//! Chart of accounts: account identity, type, and roll-up hierarchy.

use crate::error::ErpError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

/// The five fundamental account types, determining an account's normal
/// (debit- or credit-increasing) balance side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum AccountType {
    /// Resources owned by the entity. Normal balance: debit.
    Asset,
    /// Obligations owed by the entity. Normal balance: credit.
    Liability,
    /// The owners' residual claim on assets. Normal balance: credit.
    Equity,
    /// Inflows from the entity's operations. Normal balance: credit.
    Revenue,
    /// Outflows from the entity's operations. Normal balance: debit.
    Expense,
}

impl AccountType {
    /// Whether a debit posting increases (rather than decreases) this account
    /// type's normal balance. Used for presentation only — the ledger itself
    /// tracks every account uniformly as `debit - credit`, so the fundamental
    /// double-entry invariant (total debits equal total credits) holds
    /// regardless of account type.
    #[must_use]
    pub fn debit_increases(self) -> bool {
        matches!(self, Self::Asset | Self::Expense)
    }
}

/// A unique numeric identifier for a ledger account (e.g. `1000` for Cash).
///
/// `Copy + Eq + Hash`, so it can be used directly as a
/// [`casiros_dag::graph::CausalityEngine`] node for roll-up dependency tracking.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, ToSchema,
)]
pub struct AccountCode(pub u32);

/// A single entry in the chart of accounts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Account {
    /// This account's unique code.
    pub code: AccountCode,
    /// A human-readable name (e.g. `"Cash"`).
    pub name: String,
    /// Which of the five fundamental types this account is.
    pub account_type: AccountType,
    /// The roll-up parent this account consolidates into, if any.
    pub parent: Option<AccountCode>,
}

/// The complete chart of accounts: a registry of [`Account`]s plus the
/// parent/child roll-up hierarchy consumed by [`crate::ledger::consolidation`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChartOfAccounts {
    accounts: HashMap<AccountCode, Account>,
}

impl ChartOfAccounts {
    /// Creates an empty chart of accounts.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers `account`.
    ///
    /// # Errors
    ///
    /// Returns [`ErpError::DuplicateAccount`] if `account.code` is already
    /// registered, or [`ErpError::UnknownAccount`] if `account.parent` is set
    /// but not itself registered.
    pub fn register(&mut self, account: Account) -> Result<(), ErpError> {
        if self.accounts.contains_key(&account.code) {
            return Err(ErpError::DuplicateAccount(account.code));
        }
        if let Some(parent) = account.parent {
            if !self.accounts.contains_key(&parent) {
                return Err(ErpError::UnknownAccount(parent));
            }
        }
        self.accounts.insert(account.code, account);
        Ok(())
    }

    /// Returns whether `code` is registered.
    #[must_use]
    pub fn contains(&self, code: AccountCode) -> bool {
        self.accounts.contains_key(&code)
    }

    /// Looks up an account by code.
    #[must_use]
    pub fn get(&self, code: AccountCode) -> Option<&Account> {
        self.accounts.get(&code)
    }

    /// Iterates every registered account, in no particular order.
    pub fn accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.values()
    }

    /// Iterates the direct children of `parent` (accounts whose `parent` field
    /// equals `Some(parent)`).
    pub fn children_of(&self, parent: AccountCode) -> impl Iterator<Item = &Account> {
        self.accounts
            .values()
            .filter(move |account| account.parent == Some(parent))
    }
}
