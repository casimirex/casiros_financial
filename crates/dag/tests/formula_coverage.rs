//! Exercises `evaluate_dag`'s dispatch for every `FormulaNode` variant,
//! cross-checking each result against calling the underlying `casiros_core`
//! function directly with the same inputs. `evaluator.rs`'s per-formula
//! `eval_*` wrappers were previously reachable only through the handful of
//! formulas exercised via `/calculate` route tests and benchmarks — this
//! file closes that gap so the DAG dispatch layer itself (not just the core
//! math it wraps) is verified for every formula.

use casiros_core::{banking, corporate, financial, general, markets, stocks_bonds};
use casiros_dag::evaluator::{EvaluationContext, evaluate_dag};
use casiros_dag::graph::{CausalityEngine, FormulaNode};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn eval(node: FormulaNode, scalars: &[(&str, Decimal)], series: &[(&str, &[Decimal])]) -> Decimal {
    let mut ctx = EvaluationContext::new();
    for (name, value) in scalars {
        ctx.inputs.insert((*name).to_string(), *value);
    }
    for (name, values) in series {
        ctx.series_inputs
            .insert((*name).to_string(), values.to_vec());
    }
    let mut engine = CausalityEngine::new();
    engine.add_node(node);
    evaluate_dag(&engine, &mut ctx).unwrap();
    ctx.results[&node]
}

#[test]
fn future_value_dispatches_correctly() {
    let result = eval(
        FormulaNode::FutureValue,
        &[
            ("pv", dec!(1000)),
            ("rate", dec!(0.05)),
            ("periods", dec!(10)),
        ],
        &[],
    );
    assert_eq!(
        result,
        general::future_value(dec!(1000), dec!(0.05), 10).unwrap()
    );
}

#[test]
fn present_value_dispatches_correctly() {
    let result = eval(
        FormulaNode::PresentValue,
        &[
            ("fv", dec!(1100)),
            ("rate", dec!(0.10)),
            ("periods", dec!(1)),
        ],
        &[],
    );
    assert_eq!(
        result,
        general::present_value(dec!(1100), dec!(0.10), 1).unwrap()
    );
}

#[test]
fn annuity_future_value_dispatches_correctly() {
    let result = eval(
        FormulaNode::AnnuityFutureValue,
        &[
            ("pmt", dec!(1000)),
            ("rate", dec!(0.05)),
            ("periods", dec!(10)),
        ],
        &[],
    );
    assert_eq!(
        result,
        general::annuity_future_value(dec!(1000), dec!(0.05), 10).unwrap()
    );
}

#[test]
fn annuity_present_value_dispatches_correctly() {
    let result = eval(
        FormulaNode::AnnuityPresentValue,
        &[
            ("pmt", dec!(1000)),
            ("rate", dec!(0.05)),
            ("periods", dec!(10)),
        ],
        &[],
    );
    assert_eq!(
        result,
        general::annuity_present_value(dec!(1000), dec!(0.05), 10).unwrap()
    );
}

#[test]
fn perpetuity_present_value_dispatches_correctly() {
    let result = eval(
        FormulaNode::PerpetuityPresentValue,
        &[("pmt", dec!(100)), ("rate", dec!(0.05))],
        &[],
    );
    assert_eq!(
        result,
        general::perpetuity_present_value(dec!(100), dec!(0.05)).unwrap()
    );
}

#[test]
fn growing_perpetuity_dispatches_correctly() {
    let result = eval(
        FormulaNode::GrowingPerpetuity,
        &[
            ("d1", dec!(100)),
            ("rate", dec!(0.09)),
            ("growth", dec!(0.04)),
        ],
        &[],
    );
    assert_eq!(
        result,
        general::growing_perpetuity(dec!(100), dec!(0.09), dec!(0.04)).unwrap()
    );
}

#[test]
fn effective_annual_rate_dispatches_correctly() {
    let result = eval(
        FormulaNode::EffectiveAnnualRate,
        &[
            ("nominal_rate", dec!(0.12)),
            ("compounding_periods", dec!(12)),
        ],
        &[],
    );
    assert_eq!(
        result,
        general::effective_annual_rate(dec!(0.12), 12).unwrap()
    );
}

