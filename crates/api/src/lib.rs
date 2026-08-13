//! CASIROS API — the Actix-Web application logic. `main.rs` is a thin binary
//! entry point that calls into this library, which exists mainly so that
//! `tests/` (a separate compilation unit that can only link against a
//! library target) can exercise the real route handlers end-to-end.
//!
//! Implemented (per `CASIROS_BUILD_PROMPT.md` section 7.2): `/healthz`,
//! `/api/v1/calculate/{formula}`, `/api/v1/simulate`, `/ws/simulate`. Not yet
//! implemented: the ERP CRUD routes (`ledger`, `journal`, `ap`, `ar`,
//! `treasury`) — see `routes` for why.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![deny(warnings)]

pub mod error;
pub mod middleware;
pub mod routes;
