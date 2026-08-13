//! Banking metric formulas.

use crate::error::CalculationError;
use crate::types::{Dollar, Ratio};
use rust_decimal::Decimal;

/// Net Interest Margin: net interest income earned per dollar of earning assets.
///
/// # Mathematical Definition
///
/// \[ NIM = \frac{\text{Net Interest Income}}{\text{Average Earning Assets}} \]
///
/// # Constraints
///
/// - `avg_earning_assets` MUST be strictly positive.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `avg_earning_assets` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if `avg_earning_assets` is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::banking::net_interest_margin;
/// use rust_decimal_macros::dec;
///
/// let nim = net_interest_margin(dec!(40_000.0), dec!(1_000_000.0)).unwrap();
/// assert_eq!(nim, dec!(0.04));
/// assert!(nim > dec!(0.0));
/// ```
pub fn net_interest_margin(
    net_interest_income: Dollar,
    avg_earning_assets: Dollar,
) -> Result<Ratio, CalculationError> {
    require_positive(
        avg_earning_assets,
        "net_interest_margin - avg_earning_assets",
    )?;
    checked_div(
        net_interest_income,
        avg_earning_assets,
        "net_interest_margin",
    )
}

/// Loan-to-Deposit Ratio: loans funded per dollar of deposits held.
///
/// # Mathematical Definition
///
/// \[ LDR = \frac{\text{Total Loans}}{\text{Total Deposits}} \]
///
/// # Constraints
///
/// - `total_deposits` MUST be strictly positive.
/// - `total_loans` MUST be non-negative.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `total_deposits` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if either input is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::banking::loan_to_deposit_ratio;
/// use rust_decimal_macros::dec;
///
/// let ldr = loan_to_deposit_ratio(dec!(800_000.0), dec!(1_000_000.0)).unwrap();
/// assert_eq!(ldr, dec!(0.8));
/// assert!(ldr < dec!(1.0));
/// ```
pub fn loan_to_deposit_ratio(
    total_loans: Dollar,
    total_deposits: Dollar,
) -> Result<Ratio, CalculationError> {
    if total_loans < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "loan_to_deposit_ratio - total_loans",
            value: total_loans,
        });
    }
    require_positive(total_deposits, "loan_to_deposit_ratio - total_deposits")?;
    checked_div(total_loans, total_deposits, "loan_to_deposit_ratio")
}

/// Capital Adequacy Ratio: regulatory capital held per dollar of risk-weighted assets.
///
/// # Mathematical Definition
///
/// \[ CAR = \frac{\text{Tier 1} + \text{Tier 2 Capital}}{\text{Risk-Weighted Assets}} \]
///
/// # Constraints
///
/// - `risk_weighted_assets` MUST be strictly positive.
/// - `qualifying_capital` MUST be non-negative.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `risk_weighted_assets` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if either input is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::banking::capital_adequacy_ratio;
/// use rust_decimal_macros::dec;
///
/// let car = capital_adequacy_ratio(dec!(120_000.0), dec!(1_000_000.0)).unwrap();
/// assert_eq!(car, dec!(0.12));
/// assert!(car > dec!(0.08));
/// ```
pub fn capital_adequacy_ratio(
    qualifying_capital: Dollar,
    risk_weighted_assets: Dollar,
) -> Result<Ratio, CalculationError> {
    if qualifying_capital < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "capital_adequacy_ratio - qualifying_capital",
            value: qualifying_capital,
        });
    }
    require_positive(
        risk_weighted_assets,
        "capital_adequacy_ratio - risk_weighted_assets",
    )?;
    checked_div(
        qualifying_capital,
        risk_weighted_assets,
        "capital_adequacy_ratio",
    )
}

/// Provision Coverage Ratio: loan loss provisions held per dollar of non-performing loans.
///
/// # Mathematical Definition
///
/// \[ \text{Provision Coverage} = \frac{\text{Loan Loss Provisions}}{\text{Non-Performing Loans}} \]
///
/// # Constraints
///
/// - `non_performing_loans` MUST be strictly positive.
/// - `loan_loss_provisions` MUST be non-negative.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `non_performing_loans` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if either input is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::banking::provision_coverage;
/// use rust_decimal_macros::dec;
///
/// let coverage = provision_coverage(dec!(50_000.0), dec!(100_000.0)).unwrap();
/// assert_eq!(coverage, dec!(0.5));
/// assert!(coverage > dec!(0.0));
/// ```
pub fn provision_coverage(
    loan_loss_provisions: Dollar,
    non_performing_loans: Dollar,
) -> Result<Ratio, CalculationError> {
    if loan_loss_provisions < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "provision_coverage - loan_loss_provisions",
            value: loan_loss_provisions,
        });
    }
    require_positive(
        non_performing_loans,
        "provision_coverage - non_performing_loans",
    )?;
    checked_div(
        loan_loss_provisions,
        non_performing_loans,
        "provision_coverage",
    )
}

/// Validates that `value` is strictly positive, mapping zero and negative cases
/// to the appropriate [`CalculationError`] variant.
fn require_positive(value: Decimal, context: &'static str) -> Result<(), CalculationError> {
    if value.is_zero() {
        return Err(CalculationError::DivisionByZero { formula: context });
    }
    if value < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid { context, value });
    }
    Ok(())
}

/// Divides `numerator` by `denominator`, mapping overflow to [`CalculationError::Overflow`].
fn checked_div(
    numerator: Decimal,
    denominator: Decimal,
    formula: &'static str,
) -> Result<Decimal, CalculationError> {
    numerator
        .checked_div(denominator)
        .ok_or(CalculationError::Overflow { formula })
}
