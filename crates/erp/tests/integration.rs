//! Integration tests exercising `casiros-erp`'s public API end-to-end.

use casiros_erp::business_rules::can_approve_payment;
use casiros_erp::error::ErpError;
use casiros_erp::ledger::Ledger;
use casiros_erp::ledger::account::{Account, AccountCode, AccountType, ChartOfAccounts};
use casiros_erp::ledger::journal::{JournalEntry, JournalLine, SourceDocument};
use casiros_erp::ledger::period::FiscalPeriod;
use chrono::NaiveDate;
use rust_decimal_macros::dec;

const CASH: AccountCode = AccountCode(1000);
const ACCOUNTS_RECEIVABLE: AccountCode = AccountCode(1100);
const TOTAL_CURRENT_ASSETS: AccountCode = AccountCode(1900);
const REVENUE: AccountCode = AccountCode(4000);

fn sample_chart() -> ChartOfAccounts {
    let mut chart = ChartOfAccounts::new();
    chart
        .register(Account {
            code: TOTAL_CURRENT_ASSETS,
            name: "Total Current Assets".into(),
            account_type: AccountType::Asset,
            parent: None,
        })
        .unwrap();
    chart
        .register(Account {
            code: CASH,
            name: "Cash".into(),
            account_type: AccountType::Asset,
            parent: Some(TOTAL_CURRENT_ASSETS),
        })
        .unwrap();
    chart
        .register(Account {
            code: ACCOUNTS_RECEIVABLE,
            name: "Accounts Receivable".into(),
            account_type: AccountType::Asset,
            parent: Some(TOTAL_CURRENT_ASSETS),
        })
        .unwrap();
    chart
        .register(Account {
            code: REVENUE,
            name: "Revenue".into(),
            account_type: AccountType::Revenue,
            parent: None,
        })
        .unwrap();
    chart
}

fn period() -> FiscalPeriod {
    FiscalPeriod::new(2026, 1).unwrap()
}

fn posting_date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()
}

#[test]
fn chart_rejects_duplicate_account_codes() {
    let mut chart = ChartOfAccounts::new();
    let account = Account {
        code: CASH,
        name: "Cash".into(),
        account_type: AccountType::Asset,
        parent: None,
    };
    chart.register(account.clone()).unwrap();
    assert_eq!(
        chart.register(account),
        Err(ErpError::DuplicateAccount(CASH))
    );
}

#[test]
fn chart_rejects_parent_that_does_not_exist_yet() {
    let mut chart = ChartOfAccounts::new();
    let orphan = Account {
        code: CASH,
        name: "Cash".into(),
        account_type: AccountType::Asset,
        parent: Some(TOTAL_CURRENT_ASSETS),
    };
    // TOTAL_CURRENT_ASSETS was never registered: this structurally prevents the
    // roll-up hierarchy from ever containing a cycle, since a parent must exist
    // before any child can reference it.
    assert_eq!(
        chart.register(orphan),
        Err(ErpError::UnknownAccount(TOTAL_CURRENT_ASSETS))
    );
}

#[test]
fn journal_line_rejects_both_debit_and_credit_set() {
    let result = JournalLine::new(CASH, dec!(100.0), dec!(50.0), None);
    assert_eq!(result, Err(ErpError::InvalidLine(CASH)));
}

#[test]
fn journal_line_rejects_neither_debit_nor_credit_set() {
    let result = JournalLine::new(CASH, dec!(0.0), dec!(0.0), None);
    assert_eq!(result, Err(ErpError::InvalidLine(CASH)));
}

#[test]
fn journal_entry_rejects_unbalanced_lines() {
    let lines = vec![
        JournalLine::new(CASH, dec!(1000.0), dec!(0.0), None).unwrap(),
        JournalLine::new(REVENUE, dec!(0.0), dec!(900.0), None).unwrap(),
    ];
    let result = JournalEntry::new(
        posting_date(),
        "unbalanced sale",
        lines,
        None,
        SourceDocument::ManualEntry,
        period(),
    );
    assert!(matches!(result, Err(ErpError::UnbalancedEntry { .. })));
}

#[test]
fn journal_entry_rejects_empty_lines() {
    let result = JournalEntry::new(
        posting_date(),
        "empty",
        vec![],
        None,
        SourceDocument::ManualEntry,
        period(),
    );
    assert!(result.is_err());
}

