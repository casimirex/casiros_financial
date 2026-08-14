//! Postgres-backed storage for the ERP's ledger, AP, AR, and treasury state.
//!
//! Every function here stores or retrieves the *same* domain types
//! `casiros_erp` already defines and validates (`Account`, `JournalEntry`,
//! `Supplier`, `ApInvoice`, `Customer`, `ArInvoice`, `CashFlowItem`, ...) —
//! this module only changes where they live, never how they're constructed
//! or validated. `casiros_erp` itself has no dependency on `sqlx` or any
//! other I/O crate; every Postgres row <-> domain-type conversion happens
//! here, via manual `From`/`TryFrom` glue or inline mapping, not a derive on
//! the domain type.

pub mod ap;
pub mod ar;
pub mod db;
pub mod ledger;
pub mod treasury;
