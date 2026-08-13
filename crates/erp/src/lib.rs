//! # CASIROS ERP
//!
//! Enterprise logic built on the [`casiros_core`] domain layer and the
//! [`casiros_dag`] causality engine. All six ERP subsystems from the CASIROS
//! build prompt are implemented: the causal general ledger (`ledger`), the
//! pure-function business-rule pattern (`business_rules`), accounts payable
//! (`ap`), accounts receivable (`ar`), treasury (`treasury`), tax (`tax`),
//! and budget (`budget`).

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![deny(warnings)]
// See crates/core/src/lib.rs: LaTeX-free here, but "# Errors" sections and
// formula names like `WACC` trip the same misreading of backtick-free prose.
#![allow(clippy::doc_markdown)]

pub mod ap;
pub mod ar;
pub mod budget;
pub mod business_rules;
pub mod error;
pub mod ledger;
pub mod tax;
pub mod treasury;
