//! A single economic scenario ([`Universe`]) and its computed outputs ([`UniverseMetrics`]).

use casiros_core::error::CalculationError;
use casiros_core::types::{Dollar, Rate, Ratio};
use casiros_core::{corporate, financial, markets};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A single economic scenario: every raw input needed to compute a
/// [`UniverseMetrics`], grouped by category. `Universe` carries only raw
/// inputs — no derived or pre-computed values — so that every metric in
/// [`UniverseMetrics`] can be traced to exactly one `casiros_core` formula call.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Universe {
    // --- Macroeconomic ---
    /// The risk-free rate for this scenario.
    #[schema(value_type = Decimal)]
    pub risk_free_rate: Rate,
    /// The inflation rate for this scenario.
    #[schema(value_type = Decimal)]
    pub inflation_rate: Rate,
    /// The broad market's expected return for this scenario.
    #[schema(value_type = Decimal)]
    pub market_return: Rate,
    /// The company's own portfolio/stock return for this scenario, used for Sharpe.
    #[schema(value_type = Decimal)]
    pub portfolio_return: Rate,
    /// The standard deviation of the company's returns for this scenario.
    pub return_std_dev: Decimal,

    // --- Company-specific ---
    /// Total revenue for the period.
    #[schema(value_type = Decimal)]
    pub revenue: Dollar,
    /// Cost of goods sold for the period.
    #[schema(value_type = Decimal)]
    pub cogs: Dollar,
    /// Operating expenses for the period.
    #[schema(value_type = Decimal)]
    pub operating_expenses: Dollar,
    /// Interest expense for the period.
    #[schema(value_type = Decimal)]
    pub interest_expense: Dollar,
    /// The effective tax rate, in `[0, 1]`.
    #[schema(value_type = Decimal)]
    pub tax_rate: Ratio,
    /// The company's equity beta.
    pub beta: Decimal,
    /// The scenario's assumed cost of equity (e.g. CAPM-derived upstream of this crate).
    #[schema(value_type = Decimal)]
    pub cost_of_equity: Rate,
    /// The scenario's assumed pre-tax cost of debt.
    #[schema(value_type = Decimal)]
    pub cost_of_debt: Rate,

    // --- Balance sheet ---
    /// Total assets.
    #[schema(value_type = Decimal)]
    pub total_assets: Dollar,
    /// Current assets.
    #[schema(value_type = Decimal)]
    pub current_assets: Dollar,
    /// Inventory (a subset of current assets).
    #[schema(value_type = Decimal)]
    pub inventory: Dollar,
    /// Current liabilities.
    #[schema(value_type = Decimal)]
    pub current_liabilities: Dollar,
    /// Total liabilities.
    #[schema(value_type = Decimal)]
    pub total_liabilities: Dollar,
    /// Total shareholders' equity.
    #[schema(value_type = Decimal)]
    pub total_equity: Dollar,

    // --- Market ---
    /// The market share price.
    #[schema(value_type = Decimal)]
    pub share_price: Dollar,
    /// The number of shares outstanding.
    pub shares_outstanding: Decimal,
}

