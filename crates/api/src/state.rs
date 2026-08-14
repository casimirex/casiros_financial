//! Shared, in-memory application state for the ERP routes.
//!
//! No persistence layer is wired up (matching the original build prompt's
//! `casiros-api` `Cargo.toml`, which lists no database crate either); state
//! lives for the lifetime of the process, guarded by a `Mutex` per entity
//! collection so handlers can be `Send`.

use casiros_erp::ap::invoice::{ApInvoice, ApInvoiceId};
use casiros_erp::ap::supplier::{Supplier, SupplierId};
use casiros_erp::ar::customer::{Customer, CustomerId};
use casiros_erp::ar::invoice::{ArInvoice, ArInvoiceId};
use casiros_erp::budget::model::BudgetModel;
use casiros_erp::ledger::Ledger;
use casiros_erp::ledger::account::ChartOfAccounts;
use casiros_erp::treasury::cashflow::CashForecast;
use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// Shared application state, mounted once per server and cloned (cheaply,
/// via `web::Data`'s internal `Arc`) into every worker.
pub struct AppState {
    /// The causal general ledger: chart of accounts, posted entries, balances.
    pub ledger: Mutex<Ledger>,
    /// Registered AP suppliers, keyed by id.
    pub suppliers: Mutex<HashMap<SupplierId, Supplier>>,
    /// Open and settled AP invoices, keyed by id.
    pub ap_invoices: Mutex<HashMap<ApInvoiceId, ApInvoice>>,
    /// Registered AR customers, keyed by id.
    pub customers: Mutex<HashMap<CustomerId, Customer>>,
    /// Open and settled AR invoices, keyed by id.
    pub ar_invoices: Mutex<HashMap<ArInvoiceId, ArInvoice>>,
    /// The treasury cash flow forecast.
    pub cash_forecast: Mutex<CashForecast>,
    /// The driver-based budget model: named drivers plus the line items computed from them.
    pub budget_model: Mutex<BudgetModel>,
}

impl AppState {
    /// Creates empty application state: no accounts, no invoices, no forecast items.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ledger: Mutex::new(Ledger::new(ChartOfAccounts::new())),
            suppliers: Mutex::new(HashMap::new()),
            ap_invoices: Mutex::new(HashMap::new()),
            customers: Mutex::new(HashMap::new()),
            ar_invoices: Mutex::new(HashMap::new()),
            cash_forecast: Mutex::new(CashForecast::new()),
            budget_model: Mutex::new(BudgetModel::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Locks `mutex`, recovering from poisoning rather than propagating a panic:
/// a single failed request should not permanently wedge shared state for
/// every subsequent request.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}
