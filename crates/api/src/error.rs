//! The error type returned by every API handler.

use actix_web::{HttpResponse, ResponseError, http::StatusCode};
use casiros_core::error::CalculationError;
use casiros_erp::error::ErpError;
use serde::Serialize;

/// The error type returned by every API handler.
///
/// Every variant maps to an HTTP status via [`ResponseError::status_code`]
/// and is rendered as a JSON body via [`ResponseError::error_response`].
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// The request body or path was malformed (e.g. an un-parseable decimal).
    #[error("bad request: {0}")]
    BadRequest(String),

    /// The requested resource (e.g. an unknown formula name) does not exist.
    #[error("not found: {0}")]
    NotFound(String),

    /// A `casiros_core` or `casiros_dag` computation failed.
    #[error(transparent)]
    Calculation(#[from] CalculationError),

    /// A `casiros_erp` operation failed.
    #[error(transparent)]
    Erp(#[from] ErpError),

    /// An invariant the handler expected was violated; this indicates a bug
    /// rather than a client error.
    #[error("internal error: {0}")]
    Internal(String),
}

/// The JSON shape every error response takes.
#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

/// Maps one [`ErpError`] variant to the HTTP status that best fits it,
/// rather than collapsing every ERP failure onto a single status code.
fn erp_status_code(err: &ErpError) -> StatusCode {
    match err {
        ErpError::DuplicateAccount(_) | ErpError::PeriodClosed(_) => StatusCode::CONFLICT,
        ErpError::UnknownAccount(_) | ErpError::UnknownDriver(_) => StatusCode::NOT_FOUND,
        ErpError::UnbalancedEntry { .. }
        | ErpError::InvalidLine(_)
        | ErpError::InvalidRecognitionPeriod { .. }
        | ErpError::InvalidCurrencyCode(_)
        | ErpError::InvalidTaxBrackets(_)
        | ErpError::PaymentExceedsBalance { .. }
        | ErpError::CurrencyMismatch { .. } => StatusCode::BAD_REQUEST,
        // A cyclic hierarchy or an underlying calculation failure reflects
        // the current *state* being unprocessable, not a malformed request.
        ErpError::CyclicHierarchy(_) | ErpError::Calculation(_) => StatusCode::UNPROCESSABLE_ENTITY,
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            // A CalculationError surfacing at the API boundary means the
            // caller supplied inputs that violate a formula's domain
            // preconditions (e.g. a zero denominator) — that's a client error.
            Self::Calculation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Erp(err) => erp_status_code(err),
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(ErrorBody {
            error: self.to_string(),
        })
    }
}
