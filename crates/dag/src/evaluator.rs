//! Walks a [`CausalityEngine`]'s topological order, calling the appropriate
//! `casiros_core` function for each [`FormulaNode`] and recording its result.

use crate::graph::{CausalityEngine, FormulaNode};
use casiros_core::prelude::{CalculationError, Decimal, Periods};
use casiros_core::{banking, corporate, financial, general, markets, stocks_bonds};
use rust_decimal::prelude::ToPrimitive;
use std::collections::HashMap;

/// Holds the raw inputs and accumulated results for a single DAG evaluation pass.
#[derive(Debug, Default)]
pub struct EvaluationContext {
    /// Computed outputs, keyed by the node that produced them.
    pub results: HashMap<FormulaNode, Decimal>,
    /// Raw scalar inputs supplied by the caller, keyed by parameter name.
    pub inputs: HashMap<String, Decimal>,
    /// Raw cash-flow-series inputs, for the formulas ([`FormulaNode::DiscountedCashFlow`],
    /// [`FormulaNode::Duration`], [`FormulaNode::Convexity`]) that take a slice rather
    /// than a single scalar. Keyed by parameter name (conventionally `"cash_flows"`).
    pub series_inputs: HashMap<String, Vec<Decimal>>,
}

impl EvaluationContext {
    /// Creates an empty evaluation context.
    #[must_use]
    pub fn new() -> Self {
        Self {
            results: HashMap::new(),
            inputs: HashMap::new(),
            series_inputs: HashMap::new(),
        }
    }

    /// Looks up a prior result by the *name* of the [`FormulaNode`] that produced it.
    fn result_by_name(&self, name: &str) -> Option<Decimal> {
        self.results
            .iter()
            .find(|(node, _)| node.name() == name)
            .map(|(_, value)| *value)
    }
}

/// Resolves parameter `parameter` of `formula`: a prior node's result (matched by
/// name) takes precedence over a raw scalar input, matching the DAG's wiring
/// convention documented on [`FormulaNode::name`].
fn resolve(
    ctx: &EvaluationContext,
    formula: FormulaNode,
    parameter: &'static str,
) -> Result<Decimal, CalculationError> {
    if let Some(value) = ctx.result_by_name(parameter) {
        return Ok(value);
    }
    if let Some(value) = ctx.inputs.get(parameter) {
        return Ok(*value);
    }
    Err(CalculationError::MissingInput {
        formula: formula.name(),
        parameter,
    })
}

/// Resolves a parameter that must be a non-negative whole number of periods.
fn resolve_periods(
    ctx: &EvaluationContext,
    formula: FormulaNode,
    parameter: &'static str,
) -> Result<Periods, CalculationError> {
    let value = resolve(ctx, formula, parameter)?;
    if !value.fract().is_zero() || value.is_sign_negative() {
        return Err(CalculationError::RangeViolation {
            context: parameter,
            value,
        });
    }
    value.to_u32().ok_or(CalculationError::RangeViolation {
        context: parameter,
        value,
    })
}

/// Resolves a cash-flow-series parameter from [`EvaluationContext::series_inputs`].
fn resolve_series(
    ctx: &EvaluationContext,
    formula: FormulaNode,
    parameter: &'static str,
) -> Result<Vec<Decimal>, CalculationError> {
    ctx.series_inputs
        .get(parameter)
        .cloned()
        .ok_or(CalculationError::MissingInput {
            formula: formula.name(),
            parameter,
        })
}

// --- general: Time Value of Money -----------------------------------------

fn eval_future_value(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::FutureValue;
    let pv = resolve(ctx, node, "pv")?;
    let rate = resolve(ctx, node, "rate")?;
    let periods = resolve_periods(ctx, node, "periods")?;
    general::future_value(pv, rate, periods)
}

fn eval_present_value(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::PresentValue;
    let fv = resolve(ctx, node, "fv")?;
    let rate = resolve(ctx, node, "rate")?;
    let periods = resolve_periods(ctx, node, "periods")?;
    general::present_value(fv, rate, periods)
}

fn eval_annuity_future_value(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::AnnuityFutureValue;
    let pmt = resolve(ctx, node, "pmt")?;
    let rate = resolve(ctx, node, "rate")?;
    let periods = resolve_periods(ctx, node, "periods")?;
    general::annuity_future_value(pmt, rate, periods)
}

fn eval_annuity_present_value(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::AnnuityPresentValue;
    let pmt = resolve(ctx, node, "pmt")?;
    let rate = resolve(ctx, node, "rate")?;
    let periods = resolve_periods(ctx, node, "periods")?;
    general::annuity_present_value(pmt, rate, periods)
}

