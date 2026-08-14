//! Benchmarks the hot-path pure formulas in `casiros-core`: simple closed-form
//! math with no allocation, so these numbers are a floor for everything built
//! on top (the DAG evaluator, the Monte Carlo engine, the ERP layer).

use casiros_core::corporate::wacc;
use casiros_core::financial::current_ratio;
use casiros_core::general::{future_value, present_value};
use casiros_core::markets::sharpe_ratio;
use criterion::{Criterion, criterion_group, criterion_main};
use rust_decimal_macros::dec;
use std::hint::black_box;

fn bench_future_value(c: &mut Criterion) {
    c.bench_function("future_value", |b| {
        b.iter(|| future_value(black_box(dec!(1000)), black_box(dec!(0.05)), black_box(120)));
    });
}

fn bench_present_value(c: &mut Criterion) {
    c.bench_function("present_value", |b| {
        b.iter(|| present_value(black_box(dec!(1000)), black_box(dec!(0.05)), black_box(120)));
    });
}

fn bench_wacc(c: &mut Criterion) {
    c.bench_function("wacc", |b| {
        b.iter(|| {
            wacc(
                black_box(dec!(600_000)),
                black_box(dec!(400_000)),
                black_box(dec!(0.10)),
                black_box(dec!(0.06)),
                black_box(dec!(0.25)),
            )
        });
    });
}

fn bench_current_ratio(c: &mut Criterion) {
    c.bench_function("current_ratio", |b| {
        b.iter(|| current_ratio(black_box(dec!(400_000)), black_box(dec!(200_000))));
    });
}

fn bench_sharpe_ratio(c: &mut Criterion) {
    c.bench_function("sharpe_ratio", |b| {
        b.iter(|| sharpe_ratio(black_box(dec!(0.10)), black_box(dec!(0.03)), black_box(dec!(0.15))));
    });
}

criterion_group!(
    benches,
    bench_future_value,
    bench_present_value,
    bench_wacc,
    bench_current_ratio,
    bench_sharpe_ratio,
);
criterion_main!(benches);
