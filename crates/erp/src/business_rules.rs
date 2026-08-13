//! ERP business rules. Every rule here is a pure function: no I/O, no database
//! access, no hidden state — the same inputs always produce the same decision.

use casiros_core::error::CalculationError;
use casiros_core::types::Dollar;
use rust_decimal_macros::dec;

/// Determines whether a payment can be approved based on liquidity.
///
/// A payment is approvable only if the entity can cover it from cash on hand
/// *and* doing so would not leave the entity dangerously illiquid (current
/// ratio at or below `1.2`).
///
/// # Mathematical Definition
///
/// \[ \text{Approved} = (\text{payment} \le \text{cash}) \land
/// \left(\frac{\text{cash}}{\text{current liabilities}} > 1.2\right) \]
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `current_liabilities` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if `current_liabilities` is negative.
///
/// # Examples
///
/// ```
/// use casiros_erp::business_rules::can_approve_payment;
/// use rust_decimal_macros::dec;
///
/// let approved = can_approve_payment(dec!(10_000.0), dec!(50_000.0), dec!(20_000.0)).unwrap();
/// assert!(approved);
///
/// let denied = can_approve_payment(dec!(10_000.0), dec!(15_000.0), dec!(20_000.0)).unwrap();
/// assert!(!denied);
/// ```
pub fn can_approve_payment(
    payment_amount: Dollar,
    current_cash: Dollar,
    current_liabilities: Dollar,
) -> Result<bool, CalculationError> {
    let current_ratio = casiros_core::financial::current_ratio(current_cash, current_liabilities)?;
    Ok(payment_amount <= current_cash && current_ratio > dec!(1.2))
}