#[test]
fn continuous_compounding_dispatches_correctly() {
    let result = eval(
        FormulaNode::ContinuousCompounding,
        &[("pv", dec!(1000)), ("rate", dec!(0.05)), ("time", dec!(1))],
        &[],
    );
    assert_eq!(
        result,
        general::continuous_compounding(dec!(1000), dec!(0.05), dec!(1)).unwrap()
    );
}

#[test]
fn return_on_equity_dispatches_correctly() {
    let result = eval(
        FormulaNode::ReturnOnEquity,
        &[
            ("net_income", dec!(100_000)),
            ("avg_shareholders_equity", dec!(500_000)),
        ],
        &[],
    );
    assert_eq!(
        result,
        financial::return_on_equity(dec!(100_000), dec!(500_000)).unwrap()
    );
}

#[test]
fn return_on_assets_dispatches_correctly() {
    let result = eval(
        FormulaNode::ReturnOnAssets,
        &[
            ("net_income", dec!(100_000)),
            ("avg_total_assets", dec!(1_000_000)),
        ],
        &[],
    );
    assert_eq!(
        result,
        financial::return_on_assets(dec!(100_000), dec!(1_000_000)).unwrap()
    );
}

#[test]
fn return_on_investment_dispatches_correctly() {
    let result = eval(
        FormulaNode::ReturnOnInvestment,
        &[("current_value", dec!(1_200)), ("cost", dec!(1_000))],
        &[],
    );
    assert_eq!(
        result,
        financial::return_on_investment(dec!(1_200), dec!(1_000)).unwrap()
    );
}

#[test]
fn profit_margin_dispatches_correctly() {
    let result = eval(
        FormulaNode::ProfitMargin,
        &[("net_income", dec!(100_000)), ("revenue", dec!(1_000_000))],
        &[],
    );
    assert_eq!(
        result,
        financial::profit_margin(dec!(100_000), dec!(1_000_000)).unwrap()
    );
}

#[test]
fn asset_turnover_dispatches_correctly() {
    let result = eval(
        FormulaNode::AssetTurnover,
        &[
            ("net_sales", dec!(1_000_000)),
            ("avg_total_assets", dec!(500_000)),
        ],
        &[],
    );
    assert_eq!(
        result,
        financial::asset_turnover(dec!(1_000_000), dec!(500_000)).unwrap()
    );
}

#[test]
fn equity_multiplier_dispatches_correctly() {
    let result = eval(
        FormulaNode::EquityMultiplier,
        &[
            ("total_assets", dec!(1_000_000)),
            ("total_equity", dec!(500_000)),
        ],
        &[],
    );
    assert_eq!(
        result,
        financial::equity_multiplier(dec!(1_000_000), dec!(500_000)).unwrap()
    );
}

#[test]
fn dupont_roe_dispatches_correctly() {
    let result = eval(
        FormulaNode::DupontRoe,
        &[
            ("net_margin", dec!(0.10)),
            ("asset_turnover", dec!(1.5)),
            ("equity_multiplier", dec!(2.0)),
        ],
        &[],
    );
    assert_eq!(
        result,
        financial::dupont_roe(dec!(0.10), dec!(1.5), dec!(2.0)).unwrap()
    );
}

#[test]
fn current_ratio_dispatches_correctly() {
    let result = eval(
        FormulaNode::CurrentRatio,
        &[
            ("current_assets", dec!(400_000)),
            ("current_liabilities", dec!(200_000)),
        ],
        &[],
    );
    assert_eq!(
        result,
        financial::current_ratio(dec!(400_000), dec!(200_000)).unwrap()
    );
}

