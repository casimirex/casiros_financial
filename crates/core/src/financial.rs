//! Financial ratio formulas.

use crate::error::CalculationError;
use crate::types::{Dollar, Ratio};
use rust_decimal::Decimal;

/// Return on Equity: net income generated per dollar of shareholder equity.
///
/// # Mathematical Definition
///
/// \[ ROE = \frac{\text{Net Income}}{\text{Average Shareholders' Equity}} \]
///
/// # Constraints
///
/// - `avg_shareholders_equity` MUST be strictly positive.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `avg_shareholders_equity` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if `avg_shareholders_equity` is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::financial::return_on_equity;
/// use rust_decimal_macros::dec;
///
/// let roe = return_on_equity(dec!(150_000.0), dec!(1_000_000.0)).unwrap();
/// assert_eq!(roe, dec!(0.15));
/// assert!(roe > dec!(0.0));
/// ```
pub fn return_on_equity(
    net_income: Dollar,
    avg_shareholders_equity: Dollar,
) -> Result<Ratio, CalculationError> {
    require_positive(
        avg_shareholders_equity,
        "return_on_equity - avg_shareholders_equity",
    )?;
    checked_div(net_income, avg_shareholders_equity, "return_on_equity")
}

/// Return on Assets: net income generated per dollar of total assets.
///
/// # Mathematical Definition
///
/// \[ ROA = \frac{\text{Net Income}}{\text{Average Total Assets}} \]
///
/// # Constraints
///
/// - `avg_total_assets` MUST be strictly positive.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `avg_total_assets` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if `avg_total_assets` is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::financial::return_on_assets;
/// use rust_decimal_macros::dec;
///
/// let roa = return_on_assets(dec!(80_000.0), dec!(2_000_000.0)).unwrap();
/// assert_eq!(roa, dec!(0.04));
/// assert!(roa < dec!(1.0));
/// ```
pub fn return_on_assets(
    net_income: Dollar,
    avg_total_assets: Dollar,
) -> Result<Ratio, CalculationError> {
    require_positive(avg_total_assets, "return_on_assets - avg_total_assets")?;
    checked_div(net_income, avg_total_assets, "return_on_assets")
}

/// Return on Investment: gain relative to the cost of the investment.
///
/// # Mathematical Definition
///
/// \[ ROI = \frac{\text{Current Value} - \text{Cost}}{\text{Cost}} \]
///
/// # Constraints
///
/// - `cost` MUST be strictly positive.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `cost` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if `cost` is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::financial::return_on_investment;
/// use rust_decimal_macros::dec;
///
/// let roi = return_on_investment(dec!(1_200.0), dec!(1_000.0)).unwrap();
/// assert_eq!(roi, dec!(0.2));
/// assert!(roi > dec!(0.0));
/// ```
pub fn return_on_investment(
    current_value: Dollar,
    cost: Dollar,
) -> Result<Ratio, CalculationError> {
    require_positive(cost, "return_on_investment - cost")?;
    let gain = current_value
        .checked_sub(cost)
        .ok_or(CalculationError::Overflow {
            formula: "return_on_investment",
        })?;
    checked_div(gain, cost, "return_on_investment")
}

/// Profit Margin: net income generated per dollar of revenue.
///
/// # Mathematical Definition
///
/// \[ \text{Profit Margin} = \frac{\text{Net Income}}{\text{Revenue}} \]
///
/// # Constraints
///
/// - `revenue` MUST be strictly positive.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `revenue` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if `revenue` is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::financial::profit_margin;
/// use rust_decimal_macros::dec;
///
/// let margin = profit_margin(dec!(50_000.0), dec!(500_000.0)).unwrap();
/// assert_eq!(margin, dec!(0.1));
/// assert!(margin < dec!(1.0));
/// ```
pub fn profit_margin(net_income: Dollar, revenue: Dollar) -> Result<Ratio, CalculationError> {
    require_positive(revenue, "profit_margin - revenue")?;
    checked_div(net_income, revenue, "profit_margin")
}

