//! Accounts-receivable invoices, ASC 606 revenue recognition, and dunning.

use super::customer::CustomerId;
use crate::ap::supplier::PaymentTerms;
use crate::error::ErpError;
use casiros_core::error::CalculationError;
use casiros_core::types::Dollar;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A unique identifier for an [`ArInvoice`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArInvoiceId(pub Uuid);

/// An accounts-receivable invoice's collection status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArInvoiceStatus {
    /// No receipts have been applied yet.
    Open,
    /// Some, but not all, of the invoice has been collected.
    PartiallyCollected,
    /// The invoice is fully collected.
    Collected,
}

/// A simplified ASC 606 revenue recognition method: when the performance
/// obligation underlying an invoice is satisfied.
///
/// This models the two dominant patterns in practice — recognized entirely at
/// a single point in time, or ratably (straight-line) over a service period —
/// not the full 5-step ASC 606 model (contract combination, variable
/// consideration, multi-element allocation), which is a contract-drafting and
/// judgment exercise outside the scope of a pure formula/domain layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecognitionMethod {
    /// The full amount is recognized once `recognition_date` is reached (e.g.
    /// delivery of goods).
    PointInTime {
        /// The date the performance obligation is satisfied.
        recognition_date: NaiveDate,
    },
    /// The amount is recognized straight-line across `start..=end` (e.g. a
    /// subscription or service period).
    RatablyOverTime {
        /// The first day of the service period.
        start: NaiveDate,
        /// The last day of the service period.
        end: NaiveDate,
    },
}

/// An accounts-receivable invoice issued to a customer.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArInvoice {
    /// This invoice's unique id.
    pub id: ArInvoiceId,
    /// The customer this invoice was issued to.
    pub customer: CustomerId,
    /// This entity's own invoice number.
    pub invoice_number: String,
    /// The date the invoice was issued.
    pub invoice_date: NaiveDate,
    /// The total invoice amount.
    pub amount: Dollar,
    /// The payment terms governing this invoice's due date.
    pub terms: PaymentTerms,
    /// How the underlying performance obligation is recognized under ASC 606.
    pub recognition_method: RecognitionMethod,
    /// The total amount collected against this invoice so far.
    pub amount_received: Dollar,
    /// This invoice's current collection status.
    pub status: ArInvoiceStatus,
}

