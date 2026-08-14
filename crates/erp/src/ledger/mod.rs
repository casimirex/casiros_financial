//! The causal general ledger.

pub mod account;
pub mod consolidation;
pub mod journal;
pub mod period;

use crate::error::ErpError;
use account::{Account, AccountCode, ChartOfAccounts};
use casiros_core::error::CalculationError;
use casiros_core::types::Dollar;
use journal::JournalEntry;
use period::{FiscalPeriod, PeriodStatus};
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

/// The general ledger: posted journal entries plus account balances.
///
/// Balances are **not** recomputed by summing every journal line on every
/// read. [`Ledger::post`] updates each affected leaf account's balance
/// incrementally (`O(lines)`, never rescanning prior entries) and marks it
/// dirty; [`Ledger::trial_balance`] only recomputes roll-up accounts — via
/// [`consolidation::recompute_rollups`]'s topologically-ordered DAG walk —
/// and only when something is actually dirty.
#[derive(Debug, Clone, Default)]
pub struct Ledger {
    chart: ChartOfAccounts,
    entries: Vec<JournalEntry>,
    balances: HashMap<AccountCode, Dollar>,
    dirty: HashSet<AccountCode>,
    period_status: HashMap<FiscalPeriod, PeriodStatus>,
}

impl Ledger {
    /// Creates an empty ledger over `chart`.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ledger::Ledger;
    /// use casiros_erp::ledger::account::ChartOfAccounts;
    ///
    /// let ledger = Ledger::new(ChartOfAccounts::new());
    /// assert_eq!(ledger.entries().len(), 0);
    /// assert_eq!(ledger.chart().accounts().count(), 0);
    /// ```
    #[must_use]
    pub fn new(chart: ChartOfAccounts) -> Self {
        Self {
            chart,
            ..Self::default()
        }
    }

    /// The chart of accounts this ledger posts against.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ledger::Ledger;
    /// use casiros_erp::ledger::account::{Account, AccountCode, AccountType, ChartOfAccounts};
    ///
    /// let mut ledger = Ledger::new(ChartOfAccounts::new());
    /// ledger
    ///     .register_account(Account {
    ///         code: AccountCode(1000),
    ///         name: "Cash".to_string(),
    ///         account_type: AccountType::Asset,
    ///         parent: None,
    ///     })
    ///     .unwrap();
    /// assert!(ledger.chart().contains(AccountCode(1000)));
    /// assert_eq!(ledger.chart().accounts().count(), 1);
    /// ```
    #[must_use]
    pub fn chart(&self) -> &ChartOfAccounts {
        &self.chart
    }

    /// Registers a new account into this ledger's chart of accounts.
    ///
    /// # Errors
    ///
    /// Returns [`ErpError::DuplicateAccount`] if `account.code` is already
    /// registered, or [`ErpError::UnknownAccount`] if `account.parent` is set
    /// but not itself registered.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ledger::Ledger;
    /// use casiros_erp::ledger::account::{Account, AccountCode, AccountType, ChartOfAccounts};
    ///
    /// let mut ledger = Ledger::new(ChartOfAccounts::new());
    /// ledger
    ///     .register_account(Account {
    ///         code: AccountCode(1000),
    ///         name: "Cash".to_string(),
    ///         account_type: AccountType::Asset,
    ///         parent: None,
    ///     })
    ///     .unwrap();
    /// assert!(ledger.chart().contains(AccountCode(1000)));
    ///
    /// // Registering the same code twice is rejected.
    /// let duplicate = Account {
    ///     code: AccountCode(1000),
    ///     name: "Cash Again".to_string(),
    ///     account_type: AccountType::Asset,
    ///     parent: None,
    /// };
    /// assert!(ledger.register_account(duplicate).is_err());
    /// ```
    pub fn register_account(&mut self, account: Account) -> Result<(), ErpError> {
        self.chart.register(account)
    }

