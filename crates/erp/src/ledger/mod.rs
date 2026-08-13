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
    #[must_use]
    pub fn new(chart: ChartOfAccounts) -> Self {
        Self {
            chart,
            ..Self::default()
        }
    }

    /// The chart of accounts this ledger posts against.
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
    pub fn register_account(&mut self, account: Account) -> Result<(), ErpError> {
        self.chart.register(account)
    }

    /// Every entry posted so far, in posting order.
    #[must_use]
    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    /// Marks `period` closed: further postings to it are rejected.
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
