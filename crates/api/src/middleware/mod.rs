//! HTTP middleware support: request-id propagation and rate limiting.
//!
//! The actual `App::wrap_fn` closures live in `main.rs`, since Actix's
//! `wrap_fn` signature is easiest to satisfy with an inline closure; these
//! modules hold the reusable logic those closures call into.

pub mod rate_limit;
pub mod tracing;
