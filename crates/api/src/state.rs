//! Shared application state for the ERP routes.
//!
//! Ledger/journal, AP, AR, and treasury state are all Postgres-backed (via
//! `pool`, see `crate::persistence`) as of Phase 9. `budget_model` stays
//! in-memory by design (out of Phase 9's scope — see `ROADMAP.md`).

use casiros_erp::budget::model::BudgetModel;
use sqlx::PgPool;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// Shared application state, mounted once per server and cloned (cheaply —
/// `PgPool` and `web::Data`'s internal `Arc` are both cheap to clone) into
/// every worker.
pub struct AppState {
    /// The Postgres connection pool backing the ledger, journal, AP, AR, and treasury.
    pub pool: PgPool,
    /// The driver-based budget model: named drivers plus the line items computed from them.
    pub budget_model: Mutex<BudgetModel>,
}

impl AppState {
    /// Creates application state backed by `pool`. Budget starts empty;
    /// every other entity's data lives in whatever `pool` already has
    /// migrated into it.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
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