/// Asset Turnover: revenue generated per dollar of assets deployed.
///
/// # Mathematical Definition
///
/// \[ \text{Asset Turnover} = \frac{\text{Net Sales}}{\text{Average Total Assets}} \]
///
/// # Constraints
///
/// - `avg_total_assets` MUST be strictly positive.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `avg_total_assets` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if `avg_total_assets` is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::financial::asset_turnover;
/// use rust_decimal_macros::dec;
///
/// let turnover = asset_turnover(dec!(1_000_000.0), dec!(500_000.0)).unwrap();
/// assert_eq!(turnover, dec!(2.0));
/// assert!(turnover > dec!(1.0));
/// ```
pub fn asset_turnover(
    net_sales: Dollar,
    avg_total_assets: Dollar,
) -> Result<Ratio, CalculationError> {
    require_positive(avg_total_assets, "asset_turnover - avg_total_assets")?;
    checked_div(net_sales, avg_total_assets, "asset_turnover")
}

/// Equity Multiplier: financial leverage expressed as assets per dollar of equity.
///
/// # Mathematical Definition
///
/// \[ \text{Equity Multiplier} = \frac{\text{Total Assets}}{\text{Total Equity}} \]
///
/// # Constraints
///
/// - `total_equity` MUST be strictly positive.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `total_equity` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if `total_equity` is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::financial::equity_multiplier;
/// use rust_decimal_macros::dec;
///
/// let multiplier = equity_multiplier(dec!(2_000_000.0), dec!(500_000.0)).unwrap();
/// assert_eq!(multiplier, dec!(4.0));
/// assert!(multiplier >= dec!(1.0));
/// ```
pub fn equity_multiplier(
    total_assets: Dollar,
    total_equity: Dollar,
) -> Result<Ratio, CalculationError> {
    require_positive(total_equity, "equity_multiplier - total_equity")?;
    checked_div(total_assets, total_equity, "equity_multiplier")
}

/// Return on Equity decomposed via the DuPont identity.
///
/// # Mathematical Definition
///
/// \[ ROE = \text{Net Margin} \times \text{Asset Turnover} \times \text{Equity Multiplier} \]
///
/// # Constraints
///
/// - `asset_turnover` MUST be non-negative.
/// - `equity_multiplier` MUST be strictly positive.
///
/// # Errors
///
/// Returns [`CalculationError::NegativeValueInvalid`] if `asset_turnover` is negative
/// or `equity_multiplier` is not strictly positive.
///
/// # Examples
///
/// ```
/// use casiros_core::financial::dupont_roe;
/// use rust_decimal_macros::dec;
///
/// let roe = dupont_roe(dec!(0.1), dec!(2.0), dec!(4.0)).unwrap();
/// assert_eq!(roe, dec!(0.8));
/// assert!(roe > dec!(0.0));
/// ```
pub fn dupont_roe(
    net_margin: Ratio,
    asset_turnover: Ratio,
    equity_multiplier: Ratio,
) -> Result<Ratio, CalculationError> {
    if asset_turnover < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "dupont_roe - asset_turnover",
            value: asset_turnover,
        });
    }
    require_positive(equity_multiplier, "dupont_roe - equity_multiplier")?;
    net_margin
        .checked_mul(asset_turnover)
        .and_then(|v| v.checked_mul(equity_multiplier))
        .ok_or(CalculationError::Overflow {
            formula: "dupont_roe",
        })
}

