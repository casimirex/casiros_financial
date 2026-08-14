//! Exercises the precondition-validation (`Err`) branches of `casiros-core`
//! formulas that each function's own `# Examples` doc-test — necessarily
//! showing only the happy path — doesn't reach. Every case here corresponds
//! to a documented `# Errors` behavior.

use casiros_core::{banking, corporate, financial, general, markets, stocks_bonds, types};
use rust_decimal_macros::dec;

// --- banking -----------------------------------------------------------------

#[test]
fn loan_to_deposit_ratio_rejects_negative_loans_and_nonpositive_deposits() {
    assert!(banking::loan_to_deposit_ratio(dec!(-1), dec!(1_000_000)).is_err());
    assert!(banking::loan_to_deposit_ratio(dec!(800_000), dec!(0)).is_err());
    assert!(banking::loan_to_deposit_ratio(dec!(800_000), dec!(-1)).is_err());
}

#[test]
fn capital_adequacy_ratio_rejects_negative_capital_and_nonpositive_rwa() {
    assert!(banking::capital_adequacy_ratio(dec!(-1), dec!(1_000_000)).is_err());
    assert!(banking::capital_adequacy_ratio(dec!(120_000), dec!(0)).is_err());
    assert!(banking::capital_adequacy_ratio(dec!(120_000), dec!(-1)).is_err());
}

#[test]
fn provision_coverage_rejects_negative_provisions_and_nonpositive_npls() {
    assert!(banking::provision_coverage(dec!(-1), dec!(100_000)).is_err());
    assert!(banking::provision_coverage(dec!(50_000), dec!(0)).is_err());
    assert!(banking::provision_coverage(dec!(50_000), dec!(-1)).is_err());
}

// --- corporate -----------------------------------------------------------------

#[test]
fn wacc_rejects_negative_equity_or_debt_and_zero_total() {
    assert!(corporate::wacc(dec!(-1), dec!(400_000), dec!(0.10), dec!(0.06), dec!(0.25)).is_err());
    assert!(corporate::wacc(dec!(600_000), dec!(-1), dec!(0.10), dec!(0.06), dec!(0.25)).is_err());
    assert!(
        corporate::wacc(
            dec!(600_000),
            dec!(400_000),
            dec!(0.10),
            dec!(0.06),
            dec!(1.5)
        )
        .is_err()
    );
    assert!(corporate::wacc(dec!(0), dec!(0), dec!(0.10), dec!(0.06), dec!(0.25)).is_err());
}

#[test]
fn economic_value_added_rejects_negative_invested_capital() {
    assert!(corporate::economic_value_added(dec!(200_000), dec!(-1), dec!(0.08)).is_err());
}

#[test]
fn internal_growth_rate_rejects_out_of_range_retention_ratio() {
    assert!(corporate::internal_growth_rate(dec!(0.10), dec!(1.5)).is_err());
    assert!(corporate::internal_growth_rate(dec!(0.10), dec!(-0.1)).is_err());
}

#[test]
fn internal_growth_rate_rejects_full_retention_of_all_returns() {
    // roa * retention_ratio == 1 makes the denominator (1 - retained_return) zero.
    assert!(corporate::internal_growth_rate(dec!(1.0), dec!(1.0)).is_err());
}

// --- financial -----------------------------------------------------------------

#[test]
fn dupont_roe_rejects_negative_asset_turnover() {
    assert!(financial::dupont_roe(dec!(0.10), dec!(-1), dec!(2.0)).is_err());
}

#[test]
fn current_ratio_rejects_negative_current_assets() {
    assert!(financial::current_ratio(dec!(-1), dec!(200_000)).is_err());
}

#[test]
fn quick_ratio_rejects_negative_current_assets_or_inventory() {
    assert!(financial::quick_ratio(dec!(-1), dec!(100_000), dec!(200_000)).is_err());
    assert!(financial::quick_ratio(dec!(400_000), dec!(-1), dec!(200_000)).is_err());
}

#[test]
fn debt_to_equity_rejects_negative_liabilities_and_zero_equity() {
    assert!(financial::debt_to_equity(dec!(-1), dec!(500_000)).is_err());
    assert!(financial::debt_to_equity(dec!(400_000), dec!(0)).is_err());
}

#[test]
fn inventory_turnover_rejects_negative_cogs() {
    assert!(financial::inventory_turnover(dec!(-1), dec!(150_000)).is_err());
}

#[test]
fn cash_conversion_cycle_rejects_any_negative_days_component() {
    assert!(financial::cash_conversion_cycle(dec!(-1), dec!(45), dec!(30)).is_err());
    assert!(financial::cash_conversion_cycle(dec!(60), dec!(-1), dec!(30)).is_err());
    assert!(financial::cash_conversion_cycle(dec!(60), dec!(45), dec!(-1)).is_err());
}

#[test]
fn return_on_assets_rejects_zero_and_negative_avg_total_assets() {
    assert!(financial::return_on_assets(dec!(100_000), dec!(0)).is_err());
    assert!(financial::return_on_assets(dec!(100_000), dec!(-1)).is_err());
}

// --- general -----------------------------------------------------------------

#[test]
fn future_value_rejects_a_rate_at_or_below_negative_one() {
    assert!(general::future_value(dec!(1000), dec!(-1.0), 10).is_err());
    assert!(general::future_value(dec!(1000), dec!(-2.0), 10).is_err());
}

