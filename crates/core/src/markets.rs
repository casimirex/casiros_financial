//! Market metric formulas.

use crate::error::CalculationError;
use crate::types::{Dollar, Rate, Ratio};
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;

/// Beta: sensitivity of an asset's returns to market returns.
///
/// # Mathematical Definition
///
/// \[ \beta = \frac{\text{Cov}(R_i, R_m)}{\text{Var}(R_m)} \]
///
/// # Constraints
///
/// - `variance_market` MUST be strictly positive.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `variance_market` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if `variance_market` is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::markets::beta;
/// use rust_decimal_macros::dec;
///
/// let b = beta(dec!(0.02), dec!(0.01)).unwrap();
/// assert_eq!(b, dec!(2.0));
/// assert!(b > dec!(1.0));
/// ```
pub fn beta(covariance: Decimal, variance_market: Decimal) -> Result<Decimal, CalculationError> {
    require_positive(variance_market, "beta - variance_market")?;
    checked_div(covariance, variance_market, "beta")
}

/// Sharpe Ratio: excess return earned per unit of total risk.
///
/// # Mathematical Definition
///
/// \[ \text{Sharpe} = \frac{R_p - R_f}{\sigma_p} \]
///
/// # Constraints
///
/// - `std_dev` MUST be strictly positive.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `std_dev` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if `std_dev` is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::markets::sharpe_ratio;
/// use rust_decimal_macros::dec;
///
/// let sharpe = sharpe_ratio(dec!(0.12), dec!(0.02), dec!(0.10)).unwrap();
/// assert_eq!(sharpe, dec!(1.0));
/// assert!(sharpe > dec!(0.0));
/// ```
pub fn sharpe_ratio(
    portfolio_return: Rate,
    risk_free_rate: Rate,
    std_dev: Decimal,
) -> Result<Decimal, CalculationError> {
    require_positive(std_dev, "sharpe_ratio - std_dev")?;
    let excess_return =
        portfolio_return
            .checked_sub(risk_free_rate)
            .ok_or(CalculationError::Overflow {
                formula: "sharpe_ratio",
            })?;
    checked_div(excess_return, std_dev, "sharpe_ratio")
}

/// Treynor Ratio: excess return earned per unit of systematic risk.
///
/// # Mathematical Definition
///
/// \[ \text{Treynor} = \frac{R_p - R_f}{\beta} \]
///
/// # Constraints
///
/// - `portfolio_beta` MUST NOT be zero.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `portfolio_beta` is zero.
///
/// # Examples
///
/// ```
/// use casiros_core::markets::treynor_ratio;
/// use rust_decimal_macros::dec;
///
/// let treynor = treynor_ratio(dec!(0.12), dec!(0.02), dec!(2.0)).unwrap();
/// assert_eq!(treynor, dec!(0.05));
/// assert!(treynor_ratio(dec!(0.12), dec!(0.02), dec!(0.0)).is_err());
/// ```
pub fn treynor_ratio(
    portfolio_return: Rate,
    risk_free_rate: Rate,
    portfolio_beta: Decimal,
) -> Result<Decimal, CalculationError> {
    if portfolio_beta.is_zero() {
        return Err(CalculationError::DivisionByZero {
            formula: "treynor_ratio",
        });
    }
    let excess_return =
        portfolio_return
            .checked_sub(risk_free_rate)
            .ok_or(CalculationError::Overflow {
                formula: "treynor_ratio",
            })?;
    checked_div(excess_return, portfolio_beta, "treynor_ratio")
}

/// Jensen's Alpha: risk-adjusted excess return versus the CAPM-predicted return.
///
/// # Mathematical Definition
///
/// \[ \alpha = R_p - \left[ R_f + \beta (R_m - R_f) \right] \]
///
/// # Constraints
///
/// None — all real-valued inputs are accepted.
///
/// # Errors
///
/// Returns [`CalculationError::Overflow`] if any intermediate step overflows.
///
/// # Examples
///
/// ```
/// use casiros_core::markets::jensens_alpha;
/// use rust_decimal_macros::dec;
///
/// let alpha = jensens_alpha(dec!(0.12), dec!(0.02), dec!(1.5), dec!(0.10)).unwrap();
/// assert_eq!(alpha, dec!(-0.02));
/// assert!(alpha < dec!(0.0));
/// ```
pub fn jensens_alpha(
    portfolio_return: Rate,
    risk_free_rate: Rate,
    portfolio_beta: Decimal,
    market_return: Rate,
) -> Result<Decimal, CalculationError> {
    let market_premium =
        market_return
            .checked_sub(risk_free_rate)
            .ok_or(CalculationError::Overflow {
                formula: "jensens_alpha",
            })?;
    let expected_return = portfolio_beta
        .checked_mul(market_premium)
        .and_then(|v| v.checked_add(risk_free_rate))
        .ok_or(CalculationError::Overflow {
            formula: "jensens_alpha",
        })?;
    portfolio_return
        .checked_sub(expected_return)
        .ok_or(CalculationError::Overflow {
            formula: "jensens_alpha",
        })
}