/// Current Ratio: short-term liquidity coverage.
///
/// # Mathematical Definition
///
/// \[ \text{Current Ratio} = \frac{\text{Current Assets}}{\text{Current Liabilities}} \]
///
/// # Constraints
///
/// - `current_liabilities` MUST be strictly positive.
/// - `current_assets` MUST be non-negative.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `current_liabilities` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if either input is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::financial::current_ratio;
/// use rust_decimal_macros::dec;
///
/// let ratio = current_ratio(dec!(300_000.0), dec!(150_000.0)).unwrap();
/// assert_eq!(ratio, dec!(2.0));
/// assert!(ratio > dec!(1.0));
/// ```
pub fn current_ratio(
    current_assets: Dollar,
    current_liabilities: Dollar,
) -> Result<Ratio, CalculationError> {
    if current_assets < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "current_ratio - current_assets",
            value: current_assets,
        });
    }
    require_positive(current_liabilities, "current_ratio - current_liabilities")?;
    checked_div(current_assets, current_liabilities, "current_ratio")
}

/// Quick Ratio (Acid-Test): short-term liquidity excluding inventory.
///
/// # Mathematical Definition
///
/// \[ \text{Quick Ratio} = \frac{\text{Current Assets} - \text{Inventory}}{\text{Current Liabilities}} \]
///
/// # Constraints
///
/// - `current_liabilities` MUST be strictly positive.
/// - `current_assets` and `inventory` MUST be non-negative.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `current_liabilities` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if any input is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::financial::quick_ratio;
/// use rust_decimal_macros::dec;
///
/// let ratio = quick_ratio(dec!(300_000.0), dec!(100_000.0), dec!(100_000.0)).unwrap();
/// assert_eq!(ratio, dec!(2.0));
/// assert!(ratio < dec!(3.0));
/// ```
pub fn quick_ratio(
    current_assets: Dollar,
    inventory: Dollar,
    current_liabilities: Dollar,
) -> Result<Ratio, CalculationError> {
    if current_assets < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "quick_ratio - current_assets",
            value: current_assets,
        });
    }
    if inventory < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "quick_ratio - inventory",
            value: inventory,
        });
    }
    require_positive(current_liabilities, "quick_ratio - current_liabilities")?;
    let liquid_assets =
        current_assets
            .checked_sub(inventory)
            .ok_or(CalculationError::Overflow {
                formula: "quick_ratio",
            })?;
    checked_div(liquid_assets, current_liabilities, "quick_ratio")
}

/// Debt-to-Equity Ratio: financial leverage relative to shareholder equity.
///
/// # Mathematical Definition
///
/// \[ D/E = \frac{\text{Total Liabilities}}{\text{Total Shareholders' Equity}} \]
///
/// # Constraints
///
/// - `total_equity` MUST NOT be zero.
/// - `total_liabilities` MUST be non-negative.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `total_equity` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if `total_liabilities` is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::financial::debt_to_equity;
/// use rust_decimal_macros::dec;
///
/// let de = debt_to_equity(dec!(400_000.0), dec!(500_000.0)).unwrap();
/// assert_eq!(de, dec!(0.8));
/// assert!(de < dec!(1.0));
/// ```
pub fn debt_to_equity(
    total_liabilities: Dollar,
    total_equity: Dollar,
) -> Result<Ratio, CalculationError> {
    if total_liabilities < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "debt_to_equity - total_liabilities",
            value: total_liabilities,
        });
    }
    if total_equity.is_zero() {
        return Err(CalculationError::DivisionByZero {
            formula: "debt_to_equity",
        });
    }
    checked_div(total_liabilities, total_equity, "debt_to_equity")
}

/// Interest Coverage Ratio: ability to service debt from operating earnings.
///
/// # Mathematical Definition
///
/// \[ \text{Interest Coverage} = \frac{\text{EBIT}}{\text{Interest Expense}} \]
///
/// # Constraints
///
/// - `interest_expense` MUST be strictly positive.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `interest_expense` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if `interest_expense` is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::financial::interest_coverage;
/// use rust_decimal_macros::dec;
///
/// let coverage = interest_coverage(dec!(500_000.0), dec!(100_000.0)).unwrap();
/// assert_eq!(coverage, dec!(5.0));
/// assert!(coverage > dec!(1.0));
/// ```
pub fn interest_coverage(
    ebit: Dollar,
    interest_expense: Dollar,
) -> Result<Ratio, CalculationError> {
    require_positive(interest_expense, "interest_coverage - interest_expense")?;
    checked_div(ebit, interest_expense, "interest_coverage")
}