#[test]
fn quick_ratio_dispatches_correctly() {
    let result = eval(
        FormulaNode::QuickRatio,
        &[
            ("current_assets", dec!(400_000)),
            ("inventory", dec!(100_000)),
            ("current_liabilities", dec!(200_000)),
        ],
        &[],
    );
    assert_eq!(
        result,
        financial::quick_ratio(dec!(400_000), dec!(100_000), dec!(200_000)).unwrap()
    );
}

#[test]
fn debt_to_equity_dispatches_correctly() {
    let result = eval(
        FormulaNode::DebtToEquity,
        &[
            ("total_liabilities", dec!(400_000)),
            ("total_equity", dec!(500_000)),
        ],
        &[],
    );
    assert_eq!(
        result,
        financial::debt_to_equity(dec!(400_000), dec!(500_000)).unwrap()
    );
}

#[test]
fn interest_coverage_dispatches_correctly() {
    let result = eval(
        FormulaNode::InterestCoverage,
        &[("ebit", dec!(500_000)), ("interest_expense", dec!(100_000))],
        &[],
    );
    assert_eq!(
        result,
        financial::interest_coverage(dec!(500_000), dec!(100_000)).unwrap()
    );
}

#[test]
fn inventory_turnover_dispatches_correctly() {
    let result = eval(
        FormulaNode::InventoryTurnover,
        &[("cogs", dec!(600_000)), ("avg_inventory", dec!(150_000))],
        &[],
    );
    assert_eq!(
        result,
        financial::inventory_turnover(dec!(600_000), dec!(150_000)).unwrap()
    );
}

#[test]
fn cash_conversion_cycle_dispatches_correctly() {
    let result = eval(
        FormulaNode::CashConversionCycle,
        &[
            ("days_inventory_outstanding", dec!(60)),
            ("days_sales_outstanding", dec!(45)),
            ("days_payable_outstanding", dec!(30)),
        ],
        &[],
    );
    assert_eq!(
        result,
        financial::cash_conversion_cycle(dec!(60), dec!(45), dec!(30)).unwrap()
    );
}

#[test]
fn net_interest_margin_dispatches_correctly() {
    let result = eval(
        FormulaNode::NetInterestMargin,
        &[
            ("net_interest_income", dec!(40_000)),
            ("avg_earning_assets", dec!(1_000_000)),
        ],
        &[],
    );
    assert_eq!(
        result,
        banking::net_interest_margin(dec!(40_000), dec!(1_000_000)).unwrap()
    );
}

#[test]
fn loan_to_deposit_ratio_dispatches_correctly() {
    let result = eval(
        FormulaNode::LoanToDepositRatio,
        &[
            ("total_loans", dec!(800_000)),
            ("total_deposits", dec!(1_000_000)),
        ],
        &[],
    );
    assert_eq!(
        result,
        banking::loan_to_deposit_ratio(dec!(800_000), dec!(1_000_000)).unwrap()
    );
}

#[test]
fn capital_adequacy_ratio_dispatches_correctly() {
    let result = eval(
        FormulaNode::CapitalAdequacyRatio,
        &[
            ("qualifying_capital", dec!(120_000)),
            ("risk_weighted_assets", dec!(1_000_000)),
        ],
        &[],
    );
    assert_eq!(
        result,
        banking::capital_adequacy_ratio(dec!(120_000), dec!(1_000_000)).unwrap()
    );
}

#[test]
fn provision_coverage_dispatches_correctly() {
    let result = eval(
        FormulaNode::ProvisionCoverage,
        &[
            ("loan_loss_provisions", dec!(50_000)),
            ("non_performing_loans", dec!(100_000)),
        ],
        &[],
    );
    assert_eq!(
        result,
        banking::provision_coverage(dec!(50_000), dec!(100_000)).unwrap()
    );
}

#[test]
fn beta_dispatches_correctly() {
    let result = eval(
        FormulaNode::Beta,
        &[("covariance", dec!(0.02)), ("variance_market", dec!(0.04))],
        &[],
    );
    assert_eq!(result, markets::beta(dec!(0.02), dec!(0.04)).unwrap());
}

