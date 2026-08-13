//! Cash receipt allocation across a customer's open invoices.

use super::customer::CustomerId;
use super::invoice::{ArInvoice, ArInvoiceId};
use crate::error::ErpError;
use casiros_core::error::CalculationError;
use casiros_core::types::Dollar;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// A unique identifier for a [`Receipt`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ReceiptId(pub Uuid);

/// An incoming cash receipt from a customer, not yet allocated to any invoice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Receipt {
    /// This receipt's unique id.
    pub id: ReceiptId,
    /// The customer this cash was received from.
    pub customer: CustomerId,
    /// The amount received.
    #[schema(value_type = Decimal)]
    pub amount: Dollar,
    /// The date the cash was received.
    pub date: NaiveDate,
}

impl Receipt {
    /// Creates a new receipt.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::NegativeValueInvalid`] if `amount` is not strictly positive.
    pub fn new(customer: CustomerId, amount: Dollar, date: NaiveDate) -> Result<Self, ErpError> {
        if amount <= Decimal::ZERO {
            return Err(CalculationError::NegativeValueInvalid {
                context: "Receipt::new - amount",
                value: amount,
            }
            .into());
        }
        Ok(Self {
            id: ReceiptId(Uuid::new_v4()),
            customer,
            amount,
            date,
        })
    }
}

/// How much of a [`Receipt`] was applied against one invoice.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ReceiptAllocation {
    /// The invoice this portion of the receipt was applied to.
    pub invoice: ArInvoiceId,
    /// The amount applied.
    #[schema(value_type = Decimal)]
    pub amount_applied: Dollar,
}

/// Allocates `receipt` across `invoices` belonging to the same customer,
/// oldest due date first, mutating each invoice's balance via
/// [`ArInvoice::apply_receipt`] as it goes. Invoices belonging to a different
/// customer, or already fully collected, are left untouched.
///
/// Any portion of the receipt left over once every open invoice is fully
/// settled is simply not allocated (an unapplied cash balance is a treasury
/// concern, out of scope here).
///
/// # Errors
///
/// Returns [`ErpError::Calculation`] if a date or balance computation overflows.
pub fn allocate_receipt(
    receipt: &Receipt,
    invoices: &mut [ArInvoice],
) -> Result<Vec<ReceiptAllocation>, ErpError> {
    let mut candidate_indices: Vec<usize> = Vec::new();
    for (index, invoice) in invoices.iter().enumerate() {
        if invoice.customer != receipt.customer {
            continue;
        }
        if invoice.balance_due()? <= Decimal::ZERO {
            continue;
        }
        candidate_indices.push(index);
    }
    candidate_indices.sort_by_key(|&index| invoices[index].due_date().unwrap_or(NaiveDate::MAX));

    let mut remaining = receipt.amount;
    let mut allocations = Vec::new();
    for index in candidate_indices {
        if remaining <= Decimal::ZERO {
            break;
        }
        let invoice = &mut invoices[index];
        let balance_due = invoice.balance_due()?;
        let applied = remaining.min(balance_due);
        invoice.apply_receipt(applied)?;
        remaining = remaining
            .checked_sub(applied)
            .ok_or(CalculationError::Overflow {
                formula: "allocate_receipt",
            })?;
        allocations.push(ReceiptAllocation {
            invoice: invoice.id,
            amount_applied: applied,
        });
    }
    Ok(allocations)
}
