//! `POST /api/v1/calculate/{formula}` — evaluate a single `casiros_core` formula.

use crate::error::AppError;
use actix_web::{HttpResponse, web};
use casiros_dag::evaluator::{EvaluationContext, evaluate_dag};
use casiros_dag::graph::{CausalityEngine, FormulaNode};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use tracing::{error, info, instrument};
use utoipa::ToSchema;

/// The request body for `POST /api/v1/calculate/{formula}`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CalculateRequest {
    /// The formula's named parameters, as decimal strings (e.g. `"1000.00"`).
    ///
    /// Only scalar-parameter formulas are supported here — `discounted_cash_flow`,
    /// `duration`, and `convexity` take a cash-flow series, which this flat
    /// string map cannot represent; requesting one of those three returns
    /// [`AppError::BadRequest`].
    pub params: HashMap<String, String>,
}

/// The response body for `POST /api/v1/calculate/{formula}`.
#[derive(Debug, Serialize, ToSchema)]
pub struct CalculateResponse {
    /// The formula that was evaluated.
    pub formula: String,
    /// The computed result. Serializes as a JSON string (never `f64`), via
    /// `rust_decimal`'s `serde-with-str` feature.
    pub result: Decimal,
}

const SERIES_FORMULAS: [FormulaNode; 3] = [
    FormulaNode::DiscountedCashFlow,
    FormulaNode::Duration,
    FormulaNode::Convexity,
];

/// Evaluates the formula named in the request path, using `req.params` as inputs.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if the path segment does not name a known
/// formula. Returns [`AppError::BadRequest`] if a parameter is not a valid
/// decimal, or the formula requires a cash-flow series. Returns
/// [`AppError::Calculation`] if the formula's own preconditions are violated
/// (e.g. a zero denominator).
#[utoipa::path(
    post,
    path = "/api/v1/calculate/{formula}",
    params(("formula" = String, Path, description = "The formula's snake_case name, e.g. \"future_value\"")),
    request_body = CalculateRequest,
    responses(
        (status = 200, description = "The computed result", body = CalculateResponse),
        (status = 400, description = "Invalid decimal parameter, or the formula needs a cash-flow series"),
        (status = 404, description = "Unknown formula name"),
        (status = 422, description = "A formula precondition was violated (e.g. a zero denominator)"),
    ),
    tag = "calculate"
)]
#[instrument(name = "POST /calculate", skip(req))]
pub async fn handle_calculate(
    path: web::Path<String>,
    req: web::Json<CalculateRequest>,
) -> Result<HttpResponse, AppError> {
    let formula_name = path.into_inner();
    let node = FormulaNode::from_name(&formula_name).ok_or_else(|| {
        error!(formula = %formula_name, "unknown formula");
        AppError::NotFound(format!("unknown formula '{formula_name}'"))
    })?;

    if SERIES_FORMULAS.contains(&node) {
        error!(formula = %formula_name, "formula requires a cash-flow series, unsupported via this endpoint");
        return Err(AppError::BadRequest(format!(
            "'{formula_name}' takes a cash-flow series; not supported via /calculate"
        )));
    }

    let mut ctx = EvaluationContext::new();
    for (key, value) in &req.params {
        let parsed = Decimal::from_str(value).map_err(|_| {
            error!(parameter = %key, value = %value, "invalid decimal parameter");
            AppError::BadRequest(format!("invalid decimal for parameter '{key}': '{value}'"))
        })?;
        ctx.inputs.insert(key.clone(), parsed);
    }

    let mut engine = CausalityEngine::new();
    engine.add_node(node);
    evaluate_dag(&engine, &mut ctx).map_err(|err| {
        error!(formula = %formula_name, error = %err, "calculation failed");
        AppError::from(err)
    })?;

    let result = ctx.results.get(&node).copied().ok_or_else(|| {
        AppError::Internal(format!("formula '{formula_name}' did not produce a result"))
    })?;

    info!(formula = %formula_name, result = %result, "calculation succeeded");
    Ok(HttpResponse::Ok().json(CalculateResponse {
        formula: formula_name,
        result,
    }))
}
