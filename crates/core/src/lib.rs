//! # CASIROS Core Mathematics
//!
//! The fundamental, immutable financial formulas that form the computational backbone.
//!
//! ## Design Principles
//!
//! - **Purity:** Every public function is a pure computation. Same input → same output. No I/O.
//! - **Decimal Precision:** All monetary values use [`rust_decimal::Decimal`]. `f32`/`f64` is BANNED.
//! - **Defensive:** Every function validates preconditions and returns [`Result<T, CalculationError>`].
//! - **Documented:** Every public item has a doc-comment with at least one comprehensive doc-test.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(clippy::cognitive_complexity)]
#![deny(unreachable_pub)]
#![deny(rust_2018_idioms)]
#![deny(rust_2024_compatibility)]
// LaTeX math blocks (`\frac{R_p}{R_f}` etc.) are mandated by the "Mathematical Definition"
// doc-comment pattern; clippy::pedantic's doc_markdown heuristic misreads every subscripted
// variable as an un-backticked Rust identifier. Scoped allow, not a pedantic downgrade.
#![allow(clippy::doc_markdown)]

pub mod prelude {
    //! Re-exports for ergonomic use across the workspace.
    pub use crate::error::CalculationError;
    pub use crate::types::{Amounts, Dollar, Periods, Rate, Ratio};
    pub use rust_decimal::Decimal;
    pub use rust_decimal_macros::dec;
}

pub mod banking;
pub mod corporate;
pub mod error;
pub mod financial;
pub mod general;
pub mod markets;
pub mod stocks_bonds;
pub mod types;
