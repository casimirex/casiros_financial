//! Progressive tax calculation, multi-jurisdiction aggregation, and deferred tax.

use super::jurisdiction::TaxJurisdiction;
use crate::error::ErpError;
use casiros_core::error::CalculationError;
use casiros_core::types::{Dollar, Ratio};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Computes the marginal (progressive-bracket) tax owed on `taxable_income`
/// under `jurisdiction`'s rate schedule.
///
/// # Errors
///
/// Returns [`CalculationError::NegativeValueInvalid`] if `taxable_income` is negative.
/// Returns [`CalculationError::Overflow`] if any intermediate sum overflows.
pub fn calculate_tax(
    jurisdiction: &TaxJurisdiction,
    taxable_income: Dollar,
) -> Result<Dollar, ErpError> {
    if taxable_income < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: "calculate_tax - taxable_income",
            value: taxable_income,
        }
        .into());
    }
    let formula = "calculate_tax";
    let mut tax = Decimal::ZERO;
    let mut lower_bound = Decimal::ZERO;
    for bracket in jurisdiction.brackets() {
        if lower_bound >= taxable_income {
            break;
        }
        let bracket_ceiling = bracket
            .upper_bound
            .unwrap_or(taxable_income)
            .min(taxable_income);
        let taxable_in_bracket = bracket_ceiling
            .checked_sub(lower_bound)
            .ok_or(CalculationError::Overflow { formula })?;
        let bracket_tax = taxable_in_bracket
            .checked_mul(bracket.rate)
            .ok_or(CalculationError::Overflow { formula })?;
        tax = tax
            .checked_add(bracket_tax)
            .ok_or(CalculationError::Overflow { formula })?;
        lower_bound = bracket.upper_bound.unwrap_or(taxable_income);
    }
    Ok(tax)
}

/// Computes total tax owed across several jurisdictions at once (e.g. federal
/// plus state), each with its own taxable income allocation.
///
/// # Errors
///
/// Returns whatever [`calculate_tax`] returns for the first failing allocation.
pub fn calculate_multi_jurisdiction_tax(
    allocations: &[(&TaxJurisdiction, Dollar)],
) -> Result<Dollar, ErpError> {
    let mut total = Decimal::ZERO;
    for (jurisdiction, taxable_income) in allocations {
        let tax = calculate_tax(jurisdiction, *taxable_income)?;
        total = total.checked_add(tax).ok_or(CalculationError::Overflow {
            formula: "calculate_multi_jurisdiction_tax",
        })?;
    }
    Ok(total)
}

/// Whether a temporary difference produces a deferred tax liability, a
/// deferred tax asset, or neither.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum DeferredTaxPosition {
    /// A taxable temporary difference: book basis exceeds tax basis, so more
    /// tax will be owed in the future as the difference reverses.
    Liability(Dollar),
    /// A deductible temporary difference: tax basis exceeds book basis, so
    /// less tax will be owed in the future as the difference reverses.
    Asset(Dollar),
    /// Book and tax basis are equal: no temporary difference.
    None,
}

/// A single book-versus-tax basis difference for one asset or liability
/// (e.g. a fixed asset depreciated differently for book and tax purposes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporaryDifference {
    /// A human-readable description.
    pub description: String,
    /// The carrying value under GAAP/book accounting.
    pub book_basis: Dollar,
    /// The carrying value under tax accounting.
    pub tax_basis: Dollar,
    /// The tax rate expected to apply when the difference reverses.
    pub tax_rate: Ratio,
}

impl TemporaryDifference {
    /// Computes the deferred tax position arising from this temporary difference.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if the computation overflows.
    pub fn deferred_tax_position(&self) -> Result<DeferredTaxPosition, CalculationError> {
        let formula = "TemporaryDifference::deferred_tax_position";
        let diff = self
            .book_basis
            .checked_sub(self.tax_basis)
            .ok_or(CalculationError::Overflow { formula })?;
        if diff.is_zero() {
            return Ok(DeferredTaxPosition::None);
        }
        let tax_effect = diff
            .abs()
            .checked_mul(self.tax_rate)
            .ok_or(CalculationError::Overflow { formula })?;
        if diff > Decimal::ZERO {
            Ok(DeferredTaxPosition::Liability(tax_effect))
        } else {
            Ok(DeferredTaxPosition::Asset(tax_effect))
        }
    }
}
