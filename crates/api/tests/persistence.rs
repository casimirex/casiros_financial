//! Integration tests for the Postgres-backed persistence layer, exercised
//! against a real Postgres via testcontainers (see `tests/support`) — not a
//! fake/in-memory double, matching this repo's "test the real thing"
//! precedent (`actix-test`/`awc` for real HTTP, not mocked handlers).
//!
//! This suite is also the substitute for the compile-time SQL checking this
//! crate deliberately opts out of (see `crates/api/Cargo.toml`'s comment) —
//! every query in `persistence::ledger` is exercised here at least once
//! against a real schema.

mod support;

use casiros_api::error::AppError;
use casiros_api::persistence::ledger;
use casiros_erp::error::ErpError;
use casiros_erp::ledger::account::{Account, AccountCode, AccountType};
use casiros_erp::ledger::journal::{JournalEntry, JournalLine, SourceDocument};
use casiros_erp::ledger::period::FiscalPeriod;
use chrono::NaiveDate;
use rust_decimal_macros::dec;

fn asset_account(code: u32, name: &str, parent: Option<u32>) -> Account {
    Account {
        code: AccountCode(code),
        name: name.to_string(),
        account_type: AccountType::Asset,
        parent: parent.map(AccountCode),
    }
}

fn equity_account(code: u32) -> Account {
    Account {
        code: AccountCode(code),
        name: "Equity".to_string(),
        account_type: AccountType::Equity,
        parent: None,
    }
}

#[tokio::test]
async fn register_account_rejects_duplicate_code() {
    let db = support::test_db().await;

    let account = asset_account(1000, "Cash", None);
    ledger::register_account(&db.pool, &account).await.unwrap();

    let err = ledger::register_account(&db.pool, &account)
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        AppError::Erp(ErpError::DuplicateAccount(AccountCode(1000)))
    ));
}

#[tokio::test]
async fn register_account_rejects_unknown_parent() {
    let db = support::test_db().await;

    let account = asset_account(1100, "Cash", Some(9999));
    let err = ledger::register_account(&db.pool, &account)
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        AppError::Erp(ErpError::UnknownAccount(AccountCode(9999)))
    ));
}

#[tokio::test]
async fn trial_balance_computes_leaf_and_rollup_correctly() {
    let db = support::test_db().await;
    let pool = &db.pool;

    ledger::register_account(pool, &asset_account(1000, "Assets", None))
        .await
        .unwrap();
    ledger::register_account(pool, &asset_account(1100, "Cash", Some(1000)))
        .await
        .unwrap();
    ledger::register_account(pool, &asset_account(1200, "AR", Some(1000)))
        .await
        .unwrap();
    ledger::register_account(pool, &equity_account(3000))
        .await
        .unwrap();

    let entry = JournalEntry::new(
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        "Initial capital",
        vec![
            JournalLine::new(AccountCode(1100), dec!(500), dec!(0), None).unwrap(),
            JournalLine::new(AccountCode(3000), dec!(0), dec!(500), None).unwrap(),
        ],
        None,
        SourceDocument::ManualEntry,
        FiscalPeriod::new(2026, 8).unwrap(),
    )
    .unwrap();
    ledger::post_entry(pool, &entry).await.unwrap();

    let balances = ledger::trial_balance(pool).await.unwrap();
    assert_eq!(balances[&AccountCode(1100)], dec!(500)); // leaf, direct posting
    assert_eq!(balances[&AccountCode(1200)], dec!(0)); // leaf, no postings
    assert_eq!(balances[&AccountCode(1000)], dec!(500)); // rollup = sum of children
    assert_eq!(balances[&AccountCode(3000)], dec!(-500)); // credit-side leaf

    let single = ledger::account_balance(pool, AccountCode(1000))
        .await
        .unwrap();
    assert_eq!(single, dec!(500));
}