/// Inventory Turnover: how many times inventory is sold and replaced over a period.
///
/// # Mathematical Definition
///
/// \[ \text{Inventory Turnover} = \frac{\text{COGS}}{\text{Average Inventory}} \]
///
/// # Constraints
///
/// - `avg_inventory` MUST be strictly positive.
/// - `cogs` MUST be non-negative.
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `avg_inventory` is zero.
/// Returns [`CalculationError::NegativeValueInvalid`] if either input is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::financial::inventory_turnover;
/// use rust_decimal_macros::dec;
///
/// let turnover = inventory_turnover(dec!(600_000.0), dec!(150_000.0)).unwrap();
/// assert_eq!(turnover, dec!(4.0));
/// assert!(turnover > dec!(0.0));
/// ```
pub fn inventory_turnover(cogs: Dollar, avg_inventory: Dollar) -> Result<Ratio, CalculationError> {
    if cogs < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "inventory_turnover - cogs",
            value: cogs,
        });
    }
    require_positive(avg_inventory, "inventory_turnover - avg_inventory")?;
    checked_div(cogs, avg_inventory, "inventory_turnover")
}

/// Cash Conversion Cycle: days between cash outlay for inventory and cash receipt from sales.
///
/// # Mathematical Definition
///
/// \[ CCC = DIO + DSO - DPO \]
///
/// # Constraints
///
/// - `days_inventory_outstanding`, `days_sales_outstanding`, and `days_payable_outstanding`
///   MUST each be non-negative.
///
/// # Errors
///
/// Returns [`CalculationError::NegativeValueInvalid`] if any input is negative.
///
/// # Examples
///
/// ```
/// use casiros_core::financial::cash_conversion_cycle;
/// use rust_decimal_macros::dec;
///
/// let ccc = cash_conversion_cycle(dec!(60.0), dec!(45.0), dec!(30.0)).unwrap();
/// assert_eq!(ccc, dec!(75.0));
/// assert!(ccc > dec!(0.0));
/// ```
pub fn cash_conversion_cycle(
    days_inventory_outstanding: Decimal,
    days_sales_outstanding: Decimal,
    days_payable_outstanding: Decimal,
) -> Result<Decimal, CalculationError> {
    if days_inventory_outstanding < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "cash_conversion_cycle - days_inventory_outstanding",
            value: days_inventory_outstanding,
        });
    }
    if days_sales_outstanding < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "cash_conversion_cycle - days_sales_outstanding",
            value: days_sales_outstanding,
        });
    }
    if days_payable_outstanding < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "cash_conversion_cycle - days_payable_outstanding",
            value: days_payable_outstanding,
        });
    }
    days_inventory_outstanding
        .checked_add(days_sales_outstanding)
        .and_then(|v| v.checked_sub(days_payable_outstanding))
        .ok_or(CalculationError::Overflow {
            formula: "cash_conversion_cycle",
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

/// Property-based invariant checks for the liquidity ratios.
#[cfg(test)]
#[allow(clippy::missing_docs_in_private_items)]
mod proptests {
    use super::{Decimal, current_ratio, quick_ratio};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn quick_ratio_never_exceeds_current_ratio(
            assets in 1.0f64..10_000_000.0,
            inventory in 0.0f64..1_000_000.0,
            liabilities in 1.0f64..10_000_000.0,
        ) {
            let assets_dec = Decimal::from_f64_retain(assets).unwrap();
            let inventory_dec = Decimal::from_f64_retain(inventory).unwrap();
            let liabilities_dec = Decimal::from_f64_retain(liabilities).unwrap();
            let current = current_ratio(assets_dec, liabilities_dec).unwrap();
            let quick = quick_ratio(assets_dec, inventory_dec, liabilities_dec).unwrap();
            prop_assert!(quick <= current);
        }
    }
}
