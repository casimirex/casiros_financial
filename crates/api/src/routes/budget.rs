//! `/api/v1/budget/*` — driver-based budget planning and variance analysis.
//!
//! Drivers and line items accumulate in shared [`crate::state::AppState`]
//! (mirroring treasury's cash forecast), since a budget model is built up
//! incrementally across several requests. Variance analysis itself is pure —
//! stateless, comparing whatever budget/actual figures the caller supplies.

use crate::error::AppError;
use crate::state::{AppState, lock};
use actix_web::{HttpResponse, web};
use casiros_core::types::Dollar;
use casiros_erp::budget::model::DriverBasedLineItem;
use casiros_erp::budget::variance::{VarianceResult, variance_report};
use casiros_erp::ledger::account::{AccountCode, AccountType};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use utoipa::ToSchema;

/// The request body for `POST /api/v1/budget/drivers`.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct SetDriverRequest {
    /// The driver's name (e.g. `"units_sold"`).
    pub name: String,
    /// The driver's value.
    pub value: Decimal,
}

/// Sets (or overwrites) a named driver's value.
///
/// # Errors
///
/// This handler is infallible; it always returns `Ok`.
#[utoipa::path(
    post,
    path = "/api/v1/budget/drivers",
    request_body = SetDriverRequest,
    responses((status = 201, description = "Driver set", body = SetDriverRequest)),
    tag = "budget"
)]
#[instrument(name = "POST /budget/drivers", skip(state, request))]
pub async fn set_driver(
    state: web::Data<AppState>,
    request: web::Json<SetDriverRequest>,
) -> Result<HttpResponse, AppError> {
    let request = request.into_inner();
    lock(&state.budget_model).set_driver(request.name.clone(), request.value);
    info!(driver = %request.name, value = %request.value, "budget driver set");
    Ok(HttpResponse::Created().json(request))
}

/// The response body for `GET /api/v1/budget/drivers/{name}`.
#[derive(Debug, Serialize, ToSchema)]
pub struct DriverResponse {
    /// The driver's current value.
    pub value: Decimal,
}

/// Looks up a named driver's current value.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if no driver with that name has been set.
#[utoipa::path(
    get,
    path = "/api/v1/budget/drivers/{name}",
    params(("name" = String, Path, description = "The driver's name")),
    responses(
        (status = 200, description = "The driver's value", body = DriverResponse),
        (status = 404, description = "No driver with that name has been set"),
    ),
    tag = "budget"
)]
#[instrument(name = "GET /budget/drivers/{name}", skip(state))]
pub async fn get_driver(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let name = path.into_inner();
    let value = lock(&state.budget_model)
        .driver(&name)
        .ok_or_else(|| AppError::NotFound(format!("no driver named '{name}'")))?;
    Ok(HttpResponse::Ok().json(DriverResponse { value }))
}

/// Adds a line item to the budget model.
///
/// # Errors
///
/// This handler is infallible; it always returns `Ok`.
#[utoipa::path(
    post,
    path = "/api/v1/budget/line-items",
    request_body = DriverBasedLineItem,
    responses((status = 201, description = "Line item added", body = DriverBasedLineItem)),
    tag = "budget"
)]
#[instrument(name = "POST /budget/line-items", skip(state, item))]
pub async fn add_line_item(
    state: web::Data<AppState>,
    item: web::Json<DriverBasedLineItem>,
) -> Result<HttpResponse, AppError> {
    let item = item.into_inner();
    lock(&state.budget_model).add_line_item(item.clone());
    info!(description = %item.description, "budget line item added");
    Ok(HttpResponse::Created().json(item))
}

/// Lists every line item in the budget model, in insertion order.
///
/// # Errors
///
/// This handler is infallible; it always returns `Ok`.
#[utoipa::path(
    get,
    path = "/api/v1/budget/line-items",
    responses((status = 200, description = "Every budget line item", body = Vec<DriverBasedLineItem>)),
    tag = "budget"
)]
#[instrument(name = "GET /budget/line-items", skip(state))]
pub async fn list_line_items(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let items = lock(&state.budget_model).line_items().to_vec();
    Ok(HttpResponse::Ok().json(items))
}

/// The response body for `GET /api/v1/budget/total`.
#[derive(Debug, Serialize, ToSchema)]
pub struct TotalBudgetResponse {
    /// The sum of every line item's computed amount.
    #[schema(value_type = Decimal)]
    pub total: Dollar,
}

/// Computes the total budget: the sum of every line item's computed amount.
///
/// # Errors
///
/// Returns [`AppError::Erp`] (404) if a line item references an unset
/// driver. Returns [`AppError::Calculation`] (422) if a product or the total
/// overflows.
#[utoipa::path(
    get,
    path = "/api/v1/budget/total",
    responses((status = 200, description = "The total budget", body = TotalBudgetResponse)),
    tag = "budget"
)]
#[instrument(name = "GET /budget/total", skip(state))]
pub async fn total(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let total = lock(&state.budget_model).total_budget()?;
    Ok(HttpResponse::Ok().json(TotalBudgetResponse { total }))
}

/// One account's budget-versus-actual comparison, for variance analysis.
#[derive(Debug, Deserialize, ToSchema)]
pub struct VarianceEntry {
    /// The account this entry applies to.
    pub account: AccountCode,
    /// The account's type, which determines what "favorable" means.
    pub account_type: AccountType,
    /// The budgeted amount.
    pub budget: Decimal,
    /// The actual amount.
    pub actual: Decimal,
}

/// The request body for `POST /api/v1/budget/variance`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct VarianceRequest {
    /// Every account's budget-versus-actual entry to compare.
    pub entries: Vec<VarianceEntry>,
}

/// Compares budgeted to actual amounts across several accounts at once,
/// classifying each variance as favorable or unfavorable. Pure — stateless,
/// no shared application state involved.
///
/// # Errors
///
/// Returns [`AppError::Calculation`] (422) if a variance or its percentage overflows.
#[utoipa::path(
    post,
    path = "/api/v1/budget/variance",
    request_body = VarianceRequest,
    responses((status = 200, description = "The variance result for each entry", body = Vec<VarianceResult>)),
    tag = "budget"
)]
#[instrument(name = "POST /budget/variance", skip(request))]
pub async fn variance(request: web::Json<VarianceRequest>) -> Result<HttpResponse, AppError> {
    let entries: Vec<(AccountCode, AccountType, Decimal, Decimal)> = request
        .into_inner()
        .entries
        .into_iter()
        .map(|e| (e.account, e.account_type, e.budget, e.actual))
        .collect();
    let report = variance_report(&entries)?;
    Ok(HttpResponse::Ok().json(report))
}
