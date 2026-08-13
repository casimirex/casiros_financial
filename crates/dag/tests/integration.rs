//! Integration tests exercising `casiros-dag`'s public API end-to-end.

use casiros_core::corporate;
use casiros_core::error::CalculationError;
use casiros_dag::evaluator::{EvaluationContext, evaluate_dag};
use casiros_dag::graph::{CausalityEngine, FormulaNode};
use rust_decimal_macros::dec;

#[test]
fn execution_order_respects_declared_dependency() {
    let mut engine = CausalityEngine::new();
    engine.add_dependency(FormulaNode::Wacc, FormulaNode::EconomicValueAdded);

    let order = engine.execution_order().expect("two-node chain is acyclic");
    let wacc_pos = order
        .iter()
        .position(|node| *node == FormulaNode::Wacc)
        .expect("Wacc present");
    let eva_pos = order
        .iter()
        .position(|node| *node == FormulaNode::EconomicValueAdded)
        .expect("EconomicValueAdded present");
    assert!(wacc_pos < eva_pos);
}

#[test]
fn cycle_detection_is_a_hard_error() {
    let mut engine = CausalityEngine::new();
    engine.add_dependency(FormulaNode::Wacc, FormulaNode::EconomicValueAdded);
    engine.add_dependency(FormulaNode::EconomicValueAdded, FormulaNode::Wacc);

    assert!(engine.execution_order().is_err());
}

#[test]
fn evaluate_dag_chains_wacc_output_into_economic_value_added() {
    let mut engine = CausalityEngine::new();
    engine.add_dependency(FormulaNode::Wacc, FormulaNode::EconomicValueAdded);

    let mut ctx = EvaluationContext::new();
    ctx.inputs
        .insert("equity_value".to_string(), dec!(600_000.0));
    ctx.inputs.insert("debt_value".to_string(), dec!(400_000.0));
    ctx.inputs.insert("cost_of_equity".to_string(), dec!(0.10));
    ctx.inputs.insert("cost_of_debt".to_string(), dec!(0.06));
    ctx.inputs.insert("tax_rate".to_string(), dec!(0.25));
    ctx.inputs.insert("nopat".to_string(), dec!(200_000.0));
    ctx.inputs
        .insert("invested_capital".to_string(), dec!(1_000_000.0));

    evaluate_dag(&engine, &mut ctx).expect("both nodes have all required inputs");

    let expected_wacc = corporate::wacc(
        dec!(600_000.0),
        dec!(400_000.0),
        dec!(0.10),
        dec!(0.06),
        dec!(0.25),
    )
    .unwrap();
    let expected_eva =
        corporate::economic_value_added(dec!(200_000.0), dec!(1_000_000.0), expected_wacc).unwrap();

    assert_eq!(ctx.results[&FormulaNode::Wacc], expected_wacc);
    assert_eq!(ctx.results[&FormulaNode::EconomicValueAdded], expected_eva);
}

#[test]
fn evaluate_dag_reports_missing_input() {
    let mut engine = CausalityEngine::new();
    engine.add_node(FormulaNode::FutureValue);

    let mut ctx = EvaluationContext::new();
    ctx.inputs.insert("pv".to_string(), dec!(1000.0));
    ctx.inputs.insert("rate".to_string(), dec!(0.05));
    // "periods" is deliberately omitted.

    let error = evaluate_dag(&engine, &mut ctx).expect_err("periods is missing");
    assert_eq!(
        error,
        CalculationError::MissingInput {
            formula: "future_value",
            parameter: "periods"
        }
    );
}

#[test]
fn evaluate_dag_reports_cyclic_dependency() {
    let mut engine = CausalityEngine::new();
    engine.add_dependency(FormulaNode::Wacc, FormulaNode::EconomicValueAdded);
    engine.add_dependency(FormulaNode::EconomicValueAdded, FormulaNode::Wacc);

    let mut ctx = EvaluationContext::new();
    let error = evaluate_dag(&engine, &mut ctx).expect_err("graph has a cycle");
    assert!(matches!(error, CalculationError::CyclicDependency { .. }));
}

#[test]
fn evaluate_dag_handles_cash_flow_series_nodes() {
    let mut engine = CausalityEngine::new();
    engine.add_node(FormulaNode::DiscountedCashFlow);

    let mut ctx = EvaluationContext::new();
    ctx.series_inputs.insert(
        "cash_flows".to_string(),
        vec![dec!(0.0), dec!(0.0), dec!(1000.0)],
    );
    ctx.inputs.insert("rate".to_string(), dec!(0.10));

    evaluate_dag(&engine, &mut ctx).expect("cash flow series and rate are both present");

    let expected = casiros_core::stocks_bonds::discounted_cash_flow(
        &[dec!(0.0), dec!(0.0), dec!(1000.0)],
        dec!(0.10),
    )
    .unwrap();
    assert_eq!(ctx.results[&FormulaNode::DiscountedCashFlow], expected);
}
