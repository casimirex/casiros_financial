//! # CASIROS Causality Engine
//!
//! A directed acyclic graph over [`casiros_core`] formulas. An edge `A -> B` means
//! "B depends on A": A must be evaluated first, and A's result is available for B
//! to consume. Cycle detection is a hard error — a financial model that depends on
//! itself is not computable, and this crate refuses to pretend otherwise.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![deny(warnings)]

pub mod evaluator;
pub mod graph;