    /// Every entry posted so far, in posting order.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ledger::Ledger;
    /// use casiros_erp::ledger::account::ChartOfAccounts;
    ///
    /// let ledger = Ledger::new(ChartOfAccounts::new());
    /// assert_eq!(ledger.entries().len(), 0);
    /// assert!(ledger.entries().is_empty());
    /// ```
    #[must_use]
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// Marks `period` closed: further postings to it are rejected.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ledger::Ledger;
    /// use casiros_erp::ledger::account::{Account, AccountCode, AccountType, ChartOfAccounts};
    /// use casiros_erp::ledger::journal::{JournalEntry, JournalLine, SourceDocument};
    /// use casiros_erp::ledger::period::FiscalPeriod;
    /// use chrono::NaiveDate;
    /// use rust_decimal_macros::dec;
    ///
    /// let mut ledger = Ledger::new(ChartOfAccounts::new());
    /// ledger
    ///     .register_account(Account {
    ///         code: AccountCode(1000),
    ///         name: "Cash".to_string(),
    ///         account_type: AccountType::Asset,
    ///         parent: None,
    ///     })
    ///     .unwrap();
    /// ledger
    ///     .register_account(Account {
    ///         code: AccountCode(3000),
    ///         name: "Owner Equity".to_string(),
    ///         account_type: AccountType::Equity,
    ///         parent: None,
    ///     })
    ///     .unwrap();
    /// let period = FiscalPeriod::new(2026, 8).unwrap();
    /// ledger.close_period(period);
    ///
    /// let lines = vec![
    ///     JournalLine::new(AccountCode(1000), dec!(100), dec!(0), None).unwrap(),
    ///     JournalLine::new(AccountCode(3000), dec!(0), dec!(100), None).unwrap(),
    /// ];
    /// let entry = JournalEntry::new(
    ///     NaiveDate::from_ymd_opt(2026, 8, 13).unwrap(),
    ///     "Late entry",
    ///     lines,
    ///     None,
    ///     SourceDocument::ManualEntry,
    ///     period,
    /// )
    /// .unwrap();
    /// assert!(ledger.post(entry).is_err(), "posting to a closed period is rejected");
    /// assert_eq!(ledger.entries().len(), 0);
    /// ```
    pub fn close_period(&mut self, period: FiscalPeriod) {
        self.period_status.insert(period, PeriodStatus::Closed);
    }

    /// Posts `entry`: incrementally updates every affected account's balance
    /// and marks those accounts (and their roll-up ancestors) dirty.
    ///
    /// # Errors
    ///
    /// Returns [`ErpError::UnknownAccount`] if any line references an account
    /// not in the chart. Returns [`ErpError::PeriodClosed`] if `entry.period`
    /// has been closed. Returns [`ErpError::Calculation`] if a balance update
    /// overflows.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ledger::Ledger;
    /// use casiros_erp::ledger::account::{Account, AccountCode, AccountType, ChartOfAccounts};
    /// use casiros_erp::ledger::journal::{JournalEntry, JournalLine, SourceDocument};
    /// use casiros_erp::ledger::period::FiscalPeriod;
    /// use chrono::NaiveDate;
    /// use rust_decimal_macros::dec;
    ///
    /// let mut ledger = Ledger::new(ChartOfAccounts::new());
    /// ledger
    ///     .register_account(Account {
    ///         code: AccountCode(1000),
    ///         name: "Cash".to_string(),
    ///         account_type: AccountType::Asset,
    ///         parent: None,
    ///     })
    ///     .unwrap();
    /// ledger
    ///     .register_account(Account {
    ///         code: AccountCode(3000),
    ///         name: "Owner Equity".to_string(),
    ///         account_type: AccountType::Equity,
    ///         parent: None,
    ///     })
    ///     .unwrap();
    ///
    /// let lines = vec![
    ///     JournalLine::new(AccountCode(1000), dec!(100), dec!(0), None).unwrap(),
    ///     JournalLine::new(AccountCode(3000), dec!(0), dec!(100), None).unwrap(),
    /// ];
    /// let entry = JournalEntry::new(
    ///     NaiveDate::from_ymd_opt(2026, 8, 13).unwrap(),
    ///     "Initial capital",
    ///     lines,
    ///     None,
    ///     SourceDocument::ManualEntry,
    ///     FiscalPeriod::new(2026, 8).unwrap(),
    /// )
    /// .unwrap();
    /// ledger.post(entry).unwrap();
    /// assert_eq!(ledger.entries().len(), 1);
    /// assert_eq!(ledger.balance(AccountCode(1000)).unwrap(), dec!(100));
    /// ```
    pub fn post(&mut self, entry: JournalEntry) -> Result<(), ErpError> {
        if self.period_status.get(&entry.period) == Some(&PeriodStatus::Closed) {
            return Err(ErpError::PeriodClosed(entry.period));
        }
        for line in &entry.lines {
            if !self.chart.contains(line.account) {
                return Err(ErpError::UnknownAccount(line.account));
            }
        }
        for line in &entry.lines {
            let delta = line.signed_amount()?;
            let current = self.balances.entry(line.account).or_insert(Decimal::ZERO);
            *current = current
                .checked_add(delta)
                .ok_or(CalculationError::Overflow {
                    formula: "Ledger::post",
                })?;
            self.dirty.insert(line.account);
        }
        self.entries.push(entry);
        Ok(())
    }

