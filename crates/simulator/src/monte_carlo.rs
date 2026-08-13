//! Monte Carlo scenario generation and parallel evaluation.

use crate::universe::{Universe, UniverseMetrics, compute_universe_metrics};
use casiros_core::error::CalculationError;
use casiros_core::types::{Rate, Ratio};
use rand::SeedableRng;
use rand::rngs::StdRng;
use rand_distr::{Distribution, LogNormal, Normal};
use rayon::prelude::*;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};

/// Relative standard deviation applied to strictly-positive "scale" fields
/// (revenue, assets, share price, ...) via a log-normal perturbation.
const SCALE_RELATIVE_STD_DEV: f64 = 0.20;
/// Absolute standard deviation (e.g. `0.02` = 200bps) applied to rate-like
/// fields (risk-free rate, beta, cost of equity, ...) via a normal perturbation.
const RATE_ABSOLUTE_STD_DEV: f64 = 0.02;

/// Configuration for a Monte Carlo simulation run.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MonteCarloConfig {
    /// The number of scenarios to generate.
    pub iterations: u32,
    /// The seed for the reproducible pseudo-random number generator.
    pub seed: u64,
    /// Whether to track convergence and stop early once [`Self::convergence_threshold`]
    /// is satisfied.
    pub track_convergence: bool,
    /// The maximum batch-over-batch change in the running mean return-on-equity
    /// below which the simulation is considered converged.
    pub convergence_threshold: Decimal,
    /// The number of scenarios evaluated per convergence-check batch.
    pub convergence_batch_size: u32,
}

/// Perturbs a rate-like `baseline` using a normal distribution, immediately
/// converting the sampled `f64` to a `Decimal`.
fn perturb_rate(rng: &mut StdRng, baseline: Rate) -> Result<Rate, CalculationError> {
    let formula = "monte_carlo::perturb_rate";
    let mean = baseline
        .to_f64()
        .ok_or(CalculationError::Overflow { formula })?;
    let distribution = Normal::new(mean, RATE_ABSOLUTE_STD_DEV)
        .map_err(|_| CalculationError::Overflow { formula })?;
    let sample = distribution.sample(rng);
    Decimal::from_f64_retain(sample).ok_or(CalculationError::Overflow { formula })
}

/// Perturbs a `retention_ratio`/`tax_rate`-style `baseline` like [`perturb_rate`],
/// then clamps the result to `[0, 1]`.
fn perturb_unit_ratio(rng: &mut StdRng, baseline: Ratio) -> Result<Ratio, CalculationError> {
    let perturbed = perturb_rate(rng, baseline)?;
    Ok(perturbed.clamp(Decimal::ZERO, Decimal::ONE))
}

/// Perturbs a strictly-positive "scale" `baseline` (revenue, assets, price, ...)
/// using a log-normal distribution, immediately converting the sampled `f64` to
/// a `Decimal`. Non-positive baselines are returned unperturbed, since a
/// log-normal distribution cannot center on a non-positive value.
fn perturb_scale(rng: &mut StdRng, baseline: Decimal) -> Result<Decimal, CalculationError> {
    let formula = "monte_carlo::perturb_scale";
    let mean = baseline
        .to_f64()
        .ok_or(CalculationError::Overflow { formula })?;
    if mean <= 0.0 {
        return Ok(baseline);
    }
    let distribution = LogNormal::new(mean.ln(), SCALE_RELATIVE_STD_DEV)
        .map_err(|_| CalculationError::Overflow { formula })?;
    let sample = distribution.sample(rng);
    Decimal::from_f64_retain(sample).ok_or(CalculationError::Overflow { formula })
}

/// Perturbs every field of `baseline`, producing one new scenario.
fn perturb_universe(rng: &mut StdRng, baseline: &Universe) -> Result<Universe, CalculationError> {
    Ok(Universe {
        risk_free_rate: perturb_rate(rng, baseline.risk_free_rate)?,
        inflation_rate: perturb_rate(rng, baseline.inflation_rate)?,
        market_return: perturb_rate(rng, baseline.market_return)?,
        portfolio_return: perturb_rate(rng, baseline.portfolio_return)?,
        return_std_dev: perturb_scale(rng, baseline.return_std_dev)?,
        revenue: perturb_scale(rng, baseline.revenue)?,
        cogs: perturb_scale(rng, baseline.cogs)?,
        operating_expenses: perturb_scale(rng, baseline.operating_expenses)?,
        interest_expense: perturb_scale(rng, baseline.interest_expense)?,
        tax_rate: perturb_unit_ratio(rng, baseline.tax_rate)?,
        beta: perturb_rate(rng, baseline.beta)?,
        cost_of_equity: perturb_rate(rng, baseline.cost_of_equity)?,
        cost_of_debt: perturb_rate(rng, baseline.cost_of_debt)?,
        total_assets: perturb_scale(rng, baseline.total_assets)?,
        current_assets: perturb_scale(rng, baseline.current_assets)?,
        inventory: perturb_scale(rng, baseline.inventory)?,
        current_liabilities: perturb_scale(rng, baseline.current_liabilities)?,
        total_liabilities: perturb_scale(rng, baseline.total_liabilities)?,
        total_equity: perturb_scale(rng, baseline.total_equity)?,
        share_price: perturb_scale(rng, baseline.share_price)?,
        shares_outstanding: perturb_scale(rng, baseline.shares_outstanding)?,
    })
}

