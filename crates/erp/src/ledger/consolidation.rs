//! DAG-based account roll-up: a rollup account's balance is the sum of its
//! direct children's balances, recomputed in topological order rather than by
//! summing the whole ledger.

use super::account::{AccountCode, ChartOfAccounts};
use crate::error::ErpError;
use casiros_core::error::CalculationError;
use casiros_core::types::Dollar;
use casiros_dag::graph::CausalityEngine;
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::hash::BuildHasher;

/// Builds a [`CausalityEngine`] over `chart`'s parent/child roll-up hierarchy:
/// an edge `child -> parent` means the parent's balance causally depends on
/// the child's, so children are always ordered before their parents.
#[must_use]
pub fn build_rollup_graph(chart: &ChartOfAccounts) -> CausalityEngine<AccountCode> {
    let mut engine = CausalityEngine::new();
    for account in chart.accounts() {
        engine.add_node(account.code);
        if let Some(parent) = account.parent {
            engine.add_dependency(account.code, parent);
        }
    }
    engine
}

/// Recomputes every roll-up account's balance (the sum of its direct
/// children's balances) in topological order, so a grandparent is only
/// recomputed after its parent has already been refreshed. Leaf accounts
/// (with no children) are left untouched — their balances are maintained
/// incrementally by [`crate::ledger::Ledger::post`], not recomputed here.
///
/// # Errors
///
/// Returns [`ErpError::CyclicHierarchy`] if the roll-up hierarchy contains a
/// cycle. Returns [`ErpError::Calculation`] if a balance sum overflows.
pub fn recompute_rollups<S: BuildHasher>(
    chart: &ChartOfAccounts,
    balances: &mut HashMap<AccountCode, Dollar, S>,
) -> Result<(), ErpError> {
    let graph = build_rollup_graph(chart);
    let order = graph.execution_order().map_err(ErpError::CyclicHierarchy)?;

    for code in order {
        let children: Vec<AccountCode> = chart
            .children_of(code)
            .map(|account| account.code)
            .collect();
        if children.is_empty() {
            continue;
        }
        let mut total = Decimal::ZERO;
        for child_code in children {
            let child_balance = balances.get(&child_code).copied().unwrap_or(Decimal::ZERO);
            total = total
                .checked_add(child_balance)
                .ok_or(CalculationError::Overflow {
                    formula: "consolidation::recompute_rollups",
                })?;
        }
        balances.insert(code, total);
    }
    Ok(())
}