#[test]
fn sharpe_ratio_dispatches_correctly() {
    let result = eval(
        FormulaNode::SharpeRatio,
        &[
            ("portfolio_return", dec!(0.10)),
            ("risk_free_rate", dec!(0.03)),
            ("std_dev", dec!(0.15)),
        ],
        &[],
    );
    assert_eq!(
        result,
        markets::sharpe_ratio(dec!(0.10), dec!(0.03), dec!(0.15)).unwrap()
    );
}

#[test]
fn treynor_ratio_dispatches_correctly() {
    let result = eval(
        FormulaNode::TreynorRatio,
        &[
            ("portfolio_return", dec!(0.10)),
            ("risk_free_rate", dec!(0.03)),
            ("portfolio_beta", dec!(1.2)),
        ],
        &[],
    );
    assert_eq!(
        result,
        markets::treynor_ratio(dec!(0.10), dec!(0.03), dec!(1.2)).unwrap()
    );
}

#[test]
fn jensens_alpha_dispatches_correctly() {
    let result = eval(
        FormulaNode::JensensAlpha,
        &[
            ("portfolio_return", dec!(0.12)),
            ("risk_free_rate", dec!(0.03)),
            ("portfolio_beta", dec!(1.2)),
            ("market_return", dec!(0.08)),
        ],
        &[],
    );
    assert_eq!(
        result,
        markets::jensens_alpha(dec!(0.12), dec!(0.03), dec!(1.2), dec!(0.08)).unwrap()
    );
}

#[test]
fn value_at_risk_dispatches_correctly() {
    let result = eval(
        FormulaNode::ValueAtRisk,
        &[
            ("portfolio_value", dec!(1_000_000)),
            ("z_score", dec!(1.65)),
            ("std_dev", dec!(0.02)),
        ],
        &[],
    );
    assert_eq!(
        result,
        markets::value_at_risk(dec!(1_000_000), dec!(1.65), dec!(0.02)).unwrap()
    );
}

#[test]
fn expected_shortfall_dispatches_correctly() {
    let result = eval(
        FormulaNode::ExpectedShortfall,
        &[
            ("portfolio_value", dec!(1_000_000)),
            ("z_score", dec!(1.65)),
            ("std_dev", dec!(0.02)),
            ("confidence", dec!(0.95)),
        ],
        &[],
    );
    assert_eq!(
        result,
        markets::expected_shortfall(dec!(1_000_000), dec!(1.65), dec!(0.02), dec!(0.95)).unwrap()
    );
}

#[test]
fn dividend_discount_model_dispatches_correctly() {
    let result = eval(
        FormulaNode::DividendDiscountModel,
        &[
            ("next_dividend", dec!(2.10)),
            ("required_return", dec!(0.10)),
            ("growth_rate", dec!(0.05)),
        ],
        &[],
    );
    assert_eq!(
        result,
        stocks_bonds::dividend_discount_model(dec!(2.10), dec!(0.10), dec!(0.05)).unwrap()
    );
}

#[test]
fn discounted_cash_flow_dispatches_correctly() {
    let cash_flows = [dec!(0), dec!(0), dec!(1000)];
    let result = eval(
        FormulaNode::DiscountedCashFlow,
        &[("rate", dec!(0.10))],
        &[("cash_flows", &cash_flows)],
    );
    assert_eq!(
        result,
        stocks_bonds::discounted_cash_flow(&cash_flows, dec!(0.10)).unwrap()
    );
}

#[test]
fn bond_price_dispatches_correctly() {
    let result = eval(
        FormulaNode::BondPrice,
        &[
            ("face_value", dec!(1000)),
            ("coupon_rate", dec!(0.05)),
            ("market_rate", dec!(0.05)),
            ("periods", dec!(10)),
        ],
        &[],
    );
    assert_eq!(
        result,
        stocks_bonds::bond_price(dec!(1000), dec!(0.05), dec!(0.05), 10).unwrap()
    );
}

