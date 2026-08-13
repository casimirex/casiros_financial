//! Integration tests exercising `casiros-erp`'s `budget` module end-to-end.

use casiros_erp::budget::model::{BudgetModel, DriverBasedLineItem};
use casiros_erp::budget::variance::{analyze_variance, variance_report};
use casiros_erp::error::ErpError;
use casiros_erp::ledger::account::{AccountCode, AccountType};
use rust_decimal_macros::dec;

const REVENUE: AccountCode = AccountCode(4000);
const COGS: AccountCode = AccountCode(5000);

fn sample_model() -> BudgetModel {
    let mut model = BudgetModel::new();
    model.set_driver("units_sold", dec!(1000.0));
    model.set_driver("average_price", dec!(50.0));
    model.set_driver("unit_cost", dec!(30.0));
    model.add_line_item(DriverBasedLineItem {
        account: REVENUE,
        description: "revenue".into(),
        driver_names: vec!["units_sold".into(), "average_price".into()],
    });
    model.add_line_item(DriverBasedLineItem {
        account: COGS,
        description: "cogs".into(),
        driver_names: vec!["units_sold".into(), "unit_cost".into()],
    });
    model
}

#[test]
fn line_item_multiplies_named_drivers() {
    let model = sample_model();
    let revenue_item = &model.line_items()[0];
    assert_eq!(
        model.compute_line_item(revenue_item).unwrap(),
        dec!(50_000.0)
    );
}

#[test]
fn total_budget_sums_every_line_item() {
    let model = sample_model();
    assert_eq!(model.total_budget().unwrap(), dec!(80_000.0));
}

#[test]
fn changing_a_driver_propagates_to_every_line_item_referencing_it() {
    let mut model = sample_model();
    let before = model.total_budget().unwrap();

    model.set_driver("units_sold", dec!(1200.0));
    let after = model.total_budget().unwrap();

    // Revenue: 1200*50=60,000; COGS: 1200*30=36,000; total=96,000.
    assert_eq!(after, dec!(96_000.0));
    assert_ne!(before, after);
}

#[test]
fn unknown_driver_is_rejected() {
    let mut model = BudgetModel::new();
    model.set_driver("units_sold", dec!(1000.0));
    model.add_line_item(DriverBasedLineItem {
        account: REVENUE,
        description: "revenue".into(),
        driver_names: vec!["units_sold".into(), "missing_price".into()],
    });

    let result = model.total_budget();
    assert_eq!(
        result,
        Err(ErpError::UnknownDriver("missing_price".to_string()))
    );
}

#[test]
fn empty_driver_list_is_rejected() {
    let mut model = BudgetModel::new();
    model.add_line_item(DriverBasedLineItem {
        account: REVENUE,
        description: "revenue".into(),
        driver_names: vec![],
    });
    assert!(model.total_budget().is_err());
}

#[test]
fn revenue_favorable_when_actual_exceeds_budget() {
    let result = analyze_variance(
        REVENUE,
        AccountType::Revenue,
        dec!(100_000.0),
        dec!(110_000.0),
    )
    .unwrap();
    assert_eq!(result.variance, dec!(10_000.0));
    assert!(result.favorable);
    assert_eq!(result.variance_percent, Some(dec!(0.1)));
}

#[test]
fn revenue_unfavorable_when_actual_falls_short_of_budget() {
    let result = analyze_variance(
        REVENUE,
        AccountType::Revenue,
        dec!(100_000.0),
        dec!(90_000.0),
    )
    .unwrap();
    assert!(!result.favorable);
}

#[test]
fn expense_favorable_when_actual_is_below_budget() {
    let result =
        analyze_variance(COGS, AccountType::Expense, dec!(50_000.0), dec!(45_000.0)).unwrap();
    assert_eq!(result.variance, dec!(-5_000.0));
    assert!(result.favorable);
}

#[test]
fn expense_unfavorable_when_actual_exceeds_budget() {
    let result =
        analyze_variance(COGS, AccountType::Expense, dec!(50_000.0), dec!(55_000.0)).unwrap();
    assert!(!result.favorable);
}

#[test]
fn variance_percent_is_none_for_zero_budget() {
    let result = analyze_variance(REVENUE, AccountType::Revenue, dec!(0.0), dec!(500.0)).unwrap();
    assert_eq!(result.variance_percent, None);
}

#[test]
fn variance_report_runs_across_multiple_accounts() {
    let entries = [
        (
            REVENUE,
            AccountType::Revenue,
            dec!(100_000.0),
            dec!(110_000.0),
        ),
        (COGS, AccountType::Expense, dec!(50_000.0), dec!(55_000.0)),
    ];
    let report = variance_report(&entries).unwrap();
    assert_eq!(report.len(), 2);
    assert!(report[0].favorable);
    assert!(!report[1].favorable);
}