fn eval_perpetuity_present_value(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::PerpetuityPresentValue;
    let pmt = resolve(ctx, node, "pmt")?;
    let rate = resolve(ctx, node, "rate")?;
    general::perpetuity_present_value(pmt, rate)
}

fn eval_growing_perpetuity(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::GrowingPerpetuity;
    let d1 = resolve(ctx, node, "d1")?;
    let rate = resolve(ctx, node, "rate")?;
    let growth = resolve(ctx, node, "growth")?;
    general::growing_perpetuity(d1, rate, growth)
}

fn eval_effective_annual_rate(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::EffectiveAnnualRate;
    let nominal_rate = resolve(ctx, node, "nominal_rate")?;
    let compounding_periods = resolve_periods(ctx, node, "compounding_periods")?;
    general::effective_annual_rate(nominal_rate, compounding_periods)
}

fn eval_continuous_compounding(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::ContinuousCompounding;
    let pv = resolve(ctx, node, "pv")?;
    let rate = resolve(ctx, node, "rate")?;
    let time = resolve(ctx, node, "time")?;
    general::continuous_compounding(pv, rate, time)
}

// --- financial: Ratios ------------------------------------------------------

fn eval_return_on_equity(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::ReturnOnEquity;
    let net_income = resolve(ctx, node, "net_income")?;
    let avg_shareholders_equity = resolve(ctx, node, "avg_shareholders_equity")?;
    financial::return_on_equity(net_income, avg_shareholders_equity)
}

fn eval_return_on_assets(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::ReturnOnAssets;
    let net_income = resolve(ctx, node, "net_income")?;
    let avg_total_assets = resolve(ctx, node, "avg_total_assets")?;
    financial::return_on_assets(net_income, avg_total_assets)
}

fn eval_return_on_investment(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::ReturnOnInvestment;
    let current_value = resolve(ctx, node, "current_value")?;
    let cost = resolve(ctx, node, "cost")?;
    financial::return_on_investment(current_value, cost)
}

fn eval_profit_margin(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::ProfitMargin;
    let net_income = resolve(ctx, node, "net_income")?;
    let revenue = resolve(ctx, node, "revenue")?;
    financial::profit_margin(net_income, revenue)
}

fn eval_asset_turnover(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::AssetTurnover;
    let net_sales = resolve(ctx, node, "net_sales")?;
    let avg_total_assets = resolve(ctx, node, "avg_total_assets")?;
    financial::asset_turnover(net_sales, avg_total_assets)
}

fn eval_equity_multiplier(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::EquityMultiplier;
    let total_assets = resolve(ctx, node, "total_assets")?;
    let total_equity = resolve(ctx, node, "total_equity")?;
    financial::equity_multiplier(total_assets, total_equity)
}

fn eval_dupont_roe(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::DupontRoe;
    let net_margin = resolve(ctx, node, "net_margin")?;
    let asset_turnover = resolve(ctx, node, "asset_turnover")?;
    let equity_multiplier = resolve(ctx, node, "equity_multiplier")?;
    financial::dupont_roe(net_margin, asset_turnover, equity_multiplier)
}

fn eval_current_ratio(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::CurrentRatio;
    let current_assets = resolve(ctx, node, "current_assets")?;
    let current_liabilities = resolve(ctx, node, "current_liabilities")?;
    financial::current_ratio(current_assets, current_liabilities)
}

fn eval_quick_ratio(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::QuickRatio;
    let current_assets = resolve(ctx, node, "current_assets")?;
    let inventory = resolve(ctx, node, "inventory")?;
    let current_liabilities = resolve(ctx, node, "current_liabilities")?;
    financial::quick_ratio(current_assets, inventory, current_liabilities)
}

fn eval_debt_to_equity(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::DebtToEquity;
    let total_liabilities = resolve(ctx, node, "total_liabilities")?;
    let total_equity = resolve(ctx, node, "total_equity")?;
    financial::debt_to_equity(total_liabilities, total_equity)
}

fn eval_interest_coverage(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::InterestCoverage;
    let ebit = resolve(ctx, node, "ebit")?;
    let interest_expense = resolve(ctx, node, "interest_expense")?;
    financial::interest_coverage(ebit, interest_expense)
}

fn eval_inventory_turnover(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::InventoryTurnover;
    let cogs = resolve(ctx, node, "cogs")?;
    let avg_inventory = resolve(ctx, node, "avg_inventory")?;
    financial::inventory_turnover(cogs, avg_inventory)
}