#[test]
fn yield_to_maturity_dispatches_correctly() {
    let result = eval(
        FormulaNode::YieldToMaturity,
        &[
            ("price", dec!(950)),
            ("face_value", dec!(1000)),
            ("coupon_rate", dec!(0.05)),
            ("periods", dec!(10)),
        ],
        &[],
    );
    assert_eq!(
        result,
        stocks_bonds::yield_to_maturity(dec!(950), dec!(1000), dec!(0.05), 10).unwrap()
    );
}

#[test]
fn duration_dispatches_correctly() {
    let cash_flows = [dec!(50), dec!(50), dec!(1050)];
    let result = eval(
        FormulaNode::Duration,
        &[("rate", dec!(0.05))],
        &[("cash_flows", &cash_flows)],
    );
    assert_eq!(
        result,
        stocks_bonds::duration(&cash_flows, dec!(0.05)).unwrap()
    );
}

#[test]
fn modified_duration_dispatches_correctly() {
    let result = eval(
        FormulaNode::ModifiedDuration,
        &[
            ("macaulay_duration", dec!(2.8)),
            ("ytm", dec!(0.05)),
            ("periods_per_year", dec!(1)),
        ],
        &[],
    );
    assert_eq!(
        result,
        stocks_bonds::modified_duration(dec!(2.8), dec!(0.05), 1).unwrap()
    );
}

#[test]
fn convexity_dispatches_correctly() {
    let cash_flows = [dec!(0), dec!(0), dec!(0), dec!(0), dec!(1000)];
    let result = eval(
        FormulaNode::Convexity,
        &[("rate", dec!(0.05))],
        &[("cash_flows", &cash_flows)],
    );
    assert_eq!(
        result,
        stocks_bonds::convexity(&cash_flows, dec!(0.05)).unwrap()
    );
}

#[test]
fn wacc_dispatches_correctly() {
    let result = eval(
        FormulaNode::Wacc,
        &[
            ("equity_value", dec!(600_000)),
            ("debt_value", dec!(400_000)),
            ("cost_of_equity", dec!(0.10)),
            ("cost_of_debt", dec!(0.06)),
            ("tax_rate", dec!(0.25)),
        ],
        &[],
    );
    assert_eq!(
        result,
        corporate::wacc(
            dec!(600_000),
            dec!(400_000),
            dec!(0.10),
            dec!(0.06),
            dec!(0.25)
        )
        .unwrap()
    );
}

#[test]
fn free_cash_flow_to_firm_dispatches_correctly() {
    let result = eval(
        FormulaNode::FreeCashFlowToFirm,
        &[
            ("ebit", dec!(500_000)),
            ("tax_rate", dec!(0.25)),
            ("depreciation_amortization", dec!(50_000)),
            ("capex", dec!(80_000)),
            ("change_in_working_capital", dec!(20_000)),
        ],
        &[],
    );
    assert_eq!(
        result,
        corporate::free_cash_flow_to_firm(
            dec!(500_000),
            dec!(0.25),
            dec!(50_000),
            dec!(80_000),
            dec!(20_000)
        )
        .unwrap()
    );
}

#[test]
fn free_cash_flow_to_equity_dispatches_correctly() {
    let result = eval(
        FormulaNode::FreeCashFlowToEquity,
        &[
            ("net_income", dec!(300_000)),
            ("depreciation_amortization", dec!(50_000)),
            ("capex", dec!(80_000)),
            ("change_in_working_capital", dec!(20_000)),
            ("net_borrowing", dec!(10_000)),
        ],
        &[],
    );
    assert_eq!(
        result,
        corporate::free_cash_flow_to_equity(
            dec!(300_000),
            dec!(50_000),
            dec!(80_000),
            dec!(20_000),
            dec!(10_000)
        )
        .unwrap()
    );
}

#[test]
fn economic_value_added_dispatches_correctly() {
    let result = eval(
        FormulaNode::EconomicValueAdded,
        &[
            ("nopat", dec!(200_000)),
            ("invested_capital", dec!(1_000_000)),
            ("wacc", dec!(0.08)),
        ],
        &[],
    );
    assert_eq!(
        result,
        corporate::economic_value_added(dec!(200_000), dec!(1_000_000), dec!(0.08)).unwrap()
    );
}