#[test]
fn annuity_future_value_handles_zero_periods_and_zero_rate() {
    assert_eq!(
        general::annuity_future_value(dec!(1000), dec!(0.05), 0).unwrap(),
        dec!(0)
    );
    assert!(general::annuity_future_value(dec!(1000), dec!(0), 10).is_ok());
}

#[test]
fn annuity_present_value_handles_zero_periods_and_zero_rate() {
    assert_eq!(
        general::annuity_present_value(dec!(1000), dec!(0.05), 0).unwrap(),
        dec!(0)
    );
    assert!(general::annuity_present_value(dec!(1000), dec!(0), 10).is_ok());
}

#[test]
fn perpetuity_present_value_rejects_a_negative_rate() {
    assert!(general::perpetuity_present_value(dec!(100), dec!(-0.01)).is_err());
}

#[test]
fn growing_perpetuity_rejects_rate_equal_to_growth() {
    assert!(general::growing_perpetuity(dec!(100), dec!(0.05), dec!(0.05)).is_err());
}

#[test]
fn effective_annual_rate_rejects_zero_compounding_periods_and_invalid_rate() {
    assert!(general::effective_annual_rate(dec!(0.12), 0).is_err());
    assert!(general::effective_annual_rate(dec!(-100), 12).is_err());
}

#[test]
fn continuous_compounding_rejects_negative_time() {
    assert!(general::continuous_compounding(dec!(1000), dec!(0.05), dec!(-1)).is_err());
}

// --- markets -------------------------------------------------------------------

#[test]
fn expected_shortfall_rejects_confidence_outside_open_unit_interval() {
    assert!(markets::expected_shortfall(dec!(1_000_000), dec!(1.65), dec!(0.02), dec!(0)).is_err());
    assert!(markets::expected_shortfall(dec!(1_000_000), dec!(1.65), dec!(0.02), dec!(1)).is_err());
}

#[test]
fn value_at_risk_rejects_negative_portfolio_value_or_std_dev() {
    assert!(markets::value_at_risk(dec!(-1), dec!(1.65), dec!(0.02)).is_err());
    assert!(markets::value_at_risk(dec!(1_000_000), dec!(1.65), dec!(-1)).is_err());
}

#[test]
fn sharpe_ratio_rejects_a_nonpositive_std_dev() {
    assert!(markets::sharpe_ratio(dec!(0.10), dec!(0.03), dec!(0)).is_err());
    assert!(markets::sharpe_ratio(dec!(0.10), dec!(0.03), dec!(-1)).is_err());
}

// --- stocks_bonds ----------------------------------------------------------------

#[test]
fn dividend_discount_model_rejects_required_return_at_or_below_growth_rate() {
    assert!(stocks_bonds::dividend_discount_model(dec!(2.10), dec!(0.05), dec!(0.05)).is_err());
    assert!(stocks_bonds::dividend_discount_model(dec!(2.10), dec!(0.03), dec!(0.05)).is_err());
}

#[test]
fn discounted_cash_flow_rejects_empty_series_and_invalid_rate() {
    assert!(stocks_bonds::discounted_cash_flow(&[], dec!(0.10)).is_err());
    assert!(stocks_bonds::discounted_cash_flow(&[dec!(100)], dec!(-1.0)).is_err());
}

#[test]
fn bond_price_returns_face_value_at_zero_periods_and_rejects_invalid_rate() {
    assert_eq!(
        stocks_bonds::bond_price(dec!(1000), dec!(0.05), dec!(0.05), 0).unwrap(),
        dec!(1000)
    );
    assert!(stocks_bonds::bond_price(dec!(1000), dec!(0.05), dec!(-1.0), 10).is_err());
}

#[test]
fn yield_to_maturity_rejects_zero_periods_and_nonpositive_price_or_face_value() {
    assert!(stocks_bonds::yield_to_maturity(dec!(1000), dec!(1000), dec!(0.05), 0).is_err());
    assert!(stocks_bonds::yield_to_maturity(dec!(0), dec!(1000), dec!(0.05), 10).is_err());
    assert!(stocks_bonds::yield_to_maturity(dec!(1000), dec!(0), dec!(0.05), 10).is_err());
}

#[test]
fn duration_rejects_empty_series_and_invalid_rate() {
    assert!(stocks_bonds::duration(&[], dec!(0.05)).is_err());
    assert!(stocks_bonds::duration(&[dec!(100)], dec!(-1.0)).is_err());
}

#[test]
fn modified_duration_rejects_zero_periods_per_year() {
    assert!(stocks_bonds::modified_duration(dec!(5.5), dec!(0.08), 0).is_err());
}

#[test]
fn convexity_rejects_empty_series_and_invalid_rate() {
    assert!(stocks_bonds::convexity(&[], dec!(0.05)).is_err());
    assert!(stocks_bonds::convexity(&[dec!(100)], dec!(-1.0)).is_err());
}

// --- types -----------------------------------------------------------------------

#[test]
fn amounts_new_rejects_any_negative_field() {
    assert!(types::Amounts::new(dec!(-1), dec!(150), dec!(10)).is_err());
    assert!(types::Amounts::new(dec!(100), dec!(-1), dec!(10)).is_err());
    assert!(types::Amounts::new(dec!(100), dec!(150), dec!(-1)).is_err());
}