fn eval_cash_conversion_cycle(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::CashConversionCycle;
    let inventory_days = resolve(ctx, node, "days_inventory_outstanding")?;
    let sales_days = resolve(ctx, node, "days_sales_outstanding")?;
    let payable_days = resolve(ctx, node, "days_payable_outstanding")?;
    financial::cash_conversion_cycle(inventory_days, sales_days, payable_days)
}

// --- banking -----------------------------------------------------------------

fn eval_net_interest_margin(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::NetInterestMargin;
    let net_interest_income = resolve(ctx, node, "net_interest_income")?;
    let avg_earning_assets = resolve(ctx, node, "avg_earning_assets")?;
    banking::net_interest_margin(net_interest_income, avg_earning_assets)
}

fn eval_loan_to_deposit_ratio(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::LoanToDepositRatio;
    let total_loans = resolve(ctx, node, "total_loans")?;
    let total_deposits = resolve(ctx, node, "total_deposits")?;
    banking::loan_to_deposit_ratio(total_loans, total_deposits)
}

fn eval_capital_adequacy_ratio(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::CapitalAdequacyRatio;
    let qualifying_capital = resolve(ctx, node, "qualifying_capital")?;
    let risk_weighted_assets = resolve(ctx, node, "risk_weighted_assets")?;
    banking::capital_adequacy_ratio(qualifying_capital, risk_weighted_assets)
}

fn eval_provision_coverage(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::ProvisionCoverage;
    let loan_loss_provisions = resolve(ctx, node, "loan_loss_provisions")?;
    let non_performing_loans = resolve(ctx, node, "non_performing_loans")?;
    banking::provision_coverage(loan_loss_provisions, non_performing_loans)
}

// --- markets -------------------------------------------------------------------

fn eval_beta(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::Beta;
    let covariance = resolve(ctx, node, "covariance")?;
    let variance_market = resolve(ctx, node, "variance_market")?;
    markets::beta(covariance, variance_market)
}

fn eval_sharpe_ratio(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::SharpeRatio;
    let portfolio_return = resolve(ctx, node, "portfolio_return")?;
    let risk_free_rate = resolve(ctx, node, "risk_free_rate")?;
    let std_dev = resolve(ctx, node, "std_dev")?;
    markets::sharpe_ratio(portfolio_return, risk_free_rate, std_dev)
}

fn eval_treynor_ratio(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::TreynorRatio;
    let portfolio_return = resolve(ctx, node, "portfolio_return")?;
    let risk_free_rate = resolve(ctx, node, "risk_free_rate")?;
    let portfolio_beta = resolve(ctx, node, "portfolio_beta")?;
    markets::treynor_ratio(portfolio_return, risk_free_rate, portfolio_beta)
}

fn eval_jensens_alpha(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::JensensAlpha;
    let portfolio_return = resolve(ctx, node, "portfolio_return")?;
    let risk_free_rate = resolve(ctx, node, "risk_free_rate")?;
    let portfolio_beta = resolve(ctx, node, "portfolio_beta")?;
    let market_return = resolve(ctx, node, "market_return")?;
    markets::jensens_alpha(
        portfolio_return,
        risk_free_rate,
        portfolio_beta,
        market_return,
    )
}

fn eval_value_at_risk(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::ValueAtRisk;
    let portfolio_value = resolve(ctx, node, "portfolio_value")?;
    let z_score = resolve(ctx, node, "z_score")?;
    let std_dev = resolve(ctx, node, "std_dev")?;
    markets::value_at_risk(portfolio_value, z_score, std_dev)
}

fn eval_expected_shortfall(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::ExpectedShortfall;
    let portfolio_value = resolve(ctx, node, "portfolio_value")?;
    let z_score = resolve(ctx, node, "z_score")?;
    let std_dev = resolve(ctx, node, "std_dev")?;
    let confidence = resolve(ctx, node, "confidence")?;
    markets::expected_shortfall(portfolio_value, z_score, std_dev, confidence)
}

// --- stocks_bonds ----------------------------------------------------------------

fn eval_dividend_discount_model(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::DividendDiscountModel;
    let next_dividend = resolve(ctx, node, "next_dividend")?;
    let required_return = resolve(ctx, node, "required_return")?;
    let growth_rate = resolve(ctx, node, "growth_rate")?;
    stocks_bonds::dividend_discount_model(next_dividend, required_return, growth_rate)
}

fn eval_discounted_cash_flow(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::DiscountedCashFlow;
    let cash_flows = resolve_series(ctx, node, "cash_flows")?;
    let rate = resolve(ctx, node, "rate")?;
    stocks_bonds::discounted_cash_flow(&cash_flows, rate)
}

