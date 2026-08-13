//! Time Value of Money formulas.

use crate::error::CalculationError;
use crate::types::{Dollar, Periods, Rate};
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;

/// Computes `(1 + rate)^periods`, validating that `rate > -1`.
fn growth_factor(
    rate: Rate,
    periods: Periods,
    formula: &'static str,
) -> Result<Decimal, CalculationError> {
    if rate <= Decimal::NEGATIVE_ONE {
        return Err(CalculationError::InvalidRate { rate });
    }
    let base = Decimal::ONE
        .checked_add(rate)
        .ok_or(CalculationError::Overflow { formula })?;
    base.checked_powu(u64::from(periods))
        .ok_or(CalculationError::Overflow { formula })
}

/// Future Value of a lump sum under discrete compound interest.
///
/// # Mathematical Definition
///
/// \[ FV = PV \times (1 + r)^n \]
///
/// # Constraints
///
/// - `rate` MUST be greater than -1.0.
///
/// # Errors
///
/// Returns [`CalculationError::InvalidRate`] if `rate <= -1.0`.
/// Returns [`CalculationError::Overflow`] if the result exceeds `Decimal`'s range.
///
/// # Examples
///
/// ```
/// use casiros_core::general::future_value;
/// use rust_decimal_macros::dec;
///
/// let fv = future_value(dec!(1000.0), dec!(0.05), 10).unwrap();
/// assert!(fv > dec!(1000.0));
/// assert!(fv > dec!(1600.0) && fv < dec!(1700.0));
/// ```
pub fn future_value(pv: Dollar, rate: Rate, periods: Periods) -> Result<Dollar, CalculationError> {
    let factor = growth_factor(rate, periods, "future_value")?;
    pv.checked_mul(factor).ok_or(CalculationError::Overflow {
        formula: "future_value",
    })
}

/// Present Value of a future lump sum under discrete compound discounting.
///
/// # Mathematical Definition
///
/// \[ PV = \frac{FV}{(1 + r)^n} \]
///
/// # Constraints
///
/// - `rate` MUST be greater than -1.0.
///
/// # Errors
///
/// Returns [`CalculationError::InvalidRate`] if `rate <= -1.0`.
/// Returns [`CalculationError::Overflow`] if the division cannot be represented.
///
/// # Examples
///
/// ```
/// use casiros_core::general::present_value;
/// use rust_decimal_macros::dec;
///
/// let pv = present_value(dec!(1100.0), dec!(0.10), 1).unwrap();
/// assert_eq!(pv, dec!(1000));
/// assert!(pv < dec!(1100.0));
/// ```
pub fn present_value(fv: Dollar, rate: Rate, periods: Periods) -> Result<Dollar, CalculationError> {
    let factor = growth_factor(rate, periods, "present_value")?;
    fv.checked_div(factor).ok_or(CalculationError::Overflow {
        formula: "present_value",
    })
}

/// Future Value of an ordinary annuity (level payments at period end).
///
/// # Mathematical Definition
///
/// \[ FV = PMT \times \frac{(1 + r)^n - 1}{r} \]
///
/// # Constraints
///
/// - `rate` MUST be greater than -1.0.
/// - When `rate` is zero, the well-defined limit `PMT * n` is returned.
///
/// # Errors
///
/// Returns [`CalculationError::InvalidRate`] if `rate <= -1.0`.
/// Returns [`CalculationError::Overflow`] if any intermediate step overflows.
///
/// # Examples
///
/// ```
/// use casiros_core::general::annuity_future_value;
/// use rust_decimal_macros::dec;
///
/// let fv = annuity_future_value(dec!(1000.0), dec!(0.05), 10).unwrap();
/// assert!(fv > dec!(10000.0));
/// let zero_rate = annuity_future_value(dec!(1000.0), dec!(0.0), 10).unwrap();
/// assert_eq!(zero_rate, dec!(10000.0));
/// ```
pub fn annuity_future_value(
    pmt: Dollar,
    rate: Rate,
    periods: Periods,
) -> Result<Dollar, CalculationError> {
    if periods == 0 {
        return Ok(Decimal::ZERO);
    }
    if rate.is_zero() {
        return pmt
            .checked_mul(Decimal::from(periods))
            .ok_or(CalculationError::Overflow {
                formula: "annuity_future_value",
            });
    }
    let factor = growth_factor(rate, periods, "annuity_future_value")?;
    let numerator = factor
        .checked_sub(Decimal::ONE)
        .ok_or(CalculationError::Overflow {
            formula: "annuity_future_value",
        })?;
    pmt.checked_mul(numerator)
        .and_then(|v| v.checked_div(rate))
        .ok_or(CalculationError::Overflow {
            formula: "annuity_future_value",
        })
}

