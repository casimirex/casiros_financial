//! Shared application state for the ERP routes.
//!
//! Ledger/journal, AP, and AR state are Postgres-backed (via `pool`, see
//! `crate::persistence`) as of Phase 9. Treasury state is still in-memory
//! pending Phase 9's final step; `budget_model` stays in-memory by design
//! (out of Phase 9's scope — see `ROADMAP.md`).

use casiros_erp::budget::model::BudgetModel;
use casiros_erp::treasury::cashflow::CashForecast;
use sqlx::PgPool;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// Shared application state, mounted once per server and cloned (cheaply —
/// `PgPool` and `web::Data`'s internal `Arc` are both cheap to clone) into
/// every worker.
pub struct AppState {
    /// The Postgres connection pool backing the ledger, journal, AP, and AR.
    pub pool: PgPool,
    /// The treasury cash flow forecast.
    pub cash_forecast: Mutex<CashForecast>,
    /// The driver-based budget model: named drivers plus the line items computed from them.
    pub budget_model: Mutex<BudgetModel>,
}

impl AppState {
    /// Creates application state backed by `pool`. Treasury/budget start
    /// empty; ledger/journal/AP/AR data lives in whatever `pool` already
    /// has migrated into it.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            cash_forecast: Mutex::new(CashForecast::new()),
            budget_model: Mutex::new(BudgetModel::new()),
        }
    }
}

/// Locks `mutex`, recovering from poisoning rather than propagating a panic:
/// a single failed request should not permanently wedge shared state for
/// every subsequent request.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
