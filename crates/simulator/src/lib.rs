//! # CASIROS Multiverse Simulator
//!
//! Monte Carlo scenario generation and aggregation over [`casiros_core`] formulas.
//! Every field in [`universe::UniverseMetrics`] is produced by exactly one
//! `casiros_core` function call — this crate composes the domain layer, it never
//! reimplements it.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![deny(warnings)]
// See crates/core/src/lib.rs for why: LaTeX-free here, but doc comments below
// reference formula names like `WACC`/`ROE` that doc_markdown also misreads.
#![allow(clippy::doc_markdown)]

pub mod aggregation;
pub mod monte_carlo;
pub mod universe;
