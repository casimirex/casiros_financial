//! Equity & fixed income formulas.

use crate::error::CalculationError;
use crate::types::{Dollar, Periods, Rate};
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use rust_decimal_macros::dec;

/// Maximum number of Newton-Raphson iterations attempted by [`yield_to_maturity`].
const MAX_NEWTON_ITERATIONS: u32 = 100;
/// Step size used for central-difference derivative estimation in Newton-Raphson.
const NEWTON_STEP: Decimal = dec!(0.0001);
/// Twice [`NEWTON_STEP`], precomputed to avoid a repeated runtime multiplication.
const NEWTON_STEP_DOUBLE: Decimal = dec!(0.0002);
/// Absolute price-difference tolerance for Newton-Raphson convergence.
const NEWTON_TOLERANCE: Decimal = dec!(0.00000001);
/// Floor applied to candidate yields so [`bond_price`] never sees an invalid rate.
const YIELD_FLOOR: Decimal = dec!(-0.99);

/// Present Value of a growing dividend stream (Gordon Growth Model).
///
/// # Mathematical Definition
///
/// \[ PV = \frac{D_1}{r - g} \]
///
/// # Constraints
///
/// - `required_return` MUST be strictly greater than `growth_rate`.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `required_return == growth_rate`.
/// Returns [`CalculationError::RangeViolation`] if `required_return < growth_rate`.
///
/// # Examples
///
/// ```
/// use casiros_core::stocks_bonds::dividend_discount_model;
/// use rust_decimal_macros::dec;
///
/// let pv = dividend_discount_model(dec!(2.10), dec!(0.10), dec!(0.05)).unwrap();
/// assert_eq!(pv, dec!(42.0));
/// assert!(pv > dec!(0.0));
/// ```
pub fn dividend_discount_model(
    next_dividend: Dollar,
    required_return: Rate,
    growth_rate: Rate,
) -> Result<Dollar, CalculationError> {
    if required_return == growth_rate {
        return Err(CalculationError::DivisionByZero {
            formula: "dividend_discount_model",
        });
    }
    if required_return < growth_rate {
        return Err(CalculationError::RangeViolation {
            context: "dividend_discount_model - required_return must exceed growth_rate",
            value: required_return,
        });
    }
    let denom = required_return
        .checked_sub(growth_rate)
        .ok_or(CalculationError::Overflow {
            formula: "dividend_discount_model",
        })?;
    next_dividend
        .checked_div(denom)
        .ok_or(CalculationError::Overflow {
            formula: "dividend_discount_model",
        })
}

/// Present Value of a series of future cash flows (Discounted Cash Flow).
///
/// `cash_flows[i]` is treated as occurring at period `i + 1`.
///
/// # Mathematical Definition
///
/// \[ PV = \sum_{t=1}^{n} \frac{CF_t}{(1 + r)^t} \]
///
/// # Constraints
///
/// - `cash_flows` MUST be non-empty.
/// - `rate` MUST be greater than -1.0.
///
/// # Errors
///
/// Returns [`CalculationError::MissingInput`] if `cash_flows` is empty.
/// Returns [`CalculationError::InvalidRate`] if `rate <= -1.0`.
///
/// # Examples
///
/// ```
/// use casiros_core::stocks_bonds::discounted_cash_flow;
/// use casiros_core::general::present_value;
/// use rust_decimal_macros::dec;
///
/// let dcf = discounted_cash_flow(&[dec!(0.0), dec!(0.0), dec!(1000.0)], dec!(0.10)).unwrap();
/// let pv = present_value(dec!(1000.0), dec!(0.10), 3).unwrap();
/// assert_eq!(dcf, pv);
/// assert!(dcf > dec!(0.0));
/// ```
pub fn discounted_cash_flow(cash_flows: &[Dollar], rate: Rate) -> Result<Dollar, CalculationError> {
    if cash_flows.is_empty() {
        return Err(CalculationError::MissingInput {
            formula: "discounted_cash_flow",
            parameter: "cash_flows",
        });
    }
    if rate <= Decimal::NEGATIVE_ONE {
        return Err(CalculationError::InvalidRate { rate });
    }
    let base = Decimal::ONE
        .checked_add(rate)
        .ok_or(CalculationError::Overflow {
            formula: "discounted_cash_flow",
        })?;
    let mut total = Decimal::ZERO;
    for (index, cash_flow) in cash_flows.iter().enumerate() {
        let t = u64::try_from(index + 1).map_err(|_| CalculationError::Overflow {
            formula: "discounted_cash_flow",
        })?;
        let factor = base.checked_powu(t).ok_or(CalculationError::Overflow {
            formula: "discounted_cash_flow",
        })?;
        let discounted = cash_flow
            .checked_div(factor)
            .ok_or(CalculationError::Overflow {
                formula: "discounted_cash_flow",
            })?;
        total = total
            .checked_add(discounted)
            .ok_or(CalculationError::Overflow {
                formula: "discounted_cash_flow",
            })?;
    }
    Ok(total)
}