#[test]
fn sustainable_growth_rate_dispatches_correctly() {
    let result = eval(
        FormulaNode::SustainableGrowthRate,
        &[("roe", dec!(0.15)), ("retention_ratio", dec!(0.6))],
        &[],
    );
    assert_eq!(
        result,
        corporate::sustainable_growth_rate(dec!(0.15), dec!(0.6)).unwrap()
    );
}

#[test]
fn internal_growth_rate_dispatches_correctly() {
    let result = eval(
        FormulaNode::InternalGrowthRate,
        &[("roa", dec!(0.10)), ("retention_ratio", dec!(0.5))],
        &[],
    );
    assert_eq!(
        result,
        corporate::internal_growth_rate(dec!(0.10), dec!(0.5)).unwrap()
    );
}

// --- error paths: resolve()/resolve_periods()/resolve_series() -------------

#[test]
fn missing_scalar_input_is_a_missing_input_error() {
    let mut ctx = EvaluationContext::new();
    ctx.inputs.insert("rate".to_string(), dec!(0.05));
    ctx.inputs.insert("periods".to_string(), dec!(10));
    // "pv" deliberately omitted.
    let mut engine = CausalityEngine::new();
    engine.add_node(FormulaNode::FutureValue);
    let result = evaluate_dag(&engine, &mut ctx);
    assert!(result.is_err());
}

#[test]
fn non_integer_periods_is_rejected() {
    let mut ctx = EvaluationContext::new();
    ctx.inputs.insert("pv".to_string(), dec!(1000));
    ctx.inputs.insert("rate".to_string(), dec!(0.05));
    ctx.inputs.insert("periods".to_string(), dec!(10.5));
    let mut engine = CausalityEngine::new();
    engine.add_node(FormulaNode::FutureValue);
    let result = evaluate_dag(&engine, &mut ctx);
    assert!(result.is_err());
}

#[test]
fn missing_series_input_is_a_missing_input_error() {
    let mut ctx = EvaluationContext::new();
    ctx.inputs.insert("rate".to_string(), dec!(0.10));
    // "cash_flows" deliberately omitted from series_inputs.
    let mut engine = CausalityEngine::new();
    engine.add_node(FormulaNode::DiscountedCashFlow);
    let result = evaluate_dag(&engine, &mut ctx);
    assert!(result.is_err());
}

// --- FormulaNode::name / from_name round-trip, and CausalityEngine basics --

// FormulaNode::all() is itself a hand-written list (see graph.rs) — kept as
// a separate literal here (rather than simply calling it) so this file's
// round-trip test in `every_formula_node_name_round_trips_through_from_name`
// stays a genuine cross-check of two independently-written enumerations,
// not a tautology against the same list.
const ALL_FORMULA_NODES: &[FormulaNode] = &[
    FormulaNode::FutureValue,
    FormulaNode::PresentValue,
    FormulaNode::AnnuityFutureValue,
    FormulaNode::AnnuityPresentValue,
    FormulaNode::PerpetuityPresentValue,
    FormulaNode::GrowingPerpetuity,
    FormulaNode::EffectiveAnnualRate,
    FormulaNode::ContinuousCompounding,
    FormulaNode::ReturnOnEquity,
    FormulaNode::ReturnOnAssets,
    FormulaNode::ReturnOnInvestment,
    FormulaNode::ProfitMargin,
    FormulaNode::AssetTurnover,
    FormulaNode::EquityMultiplier,
    FormulaNode::DupontRoe,
    FormulaNode::CurrentRatio,
    FormulaNode::QuickRatio,
    FormulaNode::DebtToEquity,
    FormulaNode::InterestCoverage,
    FormulaNode::InventoryTurnover,
    FormulaNode::CashConversionCycle,
    FormulaNode::NetInterestMargin,
    FormulaNode::LoanToDepositRatio,
    FormulaNode::CapitalAdequacyRatio,
    FormulaNode::ProvisionCoverage,
    FormulaNode::Beta,
    FormulaNode::SharpeRatio,
    FormulaNode::TreynorRatio,
    FormulaNode::JensensAlpha,
    FormulaNode::ValueAtRisk,
    FormulaNode::ExpectedShortfall,
    FormulaNode::DividendDiscountModel,
    FormulaNode::DiscountedCashFlow,
    FormulaNode::BondPrice,
    FormulaNode::YieldToMaturity,
    FormulaNode::Duration,
    FormulaNode::ModifiedDuration,
    FormulaNode::Convexity,
    FormulaNode::Wacc,
    FormulaNode::FreeCashFlowToFirm,
    FormulaNode::FreeCashFlowToEquity,
    FormulaNode::EconomicValueAdded,
    FormulaNode::SustainableGrowthRate,
    FormulaNode::InternalGrowthRate,
];

