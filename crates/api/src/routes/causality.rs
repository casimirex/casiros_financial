//! `/api/v1/causality/*` — introspection over `casiros-dag`'s formula
//! dependency graph: for any formula, which of its parameters are raw inputs
//! versus another formula's output (by the exact-name wiring convention
//! `casiros_dag::evaluator::resolve` uses), and the transitive evaluation
//! order that follows from it. Pure — stateless, no shared application state
//! involved; every answer is derived from `FormulaNode` alone.

use crate::error::AppError;
use actix_web::{HttpResponse, web};
use casiros_dag::graph::{FormulaNode, transitive_dependencies};
use serde::Serialize;
use tracing::instrument;
use utoipa::ToSchema;

/// One parameter of a [`FormulaNode`], and — if its name matches another
/// formula's output — which formula would supply it.
#[derive(Debug, Serialize, ToSchema)]
pub struct ParameterInfo {
    /// The parameter's name, as `casiros_dag::evaluator::resolve` looks it up.
    pub name: String,
    /// The formula that would supply this parameter's value, if its name
    /// matches one. `None` means it must be supplied as a raw input.
    pub upstream_formula: Option<FormulaNode>,
}

/// A [`FormulaNode`] together with its resolved parameter wiring.
#[derive(Debug, Serialize, ToSchema)]
pub struct FormulaMetadata {
    /// The formula this metadata describes.
    pub formula: FormulaNode,
    /// Every parameter this formula's evaluator resolves, and how.
    pub parameters: Vec<ParameterInfo>,
}

fn describe(formula: FormulaNode) -> FormulaMetadata {
    FormulaMetadata {
        formula,
        parameters: formula
            .parameters()
            .iter()
            .map(|&name| ParameterInfo {
                name: name.to_string(),
                upstream_formula: FormulaNode::from_name(name).filter(|&node| node != formula),
            })
            .collect(),
    }
}

/// Every formula `casiros-dag` knows, with its full parameter wiring — the
/// static catalog a UI can browse without picking one first.
///
/// # Errors
///
/// This handler is infallible; it always returns `Ok`.
#[utoipa::path(
    get,
    path = "/api/v1/causality/formulas",
    responses((status = 200, description = "Every formula's parameter wiring", body = Vec<FormulaMetadata>)),
    tag = "causality"
)]
#[instrument(name = "GET /causality/formulas")]
pub async fn list_formulas() -> Result<HttpResponse, AppError> {
    let catalog: Vec<FormulaMetadata> = FormulaNode::all().iter().copied().map(describe).collect();
    Ok(HttpResponse::Ok().json(catalog))
}

/// The response body for `GET /api/v1/causality/formulas/{name}`.
#[derive(Debug, Serialize, ToSchema)]
pub struct DependencyGraphResponse {
    /// The formula the graph is rooted at.
    pub target: FormulaNode,
    /// `target` plus every formula transitively reachable through its
    /// upstream dependencies, each with its own parameter wiring.
    pub nodes: Vec<FormulaMetadata>,
    /// A valid evaluation order over `nodes`, via
    /// [`casiros_dag::graph::CausalityEngine::execution_order`]: every node
    /// appears after all the nodes it depends on.
    pub execution_order: Vec<FormulaNode>,
}

/// The transitive dependency graph rooted at the formula named in the path,
/// plus a valid evaluation order over it.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the path segment does not name a known formula.
#[utoipa::path(
    get,
    path = "/api/v1/causality/formulas/{name}",
    params(("name" = String, Path, description = "The formula's snake_case name, e.g. \"dupont_roe\"")),
    responses(
        (status = 200, description = "The transitive dependency graph and its evaluation order", body = DependencyGraphResponse),
        (status = 404, description = "Unknown formula name"),
    ),
    tag = "causality"
)]
#[instrument(name = "GET /causality/formulas/{name}")]
pub async fn formula_graph(path: web::Path<String>) -> Result<HttpResponse, AppError> {
    let name = path.into_inner();
    let target = FormulaNode::from_name(&name)
        .ok_or_else(|| AppError::NotFound(format!("unknown formula '{name}'")))?;

    let engine = transitive_dependencies(target);
    let execution_order = engine
        .execution_order()
        .map_err(|details| AppError::Internal(format!("cyclic formula dependency: {details}")))?;
    let nodes = execution_order.iter().copied().map(describe).collect();

    Ok(HttpResponse::Ok().json(DependencyGraphResponse {
        target,
        nodes,
        execution_order,
    }))
}

#[cfg(test)]
mod tests {
    use casiros_dag::graph::FormulaNode;

    #[test]
    fn covers_every_formula_node() {
        assert_eq!(FormulaNode::all().len(), 44);
    }
}
