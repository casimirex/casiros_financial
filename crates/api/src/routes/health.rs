//! `GET /healthz` — liveness check.

use crate::error::AppError;
use actix_web::HttpResponse;
use serde::Serialize;
use tracing::{info, instrument};
use utoipa::ToSchema;

/// The JSON body returned by [`healthz`].
#[derive(Debug, Serialize, ToSchema)]
struct HealthResponse {
    status: &'static str,
}

/// Reports that the server is up. Always succeeds; still returns
/// `Result<HttpResponse, AppError>` per the handler pattern every route follows.
///
/// # Errors
///
/// This handler is infallible; it always returns `Ok`.
#[utoipa::path(get, path = "/healthz", responses((status = 200, description = "The server is up", body = HealthResponse)), tag = "health")]
#[instrument(name = "GET /healthz")]
pub async fn healthz() -> Result<HttpResponse, AppError> {
    info!("health check ok");
    Ok(HttpResponse::Ok().json(HealthResponse { status: "ok" }))
}