/// Present Value of a coupon bond's remaining cash flows.
///
/// # Mathematical Definition
///
/// \[ P = \sum_{t=1}^{n} \frac{C}{(1 + y)^t} + \frac{F}{(1 + y)^n}, \quad C = F \times \text{coupon rate} \]
///
/// # Constraints
///
/// - `market_rate` MUST be greater than -1.0.
/// - When `periods` is zero, the bond is treated as already matured and `face_value` is returned.
///
/// # Errors
///
/// Returns [`CalculationError::InvalidRate`] if `market_rate <= -1.0`.
/// Returns [`CalculationError::Overflow`] if any intermediate step overflows.
///
/// # Examples
///
/// ```
/// use casiros_core::stocks_bonds::bond_price;
/// use rust_decimal_macros::dec;
///
/// // A bond priced at par: coupon rate equals the market rate.
/// let price = bond_price(dec!(1000.0), dec!(0.05), dec!(0.05), 10).unwrap();
/// assert!((price - dec!(1000.0)).abs() < dec!(0.0001));
/// assert!(price > dec!(0.0));
/// ```
pub fn bond_price(
    face_value: Dollar,
    coupon_rate: Rate,
    market_rate: Rate,
    periods: Periods,
) -> Result<Dollar, CalculationError> {
    if periods == 0 {
        return Ok(face_value);
    }
    if market_rate <= Decimal::NEGATIVE_ONE {
        return Err(CalculationError::InvalidRate { rate: market_rate });
    }
    let coupon = face_value
        .checked_mul(coupon_rate)
        .ok_or(CalculationError::Overflow {
            formula: "bond_price",
        })?;
    let base = Decimal::ONE
        .checked_add(market_rate)
        .ok_or(CalculationError::Overflow {
            formula: "bond_price",
        })?;
    let mut total = Decimal::ZERO;
    for t in 1..=periods {
        let factor = base
            .checked_powu(u64::from(t))
            .ok_or(CalculationError::Overflow {
                formula: "bond_price",
            })?;
        let mut cash_flow = coupon;
        if t == periods {
            cash_flow = cash_flow
                .checked_add(face_value)
                .ok_or(CalculationError::Overflow {
                    formula: "bond_price",
                })?;
        }
        let discounted = cash_flow
            .checked_div(factor)
            .ok_or(CalculationError::Overflow {
                formula: "bond_price",
            })?;
        total = total
            .checked_add(discounted)
            .ok_or(CalculationError::Overflow {
                formula: "bond_price",
            })?;
    }
    Ok(total)
}

/// Central-difference estimate of `d(bond_price)/dy` at yield `y`, used by
/// [`yield_to_maturity`]'s Newton-Raphson iteration.
fn bond_price_derivative(
    face_value: Dollar,
    coupon_rate: Rate,
    y: Rate,
    periods: Periods,
) -> Result<Decimal, CalculationError> {
    let y_plus = y
        .checked_add(NEWTON_STEP)
        .ok_or(CalculationError::Overflow {
            formula: "yield_to_maturity",
        })?;
    let y_minus = y
        .checked_sub(NEWTON_STEP)
        .ok_or(CalculationError::Overflow {
            formula: "yield_to_maturity",
        })?;
    let plus = bond_price(face_value, coupon_rate, y_plus, periods)?;
    let minus = bond_price(face_value, coupon_rate, y_minus, periods)?;
    plus.checked_sub(minus)
        .and_then(|d| d.checked_div(NEWTON_STEP_DOUBLE))
        .ok_or(CalculationError::Overflow {
            formula: "yield_to_maturity",
        })
}