/// Present Value of an ordinary annuity (level payments at period end).
///
/// # Mathematical Definition
///
/// \[ PV = PMT \times \frac{1 - (1 + r)^{-n}}{r} \]
///
/// # Constraints
///
/// - `rate` MUST be greater than -1.0.
/// - When `rate` is zero, the well-defined limit `PMT * n` is returned.
///
/// # Errors
///
/// Returns [`CalculationError::InvalidRate`] if `rate <= -1.0`.
/// Returns [`CalculationError::Overflow`] if any intermediate step overflows.
///
/// # Examples
///
/// ```
/// use casiros_core::general::annuity_present_value;
/// use rust_decimal_macros::dec;
///
/// let pv = annuity_present_value(dec!(1000.0), dec!(0.05), 10).unwrap();
/// assert!(pv > dec!(7000.0) && pv < dec!(8000.0));
/// let zero_rate = annuity_present_value(dec!(1000.0), dec!(0.0), 10).unwrap();
/// assert_eq!(zero_rate, dec!(10000.0));
/// ```
pub fn annuity_present_value(
    pmt: Dollar,
    rate: Rate,
    periods: Periods,
) -> Result<Dollar, CalculationError> {
    if periods == 0 {
        return Ok(Decimal::ZERO);
    }
    if rate.is_zero() {
        return pmt
            .checked_mul(Decimal::from(periods))
            .ok_or(CalculationError::Overflow {
                formula: "annuity_present_value",
            });
    }
    let factor = growth_factor(rate, periods, "annuity_present_value")?;
    let discount = Decimal::ONE
        .checked_div(factor)
        .ok_or(CalculationError::Overflow {
            formula: "annuity_present_value",
        })?;
    let numerator = Decimal::ONE
        .checked_sub(discount)
        .ok_or(CalculationError::Overflow {
            formula: "annuity_present_value",
        })?;
    pmt.checked_mul(numerator)
        .and_then(|v| v.checked_div(rate))
        .ok_or(CalculationError::Overflow {
            formula: "annuity_present_value",
        })
}

/// Present Value of a level (non-growing) perpetuity.
///
/// # Mathematical Definition
///
/// \[ PV = \frac{PMT}{r} \]
///
/// # Constraints
///
/// - `rate` MUST be strictly positive (a perpetuity only converges for `r > 0`).
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `rate` is zero.
/// Returns [`CalculationError::InvalidRate`] if `rate` is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::general::perpetuity_present_value;
/// use rust_decimal_macros::dec;
///
/// let pv = perpetuity_present_value(dec!(100.0), dec!(0.05)).unwrap();
/// assert_eq!(pv, dec!(2000));
/// assert!(perpetuity_present_value(dec!(100.0), dec!(0.0)).is_err());
/// ```
pub fn perpetuity_present_value(pmt: Dollar, rate: Rate) -> Result<Dollar, CalculationError> {
    if rate.is_zero() {
        return Err(CalculationError::DivisionByZero {
            formula: "perpetuity_present_value",
        });
    }
    if rate < Decimal::ZERO {
        return Err(CalculationError::InvalidRate { rate });
    }
    pmt.checked_div(rate).ok_or(CalculationError::Overflow {
        formula: "perpetuity_present_value",
    })
}

/// Present Value of a perpetuity whose payments grow at a constant rate (Gordon Growth Model).
///
/// # Mathematical Definition
///
/// \[ PV = \frac{D_1}{r - g} \]
///
/// # Constraints
///
/// - `rate` MUST be strictly greater than `growth` for the series to converge.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `rate == growth`.
/// Returns [`CalculationError::RangeViolation`] if `rate < growth`.
///
/// # Examples
///
/// ```
/// use casiros_core::general::growing_perpetuity;
/// use rust_decimal_macros::dec;
///
/// let pv = growing_perpetuity(dec!(100.0), dec!(0.09), dec!(0.04)).unwrap();
/// assert_eq!(pv, dec!(2000));
/// assert!(growing_perpetuity(dec!(100.0), dec!(0.04), dec!(0.09)).is_err());
/// ```
pub fn growing_perpetuity(
    d1: Dollar,
    rate: Rate,
    growth: Rate,
) -> Result<Dollar, CalculationError> {
    if rate == growth {
        return Err(CalculationError::DivisionByZero {
            formula: "growing_perpetuity",
        });
    }
    if rate < growth {
        return Err(CalculationError::RangeViolation {
            context: "growing_perpetuity - rate must exceed growth",
            value: rate,
        });
    }
    let denom = rate.checked_sub(growth).ok_or(CalculationError::Overflow {
        formula: "growing_perpetuity",
    })?;
    d1.checked_div(denom).ok_or(CalculationError::Overflow {
        formula: "growing_perpetuity",
    })
}