#[tokio::test]
async fn post_entry_rejects_unknown_account() {
    let db = support::test_db().await;
    let pool = &db.pool;

    ledger::register_account(pool, &asset_account(1000, "Cash", None))
        .await
        .unwrap();

    let entry = JournalEntry::new(
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        "Bad posting",
        vec![
            JournalLine::new(AccountCode(1000), dec!(100), dec!(0), None).unwrap(),
            JournalLine::new(AccountCode(9999), dec!(0), dec!(100), None).unwrap(),
        ],
        None,
        SourceDocument::ManualEntry,
        FiscalPeriod::new(2026, 8).unwrap(),
    )
    .unwrap();

    let err = ledger::post_entry(pool, &entry).await.unwrap_err();
    assert!(matches!(
        err,
        AppError::Erp(ErpError::UnknownAccount(AccountCode(9999)))
    ));

    // The whole entry must be rejected atomically — no partial line landed.
    let entries = ledger::list_entries(pool).await.unwrap();
    assert!(entries.is_empty());
}

#[tokio::test]
async fn post_entry_rejects_closed_period() {
    let db = support::test_db().await;
    let pool = &db.pool;

    ledger::register_account(pool, &asset_account(1000, "Cash", None))
        .await
        .unwrap();
    ledger::register_account(pool, &equity_account(3000))
        .await
        .unwrap();

    let period = FiscalPeriod::new(2026, 8).unwrap();
    ledger::close_period(pool, period).await.unwrap();

    let entry = JournalEntry::new(
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        "Late entry",
        vec![
            JournalLine::new(AccountCode(1000), dec!(100), dec!(0), None).unwrap(),
            JournalLine::new(AccountCode(3000), dec!(0), dec!(100), None).unwrap(),
        ],
        None,
        SourceDocument::ManualEntry,
        period,
    )
    .unwrap();

    let err = ledger::post_entry(pool, &entry).await.unwrap_err();
    assert!(matches!(err, AppError::Erp(ErpError::PeriodClosed(_))));
}

#[tokio::test]
async fn list_entries_preserves_posting_order_and_causal_links() {
    let db = support::test_db().await;
    let pool = &db.pool;

    ledger::register_account(pool, &asset_account(1000, "Cash", None))
        .await
        .unwrap();
    ledger::register_account(pool, &equity_account(3000))
        .await
        .unwrap();

    let period = FiscalPeriod::new(2026, 8).unwrap();
    let first = JournalEntry::new(
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        "First",
        vec![
            JournalLine::new(AccountCode(1000), dec!(100), dec!(0), None).unwrap(),
            JournalLine::new(AccountCode(3000), dec!(0), dec!(100), None).unwrap(),
        ],
        None,
        SourceDocument::ManualEntry,
        period,
    )
    .unwrap();
    ledger::post_entry(pool, &first).await.unwrap();

    let second = JournalEntry::new(
        NaiveDate::from_ymd_opt(2026, 8, 2).unwrap(),
        "Second, caused by the first",
        vec![
            JournalLine::new(AccountCode(1000), dec!(50), dec!(0), None).unwrap(),
            JournalLine::new(AccountCode(3000), dec!(0), dec!(50), None).unwrap(),
        ],
        Some(first.id),
        SourceDocument::ManualEntry,
        period,
    )
    .unwrap();
    ledger::post_entry(pool, &second).await.unwrap();

    let entries = ledger::list_entries(pool).await.unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].id, first.id);
    assert_eq!(entries[1].id, second.id);
    assert_eq!(entries[1].causal_parent, Some(first.id));
}

#[tokio::test]
async fn dangling_causal_parent_is_accepted_not_validated() {
    // Matches casiros_erp::ledger::journal::JournalEntry::new, which never
    // validated causal_parent against existing entries — a Postgres FK here
    // would silently tighten behavior that succeeds today.
    let db = support::test_db().await;
    let pool = &db.pool;

    ledger::register_account(pool, &asset_account(1000, "Cash", None))
        .await
        .unwrap();
    ledger::register_account(pool, &equity_account(3000))
        .await
        .unwrap();

    let entry = JournalEntry::new(
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        "Orphaned causal link",
        vec![
            JournalLine::new(AccountCode(1000), dec!(10), dec!(0), None).unwrap(),
            JournalLine::new(AccountCode(3000), dec!(0), dec!(10), None).unwrap(),
        ],
        Some(uuid::Uuid::new_v4()),
        SourceDocument::ManualEntry,
        FiscalPeriod::new(2026, 8).unwrap(),
    )
    .unwrap();

    ledger::post_entry(pool, &entry).await.unwrap();
}