/// Yield to Maturity: the discount rate that equates a bond's cash flows to its price.
///
/// Solved via Newton-Raphson, using the standard approximation formula as the
/// initial guess and a central-difference numerical derivative of [`bond_price`].
///
/// # Mathematical Definition
///
/// \[ P = \sum_{t=1}^{n} \frac{C}{(1 + y)^t} + \frac{F}{(1 + y)^n} \quad \text{solved for } y \]
///
/// # Constraints
///
/// - `periods` MUST be greater than zero.
/// - `price` and `face_value` MUST be strictly positive.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `periods` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if `price` or `face_value` is not positive.
/// Returns [`CalculationError::ConvergenceFailure`] if 100 iterations are exceeded.
///
/// # Examples
///
/// ```
/// use casiros_core::stocks_bonds::yield_to_maturity;
/// use rust_decimal_macros::dec;
///
/// // A bond priced at par: YTM must equal the coupon rate.
/// let ytm = yield_to_maturity(dec!(1000.0), dec!(1000.0), dec!(0.05), 10).unwrap();
/// assert!(ytm > dec!(0.04));
/// assert!(ytm < dec!(0.06));
/// ```
pub fn yield_to_maturity(
    price: Dollar,
    face_value: Dollar,
    coupon_rate: Rate,
    periods: Periods,
) -> Result<Rate, CalculationError> {
    if periods == 0 {
        return Err(CalculationError::DivisionByZero {
            formula: "yield_to_maturity",
        });
    }
    if price <= Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "yield_to_maturity - price",
            value: price,
        });
    }
    if face_value <= Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "yield_to_maturity - face_value",
            value: face_value,
        });
    }
    let n = Decimal::from(periods);
    let formula = "yield_to_maturity";
    let coupon = face_value
        .checked_mul(coupon_rate)
        .ok_or(CalculationError::Overflow { formula })?;
    let avg_price = face_value
        .checked_add(price)
        .and_then(|v| v.checked_div(dec!(2)))
        .ok_or(CalculationError::Overflow { formula })?;
    let redemption_gain = face_value
        .checked_sub(price)
        .and_then(|v| v.checked_div(n))
        .ok_or(CalculationError::Overflow { formula })?;
    let initial_guess = coupon
        .checked_add(redemption_gain)
        .and_then(|v| v.checked_div(avg_price))
        .ok_or(CalculationError::Overflow { formula })?;
    let mut y = initial_guess.max(YIELD_FLOOR);

    for iteration in 0..MAX_NEWTON_ITERATIONS {
        let modeled_price = bond_price(face_value, coupon_rate, y, periods)?;
        let f = modeled_price
            .checked_sub(price)
            .ok_or(CalculationError::Overflow { formula })?;
        if f.abs() < NEWTON_TOLERANCE {
            return Ok(y);
        }
        let derivative = bond_price_derivative(face_value, coupon_rate, y, periods)?;
        if derivative.is_zero() {
            return Err(CalculationError::ConvergenceFailure {
                formula,
                iterations: iteration,
            });
        }
        let step = f
            .checked_div(derivative)
            .ok_or(CalculationError::Overflow { formula })?;
        y = y
            .checked_sub(step)
            .ok_or(CalculationError::Overflow { formula })?
            .max(YIELD_FLOOR);
    }
    Err(CalculationError::ConvergenceFailure {
        formula,
        iterations: MAX_NEWTON_ITERATIONS,
    })
}

