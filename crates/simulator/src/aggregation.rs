//! Statistical aggregation of a Monte Carlo run's per-scenario results.

use casiros_core::error::CalculationError;
use rust_decimal::Decimal;
use rust_decimal::MathematicalOps;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Aggregate statistics for one metric across every scenario in a simulation run.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct SimulationResults {
    /// The number of values aggregated.
    pub sample_count: usize,
    /// The arithmetic mean.
    pub mean: Decimal,
    /// The 50th percentile.
    pub median: Decimal,
    /// The sample standard deviation (Bessel-corrected, `n - 1` denominator).
    pub std_dev: Decimal,
    /// The minimum observed value.
    pub min: Decimal,
    /// The maximum observed value.
    pub max: Decimal,
    /// The 5th percentile.
    pub percentile_5: Decimal,
    /// The 25th percentile.
    pub percentile_25: Decimal,
    /// The 75th percentile.
    pub percentile_75: Decimal,
    /// The 95th percentile.
    pub percentile_95: Decimal,
}

/// Computes the mean and sample variance of `values` using Welford's online
/// algorithm, for numerical stability across large sample counts.
fn welford_mean_variance(values: &[Decimal]) -> Result<(Decimal, Decimal), CalculationError> {
    let formula = "aggregation::welford_mean_variance";
    let mut mean = Decimal::ZERO;
    let mut sum_of_squares = Decimal::ZERO;
    let mut count = Decimal::ZERO;
    for &value in values {
        count = count
            .checked_add(Decimal::ONE)
            .ok_or(CalculationError::Overflow { formula })?;
        let delta = value
            .checked_sub(mean)
            .ok_or(CalculationError::Overflow { formula })?;
        let delta_over_n = delta
            .checked_div(count)
            .ok_or(CalculationError::Overflow { formula })?;
        mean = mean
            .checked_add(delta_over_n)
            .ok_or(CalculationError::Overflow { formula })?;
        let delta2 = value
            .checked_sub(mean)
            .ok_or(CalculationError::Overflow { formula })?;
        let term = delta
            .checked_mul(delta2)
            .ok_or(CalculationError::Overflow { formula })?;
        sum_of_squares = sum_of_squares
            .checked_add(term)
            .ok_or(CalculationError::Overflow { formula })?;
    }
    if count <= Decimal::ONE {
        return Ok((mean, Decimal::ZERO));
    }
    let denominator = count
        .checked_sub(Decimal::ONE)
        .ok_or(CalculationError::Overflow { formula })?;
    let variance = sum_of_squares
        .checked_div(denominator)
        .ok_or(CalculationError::Overflow { formula })?;
    Ok((mean, variance))
}

/// Computes the square root of a non-negative `Decimal`, entirely in
/// `Decimal` precision (via `rust_decimal`'s "maths" feature) — no `f64`
/// round-trip.
fn decimal_sqrt(value: Decimal) -> Result<Decimal, CalculationError> {
    let formula = "aggregation::decimal_sqrt";
    if value < Decimal::ZERO {
        return Err(CalculationError::NegativeValueInvalid {
            context: formula,
            value,
        });
    }
    value.sqrt().ok_or(CalculationError::Overflow { formula })
}

/// Computes the `p`-th percentile (`p` in `[0, 1]`) of an ascending-sorted slice
/// via linear interpolation between the two nearest ranks.
fn percentile(sorted: &[Decimal], p: Decimal) -> Result<Decimal, CalculationError> {
    let formula = "aggregation::percentile";
    if sorted.len() < 2 {
        return sorted
            .first()
            .copied()
            .ok_or(CalculationError::MissingInput {
                formula,
                parameter: "values",
            });
    }
    let last_index = Decimal::from(sorted.len() - 1);
    let rank = p
        .checked_mul(last_index)
        .ok_or(CalculationError::Overflow { formula })?;
    let lower_index = rank
        .trunc()
        .to_usize()
        .ok_or(CalculationError::Overflow { formula })?;
    let upper_index = (lower_index + 1).min(sorted.len() - 1);
    let fraction = rank.fract();
    let gap = sorted[upper_index]
        .checked_sub(sorted[lower_index])
        .ok_or(CalculationError::Overflow { formula })?;
    let interpolated = gap
        .checked_mul(fraction)
        .ok_or(CalculationError::Overflow { formula })?;
    sorted[lower_index]
        .checked_add(interpolated)
        .ok_or(CalculationError::Overflow { formula })
}

/// Aggregates `values` (one metric's observations across every scenario in a
/// simulation run) into a [`SimulationResults`].
///
/// # Errors
///
/// Returns [`CalculationError::MissingInput`] if `values` is empty.
/// Returns [`CalculationError::Overflow`] if any intermediate computation overflows.
pub fn aggregate(values: &[Decimal]) -> Result<SimulationResults, CalculationError> {
    if values.is_empty() {
        return Err(CalculationError::MissingInput {
            formula: "aggregation::aggregate",
            parameter: "values",
        });
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let (mean, variance) = welford_mean_variance(values)?;

    Ok(SimulationResults {
        sample_count: values.len(),
        mean,
        median: percentile(&sorted, dec!(0.5))?,
        std_dev: decimal_sqrt(variance)?,
        min: sorted[0],
        max: sorted[sorted.len() - 1],
        percentile_5: percentile(&sorted, dec!(0.05))?,
        percentile_25: percentile(&sorted, dec!(0.25))?,
        percentile_75: percentile(&sorted, dec!(0.75))?,
        percentile_95: percentile(&sorted, dec!(0.95))?,
    })
}
