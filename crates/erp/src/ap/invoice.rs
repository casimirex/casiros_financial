//! Accounts-payable invoices and aging.

use super::supplier::{PaymentTerms, SupplierId};
use crate::error::ErpError;
use casiros_core::error::CalculationError;
use casiros_core::types::Dollar;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A unique identifier for an [`ApInvoice`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApInvoiceId(pub Uuid);

/// An accounts-payable invoice's settlement status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApInvoiceStatus {
    /// No payments have been applied yet.
    Open,
    /// Some, but not all, of the invoice has been paid.
    PartiallyPaid,
    /// The invoice is fully paid.
    Paid,
}

/// An accounts-payable invoice received from a supplier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ApInvoice {
    /// This invoice's unique id.
    pub id: ApInvoiceId,
    /// The supplier that issued this invoice.
    pub supplier: SupplierId,
    /// The supplier's own invoice number, for matching and deduplication.
    pub invoice_number: String,
    /// The date the invoice was issued.
    pub invoice_date: NaiveDate,
    /// The original invoice amount.
    pub amount: Dollar,
    /// The payment terms governing this invoice's due date and any discount.
    pub terms: PaymentTerms,
    /// The total amount paid against this invoice so far.
    pub amount_paid: Dollar,
    /// This invoice's current settlement status.
    pub status: ApInvoiceStatus,
}

impl ApInvoice {
    /// Creates a new, unpaid invoice.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::NegativeValueInvalid`] if `amount` is not strictly positive.
    pub fn new(
        supplier: SupplierId,
        invoice_number: impl Into<String>,
        invoice_date: NaiveDate,
        amount: Dollar,
        terms: PaymentTerms,
    ) -> Result<Self, ErpError> {
        if amount <= Decimal::ZERO {
            return Err(CalculationError::NegativeValueInvalid {
                context: "ApInvoice::new - amount",
                value: amount,
            }
            .into());
        }
        Ok(Self {
            id: ApInvoiceId(Uuid::new_v4()),
            supplier,
            invoice_number: invoice_number.into(),
            invoice_date,
            amount,
            terms,
            amount_paid: Decimal::ZERO,
            status: ApInvoiceStatus::Open,
        })
    }

    /// The date the full amount is due.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if the resulting date is out of range.
    pub fn due_date(&self) -> Result<NaiveDate, CalculationError> {
        self.terms.due_date(self.invoice_date)
    }

    /// The remaining unpaid balance.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if the subtraction overflows.
    pub fn balance_due(&self) -> Result<Dollar, CalculationError> {
        self.amount
            .checked_sub(self.amount_paid)
            .ok_or(CalculationError::Overflow {
                formula: "ApInvoice::balance_due",
            })
    }

    /// Whether this invoice has an outstanding balance past its due date, as of `as_of`.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if a date or balance computation overflows.
    pub fn is_overdue(&self, as_of: NaiveDate) -> Result<bool, CalculationError> {
        Ok(as_of > self.due_date()? && self.balance_due()? > Decimal::ZERO)
    }

    /// Applies a payment of `amount` against this invoice, updating
    /// [`Self::amount_paid`] and [`Self::status`].
    ///
    /// # Errors
    ///
    /// Returns [`ErpError::PaymentExceedsBalance`] if `amount` exceeds the
    /// current balance due. Returns [`ErpError::Calculation`] with
    /// [`CalculationError::NegativeValueInvalid`] if `amount` is negative.
    pub fn apply_payment(&mut self, amount: Dollar) -> Result<(), ErpError> {
        if amount < Decimal::ZERO {
            return Err(CalculationError::NegativeValueInvalid {
                context: "ApInvoice::apply_payment - amount",
                value: amount,
            }
            .into());
        }
        let balance_due = self.balance_due()?;
        if amount > balance_due {
            return Err(ErpError::PaymentExceedsBalance {
                invoice: self.id.0,
                balance_due,
                payment: amount,
            });
        }
        self.amount_paid =
            self.amount_paid
                .checked_add(amount)
                .ok_or(CalculationError::Overflow {
                    formula: "ApInvoice::apply_payment",
                })?;
        self.status = if self.amount_paid == self.amount {
            ApInvoiceStatus::Paid
        } else {
            ApInvoiceStatus::PartiallyPaid
        };
        Ok(())
    }
}

/// A standard accounts-payable aging bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgingBucket {
    /// Not yet past due.
    Current,
    /// 1 to 30 days past due.
    Days1To30,
    /// 31 to 60 days past due.
    Days31To60,
    /// 61 to 90 days past due.
    Days61To90,
    /// More than 90 days past due.
    Over90,
}

/// Classifies `invoice`'s aging bucket as of `as_of`.
///
/// # Errors
///
/// Returns [`CalculationError::Overflow`] if `invoice`'s due date computation overflows.
pub fn aging_bucket(
    invoice: &ApInvoice,
    as_of: NaiveDate,
) -> Result<AgingBucket, CalculationError> {
    let days_overdue = (as_of - invoice.due_date()?).num_days();
    Ok(match days_overdue {
        i64::MIN..=0 => AgingBucket::Current,
        1..=30 => AgingBucket::Days1To30,
        31..=60 => AgingBucket::Days31To60,
        61..=90 => AgingBucket::Days61To90,
        _ => AgingBucket::Over90,
    })
}

/// Total balance due per aging bucket, across a set of invoices.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct AgingReport {
    /// Total balance due, not yet past due.
    pub current: Dollar,
    /// Total balance due, 1-30 days past due.
    pub days_1_to_30: Dollar,
    /// Total balance due, 31-60 days past due.
    pub days_31_to_60: Dollar,
    /// Total balance due, 61-90 days past due.
    pub days_61_to_90: Dollar,
    /// Total balance due, more than 90 days past due.
    pub over_90: Dollar,
}

/// Builds an [`AgingReport`] by classifying and summing every invoice's
/// balance due (only invoices with a positive balance are included).
///
/// # Errors
///
/// Returns [`CalculationError::Overflow`] if any invoice's aging classification
/// or a running total overflows.
pub fn aging_report(
    invoices: &[ApInvoice],
    as_of: NaiveDate,
) -> Result<AgingReport, CalculationError> {
    let formula = "aging_report";
    let mut report = AgingReport::default();
    for invoice in invoices {
        let balance = invoice.balance_due()?;
        if balance <= Decimal::ZERO {
            continue;
        }
        let bucket = aging_bucket(invoice, as_of)?;
        let target = match bucket {
            AgingBucket::Current => &mut report.current,
            AgingBucket::Days1To30 => &mut report.days_1_to_30,
            AgingBucket::Days31To60 => &mut report.days_31_to_60,
            AgingBucket::Days61To90 => &mut report.days_61_to_90,
            AgingBucket::Over90 => &mut report.over_90,
        };
        *target = target
            .checked_add(balance)
            .ok_or(CalculationError::Overflow { formula })?;
    }
    Ok(report)
}