/// Macaulay Duration: the weighted-average time to receive a bond's cash flows.
///
/// `cash_flows[i]` is treated as occurring at period `i + 1`.
///
/// # Mathematical Definition
///
/// \[ D = \frac{\sum_{t=1}^{n} t \times \frac{CF_t}{(1 + r)^t}}{\sum_{t=1}^{n} \frac{CF_t}{(1 + r)^t}} \]
///
/// # Constraints
///
/// - `cash_flows` MUST be non-empty.
/// - `rate` MUST be greater than -1.0.
///
/// # Errors
///
/// Returns [`CalculationError::MissingInput`] if `cash_flows` is empty.
/// Returns [`CalculationError::InvalidRate`] if `rate <= -1.0`.
/// Returns [`CalculationError::DivisionByZero`] if the discounted cash flows sum to zero.
///
/// # Examples
///
/// ```
/// use casiros_core::stocks_bonds::duration;
/// use rust_decimal_macros::dec;
///
/// // A single cash flow at t=5: duration collapses exactly to 5.
/// let cash_flows = [dec!(0.0), dec!(0.0), dec!(0.0), dec!(0.0), dec!(1000.0)];
/// let dur = duration(&cash_flows, dec!(0.05)).unwrap();
/// assert_eq!(dur, dec!(5));
/// assert!(dur > dec!(0.0));
/// ```
pub fn duration(cash_flows: &[Dollar], rate: Rate) -> Result<Decimal, CalculationError> {
    if cash_flows.is_empty() {
        return Err(CalculationError::MissingInput {
            formula: "duration",
            parameter: "cash_flows",
        });
    }
    if rate <= Decimal::NEGATIVE_ONE {
        return Err(CalculationError::InvalidRate { rate });
    }
    let base = Decimal::ONE
        .checked_add(rate)
        .ok_or(CalculationError::Overflow {
            formula: "duration",
        })?;
    let mut weighted_sum = Decimal::ZERO;
    let mut price_sum = Decimal::ZERO;
    for (index, cash_flow) in cash_flows.iter().enumerate() {
        let t = u64::try_from(index + 1).map_err(|_| CalculationError::Overflow {
            formula: "duration",
        })?;
        let factor = base.checked_powu(t).ok_or(CalculationError::Overflow {
            formula: "duration",
        })?;
        let discounted = cash_flow
            .checked_div(factor)
            .ok_or(CalculationError::Overflow {
                formula: "duration",
            })?;
        let weighted =
            discounted
                .checked_mul(Decimal::from(t))
                .ok_or(CalculationError::Overflow {
                    formula: "duration",
                })?;
        weighted_sum = weighted_sum
            .checked_add(weighted)
            .ok_or(CalculationError::Overflow {
                formula: "duration",
            })?;
        price_sum = price_sum
            .checked_add(discounted)
            .ok_or(CalculationError::Overflow {
                formula: "duration",
            })?;
    }
    if price_sum.is_zero() {
        return Err(CalculationError::DivisionByZero {
            formula: "duration",
        });
    }
    weighted_sum
        .checked_div(price_sum)
        .ok_or(CalculationError::Overflow {
            formula: "duration",
        })
}

/// Modified Duration: price sensitivity of a bond to yield changes.
///
/// # Mathematical Definition
///
/// \[ D_{mod} = \frac{D_{mac}}{1 + \frac{y}{m}} \]
///
/// # Constraints
///
/// - `periods_per_year` MUST be greater than zero.
/// - `1 + ytm / periods_per_year` MUST NOT be zero.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `periods_per_year` is zero or
/// `1 + ytm / periods_per_year` is zero.
///
/// # Examples
///
/// ```
/// use casiros_core::stocks_bonds::modified_duration;
/// use rust_decimal_macros::dec;
///
/// let mod_dur = modified_duration(dec!(5.5), dec!(0.08), 2).unwrap();
/// assert!(mod_dur < dec!(5.5));
/// assert!(mod_dur > dec!(5.0));
/// ```
pub fn modified_duration(
    macaulay_duration: Decimal,
    ytm: Rate,
    periods_per_year: Periods,
) -> Result<Decimal, CalculationError> {
    if periods_per_year == 0 {
        return Err(CalculationError::DivisionByZero {
            formula: "modified_duration",
        });
    }
    let m = Decimal::from(periods_per_year);
    let periodic_yield = ytm.checked_div(m).ok_or(CalculationError::Overflow {
        formula: "modified_duration",
    })?;
    let denom = Decimal::ONE
        .checked_add(periodic_yield)
        .ok_or(CalculationError::Overflow {
            formula: "modified_duration",
        })?;
    if denom.is_zero() {
        return Err(CalculationError::DivisionByZero {
            formula: "modified_duration",
        });
    }
    macaulay_duration
        .checked_div(denom)
        .ok_or(CalculationError::Overflow {
            formula: "modified_duration",
        })
}

