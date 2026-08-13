//! Customer master data and credit control.

use crate::ap::supplier::PaymentTerms;
use crate::ledger::account::AccountCode;
use casiros_core::error::CalculationError;
use casiros_core::types::Dollar;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// A unique identifier for a [`Customer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct CustomerId(pub Uuid);

impl CustomerId {
    /// Generates a new, random customer id.
    #[must_use]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for CustomerId {
    fn default() -> Self {
        Self::new()
    }
}

/// A customer master record.
///
/// Reuses [`PaymentTerms`] from `ap::supplier`: net-days-plus-discount terms
/// are a general business concept, not specific to either side of a
/// transaction, so AR and AP share the one implementation rather than
/// duplicating it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Customer {
    /// This customer's unique id.
    pub id: CustomerId,
    /// The customer's name.
    pub name: String,
    /// The maximum outstanding receivable balance this customer may carry.
    #[schema(value_type = Decimal)]
    pub credit_limit: Dollar,
    /// This customer's standard payment terms.
    pub payment_terms: PaymentTerms,
    /// The chart-of-accounts asset account this customer's receivables post to.
    pub receivable_account: AccountCode,
}

/// Determines whether a new invoice of `new_invoice_amount` may be extended to
/// a customer, given their current outstanding balance and credit limit.
///
/// # Errors
///
/// Returns [`CalculationError::NegativeValueInvalid`] if `new_invoice_amount`
/// or `credit_limit` is negative.
///
/// # Examples
///
/// ```
/// use casiros_erp::ar::customer::can_extend_credit;
/// use rust_decimal_macros::dec;
///
/// let approved = can_extend_credit(dec!(5_000.0), dec!(10_000.0), dec!(20_000.0)).unwrap();
/// assert!(approved);
///
/// let denied = can_extend_credit(dec!(5_000.0), dec!(18_000.0), dec!(20_000.0)).unwrap();
/// assert!(!denied);
/// ```
pub fn can_extend_credit(
    new_invoice_amount: Dollar,
    current_balance: Dollar,
    credit_limit: Dollar,
) -> Result<bool, CalculationError> {
    if new_invoice_amount < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "can_extend_credit - new_invoice_amount",
            value: new_invoice_amount,
        });
    }
    if credit_limit < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "can_extend_credit - credit_limit",
            value: credit_limit,
        });
    }
    let projected_balance =
        current_balance
            .checked_add(new_invoice_amount)
            .ok_or(CalculationError::Overflow {
                formula: "can_extend_credit",
            })?;
    Ok(projected_balance <= credit_limit)
}
