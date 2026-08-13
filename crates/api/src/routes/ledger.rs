//! `/api/v1/ledger/*` — chart of accounts and balances.

use crate::error::AppError;
use crate::state::{AppState, lock};
use actix_web::{HttpResponse, web};
use casiros_core::types::Dollar;
use casiros_erp::ledger::account::{Account, AccountCode};
use serde::Serialize;
use tracing::{error, info, instrument};
use utoipa::ToSchema;

/// One account's current balance.
#[derive(Debug, Serialize, ToSchema)]
pub struct AccountBalance {
    /// The account's code.
    pub code: AccountCode,
    /// The account's current balance.
    #[schema(value_type = Decimal)]
    pub balance: Dollar,
}

/// Registers a new account in the chart of accounts.
///
/// # Errors
///
/// Returns [`AppError::Erp`] (409) if the code is already registered, or
/// (404) if `account.parent` is set but not itself registered.
#[utoipa::path(
    post,
    path = "/api/v1/ledger/accounts",
    request_body = Account,
    responses(
        (status = 201, description = "Account registered", body = Account),
        (status = 404, description = "Parent account not registered"),
        (status = 409, description = "Account code already registered"),
    ),
    tag = "ledger"
)]
#[instrument(name = "POST /ledger/accounts", skip(state, account))]
pub async fn register_account(
    state: web::Data<AppState>,
    account: web::Json<Account>,
) -> Result<HttpResponse, AppError> {
    let account = account.into_inner();
    let mut ledger = lock(&state.ledger);
    ledger.register_account(account.clone())?;
    drop(ledger);
    info!(code = ?account.code, "account registered");
    Ok(HttpResponse::Created().json(account))
}

/// Lists every registered account.
///
/// # Errors
///
/// This handler is infallible; it always returns `Ok`.
#[utoipa::path(
    get,
    path = "/api/v1/ledger/accounts",
    responses((status = 200, description = "Every registered account", body = Vec<Account>)),
    tag = "ledger"
)]
#[instrument(name = "GET /ledger/accounts", skip(state))]
pub async fn list_accounts(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let ledger = lock(&state.ledger);
    let accounts: Vec<Account> = ledger.chart().accounts().cloned().collect();
    drop(ledger);
    info!(count = accounts.len(), "listed accounts");
    Ok(HttpResponse::Ok().json(accounts))
}

/// Gets one account by code.
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if no account has that code.
#[utoipa::path(
    get,
    path = "/api/v1/ledger/accounts/{code}",
    params(("code" = u32, Path, description = "The account code")),
    responses(
        (status = 200, description = "The account", body = Account),
        (status = 404, description = "No account with that code"),
    ),
    tag = "ledger"
)]
#[instrument(name = "GET /ledger/accounts/{code}", skip(state))]
pub async fn get_account(
    state: web::Data<AppState>,
    code: web::Path<u32>,
) -> Result<HttpResponse, AppError> {
    let code = AccountCode(code.into_inner());
    let ledger = lock(&state.ledger);
    let account = ledger.chart().get(code).cloned().ok_or_else(|| {
        error!(?code, "account not found");
        AppError::NotFound(format!("no account with code {}", code.0))
    })?;
    drop(ledger);
    Ok(HttpResponse::Ok().json(account))
}

/// Gets one account's current balance.
///
/// # Errors
///
/// Returns [`AppError::Erp`] (404) if no account has that code, or (422) if
/// the roll-up hierarchy contains a cycle.
#[utoipa::path(
    get,
    path = "/api/v1/ledger/accounts/{code}/balance",
    params(("code" = u32, Path, description = "The account code")),
    responses(
        (status = 200, description = "The account's balance", body = AccountBalance),
        (status = 404, description = "No account with that code"),
    ),
    tag = "ledger"
)]
#[instrument(name = "GET /ledger/accounts/{code}/balance", skip(state))]
pub async fn get_account_balance(
    state: web::Data<AppState>,
    code: web::Path<u32>,
) -> Result<HttpResponse, AppError> {
    let code = AccountCode(code.into_inner());
    let mut ledger = lock(&state.ledger);
    let balance = ledger.balance(code)?;
    drop(ledger);
    info!(?code, %balance, "balance computed");
    Ok(HttpResponse::Ok().json(AccountBalance { code, balance }))
}

/// Gets the full trial balance: every account's current balance.
///
/// # Errors
///
/// Returns [`AppError::Erp`] (422) if the roll-up hierarchy contains a cycle.
#[utoipa::path(
    get,
    path = "/api/v1/ledger/trial-balance",
    responses((status = 200, description = "Every account's current balance", body = Vec<AccountBalance>)),
    tag = "ledger"
)]
#[instrument(name = "GET /ledger/trial-balance", skip(state))]
pub async fn trial_balance(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let mut ledger = lock(&state.ledger);
    let balances = ledger.trial_balance()?;
    drop(ledger);
    let response: Vec<AccountBalance> = balances
        .into_iter()
        .map(|(code, balance)| AccountBalance { code, balance })
        .collect();
    info!(count = response.len(), "trial balance computed");
    Ok(HttpResponse::Ok().json(response))
}
