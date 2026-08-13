//! # CASIROS ERP
//!
//! Enterprise logic built on the [`casiros_core`] domain layer and the
//! [`casiros_dag`] causality engine. This crate is being built incrementally:
//! the causal general ledger (`ledger`) and the pure-function business-rule
//! pattern (`business_rules`) are complete; accounts payable, accounts
//! receivable, treasury, tax, and budget are follow-up work.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![deny(warnings)]
// See crates/core/src/lib.rs: LaTeX-free here, but "# Errors" sections and
// formula names like `WACC` trip the same misreading of backtick-free prose.
#![allow(clippy::doc_markdown)]

pub mod business_rules;
pub mod error;
pub mod ledger;
