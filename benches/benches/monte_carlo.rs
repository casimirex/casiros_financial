//! Benchmarks the Monte Carlo engine's actual hot path — scenario generation
//! plus evaluation — at a realistic default iteration count. This is the
//! single most performance-sensitive subsystem in CASIROS (the build prompt
//! configures up to 1,000,000 iterations per run).

use casiros_simulator::monte_carlo::{MonteCarloConfig, generate_scenarios, run_simulation};
use casiros_simulator::universe::Universe;
use criterion::{Criterion, criterion_group, criterion_main};
use rust_decimal_macros::dec;
use std::hint::black_box;

fn sample_universe() -> Universe {
    Universe {
        risk_free_rate: dec!(0.03),
        inflation_rate: dec!(0.02),
        market_return: dec!(0.08),
        portfolio_return: dec!(0.10),
        return_std_dev: dec!(0.15),
        revenue: dec!(1_000_000),
        cogs: dec!(600_000),
        operating_expenses: dec!(200_000),
        interest_expense: dec!(50_000),
        tax_rate: dec!(0.25),
        beta: dec!(1.2),
        cost_of_equity: dec!(0.11),
        cost_of_debt: dec!(0.06),
        total_assets: dec!(1_500_000),
        current_assets: dec!(400_000),
        inventory: dec!(100_000),
        current_liabilities: dec!(200_000),
        total_liabilities: dec!(750_000),
        total_equity: dec!(750_000),
        share_price: dec!(50),
        shares_outstanding: dec!(20_000),
    }
}

fn config(iterations: u32) -> MonteCarloConfig {
    MonteCarloConfig {
        iterations,
        seed: 42,
        track_convergence: false,
        convergence_threshold: dec!(0.0001),
        convergence_batch_size: 1_000,
    }
}

fn bench_generate_and_run_10k(c: &mut Criterion) {
    let baseline = sample_universe();
    let cfg = config(10_000);
    c.bench_function("monte_carlo_10k_iterations", |b| {
        b.iter(|| {
            let scenarios = generate_scenarios(black_box(&baseline), black_box(&cfg)).unwrap();
            run_simulation(black_box(&scenarios), black_box(&cfg))
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(20);
    targets = bench_generate_and_run_10k
}
criterion_main!(benches);