#[test]
fn all_matches_the_independently_written_test_list() {
    assert_eq!(FormulaNode::all(), ALL_FORMULA_NODES);
}

#[test]
fn every_formula_node_name_round_trips_through_from_name() {
    assert_eq!(ALL_FORMULA_NODES.len(), 44);
    for node in ALL_FORMULA_NODES {
        assert_eq!(FormulaNode::from_name(node.name()), Some(*node));
    }
}

#[test]
fn from_name_rejects_an_unknown_string() {
    assert_eq!(FormulaNode::from_name("not_a_real_formula"), None);
    assert_eq!(FormulaNode::from_name(""), None);
}

#[test]
fn add_node_is_idempotent() {
    let mut engine = CausalityEngine::new();
    let first = engine.add_node(FormulaNode::Wacc);
    let second = engine.add_node(FormulaNode::Wacc);
    assert_eq!(first, second);
    assert_eq!(engine.execution_order().unwrap().len(), 1);
}

#[test]
fn cyclic_dependency_is_rejected() {
    let mut ctx = EvaluationContext::new();
    let mut engine = CausalityEngine::new();
    engine.add_dependency(FormulaNode::Wacc, FormulaNode::EconomicValueAdded);
    engine.add_dependency(FormulaNode::EconomicValueAdded, FormulaNode::Wacc);
    let result = evaluate_dag(&engine, &mut ctx);
    assert!(result.is_err());
}

// --- FormulaNode::parameters cross-checked against real evaluation --------
//
// `parameters()` hand-transcribes each eval_* function's `resolve` calls
// (see graph.rs's doc comment on it) rather than deriving them, so an
// omitted name there wouldn't fail to compile — it would just silently
// misreport a formula's dependency wiring. This test catches that: for
// every node, supplying exactly the names `parameters()` claims (as a safe
// nonzero, whole-number scalar, or as a two-point series where the node
// takes one) must never fail with `MissingInput` — a `MissingInput` error
// here proves a real parameter was left out of the hand-written list.
// (It cannot catch the opposite mistake — a spurious extra name — since an
// unused input is silently ignored; the risk there is the same as any
// hand-transcribed list, and each entry was cross-checked against
// `frontend/src/features/calculator/formula-registry.ts`'s independently
// maintained copy when this test was written.)

#[test]
fn parameters_lists_every_name_evaluation_actually_needs() {
    for &node in ALL_FORMULA_NODES {
        let mut ctx = EvaluationContext::new();
        for &parameter in node.parameters() {
            if parameter == "cash_flows" {
                ctx.series_inputs
                    .insert(parameter.to_string(), vec![dec!(2), dec!(2)]);
            } else {
                ctx.inputs.insert(parameter.to_string(), dec!(2));
            }
        }
        let mut engine = CausalityEngine::new();
        engine.add_node(node);
        let result = evaluate_dag(&engine, &mut ctx);
        if let Err(casiros_core::error::CalculationError::MissingInput { formula, parameter }) =
            &result
        {
            panic!(
                "{node:?}::parameters() is missing {parameter:?} — {formula} required it during evaluation"
            );
        }
    }
}
