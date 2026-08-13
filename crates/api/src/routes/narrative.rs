//! `POST /api/v1/narrative` — CFO-style memo generation from ERP metrics.

use crate::error::AppError;
use actix_web::{HttpResponse, web};
use casiros_erp::narrative::{NarrativeInputs, generate_narrative};
use serde::Serialize;
use tracing::{info, instrument};
use utoipa::ToSchema;

/// The response body for `POST /api/v1/narrative`.
#[derive(Debug, Serialize, ToSchema)]
pub struct NarrativeResponse {
    /// The generated markdown memo.
    pub memo: String,
}

/// Generates a CFO-style markdown analysis memo from whichever metrics the
/// caller supplies. Absent metrics are simply omitted from the memo.
///
/// # Errors
///
/// This handler is infallible; it always returns `Ok`.
#[utoipa::path(
    post,
    path = "/api/v1/narrative",
    request_body = NarrativeInputs,
    responses((status = 200, description = "The generated memo", body = NarrativeResponse)),
    tag = "narrative"
)]
#[instrument(name = "POST /narrative", skip(inputs))]
pub async fn generate(inputs: web::Json<NarrativeInputs>) -> Result<HttpResponse, AppError> {
    let inputs = inputs.into_inner();
    let memo = generate_narrative(&inputs);
    info!(company = %inputs.company, "narrative generated");
    Ok(HttpResponse::Ok().json(NarrativeResponse { memo }))
}
