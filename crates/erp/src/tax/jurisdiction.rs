//! Tax jurisdictions and their progressive bracket structure.

use crate::error::ErpError;
use casiros_core::types::{Dollar, Ratio};
use serde::{Deserialize, Serialize};

/// An identifier for a taxing jurisdiction (e.g. `"US-FEDERAL"`, `"US-CA"`, `"DE"`).
/// Unlike [`crate::treasury::fx::CurrencyCode`], jurisdictions are not
/// fixed-width (national, state/provincial, and municipal codes all coexist),
/// so this simply wraps a `String`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JurisdictionCode(pub String);

/// One bracket of a progressive tax schedule: income up to `upper_bound`
/// (exclusive of any lower brackets' ranges) is taxed at `rate`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TaxBracket {
    /// The upper bound of this bracket's income range, or `None` for the top
    /// (unbounded) bracket.
    pub upper_bound: Option<Dollar>,
    /// The marginal rate applied to income within this bracket.
    pub rate: Ratio,
}

/// A taxing jurisdiction's progressive rate schedule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaxJurisdiction {
    /// This jurisdiction's identifier.
    pub code: JurisdictionCode,
    /// A human-readable name.
    pub name: String,
    /// The progressive brackets, in ascending order. Every bracket except the
    /// last must have `upper_bound: Some(_)`; the last must be `None`
    /// (unbounded), so all income above the highest finite threshold is covered.
    brackets: Vec<TaxBracket>,
}

impl TaxJurisdiction {
    /// Creates a jurisdiction with the given progressive `brackets`.
    ///
    /// # Errors
    ///
    /// Returns [`ErpError::InvalidTaxBrackets`] if `brackets` is empty, if any
    /// bracket but the last has `upper_bound: None`, or if the last bracket
    /// has `upper_bound: Some(_)`.
    pub fn new(
        code: JurisdictionCode,
        name: impl Into<String>,
        brackets: Vec<TaxBracket>,
    ) -> Result<Self, ErpError> {
        let Some((last, rest)) = brackets.split_last() else {
            return Err(ErpError::InvalidTaxBrackets(
                "at least one bracket is required".to_string(),
            ));
        };
        if rest.iter().any(|bracket| bracket.upper_bound.is_none()) {
            return Err(ErpError::InvalidTaxBrackets(
                "only the last bracket may be unbounded".to_string(),
            ));
        }
        if last.upper_bound.is_some() {
            return Err(ErpError::InvalidTaxBrackets(
                "the last bracket must be unbounded".to_string(),
            ));
        }
        Ok(Self {
            code,
            name: name.into(),
            brackets,
        })
    }

    /// This jurisdiction's brackets, in ascending order.
    #[must_use]
    pub fn brackets(&self) -> &[TaxBracket] {
        &self.brackets
    }
}
