//! `/api/v1/tax/*` — progressive tax calculation, multi-jurisdiction
//! aggregation, and deferred tax. Pure — stateless, no shared application
//! state involved; the caller sends the full jurisdiction (or temporary
//! difference) on every request.

use crate::error::AppError;
use actix_web::{HttpResponse, web};
use casiros_core::types::Dollar;
use casiros_erp::tax::calculation::{
    DeferredTaxPosition, TemporaryDifference, calculate_multi_jurisdiction_tax, calculate_tax,
};
use casiros_erp::tax::jurisdiction::TaxJurisdiction;
use serde::{Deserialize, Serialize};
use tracing::{info, instrument};
use utoipa::ToSchema;

/// The request body for `POST /api/v1/tax/calculate`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CalculateTaxRequest {
    /// The jurisdiction whose bracket schedule to apply.
    pub jurisdiction: TaxJurisdiction,
    /// The income to compute tax on.
    #[schema(value_type = Decimal)]
    pub taxable_income: Dollar,
}

/// The response body for `POST /api/v1/tax/calculate`.
#[derive(Debug, Serialize, ToSchema)]
pub struct CalculateTaxResponse {
    /// The tax owed under `jurisdiction`'s progressive bracket schedule.
    #[schema(value_type = Decimal)]
    pub tax: Dollar,
}

/// Computes the marginal (progressive-bracket) tax owed on `taxable_income`.
///
/// # Errors
///
/// Returns [`AppError::Calculation`] (422) if `taxable_income` is negative or
/// an intermediate sum overflows.
#[utoipa::path(
    post,
    path = "/api/v1/tax/calculate",
    request_body = CalculateTaxRequest,
    responses((status = 200, description = "The computed tax", body = CalculateTaxResponse)),
    tag = "tax"
)]
#[instrument(name = "POST /tax/calculate", skip(request))]
pub async fn calculate(request: web::Json<CalculateTaxRequest>) -> Result<HttpResponse, AppError> {
    let request = request.into_inner();
    let tax = calculate_tax(&request.jurisdiction, request.taxable_income)?;
    info!(tax = %tax, "tax calculated");
    Ok(HttpResponse::Ok().json(CalculateTaxResponse { tax }))
}

/// One jurisdiction's income allocation, for multi-jurisdiction aggregation.
#[derive(Debug, Deserialize, ToSchema)]
pub struct JurisdictionAllocation {
    /// The jurisdiction whose bracket schedule to apply.
    pub jurisdiction: TaxJurisdiction,
    /// The income allocated to this jurisdiction.
    #[schema(value_type = Decimal)]
    pub taxable_income: Dollar,
}

/// The request body for `POST /api/v1/tax/multi-jurisdiction`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct MultiJurisdictionRequest {
    /// Every jurisdiction's income allocation (e.g. federal plus state).
    pub allocations: Vec<JurisdictionAllocation>,
}

/// The response body for `POST /api/v1/tax/multi-jurisdiction`.
#[derive(Debug, Serialize, ToSchema)]
pub struct MultiJurisdictionResponse {
    /// The total tax owed across every allocation.
    #[schema(value_type = Decimal)]
    pub total_tax: Dollar,
}

/// Computes total tax owed across several jurisdictions at once.
///
/// # Errors
///
/// Returns whatever [`calculate_tax`] returns for the first failing allocation.
#[utoipa::path(
    post,
    path = "/api/v1/tax/multi-jurisdiction",
    request_body = MultiJurisdictionRequest,
    responses((status = 200, description = "The total tax across all allocations", body = MultiJurisdictionResponse)),
    tag = "tax"
)]
#[instrument(name = "POST /tax/multi-jurisdiction", skip(request))]
pub async fn multi_jurisdiction(
    request: web::Json<MultiJurisdictionRequest>,
) -> Result<HttpResponse, AppError> {
    let request = request.into_inner();
    let allocations: Vec<(&TaxJurisdiction, Dollar)> = request
        .allocations
        .iter()
        .map(|a| (&a.jurisdiction, a.taxable_income))
        .collect();
    let total_tax = calculate_multi_jurisdiction_tax(&allocations)?;
    info!(total_tax = %total_tax, jurisdictions = allocations.len(), "multi-jurisdiction tax calculated");
    Ok(HttpResponse::Ok().json(MultiJurisdictionResponse { total_tax }))
}

/// Computes the deferred tax position arising from a book-versus-tax basis difference.
///
/// # Errors
///
/// Returns [`AppError::Calculation`] (422) if the computation overflows.
#[utoipa::path(
    post,
    path = "/api/v1/tax/deferred-position",
    request_body = TemporaryDifference,
    responses((status = 200, description = "The resulting deferred tax position", body = DeferredTaxPosition)),
    tag = "tax"
)]
#[instrument(name = "POST /tax/deferred-position", skip(request))]
pub async fn deferred_position(
    request: web::Json<TemporaryDifference>,
) -> Result<HttpResponse, AppError> {
    let position = request.into_inner().deferred_tax_position()?;
    Ok(HttpResponse::Ok().json(position))
}
