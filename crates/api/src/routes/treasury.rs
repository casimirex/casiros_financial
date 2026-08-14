//! `/api/v1/treasury/*` — cash forecasting, FX conversion, and hedge effectiveness.

use crate::error::AppError;
use crate::persistence::treasury;
use crate::state::AppState;
use actix_web::{HttpResponse, web};
use casiros_core::types::Dollar;
use casiros_erp::treasury::cashflow::CashFlowItem;
use casiros_erp::treasury::fx::{ExchangeRate, FxExposure};
use casiros_erp::treasury::hedge::{hedge_effectiveness, is_highly_effective};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use utoipa::{IntoParams, ToSchema};

/// Adds an item to the cash flow forecast.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
#[utoipa::path(
    post,
    path = "/api/v1/treasury/cashflow/items",
    request_body = CashFlowItem,
    responses((status = 201, description = "Item added", body = CashFlowItem)),
    tag = "treasury"
)]
#[instrument(name = "POST /treasury/cashflow/items", skip(state, item))]
pub async fn add_cashflow_item(
    state: web::Data<AppState>,
    item: web::Json<CashFlowItem>,
) -> Result<HttpResponse, AppError> {
    let item = item.into_inner();
    treasury::add_item(&state.pool, &item).await?;
    info!(date = %item.date, "cash flow item added");
    Ok(HttpResponse::Created().json(item))
}

/// Query parameters for `GET /api/v1/treasury/cashflow/projection`.
#[derive(Debug, Deserialize, IntoParams)]
pub struct ProjectionQuery {
    /// The starting cash balance.
    #[param(value_type = Decimal)]
    pub opening_balance: Dollar,
    /// The date to project the balance forward to.
    pub as_of: NaiveDate,
}

/// The response body for `GET /api/v1/treasury/cashflow/projection`.
#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectionResponse {
    /// The projected cash balance.
    #[schema(value_type = Decimal)]
    pub balance: Dollar,
}

/// Projects the cash balance forward to `as_of`.
///
/// # Errors
///
/// Returns [`AppError::Calculation`] (422) if a running total overflows.
#[utoipa::path(
    get,
    path = "/api/v1/treasury/cashflow/projection",
    params(ProjectionQuery),
    responses((status = 200, description = "The projected balance", body = ProjectionResponse)),
    tag = "treasury"
)]
#[instrument(name = "GET /treasury/cashflow/projection", skip(state))]
pub async fn projection(
    state: web::Data<AppState>,
    query: web::Query<ProjectionQuery>,
) -> Result<HttpResponse, AppError> {
    let forecast = treasury::load_forecast(&state.pool).await?;
    let balance = forecast.projected_balance(query.opening_balance, query.as_of)?;
    Ok(HttpResponse::Ok().json(ProjectionResponse { balance }))
}

/// Query parameters for `GET /api/v1/treasury/cashflow/shortfall`.
#[derive(Debug, Deserialize, IntoParams)]
pub struct ShortfallQuery {
    /// The starting cash balance.
    #[param(value_type = Decimal)]
    pub opening_balance: Dollar,
}

/// The response body for `GET /api/v1/treasury/cashflow/shortfall`.
#[derive(Debug, Serialize, ToSchema)]
pub struct ShortfallResponse {
    /// The earliest date the running balance would go negative, if any.
    pub shortfall_date: Option<NaiveDate>,
}

/// Finds the earliest date the cash balance would go negative, if any.
///
/// # Errors
///
/// Returns [`AppError::Calculation`] (422) if a running total overflows.
#[utoipa::path(
    get,
    path = "/api/v1/treasury/cashflow/shortfall",
    params(ShortfallQuery),
    responses((status = 200, description = "The first shortfall date, if any", body = ShortfallResponse)),
    tag = "treasury"
)]
#[instrument(name = "GET /treasury/cashflow/shortfall", skip(state))]
pub async fn shortfall(
    state: web::Data<AppState>,
    query: web::Query<ShortfallQuery>,
) -> Result<HttpResponse, AppError> {
    let forecast = treasury::load_forecast(&state.pool).await?;
    let shortfall_date = forecast.first_shortfall_date(query.opening_balance)?;
    Ok(HttpResponse::Ok().json(ShortfallResponse { shortfall_date }))
}

/// The request body for `POST /api/v1/treasury/fx/convert`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ConvertRequest {
    /// The foreign-currency-denominated balance to convert.
    pub exposure: FxExposure,
    /// The exchange rate to convert at.
    pub rate: ExchangeRate,
}

/// The response body for `POST /api/v1/treasury/fx/convert`.
#[derive(Debug, Serialize, ToSchema)]
pub struct ConvertResponse {
    /// The converted amount, in `rate.to`.
    #[schema(value_type = Decimal)]
    pub converted: Dollar,
}

/// Converts an FX exposure to another currency at a given rate. Pure —
/// stateless, no shared application state involved.
///
/// # Errors
///
/// Returns [`AppError::Erp`] (400) if `rate.from` does not match
/// `exposure.currency`.
#[utoipa::path(
    post,
    path = "/api/v1/treasury/fx/convert",
    request_body = ConvertRequest,
    responses(
        (status = 200, description = "The converted amount", body = ConvertResponse),
        (status = 400, description = "Currency mismatch between exposure and rate"),
    ),
    tag = "treasury"
)]
#[instrument(name = "POST /treasury/fx/convert", skip(request))]
pub async fn convert(request: web::Json<ConvertRequest>) -> Result<HttpResponse, AppError> {
    let request = request.into_inner();
    let converted = request.exposure.convert(&request.rate)?;
    Ok(HttpResponse::Ok().json(ConvertResponse { converted }))
}

/// The request body for `POST /api/v1/treasury/hedge/effectiveness`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct HedgeEffectivenessRequest {
    /// The hedge instrument's gain (positive) or loss (negative).
    pub hedge_gain_loss: Decimal,
    /// The underlying exposure's gain (positive) or loss (negative).
    pub exposure_gain_loss: Decimal,
}

/// The response body for `POST /api/v1/treasury/hedge/effectiveness`.
#[derive(Debug, Serialize, ToSchema)]
pub struct HedgeEffectivenessResponse {
    /// The dollar-offset effectiveness ratio.
    pub effectiveness: Decimal,
    /// Whether `effectiveness` falls within the 80%-125% dollar-offset range.
    pub highly_effective: bool,
}

/// Computes hedge effectiveness and whether it qualifies as highly effective.
/// Pure — stateless, no shared application state involved.
///
/// # Errors
///
/// Returns [`AppError::Calculation`] (422) if `exposure_gain_loss` is zero.
#[utoipa::path(
    post,
    path = "/api/v1/treasury/hedge/effectiveness",
    request_body = HedgeEffectivenessRequest,
    responses((status = 200, description = "The effectiveness ratio and qualification", body = HedgeEffectivenessResponse)),
    tag = "treasury"
)]
#[instrument(name = "POST /treasury/hedge/effectiveness", skip(request))]
pub async fn hedge_effectiveness_handler(
    request: web::Json<HedgeEffectivenessRequest>,
) -> Result<HttpResponse, AppError> {
    let request = request.into_inner();
    let effectiveness = hedge_effectiveness(request.hedge_gain_loss, request.exposure_gain_loss)?;
    let highly_effective = is_highly_effective(effectiveness);
    Ok(HttpResponse::Ok().json(HedgeEffectivenessResponse {
        effectiveness,
        highly_effective,
    }))
}
