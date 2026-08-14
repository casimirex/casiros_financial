//! Fuzzes `casiros-dag`'s evaluator: `evaluate_dag` must return `Result`,
//! never panic, no matter how adversarial the raw `Decimal` inputs are —
//! including negative periods, zero denominators, and overflow-prone
//! magnitudes.

#![no_main]

use arbitrary::Unstructured;
use casiros_dag::evaluator::{EvaluationContext, evaluate_dag};
use casiros_dag::graph::{CausalityEngine, FormulaNode};
use casiros_fuzz_shared::arbitrary_decimal;
use libfuzzer_sys::fuzz_target;

const INPUT_NAMES: &[&str] = &[
    "pv",
    "rate",
    "periods",
    "current_assets",
    "current_liabilities",
    "equity_value",
    "debt_value",
    "cost_of_equity",
    "cost_of_debt",
    "tax_rate",
];

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let mut ctx = EvaluationContext::new();
    for name in INPUT_NAMES {
        let Ok(value) = arbitrary_decimal(&mut u) else {
            return;
        };
        ctx.inputs.insert((*name).to_string(), value);
    }

    let mut engine = CausalityEngine::new();
    engine.add_node(FormulaNode::FutureValue);
    engine.add_node(FormulaNode::CurrentRatio);
    engine.add_dependency(FormulaNode::Wacc, FormulaNode::EconomicValueAdded);

    let _ = evaluate_dag(&engine, &mut ctx);
});