/// The computed outputs for a single [`Universe`]. Every field is produced by
/// exactly one `casiros_core` function call in [`compute_universe_metrics`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct UniverseMetrics {
    /// Earnings before interest and taxes (`revenue - cogs - operating_expenses`).
    #[schema(value_type = Decimal)]
    pub ebit: Dollar,
    /// Net income after interest and tax.
    #[schema(value_type = Decimal)]
    pub net_income: Dollar,
    /// [`financial::profit_margin`] of `net_income` over `revenue`.
    #[schema(value_type = Decimal)]
    pub profit_margin: Ratio,
    /// [`financial::return_on_equity`].
    #[schema(value_type = Decimal)]
    pub return_on_equity: Ratio,
    /// [`financial::return_on_assets`].
    #[schema(value_type = Decimal)]
    pub return_on_assets: Ratio,
    /// [`financial::current_ratio`].
    #[schema(value_type = Decimal)]
    pub current_ratio: Ratio,
    /// [`financial::quick_ratio`].
    #[schema(value_type = Decimal)]
    pub quick_ratio: Ratio,
    /// [`financial::debt_to_equity`].
    #[schema(value_type = Decimal)]
    pub debt_to_equity: Ratio,
    /// [`financial::interest_coverage`].
    #[schema(value_type = Decimal)]
    pub interest_coverage: Ratio,
    /// [`financial::asset_turnover`].
    #[schema(value_type = Decimal)]
    pub asset_turnover: Ratio,
    /// [`corporate::wacc`], using market capitalization as the equity value.
    #[schema(value_type = Decimal)]
    pub wacc: Rate,
    /// [`markets::sharpe_ratio`] of the scenario's portfolio return.
    pub sharpe_ratio: Decimal,
}

/// Computes `ebit` and `net_income` from a universe's raw income-statement inputs.
///
/// This is ordinary line-item arithmetic (sums and differences of the inputs
/// that *define* EBIT and net income), not a named formula — it has no
/// `casiros_core` equivalent to delegate to.
fn income_statement(universe: &Universe) -> Result<(Dollar, Dollar), CalculationError> {
    let formula = "universe::income_statement";
    let ebit = universe
        .revenue
        .checked_sub(universe.cogs)
        .and_then(|v| v.checked_sub(universe.operating_expenses))
        .ok_or(CalculationError::Overflow { formula })?;
    let pretax_income = ebit
        .checked_sub(universe.interest_expense)
        .ok_or(CalculationError::Overflow { formula })?;
    let retained_fraction = Decimal::ONE
        .checked_sub(universe.tax_rate)
        .ok_or(CalculationError::Overflow { formula })?;
    let net_income = pretax_income
        .checked_mul(retained_fraction)
        .ok_or(CalculationError::Overflow { formula })?;
    Ok((ebit, net_income))
}

/// Computes every [`UniverseMetrics`] field for `universe` by calling the
/// corresponding `casiros_core` formula.
///
/// # Errors
///
/// Returns whatever error the first failing `casiros_core` call produces
/// (e.g. [`CalculationError::DivisionByZero`] if a scenario perturbed a
/// denominator, such as `total_equity`, to zero).
pub fn compute_universe_metrics(universe: &Universe) -> Result<UniverseMetrics, CalculationError> {
    let (ebit, net_income) = income_statement(universe)?;
    let market_capitalization = universe
        .share_price
        .checked_mul(universe.shares_outstanding)
        .ok_or(CalculationError::Overflow {
            formula: "universe::compute_universe_metrics",
        })?;

    Ok(UniverseMetrics {
        ebit,
        net_income,
        profit_margin: financial::profit_margin(net_income, universe.revenue)?,
        return_on_equity: financial::return_on_equity(net_income, universe.total_equity)?,
        return_on_assets: financial::return_on_assets(net_income, universe.total_assets)?,
        current_ratio: financial::current_ratio(
            universe.current_assets,
            universe.current_liabilities,
        )?,
        quick_ratio: financial::quick_ratio(
            universe.current_assets,
            universe.inventory,
            universe.current_liabilities,
        )?,
        debt_to_equity: financial::debt_to_equity(
            universe.total_liabilities,
            universe.total_equity,
        )?,
        interest_coverage: financial::interest_coverage(ebit, universe.interest_expense)?,
        asset_turnover: financial::asset_turnover(universe.revenue, universe.total_assets)?,
        wacc: corporate::wacc(
            market_capitalization,
            universe.total_liabilities,
            universe.cost_of_equity,
            universe.cost_of_debt,
            universe.tax_rate,
        )?,
        sharpe_ratio: markets::sharpe_ratio(
            universe.portfolio_return,
            universe.risk_free_rate,
            universe.return_std_dev,
        )?,
    })
}