fn cash_sale_entry(amount: rust_decimal::Decimal) -> JournalEntry {
    let lines = vec![
        JournalLine::new(CASH, amount, dec!(0.0), None).unwrap(),
        JournalLine::new(REVENUE, dec!(0.0), amount, None).unwrap(),
    ];
    JournalEntry::new(
        posting_date(),
        "cash sale",
        lines,
        None,
        SourceDocument::ManualEntry,
        period(),
    )
    .unwrap()
}

#[test]
fn posting_updates_leaf_balances_incrementally() {
    let mut ledger = Ledger::new(sample_chart());
    ledger.post(cash_sale_entry(dec!(1000.0))).unwrap();
    ledger.post(cash_sale_entry(dec!(500.0))).unwrap();

    // Cash is debit-normal: two debits of 1000 and 500 sum to 1500.
    assert_eq!(ledger.balance(CASH).unwrap(), dec!(1500.0));
    // Revenue is tracked uniformly as debit - credit, so two credits net -1500.
    assert_eq!(ledger.balance(REVENUE).unwrap(), dec!(-1500.0));
}

#[test]
fn trial_balance_rollup_equals_sum_of_children_via_the_dag() {
    let mut ledger = Ledger::new(sample_chart());
    ledger.post(cash_sale_entry(dec!(1000.0))).unwrap();

    let receivable_line_debit =
        JournalLine::new(ACCOUNTS_RECEIVABLE, dec!(2000.0), dec!(0.0), None).unwrap();
    let receivable_line_credit = JournalLine::new(REVENUE, dec!(0.0), dec!(2000.0), None).unwrap();
    let credit_sale = JournalEntry::new(
        posting_date(),
        "credit sale",
        vec![receivable_line_debit, receivable_line_credit],
        None,
        SourceDocument::ManualEntry,
        period(),
    )
    .unwrap();
    ledger.post(credit_sale).unwrap();

    // TOTAL_CURRENT_ASSETS was never posted to directly: its balance only
    // exists because consolidation::recompute_rollups summed its children
    // (Cash=1000, AR=2000) via the topologically-ordered DAG walk.
    assert_eq!(ledger.balance(TOTAL_CURRENT_ASSETS).unwrap(), dec!(3000.0));

    let trial_balance = ledger.trial_balance().unwrap();
    assert_eq!(trial_balance[&CASH], dec!(1000.0));
    assert_eq!(trial_balance[&ACCOUNTS_RECEIVABLE], dec!(2000.0));
    assert_eq!(trial_balance[&TOTAL_CURRENT_ASSETS], dec!(3000.0));
}

#[test]
fn posting_to_an_unknown_account_is_rejected() {
    let mut ledger = Ledger::new(sample_chart());
    let unknown = AccountCode(9999);
    let lines = vec![
        JournalLine::new(unknown, dec!(100.0), dec!(0.0), None).unwrap(),
        JournalLine::new(REVENUE, dec!(0.0), dec!(100.0), None).unwrap(),
    ];
    let entry = JournalEntry::new(
        posting_date(),
        "bad account",
        lines,
        None,
        SourceDocument::ManualEntry,
        period(),
    )
    .unwrap();
    assert_eq!(ledger.post(entry), Err(ErpError::UnknownAccount(unknown)));
}

#[test]
fn posting_to_a_closed_period_is_rejected() {
    let mut ledger = Ledger::new(sample_chart());
    ledger.close_period(period());
    let result = ledger.post(cash_sale_entry(dec!(100.0)));
    assert_eq!(result, Err(ErpError::PeriodClosed(period())));
}

#[test]
fn can_approve_payment_requires_both_sufficient_cash_and_healthy_ratio() {
    // Sufficient cash, but ratio (10_000 / 10_000 = 1.0) is not > 1.2.
    assert!(!can_approve_payment(dec!(5_000.0), dec!(10_000.0), dec!(10_000.0)).unwrap());
    // Ratio exactly 1.2 is still not strictly greater than 1.2.
    assert!(!can_approve_payment(dec!(5_000.0), dec!(12_000.0), dec!(10_000.0)).unwrap());
    // Ratio just above 1.2 and cash covers the payment.
    assert!(can_approve_payment(dec!(5_000.0), dec!(12_000.01), dec!(10_000.0)).unwrap());
}
