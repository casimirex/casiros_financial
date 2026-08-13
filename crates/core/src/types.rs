//! Newtype pattern for type safety. No naked `Decimal`s in function signatures.

use rust_decimal::Decimal;

/// Monetary value in the base currency (e.g., USD).
pub type Dollar = Decimal;

/// An interest rate, discount rate, or growth rate expressed as a decimal.
/// Example: 5% = `dec!(0.05)`.
pub type Rate = Decimal;

/// A dimensionless ratio (e.g., 0.6 for 60%).
pub type Ratio = Decimal;

/// A number of compounding periods (years, months, quarters).
pub type Periods = u32;

/// The three fundamental time-value-of-money quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Amounts {
    /// Present value (PV)
    pub principal: Dollar,
    /// Future value (FV)
    pub future_value: Dollar,
    /// Periodic payment (PMT)
    pub payment: Dollar,
}

impl Amounts {
    /// Creates a new `Amounts` with all fields validated as non-negative.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::CalculationError::NegativeValueInvalid`] if any
    /// of `principal`, `future_value`, or `payment` is negative.
    ///
    /// # Examples
    ///
    /// ```
    /// use casiros_core::types::Amounts;
    /// use rust_decimal_macros::dec;
    ///
    /// let amounts = Amounts::new(dec!(100.0), dec!(150.0), dec!(10.0)).unwrap();
    /// assert_eq!(amounts.principal, dec!(100.0));
    /// assert!(amounts.future_value > amounts.principal);
    /// ```
    pub fn new(
        principal: Dollar,
        future_value: Dollar,
        payment: Dollar,
    ) -> Result<Self, crate::error::CalculationError> {
        use crate::error::CalculationError;
        if principal < Decimal::ZERO {
            return Err(CalculationError::NegativeValueInvalid {
                context: "Amounts::principal",
                value: principal,
            });
        }
        if future_value < Decimal::ZERO {
            return Err(CalculationError::NegativeValueInvalid {
                context: "Amounts::future_value",
                value: future_value,
            });
        }
        if payment < Decimal::ZERO {
            return Err(CalculationError::NegativeValueInvalid {
                context: "Amounts::payment",
                value: payment,
            });
        }
        Ok(Self {
            principal,
            future_value,
            payment,
        })
    }
}
