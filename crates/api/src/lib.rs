//! CASIROS API — the Actix-Web application logic. `main.rs` is a thin binary
//! entry point that calls into this library, which exists mainly so that
//! `tests/` (a separate compilation unit that can only link against a
//! library target) can exercise the real route handlers end-to-end.
//!
//! Every route in `CASIROS_BUILD_PROMPT.md` section 7.2 is implemented:
//! `/healthz`, `/api/v1/calculate/{formula}`, `/api/v1/simulate`,
//! `/ws/simulate`, and the ERP CRUD groups (`/api/v1/ledger/*`,
//! `/api/v1/journal/*`, `/api/v1/ap/*`, `/api/v1/ar/*`, `/api/v1/treasury/*`).
//! Interactive API documentation is served at `/swagger-ui/` (generated
//! from `ApiDoc`, in `routes`), backed by the raw `OpenAPI` document at
//! `/api-docs/openapi.json`.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![deny(warnings)]

pub mod error;
pub mod middleware;
pub mod routes;
pub mod state;
