//! Procedural macros for CASIROS. Currently just [`generate_narrative!`],
//! which expands a `key: value` list of financial metrics into a call
//! building a `casiros_erp::narrative::NarrativeInputs` and generating the
//! CFO-style memo. The actual interpretation logic lives in
//! `casiros_erp::narrative` — this crate is pure syntax and produces no
//! runtime behavior of its own.

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![deny(clippy::pedantic)]
#![deny(warnings)]

use proc_macro::TokenStream;

mod narrative;

/// Builds a CFO-style financial analysis memo from a `key: value` list of
/// metrics, expanding to a call into `casiros_erp::narrative`.
///
/// `company` is required and must be a string-valued expression (e.g. a
/// string literal). Every other key must name one of the metrics on
/// `casiros_erp::narrative::NarrativeInputs` (`roe`, `roa`,
/// `debt_to_equity`, `current_ratio`, `quick_ratio`, `profit_margin`,
/// `net_income`, `interest_coverage`, `asset_turnover`); each value must be
/// an expression evaluating to a `rust_decimal::Decimal`. An unknown key, a
/// missing `company`, or a duplicate key is a compile error.
///
/// # Examples
///
/// ```
/// use casiros_macros::generate_narrative;
/// use rust_decimal_macros::dec;
///
/// let memo = generate_narrative!(
///     company: "Acme Corp",
///     roe: dec!(0.15),
///     debt_to_equity: dec!(0.8),
///     current_ratio: dec!(2.0),
/// );
///
/// assert!(memo.starts_with("## Financial Analysis Memo: Acme Corp"));
/// assert!(memo.contains("Return on Equity"));
/// ```
#[proc_macro]
pub fn generate_narrative(input: TokenStream) -> TokenStream {
    narrative::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