fn eval_bond_price(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::BondPrice;
    let face_value = resolve(ctx, node, "face_value")?;
    let coupon_rate = resolve(ctx, node, "coupon_rate")?;
    let market_rate = resolve(ctx, node, "market_rate")?;
    let periods = resolve_periods(ctx, node, "periods")?;
    stocks_bonds::bond_price(face_value, coupon_rate, market_rate, periods)
}

fn eval_yield_to_maturity(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::YieldToMaturity;
    let price = resolve(ctx, node, "price")?;
    let face_value = resolve(ctx, node, "face_value")?;
    let coupon_rate = resolve(ctx, node, "coupon_rate")?;
    let periods = resolve_periods(ctx, node, "periods")?;
    stocks_bonds::yield_to_maturity(price, face_value, coupon_rate, periods)
}

fn eval_duration(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::Duration;
    let cash_flows = resolve_series(ctx, node, "cash_flows")?;
    let rate = resolve(ctx, node, "rate")?;
    stocks_bonds::duration(&cash_flows, rate)
}

fn eval_modified_duration(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::ModifiedDuration;
    let macaulay_duration = resolve(ctx, node, "macaulay_duration")?;
    let ytm = resolve(ctx, node, "ytm")?;
    let periods_per_year = resolve_periods(ctx, node, "periods_per_year")?;
    stocks_bonds::modified_duration(macaulay_duration, ytm, periods_per_year)
}

fn eval_convexity(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::Convexity;
    let cash_flows = resolve_series(ctx, node, "cash_flows")?;
    let rate = resolve(ctx, node, "rate")?;
    stocks_bonds::convexity(&cash_flows, rate)
}

// --- corporate -------------------------------------------------------------------

fn eval_wacc(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::Wacc;
    let equity_value = resolve(ctx, node, "equity_value")?;
    let debt_value = resolve(ctx, node, "debt_value")?;
    let cost_of_equity = resolve(ctx, node, "cost_of_equity")?;
    let cost_of_debt = resolve(ctx, node, "cost_of_debt")?;
    let tax_rate = resolve(ctx, node, "tax_rate")?;
    corporate::wacc(
        equity_value,
        debt_value,
        cost_of_equity,
        cost_of_debt,
        tax_rate,
    )
}

fn eval_free_cash_flow_to_firm(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::FreeCashFlowToFirm;
    let ebit = resolve(ctx, node, "ebit")?;
    let tax_rate = resolve(ctx, node, "tax_rate")?;
    let depreciation_amortization = resolve(ctx, node, "depreciation_amortization")?;
    let capex = resolve(ctx, node, "capex")?;
    let change_in_working_capital = resolve(ctx, node, "change_in_working_capital")?;
    corporate::free_cash_flow_to_firm(
        ebit,
        tax_rate,
        depreciation_amortization,
        capex,
        change_in_working_capital,
    )
}

fn eval_free_cash_flow_to_equity(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::FreeCashFlowToEquity;
    let net_income = resolve(ctx, node, "net_income")?;
    let depreciation_amortization = resolve(ctx, node, "depreciation_amortization")?;
    let capex = resolve(ctx, node, "capex")?;
    let change_in_working_capital = resolve(ctx, node, "change_in_working_capital")?;
    let net_borrowing = resolve(ctx, node, "net_borrowing")?;
    corporate::free_cash_flow_to_equity(
        net_income,
        depreciation_amortization,
        capex,
        change_in_working_capital,
        net_borrowing,
    )
}

fn eval_economic_value_added(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::EconomicValueAdded;
    let nopat = resolve(ctx, node, "nopat")?;
    let invested_capital = resolve(ctx, node, "invested_capital")?;
    let wacc = resolve(ctx, node, "wacc")?;
    corporate::economic_value_added(nopat, invested_capital, wacc)
}

fn eval_sustainable_growth_rate(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::SustainableGrowthRate;
    let roe = resolve(ctx, node, "roe")?;
    let retention_ratio = resolve(ctx, node, "retention_ratio")?;
    corporate::sustainable_growth_rate(roe, retention_ratio)
}

fn eval_internal_growth_rate(ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    let node = FormulaNode::InternalGrowthRate;
    let roa = resolve(ctx, node, "roa")?;
    let retention_ratio = resolve(ctx, node, "retention_ratio")?;
    corporate::internal_growth_rate(roa, retention_ratio)
}