/// Generates `config.iterations` scenarios by perturbing `baseline`, using
/// `config.seed` for reproducibility: the same baseline and config always
/// produce the same scenarios.
///
/// # Errors
///
/// Returns [`CalculationError::Overflow`] if a sampled value cannot be
/// represented as a `Decimal`.
pub fn generate_scenarios(
    baseline: &Universe,
    config: &MonteCarloConfig,
) -> Result<Vec<Universe>, CalculationError> {
    let mut rng = StdRng::seed_from_u64(config.seed);
    let mut scenarios = Vec::with_capacity(config.iterations as usize);
    for _ in 0..config.iterations {
        scenarios.push(perturb_universe(&mut rng, baseline)?);
    }
    Ok(scenarios)
}

/// Folds each successfully-computed universe's return-on-equity into the
/// running Welford mean, skipping (rather than failing) on arithmetic overflow.
fn update_running_mean(
    batch: &[Result<UniverseMetrics, CalculationError>],
    mean: &mut Decimal,
    count: &mut Decimal,
) {
    for metrics in batch.iter().flatten() {
        let Some(next_count) = count.checked_add(Decimal::ONE) else {
            continue;
        };
        let Some(delta) = metrics.return_on_equity.checked_sub(*mean) else {
            continue;
        };
        let Some(delta_over_n) = delta.checked_div(next_count) else {
            continue;
        };
        let Some(next_mean) = mean.checked_add(delta_over_n) else {
            continue;
        };
        *count = next_count;
        *mean = next_mean;
    }
}

/// True once the batch-over-batch change in the running return-on-equity mean
/// falls below `threshold`.
fn has_converged(
    previous_mean: Decimal,
    current_mean: Decimal,
    count: Decimal,
    threshold: Decimal,
) -> bool {
    if count.is_zero() {
        return false;
    }
    current_mean
        .checked_sub(previous_mean)
        .map(|change| change.abs())
        .is_some_and(|change| change < threshold)
}

/// Evaluates `scenarios` in batches of `config.convergence_batch_size`, tracking
/// the running return-on-equity mean and stopping early once it has converged.
fn run_batched_with_convergence(
    scenarios: &[Universe],
    config: &MonteCarloConfig,
) -> Vec<Result<UniverseMetrics, CalculationError>> {
    let batch_size = usize::try_from(config.convergence_batch_size)
        .unwrap_or(scenarios.len())
        .max(1);
    let mut results = Vec::with_capacity(scenarios.len());
    let mut running_mean = Decimal::ZERO;
    let mut running_count = Decimal::ZERO;

    for batch in scenarios.chunks(batch_size) {
        let batch_results: Vec<_> = batch.par_iter().map(compute_universe_metrics).collect();
        let previous_mean = running_mean;
        update_running_mean(&batch_results, &mut running_mean, &mut running_count);
        results.extend(batch_results);
        if has_converged(
            previous_mean,
            running_mean,
            running_count,
            config.convergence_threshold,
        ) {
            break;
        }
    }
    results
}

/// Computes [`UniverseMetrics`] for every scenario in `scenarios`, using `rayon`
/// for parallel evaluation. Each scenario's result is independent: one
/// scenario's error does not prevent the others from being computed.
///
/// When `config.track_convergence` is set, scenarios are evaluated in batches
/// of `config.convergence_batch_size` and evaluation stops early once the
/// batch-over-batch change in the running return-on-equity mean falls below
/// `config.convergence_threshold` — the returned `Vec` may then be shorter
/// than `scenarios`.
#[must_use]
pub fn run_simulation(
    scenarios: &[Universe],
    config: &MonteCarloConfig,
) -> Vec<Result<UniverseMetrics, CalculationError>> {
    if config.track_convergence && config.convergence_batch_size > 0 {
        return run_batched_with_convergence(scenarios, config);
    }
    scenarios.par_iter().map(compute_universe_metrics).collect()
}
