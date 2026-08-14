//! Benchmarks `casiros-dag`'s topological-sort + evaluation overhead: several
//! independent nodes plus one real causal chain (`Wacc` feeds
//! `EconomicValueAdded`'s `wacc` parameter by name), evaluated together in a
//! single pass.

use casiros_dag::evaluator::{EvaluationContext, evaluate_dag};
use casiros_dag::graph::{CausalityEngine, FormulaNode};
use criterion::{Criterion, criterion_group, criterion_main};
use rust_decimal_macros::dec;
use std::hint::black_box;

fn build_context() -> EvaluationContext {
    let mut ctx = EvaluationContext::new();
    let inputs = [
        ("pv", dec!(1000)),
        ("rate", dec!(0.05)),
        ("current_assets", dec!(400_000)),
        ("current_liabilities", dec!(200_000)),
        ("equity_value", dec!(600_000)),
        ("debt_value", dec!(400_000)),
        ("cost_of_equity", dec!(0.10)),
        ("cost_of_debt", dec!(0.06)),
        ("tax_rate", dec!(0.25)),
        ("nopat", dec!(150_000)),
        ("invested_capital", dec!(1_000_000)),
    ];
    for (name, value) in inputs {
        ctx.inputs.insert(name.to_string(), value);
    }
    ctx.inputs.insert("periods".to_string(), dec!(120));
    ctx
}

fn build_engine() -> CausalityEngine<FormulaNode> {
    let mut engine = CausalityEngine::new();
    engine.add_node(FormulaNode::FutureValue);
    engine.add_node(FormulaNode::CurrentRatio);
    // The one real causal edge: EconomicValueAdded's "wacc" parameter is
    // resolved from Wacc's own result, by name (see FormulaNode::name).
    engine.add_dependency(FormulaNode::Wacc, FormulaNode::EconomicValueAdded);
    engine
}

fn bench_multi_node_dag(c: &mut Criterion) {
    let engine = build_engine();
    c.bench_function("evaluate_dag_multi_node", |b| {
        b.iter(|| {
            let mut ctx = build_context();
            evaluate_dag(black_box(&engine), black_box(&mut ctx))
        });
    });
}

criterion_group!(benches, bench_multi_node_dag);
criterion_main!(benches);
