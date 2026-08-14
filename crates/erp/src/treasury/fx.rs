//! Foreign exchange: currency codes, exchange rates, and exposure revaluation.

use crate::error::ErpError;
use casiros_core::error::CalculationError;
use casiros_core::types::Dollar;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

/// An ISO-4217-style three-letter currency code (e.g. `"USD"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct CurrencyCode(pub [u8; 3]);

impl CurrencyCode {
    /// Parses a three-letter ASCII-uppercase currency code.
    ///
    /// # Errors
    ///
    /// Returns [`ErpError::InvalidCurrencyCode`] if `code` is not exactly
    /// three ASCII uppercase letters.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::treasury::fx::CurrencyCode;
    ///
    /// let usd = CurrencyCode::new("USD").unwrap();
    /// assert_eq!(usd.to_string(), "USD");
    /// assert!(CurrencyCode::new("us").is_err());
    /// ```
    pub fn new(code: &str) -> Result<Self, ErpError> {
        let bytes = code.as_bytes();
        if bytes.len() != 3 || !bytes.iter().all(u8::is_ascii_uppercase) {
            return Err(ErpError::InvalidCurrencyCode(code.to_string()));
        }
        Ok(Self([bytes[0], bytes[1], bytes[2]]))
    }
}

impl fmt::Display for CurrencyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", std::str::from_utf8(&self.0).unwrap_or("???"))
    }
}

/// A quoted exchange rate between two currencies as of a given date: one unit
/// of `from` is worth `rate` units of `to`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct ExchangeRate {
    /// The currency being converted from.
    pub from: CurrencyCode,
    /// The currency being converted to.
    pub to: CurrencyCode,
    /// Units of `to` per one unit of `from`.
    pub rate: Decimal,
    /// The date this rate was quoted.
    pub as_of: NaiveDate,
}

/// A balance denominated in a foreign (non-functional) currency.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct FxExposure {
    /// The currency this exposure is denominated in.
    pub currency: CurrencyCode,
    /// The amount, in `currency`.
    pub amount: Decimal,
}

impl FxExposure {
    /// Converts this exposure to `rate.to` using `rate`.
    ///
    /// # Errors
    ///
    /// Returns [`ErpError::CurrencyMismatch`] if `rate.from` does not match
    /// [`Self::currency`]. Returns [`ErpError::Calculation`] if the
    /// multiplication overflows.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_erp::treasury::fx::{CurrencyCode, ExchangeRate, FxExposure};
    /// use chrono::NaiveDate;
    /// use rust_decimal_macros::dec;
    ///
    /// let eur = CurrencyCode::new("EUR").unwrap();
    /// let usd = CurrencyCode::new("USD").unwrap();
    /// let exposure = FxExposure { currency: eur, amount: dec!(100) };
    /// let rate = ExchangeRate {
    ///     from: eur,
    ///     to: usd,
    ///     rate: dec!(1.10),
    ///     as_of: NaiveDate::from_ymd_opt(2026, 8, 13).unwrap(),
    /// };
    /// assert_eq!(exposure.convert(&rate).unwrap(), dec!(110.00));
    ///
    /// let wrong_rate = ExchangeRate { from: usd, ..rate };
    /// assert!(exposure.convert(&wrong_rate).is_err());
    /// ```
    pub fn convert(&self, rate: &ExchangeRate) -> Result<Dollar, ErpError> {
        if rate.from != self.currency {
            return Err(ErpError::CurrencyMismatch {
                expected: self.currency,
                actual: rate.from,
            });
        }
        self.amount.checked_mul(rate.rate).ok_or(
            CalculationError::Overflow {
                formula: "FxExposure::convert",
            }
            .into(),
        )
    }
}

/// The unrealized FX revaluation gain (positive) or loss (negative) on
/// `exposure` between `old_rate` and `new_rate`.
///
/// # Errors
///
/// Returns [`ErpError::CurrencyMismatch`] if either rate's `from` does not
/// match `exposure.currency`. Returns [`ErpError::Calculation`] if a
/// conversion or the subtraction overflows.
///
/// # Examples
///
/// ```
/// use casiros_erp::treasury::fx::{CurrencyCode, ExchangeRate, FxExposure, revaluation_gain_loss};
/// use chrono::NaiveDate;
/// use rust_decimal_macros::dec;
///
/// let eur = CurrencyCode::new("EUR").unwrap();
/// let usd = CurrencyCode::new("USD").unwrap();
/// let exposure = FxExposure { currency: eur, amount: dec!(100) };
/// let old_rate = ExchangeRate {
///     from: eur,
///     to: usd,
///     rate: dec!(1.10),
///     as_of: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
/// };
/// let new_rate = ExchangeRate { rate: dec!(1.15), ..old_rate };
///
/// let gain = revaluation_gain_loss(&exposure, &old_rate, &new_rate).unwrap();
/// assert_eq!(gain, dec!(5.00));
/// assert!(gain > dec!(0));
/// ```
pub fn revaluation_gain_loss(
    exposure: &FxExposure,
    old_rate: &ExchangeRate,
    new_rate: &ExchangeRate,
) -> Result<Dollar, ErpError> {
    let old_value = exposure.convert(old_rate)?;
    let new_value = exposure.convert(new_rate)?;
    new_value.checked_sub(old_value).ok_or(
        CalculationError::Overflow {
            formula: "revaluation_gain_loss",
        }
        .into(),
    )
}
