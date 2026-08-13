//! Corporate finance formulas.

use crate::error::CalculationError;
use crate::types::{Dollar, Rate, Ratio};
use rust_decimal::Decimal;

/// Validates that `value` lies within the inclusive range `[0, 1]`.
fn require_unit_range(value: Decimal, context: &'static str) -> Result<(), CalculationError> {
    if value < Decimal::ZERO || value > Decimal::ONE {
        return Err(CalculationError::RangeViolation { context, value });
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

/// Weighted Average Cost of Capital: the blended required return across a firm's capital structure.
///
/// # Mathematical Definition
///
/// \[ WACC = \frac{E}{E+D} R_e + \frac{D}{E+D} R_d (1 - T) \]
///
/// # Constraints
///
/// - `equity_value` and `debt_value` MUST be non-negative, and their sum MUST be positive.
/// - `tax_rate` MUST lie within `[0, 1]`.
///
/// # Errors
///
/// Returns [`CalculationError::NegativeValueInvalid`] if `equity_value` or `debt_value` is negative.
/// Returns [`CalculationError::DivisionByZero`] if `equity_value + debt_value` is zero.
/// Returns [`CalculationError::RangeViolation`] if `tax_rate` is outside `[0, 1]`.
///
/// # Examples
///
/// ```
/// use casiros_core::corporate::wacc;
/// use rust_decimal_macros::dec;
///
/// let result = wacc(dec!(600_000.0), dec!(400_000.0), dec!(0.10), dec!(0.06), dec!(0.25)).unwrap();
/// assert_eq!(result, dec!(0.078));
/// assert!(result > dec!(0.0));
/// ```
pub fn wacc(
    equity_value: Dollar,
    debt_value: Dollar,
    cost_of_equity: Rate,
    cost_of_debt: Rate,
    tax_rate: Ratio,
) -> Result<Rate, CalculationError> {
    if equity_value < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "wacc - equity_value",
            value: equity_value,
        });
    }
    if debt_value < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "wacc - debt_value",
            value: debt_value,
        });
    }
    require_unit_range(tax_rate, "wacc - tax_rate")?;
    let total = equity_value
        .checked_add(debt_value)
        .ok_or(CalculationError::Overflow { formula: "wacc" })?;
    if total.is_zero() {
        return Err(CalculationError::DivisionByZero { formula: "wacc" });
    }
    let equity_weight = checked_div(equity_value, total, "wacc")?;
    let debt_weight = checked_div(debt_value, total, "wacc")?;
    let retained_fraction = Decimal::ONE
        .checked_sub(tax_rate)
        .ok_or(CalculationError::Overflow { formula: "wacc" })?;
    let equity_component = equity_weight
        .checked_mul(cost_of_equity)
        .ok_or(CalculationError::Overflow { formula: "wacc" })?;
    let debt_component = debt_weight
        .checked_mul(cost_of_debt)
        .and_then(|v| v.checked_mul(retained_fraction))
        .ok_or(CalculationError::Overflow { formula: "wacc" })?;
    equity_component
        .checked_add(debt_component)
        .ok_or(CalculationError::Overflow { formula: "wacc" })
}

/// Free Cash Flow to Firm: cash generated for all capital providers before financing.
///
/// # Mathematical Definition
///
/// \[ FCFF = EBIT \times (1 - T) + D\&A - CapEx - \Delta WC \]
///
/// # Constraints
///
/// - `tax_rate` MUST lie within `[0, 1]`.
///
/// # Errors
///
/// Returns [`CalculationError::RangeViolation`] if `tax_rate` is outside `[0, 1]`.
///
/// # Examples
///
/// ```
/// use casiros_core::corporate::free_cash_flow_to_firm;
/// use rust_decimal_macros::dec;
///
/// let fcff = free_cash_flow_to_firm(dec!(500_000.0), dec!(0.25), dec!(50_000.0), dec!(80_000.0), dec!(20_000.0)).unwrap();
/// assert_eq!(fcff, dec!(325_000.0));
/// assert!(fcff > dec!(0.0));
/// ```
pub fn free_cash_flow_to_firm(
    ebit: Dollar,
    tax_rate: Ratio,
    depreciation_amortization: Dollar,
    capex: Dollar,
    change_in_working_capital: Dollar,
) -> Result<Dollar, CalculationError> {
    require_unit_range(tax_rate, "free_cash_flow_to_firm - tax_rate")?;
    let retained_fraction =
        Decimal::ONE
            .checked_sub(tax_rate)
            .ok_or(CalculationError::Overflow {
                formula: "free_cash_flow_to_firm",
            })?;
    let nopat = ebit
        .checked_mul(retained_fraction)
        .ok_or(CalculationError::Overflow {
            formula: "free_cash_flow_to_firm",
        })?;
    nopat
        .checked_add(depreciation_amortization)
        .and_then(|v| v.checked_sub(capex))
        .and_then(|v| v.checked_sub(change_in_working_capital))
        .ok_or(CalculationError::Overflow {
            formula: "free_cash_flow_to_firm",
        })
}