/// Dispatches a single node to its `casiros_core` function, resolving parameters
/// from `ctx`. Exhaustive over [`FormulaNode`]: adding a variant without adding
/// its arm here is a compile error, not a silent gap.
fn evaluate_node(node: FormulaNode, ctx: &EvaluationContext) -> Result<Decimal, CalculationError> {
    match node {
        FormulaNode::FutureValue => eval_future_value(ctx),
        FormulaNode::PresentValue => eval_present_value(ctx),
        FormulaNode::AnnuityFutureValue => eval_annuity_future_value(ctx),
        FormulaNode::AnnuityPresentValue => eval_annuity_present_value(ctx),
        FormulaNode::PerpetuityPresentValue => eval_perpetuity_present_value(ctx),
        FormulaNode::GrowingPerpetuity => eval_growing_perpetuity(ctx),
        FormulaNode::EffectiveAnnualRate => eval_effective_annual_rate(ctx),
        FormulaNode::ContinuousCompounding => eval_continuous_compounding(ctx),
        FormulaNode::ReturnOnEquity => eval_return_on_equity(ctx),
        FormulaNode::ReturnOnAssets => eval_return_on_assets(ctx),
        FormulaNode::ReturnOnInvestment => eval_return_on_investment(ctx),
        FormulaNode::ProfitMargin => eval_profit_margin(ctx),
        FormulaNode::AssetTurnover => eval_asset_turnover(ctx),
        FormulaNode::EquityMultiplier => eval_equity_multiplier(ctx),
        FormulaNode::DupontRoe => eval_dupont_roe(ctx),
        FormulaNode::CurrentRatio => eval_current_ratio(ctx),
        FormulaNode::QuickRatio => eval_quick_ratio(ctx),
        FormulaNode::DebtToEquity => eval_debt_to_equity(ctx),
        FormulaNode::InterestCoverage => eval_interest_coverage(ctx),
        FormulaNode::InventoryTurnover => eval_inventory_turnover(ctx),
        FormulaNode::CashConversionCycle => eval_cash_conversion_cycle(ctx),
        FormulaNode::NetInterestMargin => eval_net_interest_margin(ctx),
        FormulaNode::LoanToDepositRatio => eval_loan_to_deposit_ratio(ctx),
        FormulaNode::CapitalAdequacyRatio => eval_capital_adequacy_ratio(ctx),
        FormulaNode::ProvisionCoverage => eval_provision_coverage(ctx),
        FormulaNode::Beta => eval_beta(ctx),
        FormulaNode::SharpeRatio => eval_sharpe_ratio(ctx),
        FormulaNode::TreynorRatio => eval_treynor_ratio(ctx),
        FormulaNode::JensensAlpha => eval_jensens_alpha(ctx),
        FormulaNode::ValueAtRisk => eval_value_at_risk(ctx),
        FormulaNode::ExpectedShortfall => eval_expected_shortfall(ctx),
        FormulaNode::DividendDiscountModel => eval_dividend_discount_model(ctx),
        FormulaNode::DiscountedCashFlow => eval_discounted_cash_flow(ctx),
        FormulaNode::BondPrice => eval_bond_price(ctx),
        FormulaNode::YieldToMaturity => eval_yield_to_maturity(ctx),
        FormulaNode::Duration => eval_duration(ctx),
        FormulaNode::ModifiedDuration => eval_modified_duration(ctx),
        FormulaNode::Convexity => eval_convexity(ctx),
        FormulaNode::Wacc => eval_wacc(ctx),
        FormulaNode::FreeCashFlowToFirm => eval_free_cash_flow_to_firm(ctx),
        FormulaNode::FreeCashFlowToEquity => eval_free_cash_flow_to_equity(ctx),
        FormulaNode::EconomicValueAdded => eval_economic_value_added(ctx),
        FormulaNode::SustainableGrowthRate => eval_sustainable_growth_rate(ctx),
        FormulaNode::InternalGrowthRate => eval_internal_growth_rate(ctx),
    }
}

/// Evaluates every node in `engine` in dependency order, populating `ctx.results`.
///
/// # Errors
///
/// Returns [`CalculationError::CyclicDependency`] if `engine` is not a DAG.
/// Returns whatever error the first failing node's `casiros_core` function
/// produces (including [`CalculationError::MissingInput`] if a required
/// parameter is present in neither `ctx.results` nor `ctx.inputs`).
pub fn evaluate_dag(
    engine: &CausalityEngine<FormulaNode>,
    ctx: &mut EvaluationContext,
) -> Result<(), CalculationError> {
    let order = engine
        .execution_order()
        .map_err(|details| CalculationError::CyclicDependency { details })?;
    for node in order {
        let value = evaluate_node(node, ctx)?;
        ctx.results.insert(node, value);
    }
    Ok(())
}
