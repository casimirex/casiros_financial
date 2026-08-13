//! Causally-linked journal entries: not just debits and credits, but a record
//! of *why* the entry exists.

use super::account::AccountCode;
use super::period::FiscalPeriod;
use crate::error::ErpError;
use casiros_core::error::CalculationError;
use casiros_core::types::Dollar;
use casiros_dag::graph::FormulaNode;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identifies what kind of business document originated a [`JournalEntry`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceDocument {
    /// A manually-entered adjustment, with no upstream business document.
    ManualEntry,
    /// Originated from an accounts-payable or accounts-receivable invoice.
    Invoice {
        /// The invoice's id.
        id: Uuid,
    },
    /// Originated from a payment.
    Payment {
        /// The payment's id.
        id: Uuid,
    },
    /// Originated from a cash receipt.
    Receipt {
        /// The receipt's id.
        id: Uuid,
    },
    /// A period-end accrual (e.g. interest, depreciation).
    Accrual,
    /// A consolidation/roll-up entry produced by [`crate::ledger::consolidation`].
    Consolidation,
}

/// A single debit or credit line within a [`JournalEntry`]. Exactly one of
/// `debit`/`credit` is non-zero.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalLine {
    /// The account this line posts to.
    pub account: AccountCode,
    /// The debit amount, or zero if this is a credit line.
    pub debit: Dollar,
    /// The credit amount, or zero if this is a debit line.
    pub credit: Dollar,
    /// If this line's amount was computed by a `casiros_core` formula, which one.
    pub causal_formula: Option<FormulaNode>,
}

impl JournalLine {
    /// Creates a validated journal line.
    ///
    /// # Errors
    ///
    /// Returns [`ErpError::InvalidLine`] unless exactly one of `debit`/`credit`
    /// is strictly positive and the other is exactly zero.
    pub fn new(
        account: AccountCode,
        debit: Dollar,
        credit: Dollar,
        causal_formula: Option<FormulaNode>,
    ) -> Result<Self, ErpError> {
        let debit_set = debit > Decimal::ZERO;
        let credit_set = credit > Decimal::ZERO;
        if debit < Decimal::ZERO || credit < Decimal::ZERO || debit_set == credit_set {
            return Err(ErpError::InvalidLine(account));
        }
        Ok(Self {
            account,
            debit,
            credit,
            causal_formula,
        })
    }

    /// This line's signed contribution to its account's balance, tracked
    /// uniformly as `debit - credit` regardless of account type.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if the subtraction overflows.
    pub fn signed_amount(&self) -> Result<Decimal, CalculationError> {
        self.debit
            .checked_sub(self.credit)
            .ok_or(CalculationError::Overflow {
                formula: "JournalLine::signed_amount",
            })
    }
}

/// A causally-linked general ledger entry: not just debits and credits, but a
/// record of *why* the entry exists, via [`Self::causal_parent`] and
/// [`Self::source_document`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalEntry {
    /// A unique identifier for this entry.
    pub id: Uuid,
    /// The posting date.
    pub date: NaiveDate,
    /// A human-readable description.
    pub description: String,
    /// The debit/credit lines. Always non-empty and balanced.
    pub lines: Vec<JournalLine>,
    /// The entry that causally produced this one, if any (e.g. an accrual
    /// entry's parent might be the invoice entry that triggered it).
    pub causal_parent: Option<Uuid>,
    /// What kind of business document originated this entry.
    pub source_document: SourceDocument,
    /// The fiscal period this entry posts into.
    pub period: FiscalPeriod,
}

fn sum_debits(lines: &[JournalLine]) -> Result<Dollar, CalculationError> {
    lines
        .iter()
        .try_fold(Decimal::ZERO, |total, line| total.checked_add(line.debit))
        .ok_or(CalculationError::Overflow {
            formula: "JournalEntry::total_debits",
        })
}

fn sum_credits(lines: &[JournalLine]) -> Result<Dollar, CalculationError> {
    lines
        .iter()
        .try_fold(Decimal::ZERO, |total, line| total.checked_add(line.credit))
        .ok_or(CalculationError::Overflow {
            formula: "JournalEntry::total_credits",
        })
}

impl JournalEntry {
    /// Creates a new, validated journal entry with a freshly-generated id.
    ///
    /// # Errors
    ///
    /// Returns an error (via [`CalculationError::MissingInput`]) if `lines` is
    /// empty, or [`ErpError::UnbalancedEntry`] if total debits do not equal
    /// total credits.
    pub fn new(
        date: NaiveDate,
        description: impl Into<String>,
        lines: Vec<JournalLine>,
        causal_parent: Option<Uuid>,
        source_document: SourceDocument,
        period: FiscalPeriod,
    ) -> Result<Self, ErpError> {
        if lines.is_empty() {
            return Err(CalculationError::MissingInput {
                formula: "JournalEntry::new",
                parameter: "lines",
            }
            .into());
        }
        let id = Uuid::new_v4();
        let debits = sum_debits(&lines)?;
        let credits = sum_credits(&lines)?;
        if debits != credits {
            return Err(ErpError::UnbalancedEntry {
                id,
                debits,
                credits,
            });
        }
        Ok(Self {
            id,
            date,
            description: description.into(),
            lines,
            causal_parent,
            source_document,
            period,
        })
    }

    /// The sum of this entry's debit lines.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if the sum overflows.
    pub fn total_debits(&self) -> Result<Dollar, CalculationError> {
        sum_debits(&self.lines)
    }

    /// The sum of this entry's credit lines.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if the sum overflows.
    pub fn total_credits(&self) -> Result<Dollar, CalculationError> {
        sum_credits(&self.lines)
    }
}