/// Free Cash Flow to Equity: cash generated for equity holders after debt financing.
///
/// # Mathematical Definition
///
/// \[ FCFE = NI + D\&A - CapEx - \Delta WC + \text{Net Borrowing} \]
///
/// # Constraints
///
/// None — all real-valued inputs are accepted.
///
/// # Errors
///
/// Returns [`CalculationError::Overflow`] if the accumulated result overflows.
///
/// # Examples
///
/// ```
/// use casiros_core::corporate::free_cash_flow_to_equity;
/// use rust_decimal_macros::dec;
///
/// let fcfe = free_cash_flow_to_equity(dec!(300_000.0), dec!(50_000.0), dec!(80_000.0), dec!(20_000.0), dec!(10_000.0)).unwrap();
/// assert_eq!(fcfe, dec!(260_000.0));
/// assert!(fcfe > dec!(0.0));
/// ```
pub fn free_cash_flow_to_equity(
    net_income: Dollar,
    depreciation_amortization: Dollar,
    capex: Dollar,
    change_in_working_capital: Dollar,
    net_borrowing: Dollar,
) -> Result<Dollar, CalculationError> {
    net_income
        .checked_add(depreciation_amortization)
        .and_then(|v| v.checked_sub(capex))
        .and_then(|v| v.checked_sub(change_in_working_capital))
        .and_then(|v| v.checked_add(net_borrowing))
        .ok_or(CalculationError::Overflow {
            formula: "free_cash_flow_to_equity",
        })
}

/// Economic Value Added: residual profit after charging for the cost of invested capital.
///
/// # Mathematical Definition
///
/// \[ EVA = NOPAT - (\text{Invested Capital} \times WACC) \]
///
/// # Constraints
///
/// - `invested_capital` MUST be non-negative.
///
/// # Errors
///
/// Returns [`CalculationError::NegativeValueInvalid`] if `invested_capital` is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::corporate::economic_value_added;
/// use rust_decimal_macros::dec;
///
/// let eva = economic_value_added(dec!(200_000.0), dec!(1_000_000.0), dec!(0.08)).unwrap();
/// assert_eq!(eva, dec!(120_000.0));
/// assert!(eva > dec!(0.0));
/// ```
pub fn economic_value_added(
    nopat: Dollar,
    invested_capital: Dollar,
    wacc: Rate,
) -> Result<Dollar, CalculationError> {
    if invested_capital < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "economic_value_added - invested_capital",
            value: invested_capital,
        });
    }
    let capital_charge = invested_capital
        .checked_mul(wacc)
        .ok_or(CalculationError::Overflow {
            formula: "economic_value_added",
        })?;
    nopat
        .checked_sub(capital_charge)
        .ok_or(CalculationError::Overflow {
            formula: "economic_value_added",
        })
}

/// Sustainable Growth Rate: the growth rate a firm can sustain without external equity financing.
///
/// # Mathematical Definition
///
/// \[ SGR = ROE \times b \]
///
/// # Constraints
///
/// - `retention_ratio` MUST lie within `[0, 1]`.
///
/// # Errors
///
/// Returns [`CalculationError::RangeViolation`] if `retention_ratio` is outside `[0, 1]`.
///
/// # Examples
///
/// ```
/// use casiros_core::corporate::sustainable_growth_rate;
/// use rust_decimal_macros::dec;
///
/// let sgr = sustainable_growth_rate(dec!(0.15), dec!(0.6)).unwrap();
/// assert_eq!(sgr, dec!(0.09));
/// assert!(sgr < dec!(0.15));
/// ```
pub fn sustainable_growth_rate(
    roe: Ratio,
    retention_ratio: Ratio,
) -> Result<Ratio, CalculationError> {
    require_unit_range(retention_ratio, "sustainable_growth_rate - retention_ratio")?;
    roe.checked_mul(retention_ratio)
        .ok_or(CalculationError::Overflow {
            formula: "sustainable_growth_rate",
        })
}

/// Internal Growth Rate: the growth rate achievable using only retained earnings.
///
/// # Mathematical Definition
///
/// \[ IGR = \frac{ROA \times b}{1 - ROA \times b} \]
///
/// # Constraints
///
/// - `retention_ratio` MUST lie within `[0, 1]`.
/// - `1 - (ROA \times b)` MUST NOT be zero.
///
/// # Errors
///
/// Returns [`CalculationError::RangeViolation`] if `retention_ratio` is outside `[0, 1]`.
/// Returns [`CalculationError::DivisionByZero`] if `ROA \times b == 1`.
///
/// # Examples
///
/// ```
/// use casiros_core::corporate::internal_growth_rate;
/// use rust_decimal_macros::dec;
///
/// let igr = internal_growth_rate(dec!(0.10), dec!(0.5)).unwrap();
/// assert!(igr > dec!(0.05));
/// assert!(igr < dec!(0.06));
/// ```
pub fn internal_growth_rate(roa: Ratio, retention_ratio: Ratio) -> Result<Ratio, CalculationError> {
    require_unit_range(retention_ratio, "internal_growth_rate - retention_ratio")?;
    let retained_return = roa
        .checked_mul(retention_ratio)
        .ok_or(CalculationError::Overflow {
            formula: "internal_growth_rate",
        })?;
    let denom = Decimal::ONE
        .checked_sub(retained_return)
        .ok_or(CalculationError::Overflow {
            formula: "internal_growth_rate",
        })?;
    if denom.is_zero() {
        return Err(CalculationError::DivisionByZero {
            formula: "internal_growth_rate",
        });
    }
    checked_div(retained_return, denom, "internal_growth_rate")
}
