//! Benchmarks the causal general ledger's two hottest operations: posting a
//! batch of journal entries, and recomputing the roll-up trial balance (the
//! DAG-based consolidation walk in `casiros_erp::ledger::consolidation`).

use casiros_erp::ledger::Ledger;
use casiros_erp::ledger::account::{Account, AccountCode, AccountType, ChartOfAccounts};
use casiros_erp::ledger::journal::{JournalEntry, JournalLine, SourceDocument};
use casiros_erp::ledger::period::FiscalPeriod;
use chrono::NaiveDate;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use rust_decimal_macros::dec;
use std::hint::black_box;

const LEAF_COUNT: u32 = 20;
const ENTRY_COUNT: u32 = 100;

fn build_ledger_with_accounts() -> Ledger {
    let mut ledger = Ledger::new(ChartOfAccounts::new());
    ledger
        .register_account(Account {
            code: AccountCode(1),
            name: "Total Assets".to_string(),
            account_type: AccountType::Asset,
            parent: None,
        })
        .unwrap();
    for i in 0..LEAF_COUNT {
        ledger
            .register_account(Account {
                code: AccountCode(100 + i),
                name: format!("Cash Account {i}"),
                account_type: AccountType::Asset,
                parent: Some(AccountCode(1)),
            })
            .unwrap();
    }
    ledger
        .register_account(Account {
            code: AccountCode(2),
            name: "Owner Equity".to_string(),
            account_type: AccountType::Equity,
            parent: None,
        })
        .unwrap();
    ledger
}

fn sample_entries() -> Vec<JournalEntry> {
    let date = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let period = FiscalPeriod::new(2026, 1).unwrap();
    (0..ENTRY_COUNT)
        .map(|i| {
            let leaf = AccountCode(100 + (i % LEAF_COUNT));
            let lines = vec![
                JournalLine::new(leaf, dec!(100), dec!(0), None).unwrap(),
                JournalLine::new(AccountCode(2), dec!(0), dec!(100), None).unwrap(),
            ];
            JournalEntry::new(
                date,
                format!("Entry {i}"),
                lines,
                None,
                SourceDocument::ManualEntry,
                period,
            )
            .unwrap()
        })
        .collect()
}

fn bench_post_entries(c: &mut Criterion) {
    let entries = sample_entries();
    c.bench_function("ledger_post_100_entries", |b| {
        b.iter_batched(
            build_ledger_with_accounts,
            |mut ledger| {
                for entry in &entries {
                    ledger.post(black_box(entry.clone())).unwrap();
                }
            },
            BatchSize::SmallInput,
        );
    });
}

fn bench_trial_balance_rollup(c: &mut Criterion) {
    let mut ledger = build_ledger_with_accounts();
    for entry in sample_entries() {
        ledger.post(entry).unwrap();
    }
    c.bench_function("ledger_trial_balance_rollup", |b| {
        b.iter_batched(
            || ledger.clone(),
            |mut cloned| black_box(cloned.trial_balance()),
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, bench_post_entries, bench_trial_balance_rollup);
criterion_main!(benches);
