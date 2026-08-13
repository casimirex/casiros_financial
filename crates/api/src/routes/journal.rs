//! `/api/v1/journal/*` — posting and listing journal entries.

use crate::error::AppError;
use crate::state::{AppState, lock};
use actix_web::{HttpResponse, web};
use casiros_erp::ledger::journal::{JournalEntry, JournalLine, SourceDocument};
use casiros_erp::ledger::period::FiscalPeriod;
use chrono::NaiveDate;
use serde::Deserialize;
use tracing::{error, info, instrument};
use utoipa::ToSchema;
use uuid::Uuid;

/// The request body for `POST /api/v1/journal/entries`.
///
/// `lines` are re-validated through [`JournalLine::new`] rather than trusted
/// as already-valid deserialized data — a client could otherwise construct a
/// [`JournalLine`] JSON object with both `debit` and `credit` set, since
/// `JournalLine`'s fields are public and its `Deserialize` impl does not run
/// the same checks its constructor does.
#[derive(Debug, Deserialize, ToSchema)]
pub struct PostJournalEntryRequest {
    /// The posting date.
    pub date: NaiveDate,
    /// A human-readable description.
    pub description: String,
    /// The debit/credit lines.
    pub lines: Vec<JournalLine>,
    /// The entry that causally produced this one, if any.
    pub causal_parent: Option<Uuid>,
    /// What kind of business document originated this entry.
    pub source_document: SourceDocument,
    /// The fiscal period this entry posts into.
    pub period: FiscalPeriod,
}

/// Posts a new journal entry to the ledger.
///
/// # Errors
///
/// Returns [`AppError::Erp`] (400) if `lines` is empty, contains an invalid
/// line, or does not balance; (404) if a line references an unregistered
/// account; (409) if the target period is closed.
#[utoipa::path(
    post,
    path = "/api/v1/journal/entries",
    request_body = PostJournalEntryRequest,
    responses(
        (status = 201, description = "Entry posted", body = JournalEntry),
        (status = 400, description = "Empty, invalid, or unbalanced entry"),
        (status = 404, description = "A line references an unregistered account"),
        (status = 409, description = "The target fiscal period is closed"),
    ),
    tag = "journal"
)]
#[instrument(name = "POST /journal/entries", skip(state, request))]
pub async fn post_entry(
    state: web::Data<AppState>,
    request: web::Json<PostJournalEntryRequest>,
) -> Result<HttpResponse, AppError> {
    let request = request.into_inner();
    let mut validated_lines = Vec::with_capacity(request.lines.len());
    for line in request.lines {
        validated_lines.push(JournalLine::new(
            line.account,
            line.debit,
            line.credit,
            line.causal_formula,
        )?);
    }

    let entry = JournalEntry::new(
        request.date,
        request.description,
        validated_lines,
        request.causal_parent,
        request.source_document,
        request.period,
    )?;

    let mut ledger = lock(&state.ledger);
    ledger.post(entry.clone()).map_err(|err| {
        error!(entry_id = %entry.id, error = %err, "failed to post journal entry");
        err
    })?;
    drop(ledger);
    info!(entry_id = %entry.id, "journal entry posted");
    Ok(HttpResponse::Created().json(entry))
}

/// Lists every journal entry posted so far, in posting order.
///
/// # Errors
///
/// This handler is infallible; it always returns `Ok`.
#[utoipa::path(
    get,
    path = "/api/v1/journal/entries",
    responses((status = 200, description = "Every posted journal entry", body = Vec<JournalEntry>)),
    tag = "journal"
)]
#[instrument(name = "GET /journal/entries", skip(state))]
pub async fn list_entries(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let ledger = lock(&state.ledger);
    let entries: Vec<JournalEntry> = ledger.entries().to_vec();
    drop(ledger);
    info!(count = entries.len(), "listed journal entries");
    Ok(HttpResponse::Ok().json(entries))
}
