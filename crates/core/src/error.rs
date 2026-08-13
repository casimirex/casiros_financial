//! The universal error type for all CASIROS computations.

use rust_decimal::Decimal;
use thiserror::Error;

/// The universal error type for all CASIROS computations.
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum CalculationError {
    /// A formula attempted to divide by zero.
    #[error("Division by zero in {formula}")]
    DivisionByZero {
        /// The name of the formula that failed.
        formula: &'static str,
    },

    /// A value that must be strictly positive was negative or zero.
    #[error("Invalid value {value} in {context}: must be strictly positive")]
    NegativeValueInvalid {
        /// Where the invalid value was encountered.
        context: &'static str,
        /// The offending value.
        value: Decimal,
    },

    /// A value that must lie in `[0, 1]` was outside that range.
    #[error("Value {value} in {context} is outside the valid range [0, 1]")]
    RangeViolation {
        /// Where the invalid value was encountered.
        context: &'static str,
        /// The offending value.
        value: Decimal,
    },

    /// A logarithm was attempted on a non-positive value.
    #[error("Cannot compute logarithm of {value}: must be strictly positive")]
    LogarithmDomainError {
        /// The offending value.
        value: Decimal,
    },

    /// A rate was less than or equal to -1.0, making `(1 + rate)` non-positive.
    #[error("Invalid rate {rate}: must be greater than -1.0")]
    InvalidRate {
        /// The offending rate.
        rate: Decimal,
    },

    /// A computation exceeded the representable range.
    #[error("Numeric overflow in {formula}")]
    Overflow {
        /// The name of the formula that failed.
        formula: &'static str,
    },

    /// An iterative formula (e.g. Newton-Raphson) failed to converge.
    #[error("{formula} failed to converge after {iterations} iterations")]
    ConvergenceFailure {
        /// The name of the formula that failed.
        formula: &'static str,
        /// The number of iterations attempted before giving up.
        iterations: u32,
    },

    /// A required input parameter was not supplied.
    #[error("Missing required input '{parameter}' for formula '{formula}'")]
    MissingInput {
        /// The name of the formula that was missing an input.
        formula: &'static str,
        /// The name of the missing parameter.
        parameter: &'static str,
    },
}
