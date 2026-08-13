//! The error type for all CASIROS ERP operations.

use crate::ledger::account::AccountCode;
use casiros_core::error::CalculationError;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use thiserror::Error;
use uuid::Uuid;

/// The error type for all CASIROS ERP operations.
#[derive(Debug, Clone, Error, PartialEq)]
pub enum ErpError {
    /// An account code was registered twice in the same chart of accounts.
    #[error("account {0:?} is already registered in the chart of accounts")]
    DuplicateAccount(AccountCode),

    /// An account code was referenced but never registered.
    #[error("account {0:?} is not registered in the chart of accounts")]
    UnknownAccount(AccountCode),

    /// A journal entry's lines were empty, or its total debits did not equal
    /// its total credits.
    #[error("journal entry {id} is not balanced: debits {debits} != credits {credits}")]
    UnbalancedEntry {
        /// The offending entry's id.
        id: Uuid,
        /// The sum of its debit lines.
        debits: Decimal,
        /// The sum of its credit lines.
        credits: Decimal,
    },

    /// A journal line had both a debit and a credit, or neither.
    #[error("journal line for account {0:?} must have exactly one of debit or credit set")]
    InvalidLine(AccountCode),

    /// An entry was posted to a fiscal period that has been closed.
    #[error("fiscal period {0:?} is closed and cannot accept new postings")]
    PeriodClosed(super::ledger::period::FiscalPeriod),

    /// The account roll-up hierarchy contains a cycle and cannot be
    /// topologically ordered for consolidation.
    #[error("cyclic account roll-up hierarchy: {0}")]
    CyclicHierarchy(String),

    /// A payment against an AP invoice exceeded that invoice's remaining balance due.
    #[error(
        "payment of {payment} against invoice {invoice} exceeds its balance due of {balance_due}"
    )]
    PaymentExceedsBalance {
        /// The invoice's id.
        invoice: Uuid,
        /// The invoice's balance due before this payment.
        balance_due: Decimal,
        /// The rejected payment amount.
        payment: Decimal,
    },

    /// An ASC 606 ratable-recognition period had its end on or before its start.
    #[error("invalid revenue recognition period: end {end} is not after start {start}")]
    InvalidRecognitionPeriod {
        /// The period's start date.
        start: NaiveDate,
        /// The period's (invalid) end date.
        end: NaiveDate,
    },

    /// A currency code was not exactly three ASCII uppercase letters (ISO 4217 style).
    #[error("invalid currency code {0:?}: must be exactly three ASCII uppercase letters")]
    InvalidCurrencyCode(String),

    /// An exchange rate's `from` currency did not match the exposure being converted.
    #[error(
        "exchange rate is denominated in {actual:?}, but the exposure is denominated in {expected:?}"
    )]
    CurrencyMismatch {
        /// The currency the exposure is actually denominated in.
        expected: super::treasury::fx::CurrencyCode,
        /// The currency the exchange rate's `from` field specified.
        actual: super::treasury::fx::CurrencyCode,
    },

    /// A tax jurisdiction's brackets were malformed (empty, or a non-final
    /// bracket left unbounded).
    #[error("invalid tax brackets: {0}")]
    InvalidTaxBrackets(String),

    /// A budget line item referenced a driver name not present in the model.
    #[error("budget model has no driver named {0:?}")]
    UnknownDriver(String),

    /// A `casiros_core` formula call failed while computing a ledger value.
    #[error(transparent)]
    Calculation(#[from] CalculationError),
}