    /// Returns `account`'s current balance (`0` if it has never been posted to).
    ///
    /// # Errors
    ///
    /// Returns [`ErpError::UnknownAccount`] if `account` is not registered.
    /// Returns [`ErpError::CyclicHierarchy`] if a pending roll-up recompute
    /// finds a cycle in the account hierarchy.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ledger::Ledger;
    /// use casiros_erp::ledger::account::{Account, AccountCode, AccountType, ChartOfAccounts};
    /// use rust_decimal_macros::dec;
    ///
    /// let mut ledger = Ledger::new(ChartOfAccounts::new());
    /// ledger
    ///     .register_account(Account {
    ///         code: AccountCode(1000),
    ///         name: "Cash".to_string(),
    ///         account_type: AccountType::Asset,
    ///         parent: None,
    ///     })
    ///     .unwrap();
    /// // Never posted to: balance defaults to zero.
    /// assert_eq!(ledger.balance(AccountCode(1000)).unwrap(), dec!(0));
    /// assert!(ledger.balance(AccountCode(9999)).is_err());
    /// ```
    pub fn balance(&mut self, account: AccountCode) -> Result<Dollar, ErpError> {
        if !self.chart.contains(account) {
            return Err(ErpError::UnknownAccount(account));
        }
        self.refresh_rollups_if_dirty()?;
        Ok(self
            .balances
            .get(&account)
            .copied()
            .unwrap_or(Decimal::ZERO))
    }

    /// Computes the full trial balance: every registered account's current balance.
    ///
    /// # Errors
    ///
    /// Returns [`ErpError::CyclicHierarchy`] if the roll-up hierarchy contains a cycle.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::ledger::Ledger;
    /// use casiros_erp::ledger::account::{Account, AccountCode, AccountType, ChartOfAccounts};
    /// use casiros_erp::ledger::journal::{JournalEntry, JournalLine, SourceDocument};
    /// use casiros_erp::ledger::period::FiscalPeriod;
    /// use chrono::NaiveDate;
    /// use rust_decimal_macros::dec;
    ///
    /// let mut ledger = Ledger::new(ChartOfAccounts::new());
    /// ledger
    ///     .register_account(Account {
    ///         code: AccountCode(1000),
    ///         name: "Cash".to_string(),
    ///         account_type: AccountType::Asset,
    ///         parent: None,
    ///     })
    ///     .unwrap();
    /// ledger
    ///     .register_account(Account {
    ///         code: AccountCode(3000),
    ///         name: "Owner Equity".to_string(),
    ///         account_type: AccountType::Equity,
    ///         parent: None,
    ///     })
    ///     .unwrap();
    ///
    /// let lines = vec![
    ///     JournalLine::new(AccountCode(1000), dec!(100), dec!(0), None).unwrap(),
    ///     JournalLine::new(AccountCode(3000), dec!(0), dec!(100), None).unwrap(),
    /// ];
    /// let entry = JournalEntry::new(
    ///     NaiveDate::from_ymd_opt(2026, 8, 13).unwrap(),
    ///     "Initial capital",
    ///     lines,
    ///     None,
    ///     SourceDocument::ManualEntry,
    ///     FiscalPeriod::new(2026, 8).unwrap(),
    /// )
    /// .unwrap();
    /// ledger.post(entry).unwrap();
    ///
    /// let trial_balance = ledger.trial_balance().unwrap();
    /// assert_eq!(trial_balance.len(), 2);
    /// assert_eq!(trial_balance[&AccountCode(1000)], dec!(100));
    /// ```
    pub fn trial_balance(&mut self) -> Result<HashMap<AccountCode, Dollar>, ErpError> {
        self.refresh_rollups_if_dirty()?;
        Ok(self
            .chart
            .accounts()
            .map(|account| {
                (
                    account.code,
                    self.balances
                        .get(&account.code)
                        .copied()
                        .unwrap_or(Decimal::ZERO),
                )
            })
            .collect())
    }

    /// Recomputes roll-up account balances via the DAG if any account has
    /// been posted to since the last recompute.
    fn refresh_rollups_if_dirty(&mut self) -> Result<(), ErpError> {
        if self.dirty.is_empty() {
            return Ok(());
        }
        consolidation::recompute_rollups(&self.chart, &mut self.balances)?;
        self.dirty.clear();
        Ok(())
    }
}
