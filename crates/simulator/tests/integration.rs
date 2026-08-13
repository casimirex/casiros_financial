//! Integration tests exercising `casiros-simulator`'s public API end-to-end.

use casiros_core::{corporate, financial, markets};
use casiros_simulator::aggregation::aggregate;
use casiros_simulator::monte_carlo::{MonteCarloConfig, generate_scenarios, run_simulation};
use casiros_simulator::universe::{Universe, compute_universe_metrics};
use rust_decimal_macros::dec;

/// A clean baseline scenario. Values are chosen for readability, not realism.
fn baseline_universe() -> Universe {
    Universe {
        risk_free_rate: dec!(0.03),
        inflation_rate: dec!(0.02),
        market_return: dec!(0.08),
        portfolio_return: dec!(0.10),
        return_std_dev: dec!(0.15),
        revenue: dec!(1_000_000.0),
        cogs: dec!(600_000.0),
        operating_expenses: dec!(200_000.0),
        interest_expense: dec!(50_000.0),
        tax_rate: dec!(0.25),
        beta: dec!(1.2),
        cost_of_equity: dec!(0.11),
        cost_of_debt: dec!(0.06),
        total_assets: dec!(1_500_000.0),
        current_assets: dec!(400_000.0),
        inventory: dec!(100_000.0),
        current_liabilities: dec!(200_000.0),
        total_liabilities: dec!(750_000.0),
        total_equity: dec!(750_000.0),
        share_price: dec!(50.0),
        shares_outstanding: dec!(20_000.0),
    }
}

fn base_config() -> MonteCarloConfig {
    MonteCarloConfig {
        iterations: 200,
        seed: 42,
        track_convergence: false,
        convergence_threshold: dec!(0.0001),
        convergence_batch_size: 20,
    }
}

#[test]
fn compute_universe_metrics_matches_direct_core_calls() {
    let universe = baseline_universe();
    let metrics = compute_universe_metrics(&universe).expect("clean baseline computes cleanly");

    assert_eq!(metrics.ebit, dec!(200_000.0));
    assert_eq!(metrics.net_income, dec!(112_500.0));
    assert_eq!(
        metrics.profit_margin,
        financial::profit_margin(metrics.net_income, universe.revenue).unwrap()
    );
    assert_eq!(
        metrics.return_on_equity,
        financial::return_on_equity(metrics.net_income, universe.total_equity).unwrap()
    );
    assert_eq!(
        metrics.current_ratio,
        financial::current_ratio(universe.current_assets, universe.current_liabilities).unwrap()
    );
    assert_eq!(
        metrics.debt_to_equity,
        financial::debt_to_equity(universe.total_liabilities, universe.total_equity).unwrap()
    );

    let expected_market_cap = universe.share_price * universe.shares_outstanding;
    let expected_wacc = corporate::wacc(
        expected_market_cap,
        universe.total_liabilities,
        universe.cost_of_equity,
        universe.cost_of_debt,
        universe.tax_rate,
    )
    .unwrap();
    assert_eq!(metrics.wacc, expected_wacc);

    let expected_sharpe = markets::sharpe_ratio(
        universe.portfolio_return,
        universe.risk_free_rate,
        universe.return_std_dev,
    )
    .unwrap();
    assert_eq!(metrics.sharpe_ratio, expected_sharpe);
}

#[test]
fn generate_scenarios_is_reproducible_under_a_fixed_seed() {
    let baseline = baseline_universe();
    let config = base_config();

    let first_run = generate_scenarios(&baseline, &config).expect("perturbation succeeds");
    let second_run = generate_scenarios(&baseline, &config).expect("perturbation succeeds");

    assert_eq!(first_run.len(), config.iterations as usize);
    assert_eq!(first_run, second_run);
}

#[test]
fn generate_scenarios_with_different_seeds_diverge() {
    let baseline = baseline_universe();
    let mut config_a = base_config();
    config_a.iterations = 5;
    let mut config_b = config_a;
    config_b.seed = config_a.seed + 1;

    let run_a = generate_scenarios(&baseline, &config_a).expect("perturbation succeeds");
    let run_b = generate_scenarios(&baseline, &config_b).expect("perturbation succeeds");

    assert_ne!(run_a, run_b);
}

#[test]
fn run_simulation_without_convergence_evaluates_every_scenario() {
    let baseline = baseline_universe();
    let config = base_config();
    let scenarios = generate_scenarios(&baseline, &config).expect("perturbation succeeds");

    let results = run_simulation(&scenarios, &config);

    assert_eq!(results.len(), scenarios.len());
    assert!(results.iter().all(Result::is_ok));
}

#[test]
fn run_simulation_with_convergence_stops_early_on_identical_scenarios() {
    let universe = baseline_universe();
    let scenarios = vec![universe; 10];
    let mut config = base_config();
    config.track_convergence = true;
    config.convergence_batch_size = 2;
    config.convergence_threshold = dec!(0.0000001);

    let results = run_simulation(&scenarios, &config);

    assert!(results.len() < scenarios.len());
    assert_eq!(results.len(), 4);
}

#[test]
fn aggregate_computes_correct_central_tendency() {
    let values: Vec<_> = (1..=10).map(rust_decimal::Decimal::from).collect();
    let stats = aggregate(&values).expect("non-empty input aggregates");

    assert_eq!(stats.sample_count, 10);
    assert_eq!(stats.mean, dec!(5.5));
    assert_eq!(stats.median, dec!(5.5));
    assert_eq!(stats.min, dec!(1));
    assert_eq!(stats.max, dec!(10));
    assert!(stats.std_dev > dec!(3.02) && stats.std_dev < dec!(3.03));
}

#[test]
fn aggregate_computes_correct_percentiles() {
    let values: Vec<_> = (1..=10).map(rust_decimal::Decimal::from).collect();
    let stats = aggregate(&values).expect("non-empty input aggregates");

    assert_eq!(stats.percentile_5, dec!(1.45));
    assert_eq!(stats.percentile_25, dec!(3.25));
    assert_eq!(stats.percentile_75, dec!(7.75));
    assert_eq!(stats.percentile_95, dec!(9.55));
}

#[test]
fn aggregate_rejects_empty_input() {
    let values: Vec<rust_decimal::Decimal> = Vec::new();
    assert!(aggregate(&values).is_err());
}
