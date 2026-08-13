//! FX forward contracts and hedge effectiveness testing.

use super::fx::CurrencyCode;
use casiros_core::error::CalculationError;
use casiros_core::types::{Dollar, Ratio};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

/// An FX forward contract: an agreement to exchange `notional` units of
/// `currency` at `forward_rate` on `settlement_date`, used to hedge an
/// offsetting foreign-currency exposure.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ForwardContract {
    /// The amount of foreign currency being hedged.
    pub notional: Dollar,
    /// The foreign currency this contract is denominated in.
    pub currency: CurrencyCode,
    /// The rate agreed at contract inception.
    pub forward_rate: Decimal,
    /// The date the contract settles.
    pub settlement_date: NaiveDate,
}

impl ForwardContract {
    /// The gain (positive) or loss (negative) on this contract, for a
    /// contract to *sell* `currency` at `forward_rate`, given the actual spot
    /// rate observed at settlement.
    ///
    /// # Errors
    ///
    /// Returns [`CalculationError::Overflow`] if the computation overflows.
    pub fn gain_loss(&self, spot_rate_at_settlement: Decimal) -> Result<Dollar, CalculationError> {
        let formula = "ForwardContract::gain_loss";
        let rate_difference = self
            .forward_rate
            .checked_sub(spot_rate_at_settlement)
            .ok_or(CalculationError::Overflow { formula })?;
        rate_difference
            .checked_mul(self.notional)
            .ok_or(CalculationError::Overflow { formula })
    }
}

/// The dollar-offset hedge effectiveness ratio: how much of the underlying
/// exposure's gain or loss the hedge offsets. A ratio of exactly `1.0` means
/// the hedge perfectly offsets the exposure.
///
/// # Mathematical Definition
///
/// \[ \text{Effectiveness} = \frac{-\text{Hedge Gain/Loss}}{\text{Exposure Gain/Loss}} \]
///
/// # Errors
///
/// Returns [`CalculationError::DivisionByZero`] if `exposure_gain_loss` is zero.
///
/// # Examples
///
/// ```
/// use casiros_erp::treasury::hedge::hedge_effectiveness;
/// use rust_decimal_macros::dec;
///
/// // The hedge lost exactly what the exposure gained: a perfect offset.
/// let ratio = hedge_effectiveness(dec!(-1000.0), dec!(1000.0)).unwrap();
/// assert_eq!(ratio, dec!(1.0));
/// assert!(ratio > dec!(0.0));
/// ```
pub fn hedge_effectiveness(
    hedge_gain_loss: Decimal,
    exposure_gain_loss: Decimal,
) -> Result<Ratio, CalculationError> {
    if exposure_gain_loss.is_zero() {
        return Err(CalculationError::DivisionByZero {
            formula: "hedge_effectiveness",
        });
    }
    (-hedge_gain_loss)
        .checked_div(exposure_gain_loss)
        .ok_or(CalculationError::Overflow {
            formula: "hedge_effectiveness",
        })
}

/// Whether `effectiveness` falls within the `80%..=125%` dollar-offset range
/// conventionally required to qualify a hedge for hedge accounting treatment
/// (per ASC 815 / IFRS 9 practice).
#[must_use]
pub fn is_highly_effective(effectiveness: Ratio) -> bool {
    effectiveness >= dec!(0.80) && effectiveness <= dec!(1.25)
}