impl ArInvoice {
    /// Creates a new, unpaid invoice.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::NegativeValueInvalid`] if `amount` is not
    /// strictly positive. Returns [`ErpError::InvalidRecognitionPeriod`] if
    /// `recognition_method` is [`RecognitionMethod::RatablyOverTime`] with
    /// `end <= start`.
    pub fn new(
        customer: CustomerId,
        invoice_number: impl Into<String>,
        invoice_date: NaiveDate,
        amount: Dollar,
        terms: PaymentTerms,
        recognition_method: RecognitionMethod,
    ) -> Result<Self, ErpError> {
        if amount <= Decimal::ZERO {
            return Err(CalculationError::NegativeValueInvalid {
                context: "ArInvoice::new - amount",
                value: amount,
            }
            .into());
        }
        if let RecognitionMethod::RatablyOverTime { start, end } = recognition_method {
            if end <= start {
                return Err(ErpError::InvalidRecognitionPeriod { start, end });
            }
        }
        Ok(Self {
            id: ArInvoiceId(Uuid::new_v4()),
            customer,
            invoice_number: invoice_number.into(),
            invoice_date,
            amount,
            terms,
            recognition_method,
            amount_received: Decimal::ZERO,
            status: ArInvoiceStatus::Open,
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

    /// The remaining uncollected balance.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if the subtraction overflows.
    pub fn balance_due(&self) -> Result<Dollar, CalculationError> {
        self.amount
            .checked_sub(self.amount_received)
            .ok_or(CalculationError::Overflow {
                formula: "ArInvoice::balance_due",
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

    /// Applies a cash receipt of `amount` against this invoice, updating
    /// [`Self::amount_received`] and [`Self::status`].
    ///
    /// # Errors
    ///
    /// Returns [`ErpError::PaymentExceedsBalance`] if `amount` exceeds the
    /// current balance due. Returns [`ErpError::Calculation`] with
    /// [`CalculationError::NegativeValueInvalid`] if `amount` is negative.
    pub fn apply_receipt(&mut self, amount: Dollar) -> Result<(), ErpError> {
        if amount < Decimal::ZERO {
            return Err(CalculationError::NegativeValueInvalid {
                context: "ArInvoice::apply_receipt - amount",
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
        self.amount_received =
            self.amount_received
                .checked_add(amount)
                .ok_or(CalculationError::Overflow {
                    formula: "ArInvoice::apply_receipt",
                })?;
        self.status = if self.amount_received == self.amount {
            ArInvoiceStatus::Collected
        } else {
            ArInvoiceStatus::PartiallyCollected
        };
        Ok(())
    }

    /// The cumulative revenue recognizable by `as_of`, under [`Self::recognition_method`].
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if the proration computation overflows.
    pub fn recognized_revenue_as_of(&self, as_of: NaiveDate) -> Result<Dollar, CalculationError> {
        match self.recognition_method {
            RecognitionMethod::PointInTime { recognition_date } => {
                if as_of >= recognition_date {
                    Ok(self.amount)
                } else {
                    Ok(Decimal::ZERO)
                }
            }
            RecognitionMethod::RatablyOverTime { start, end } => {
                if as_of < start {
                    return Ok(Decimal::ZERO);
                }
                if as_of >= end {
                    return Ok(self.amount);
                }
                let formula = "ArInvoice::recognized_revenue_as_of";
                let elapsed_days = Decimal::from((as_of - start).num_days());
                let total_days = Decimal::from((end - start).num_days());
                let fraction = elapsed_days
                    .checked_div(total_days)
                    .ok_or(CalculationError::Overflow { formula })?;
                self.amount
                    .checked_mul(fraction)
                    .ok_or(CalculationError::Overflow { formula })
            }
        }
    }

    /// The deferred revenue (contract liability) balance as of `as_of`: the
    /// portion of `amount` not yet recognizable under ASC 606.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if the underlying computation overflows.
    pub fn deferred_revenue_as_of(&self, as_of: NaiveDate) -> Result<Dollar, CalculationError> {
        let recognized = self.recognized_revenue_as_of(as_of)?;
        self.amount
            .checked_sub(recognized)
            .ok_or(CalculationError::Overflow {
                formula: "ArInvoice::deferred_revenue_as_of",
            })
    }
}

/// An escalating collections action, driven by how overdue an invoice is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DunningLevel {
    /// Not overdue: no action needed.
    None,
    /// Mildly overdue: a friendly reminder.
    Reminder,
    /// Overdue: a formal first notice.
    FirstNotice,
    /// Significantly overdue: a final notice before escalation.
    FinalNotice,
    /// Severely overdue: escalate to collections.
    Collections,
}

/// Classifies the dunning action `invoice` warrants as of `as_of`.
///
/// # Errors
///
/// Returns [`CalculationError::Overflow`] if `invoice`'s due date computation overflows.
pub fn dunning_level(
    invoice: &ArInvoice,
    as_of: NaiveDate,
) -> Result<DunningLevel, CalculationError> {
    if invoice.balance_due()? <= Decimal::ZERO {
        return Ok(DunningLevel::None);
    }
    let days_overdue = (as_of - invoice.due_date()?).num_days();
    Ok(match days_overdue {
        i64::MIN..=0 => DunningLevel::None,
        1..=15 => DunningLevel::Reminder,
        16..=45 => DunningLevel::FirstNotice,
        46..=90 => DunningLevel::FinalNotice,
        _ => DunningLevel::Collections,
    })
}