/// One period's discounted cash flow and its weighted convexity contribution
/// (`CF_t / (1+r)^t` and the `t(t+1)`-weighted term from convexity's sum),
/// factored out of [`convexity`] to keep that function's body within the
/// project's 60-line limit.
fn convexity_term(
    cash_flow: Dollar,
    base: Decimal,
    square: Decimal,
    t: u64,
) -> Result<(Decimal, Decimal), CalculationError> {
    let factor = base.checked_powu(t).ok_or(CalculationError::Overflow {
        formula: "convexity",
    })?;
    let discounted = cash_flow
        .checked_div(factor)
        .ok_or(CalculationError::Overflow {
            formula: "convexity",
        })?;
    let weight =
        Decimal::from(t)
            .checked_mul(Decimal::from(t + 1))
            .ok_or(CalculationError::Overflow {
                formula: "convexity",
            })?;
    let term = discounted
        .checked_mul(weight)
        .and_then(|v| v.checked_div(square))
        .ok_or(CalculationError::Overflow {
            formula: "convexity",
        })?;
    Ok((discounted, term))
}

/// Convexity: the curvature of a bond's price-yield relationship.
///
/// `cash_flows[i]` is treated as occurring at period `i + 1`.
///
/// # Mathematical Definition
///
/// \[ C = \frac{1}{P} \sum_{t=1}^{n} \frac{CF_t \times t \times (t + 1)}{(1 + r)^{t + 2}}, \quad
/// P = \sum_{t=1}^{n} \frac{CF_t}{(1 + r)^t} \]
///
/// # Constraints
///
/// - `cash_flows` MUST be non-empty.
/// - `rate` MUST be greater than -1.0.
///
/// # Errors
///
/// Returns [`CalculationError::MissingInput`] if `cash_flows` is empty.
/// Returns [`CalculationError::InvalidRate`] if `rate <= -1.0`.
/// Returns [`CalculationError::DivisionByZero`] if the discounted cash flows sum to zero.
///
/// # Examples
///
/// ```
/// use casiros_core::stocks_bonds::convexity;
/// use rust_decimal_macros::dec;
///
/// // A single cash flow at t=5: convexity reduces to n(n+1)/(1+r)^2 = 30/1.1025.
/// let cash_flows = [dec!(0.0), dec!(0.0), dec!(0.0), dec!(0.0), dec!(1000.0)];
/// let cvx = convexity(&cash_flows, dec!(0.05)).unwrap();
/// assert!(cvx > dec!(27.0));
/// assert!(cvx < dec!(27.5));
/// ```
pub fn convexity(cash_flows: &[Dollar], rate: Rate) -> Result<Decimal, CalculationError> {
    if cash_flows.is_empty() {
        return Err(CalculationError::MissingInput {
            formula: "convexity",
            parameter: "cash_flows",
        });
    }
    if rate <= Decimal::NEGATIVE_ONE {
        return Err(CalculationError::InvalidRate { rate });
    }
    let base = Decimal::ONE
        .checked_add(rate)
        .ok_or(CalculationError::Overflow {
            formula: "convexity",
        })?;
    let square = base.checked_powu(2).ok_or(CalculationError::Overflow {
        formula: "convexity",
    })?;
    let mut convexity_sum = Decimal::ZERO;
    let mut price_sum = Decimal::ZERO;
    for (index, cash_flow) in cash_flows.iter().enumerate() {
        let t = u64::try_from(index + 1).map_err(|_| CalculationError::Overflow {
            formula: "convexity",
        })?;
        let (discounted, term) = convexity_term(*cash_flow, base, square, t)?;
        price_sum = price_sum
            .checked_add(discounted)
            .ok_or(CalculationError::Overflow {
                formula: "convexity",
            })?;
        convexity_sum = convexity_sum
            .checked_add(term)
            .ok_or(CalculationError::Overflow {
                formula: "convexity",
            })?;
    }
    if price_sum.is_zero() {
        return Err(CalculationError::DivisionByZero {
            formula: "convexity",
        });
    }
    convexity_sum
        .checked_div(price_sum)
        .ok_or(CalculationError::Overflow {
            formula: "convexity",
        })
}
