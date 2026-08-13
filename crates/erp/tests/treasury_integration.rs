//! Integration tests exercising `casiros-erp`'s `treasury` module end-to-end.

use casiros_erp::error::ErpError;
use casiros_erp::treasury::cashflow::{CashFlowCategory, CashFlowItem, CashForecast};
use casiros_erp::treasury::fx::{CurrencyCode, ExchangeRate, FxExposure};
use casiros_erp::treasury::hedge::{ForwardContract, hedge_effectiveness, is_highly_effective};
use chrono::NaiveDate;
use rust_decimal_macros::dec;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn sample_forecast() -> CashForecast {
    let mut forecast = CashForecast::new();
    forecast.add(CashFlowItem {
        category: CashFlowCategory::Operating,
        description: "collections".into(),
        amount: dec!(1000.0),
        date: date(2026, 1, 5),
    });
    forecast.add(CashFlowItem {
        category: CashFlowCategory::Operating,
        description: "payroll".into(),
        amount: dec!(-300.0),
        date: date(2026, 1, 10),
    });
    forecast.add(CashFlowItem {
        category: CashFlowCategory::Investing,
        description: "equipment".into(),
        amount: dec!(-200.0),
        date: date(2026, 1, 15),
    });
    forecast.add(CashFlowItem {
        category: CashFlowCategory::Financing,
        description: "loan draw".into(),
        amount: dec!(500.0),
        date: date(2026, 1, 20),
    });
    forecast
}

#[test]
fn net_cash_flow_sums_within_range_and_by_category() {
    let forecast = sample_forecast();
    assert_eq!(
        forecast
            .net_cash_flow(date(2026, 1, 1), date(2026, 1, 31), None)
            .unwrap(),
        dec!(1000.0)
    );
    assert_eq!(
        forecast
            .net_cash_flow(
                date(2026, 1, 1),
                date(2026, 1, 31),
                Some(CashFlowCategory::Operating)
            )
            .unwrap(),
        dec!(700.0)
    );
}

#[test]
fn projected_balance_only_counts_items_up_to_as_of() {
    let forecast = sample_forecast();
    // Only the Jan-5 (+1000) and Jan-10 (-300) items are on or before Jan-12.
    assert_eq!(
        forecast
            .projected_balance(dec!(2000.0), date(2026, 1, 12))
            .unwrap(),
        dec!(2700.0)
    );
}

#[test]
fn first_shortfall_date_detects_the_first_negative_balance() {
    let mut forecast = CashForecast::new();
    forecast.add(CashFlowItem {
        category: CashFlowCategory::Operating,
        description: "a".into(),
        amount: dec!(-150.0),
        date: date(2026, 1, 5),
    });
    forecast.add(CashFlowItem {
        category: CashFlowCategory::Operating,
        description: "b".into(),
        amount: dec!(200.0),
        date: date(2026, 1, 10),
    });

    assert_eq!(
        forecast.first_shortfall_date(dec!(100.0)).unwrap(),
        Some(date(2026, 1, 5))
    );
    assert_eq!(forecast.first_shortfall_date(dec!(200.0)).unwrap(), None);
}

#[test]
fn currency_code_requires_three_ascii_uppercase_letters() {
    assert!(CurrencyCode::new("USD").is_ok());
    assert!(CurrencyCode::new("us").is_err());
    assert!(CurrencyCode::new("usd").is_err());
    assert!(CurrencyCode::new("USDD").is_err());
}

#[test]
fn fx_conversion_rejects_currency_mismatch() {
    let eur = CurrencyCode::new("EUR").unwrap();
    let gbp = CurrencyCode::new("GBP").unwrap();
    let usd = CurrencyCode::new("USD").unwrap();
    let exposure = FxExposure {
        currency: eur,
        amount: dec!(1000.0),
    };
    let mismatched_rate = ExchangeRate {
        from: gbp,
        to: usd,
        rate: dec!(1.25),
        as_of: date(2026, 1, 1),
    };

    let result = exposure.convert(&mismatched_rate);
    assert!(matches!(result, Err(ErpError::CurrencyMismatch { .. })));
}

#[test]
fn fx_revaluation_and_hedge_offset_each_other() {
    let eur = CurrencyCode::new("EUR").unwrap();
    let usd = CurrencyCode::new("USD").unwrap();
    let exposure = FxExposure {
        currency: eur,
        amount: dec!(1000.0),
    };

    let old_rate = ExchangeRate {
        from: eur,
        to: usd,
        rate: dec!(1.10),
        as_of: date(2026, 1, 1),
    };
    let new_rate = ExchangeRate {
        from: eur,
        to: usd,
        rate: dec!(1.15),
        as_of: date(2026, 2, 1),
    };

    let exposure_gain =
        casiros_erp::treasury::fx::revaluation_gain_loss(&exposure, &old_rate, &new_rate).unwrap();
    assert_eq!(exposure_gain, dec!(50.0));

    // A forward contract to SELL EUR at 1.10 loses value as EUR strengthens
    // past that locked-in rate: exactly enough to offset the exposure's gain.
    let forward = ForwardContract {
        notional: dec!(1000.0),
        currency: eur,
        forward_rate: dec!(1.10),
        settlement_date: date(2026, 2, 1),
    };
    let hedge_gain = forward.gain_loss(dec!(1.15)).unwrap();
    assert_eq!(hedge_gain, dec!(-50.0));

    let effectiveness = hedge_effectiveness(hedge_gain, exposure_gain).unwrap();
    assert_eq!(effectiveness, dec!(1.0));
    assert!(is_highly_effective(effectiveness));
}

#[test]
fn hedge_effectiveness_boundary_is_inclusive_on_both_sides() {
    assert!(is_highly_effective(dec!(0.80)));
    assert!(!is_highly_effective(dec!(0.79)));
    assert!(is_highly_effective(dec!(1.25)));
    assert!(!is_highly_effective(dec!(1.26)));
}

#[test]
fn hedge_effectiveness_rejects_zero_exposure_movement() {
    let result = hedge_effectiveness(dec!(-50.0), dec!(0.0));
    assert!(result.is_err());
}