/// Effective Annual Rate implied by a nominal rate compounded `n` times per year.
///
/// # Mathematical Definition
///
/// \[ EAR = \left(1 + \frac{r}{n}\right)^n - 1 \]
///
/// # Constraints
///
/// - `compounding_periods` MUST be greater than zero.
/// - The resulting periodic rate `r/n` MUST be greater than -1.0.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `compounding_periods` is zero.
/// Returns [`CalculationError::InvalidRate`] if the periodic rate is `<= -1.0`.
///
/// # Examples
///
/// ```
/// use casiros_core::general::effective_annual_rate;
/// use rust_decimal_macros::dec;
///
/// let ear = effective_annual_rate(dec!(0.12), 12).unwrap();
/// assert!(ear > dec!(0.12));
/// assert!(ear < dec!(0.13));
/// ```
pub fn effective_annual_rate(
    nominal_rate: Rate,
    compounding_periods: Periods,
) -> Result<Rate, CalculationError> {
    if compounding_periods == 0 {
        return Err(CalculationError::DivisionByZero {
            formula: "effective_annual_rate",
        });
    }
    let n = Decimal::from(compounding_periods);
    let periodic_rate = nominal_rate
        .checked_div(n)
        .ok_or(CalculationError::Overflow {
            formula: "effective_annual_rate",
        })?;
    if periodic_rate <= Decimal::NEGATIVE_ONE {
        return Err(CalculationError::InvalidRate { rate: nominal_rate });
    }
    let base = Decimal::ONE
        .checked_add(periodic_rate)
        .ok_or(CalculationError::Overflow {
            formula: "effective_annual_rate",
        })?;
    let factor =
        base.checked_powu(u64::from(compounding_periods))
            .ok_or(CalculationError::Overflow {
                formula: "effective_annual_rate",
            })?;
    factor
        .checked_sub(Decimal::ONE)
        .ok_or(CalculationError::Overflow {
            formula: "effective_annual_rate",
        })
}

/// Future Value of a lump sum under continuous compounding.
///
/// # Mathematical Definition
///
/// \[ FV = PV \times e^{rt} \]
///
/// # Constraints
///
/// - `time` MUST be non-negative.
///
/// # Errors
///
/// Returns [`CalculationError::NegativeValueInvalid`] if `time` is negative.
/// Returns [`CalculationError::Overflow`] if any intermediate step overflows.
///
/// # Examples
///
/// ```
/// use casiros_core::general::continuous_compounding;
/// use rust_decimal_macros::dec;
///
/// let fv = continuous_compounding(dec!(1000.0), dec!(0.05), dec!(1.0)).unwrap();
/// assert!(fv > dec!(1000.0));
/// assert!(fv > dec!(1050.0) && fv < dec!(1060.0));
/// ```
pub fn continuous_compounding(
    pv: Dollar,
    rate: Rate,
    time: Decimal,
) -> Result<Dollar, CalculationError> {
    if time < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "continuous_compounding - time",
            value: time,
        });
    }
    let exponent = rate.checked_mul(time).ok_or(CalculationError::Overflow {
        formula: "continuous_compounding",
    })?;
    let factor = exponent.checked_exp().ok_or(CalculationError::Overflow {
        formula: "continuous_compounding",
    })?;
    pv.checked_mul(factor).ok_or(CalculationError::Overflow {
        formula: "continuous_compounding",
    })
}

/// Property-based invariant checks for the time-value-of-money formulas.
#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items)]
mod proptests {
    use super::{Decimal, future_value, present_value};
    use proptest::prelude::*;
    use rust_decimal_macros::dec;

    proptest! {
        #[test]
        fn present_value_is_inverse_of_future_value(
            pv in 0.0f64..1_000_000.0,
            rate in 0.0f64..0.5,
            periods in 1u32..50,
        ) {
            let pv_dec = Decimal::from_f64_retain(pv).unwrap();
            let rate_dec = Decimal::from_f64_retain(rate).unwrap();
            let fv = future_value(pv_dec, rate_dec, periods).unwrap();
            let recovered_pv = present_value(fv, rate_dec, periods).unwrap();
            let diff = (recovered_pv - pv_dec).abs();
            prop_assert!(diff < dec!(0.01));
        }
    }
}
