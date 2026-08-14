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
///
/// # Examples
///
/// ```
/// use casiros_erp::ledger::account::{Account, AccountCode, AccountType, ChartOfAccounts};
/// use casiros_erp::ledger::consolidation::build_rollup_graph;
///
/// let mut chart = ChartOfAccounts::new();
/// chart
///     .register(Account {
///         code: AccountCode(1000),
///         name: "Total Assets".to_string(),
///         account_type: AccountType::Asset,
///         parent: None,
///     })
///     .unwrap();
/// chart
///     .register(Account {
///         code: AccountCode(1010),
///         name: "Cash".to_string(),
///         account_type: AccountType::Asset,
///         parent: Some(AccountCode(1000)),
///     })
///     .unwrap();
///
/// let graph = build_rollup_graph(&chart);
/// let order = graph.execution_order().unwrap();
/// assert_eq!(order.len(), 2);
/// let child_index = order.iter().position(|c| *c == AccountCode(1010)).unwrap();
/// let parent_index = order.iter().position(|c| *c == AccountCode(1000)).unwrap();
/// assert!(child_index < parent_index);
/// ```
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
///
/// # Examples
///
/// ```
/// use casiros_erp::ledger::account::{Account, AccountCode, AccountType, ChartOfAccounts};
/// use casiros_erp::ledger::consolidation::recompute_rollups;
/// use rust_decimal_macros::dec;
/// use std::collections::HashMap;
///
/// let mut chart = ChartOfAccounts::new();
/// chart
///     .register(Account {
///         code: AccountCode(1000),
///         name: "Total Assets".to_string(),
///         account_type: AccountType::Asset,
///         parent: None,
///     })
///     .unwrap();
/// chart
///     .register(Account {
///         code: AccountCode(1010),
///         name: "Cash".to_string(),
///         account_type: AccountType::Asset,
///         parent: Some(AccountCode(1000)),
///     })
///     .unwrap();
///
/// let mut balances = HashMap::new();
/// balances.insert(AccountCode(1010), dec!(500));
/// recompute_rollups(&chart, &mut balances).unwrap();
/// assert_eq!(balances[&AccountCode(1000)], dec!(500));
/// assert_eq!(balances[&AccountCode(1010)], dec!(500));
/// ```
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