/// Parametric (variance-covariance) Value at Risk for a given confidence level.
///
/// # Mathematical Definition
///
/// \[ VaR = \text{Portfolio Value} \times z \times \sigma \]
///
/// # Constraints
///
/// - `portfolio_value` and `std_dev` MUST be non-negative.
/// - `z_score` MUST be non-negative (it encodes the magnitude of a downside move).
///
/// # Errors
///
/// Returns [`CalculationError::NegativeValueInvalid`] if any input is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::markets::value_at_risk;
/// use rust_decimal_macros::dec;
///
/// let var = value_at_risk(dec!(1_000_000.0), dec!(1.645), dec!(0.02)).unwrap();
/// assert_eq!(var, dec!(32900.00));
/// assert!(var > dec!(0.0));
/// ```
pub fn value_at_risk(
    portfolio_value: Dollar,
    z_score: Decimal,
    std_dev: Decimal,
) -> Result<Dollar, CalculationError> {
    non_negative(portfolio_value, "value_at_risk - portfolio_value")?;
    non_negative(z_score, "value_at_risk - z_score")?;
    non_negative(std_dev, "value_at_risk - std_dev")?;
    portfolio_value
        .checked_mul(z_score)
        .and_then(|v| v.checked_mul(std_dev))
        .ok_or(CalculationError::Overflow {
            formula: "value_at_risk",
        })
}

/// Expected Shortfall (Conditional VaR) under a Normal-distribution assumption.
///
/// # Mathematical Definition
///
/// \[ ES = \text{Portfolio Value} \times \sigma \times \frac{\varphi(z)}{1 - c} \]
///
/// where `\varphi` is the standard normal probability density function and `c`
/// is the confidence level (e.g. `0.95`).
///
/// # Constraints
///
/// - `confidence` MUST lie strictly within `(0, 1)`.
/// - `portfolio_value`, `std_dev`, and `z_score` MUST be non-negative.
///
/// # Errors
///
/// Returns [`CalculationError::RangeViolation`] if `confidence` is outside `(0, 1)`.
/// Returns [`CalculationError::NegativeValueInvalid`] if any other input is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::markets::{expected_shortfall, value_at_risk};
/// use rust_decimal_macros::dec;
///
/// let es = expected_shortfall(dec!(1_000_000.0), dec!(1.645), dec!(0.02), dec!(0.95)).unwrap();
/// let var = value_at_risk(dec!(1_000_000.0), dec!(1.645), dec!(0.02)).unwrap();
/// assert!(es > var);
/// assert!(es > dec!(0.0));
/// ```
pub fn expected_shortfall(
    portfolio_value: Dollar,
    z_score: Decimal,
    std_dev: Decimal,
    confidence: Ratio,
) -> Result<Dollar, CalculationError> {
    non_negative(portfolio_value, "expected_shortfall - portfolio_value")?;
    non_negative(z_score, "expected_shortfall - z_score")?;
    non_negative(std_dev, "expected_shortfall - std_dev")?;
    if confidence <= Decimal::ZERO || confidence >= Decimal::ONE {
        return Err(CalculationError::RangeViolation {
            context: "expected_shortfall - confidence",
            value: confidence,
        });
    }
    let density = z_score
        .checked_norm_pdf()
        .ok_or(CalculationError::Overflow {
            formula: "expected_shortfall",
        })?;
    let tail_mass = Decimal::ONE
        .checked_sub(confidence)
        .ok_or(CalculationError::Overflow {
            formula: "expected_shortfall",
        })?;
    let tail_factor = checked_div(density, tail_mass, "expected_shortfall")?;
    portfolio_value
        .checked_mul(std_dev)
        .and_then(|v| v.checked_mul(tail_factor))
        .ok_or(CalculationError::Overflow {
            formula: "expected_shortfall",
        })
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

/// Validates that `value` is non-negative.
fn non_negative(value: Decimal, context: &'static str) -> Result<(), CalculationError> {
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
