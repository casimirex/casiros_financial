//! Postgres-backed chart of accounts, journal, and balances.
//!
//! Balances are computed by SQL aggregation on read, not incrementally
//! maintained — this replaces the in-memory `Ledger`'s entire dirty-tracking
//! subsystem. A leaf account's balance is `SUM(debit - credit)` over its own
//! journal lines; a roll-up account's balance is the same sum over every
//! *leaf* account in its subtree (its own direct postings, if any, are
//! excluded, matching `casiros_erp::ledger::consolidation::recompute_rollups`'s
//! existing behavior of overwriting a parent's balance with its children's
//! sum rather than adding to it).
//!
//! `ChartOfAccounts::register`'s two invariants (no duplicate code, parent
//! must already exist) are enforced by Postgres's own `PRIMARY KEY` and
//! `FOREIGN KEY` constraints rather than re-implemented here — a cycle in
//! the account hierarchy is therefore unreachable through this module's own
//! functions, but the balance query still guards against one (via a
//! recursive CTE `CYCLE` clause) in case a hierarchy is ever constructed by
//! a route this module doesn't know about.

use crate::error::AppError;
use casiros_core::types::Dollar;
use casiros_dag::graph::FormulaNode;
use casiros_erp::error::ErpError;
use casiros_erp::ledger::account::{Account, AccountCode, AccountType};
use casiros_erp::ledger::journal::{JournalEntry, JournalLine, SourceDocument};
use casiros_erp::ledger::period::FiscalPeriod;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{PgPool, Postgres, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

fn account_type_to_text(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Asset => "Asset",
        AccountType::Liability => "Liability",
        AccountType::Equity => "Equity",
        AccountType::Revenue => "Revenue",
        AccountType::Expense => "Expense",
    }
}

fn account_type_from_text(text: &str) -> Result<AccountType, AppError> {
    match text {
        "Asset" => Ok(AccountType::Asset),
        "Liability" => Ok(AccountType::Liability),
        "Equity" => Ok(AccountType::Equity),
        "Revenue" => Ok(AccountType::Revenue),
        "Expense" => Ok(AccountType::Expense),
        other => Err(AppError::Internal(format!(
            "corrupt account_type {other:?} in database"
        ))),
    }
}

fn code_to_i64(code: AccountCode) -> i64 {
    i64::from(code.0)
}

fn code_from_i64(value: i64) -> Result<AccountCode, AppError> {
    u32::try_from(value)
        .map(AccountCode)
        .map_err(|_| AppError::Internal(format!("account code {value} out of u32 range")))
}

#[derive(sqlx::FromRow)]
struct AccountRow {
    code: i64,
    name: String,
    account_type: String,
    parent_code: Option<i64>,
}

impl TryFrom<AccountRow> for Account {
    type Error = AppError;

    fn try_from(row: AccountRow) -> Result<Self, Self::Error> {
        Ok(Account {
            code: code_from_i64(row.code)?,
            name: row.name,
            account_type: account_type_from_text(&row.account_type)?,
            parent: row.parent_code.map(code_from_i64).transpose()?,
        })
    }
}

/// Registers a new account in the chart of accounts.
///
/// # Errors
///
/// Returns [`ErpError::DuplicateAccount`] if `account.code` is already
/// registered, or [`ErpError::UnknownAccount`] if `account.parent` is set
/// but not itself registered.
pub async fn register_account(pool: &PgPool, account: &Account) -> Result<(), AppError> {
    let result = sqlx::query(
        "INSERT INTO accounts (code, name, account_type, parent_code) VALUES ($1, $2, $3, $4)",
    )
    .bind(code_to_i64(account.code))
    .bind(&account.name)
    .bind(account_type_to_text(account.account_type))
    .bind(account.parent.map(code_to_i64))
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(map_constraint_violation(err, account.code, account.parent)),
    }
}

/// Maps a Postgres constraint violation from [`register_account`] onto the
/// same [`ErpError`] variant the in-memory `ChartOfAccounts::register` used
/// to return for the equivalent condition.
fn map_constraint_violation(
    err: sqlx::Error,
    code: AccountCode,
    parent: Option<AccountCode>,
) -> AppError {
    let sqlstate = err
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code);
    match sqlstate.as_deref() {
        Some("23505") => AppError::Erp(ErpError::DuplicateAccount(code)),
        Some("23503") => AppError::Erp(ErpError::UnknownAccount(parent.unwrap_or(code))),
        _ => AppError::Database(err),
    }
}

/// Lists every registered account.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
pub async fn list_accounts(pool: &PgPool) -> Result<Vec<Account>, AppError> {
    let rows: Vec<AccountRow> =
        sqlx::query_as("SELECT code, name, account_type, parent_code FROM accounts")
            .fetch_all(pool)
            .await?;
    rows.into_iter().map(Account::try_from).collect()
}

/// Gets one account by code.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
pub async fn get_account(pool: &PgPool, code: AccountCode) -> Result<Option<Account>, AppError> {
    let row: Option<AccountRow> = sqlx::query_as(
        "SELECT code, name, account_type, parent_code FROM accounts WHERE code = $1",
    )
    .bind(code_to_i64(code))
    .fetch_optional(pool)
    .await?;
    row.map(Account::try_from).transpose()
}

/// The trial-balance recursive CTE: `root`'s balance is the sum of
/// `debit - credit` over every *leaf* account in `root`'s subtree
/// (including `root` itself, if it is itself a leaf). The `CYCLE` clause
/// guards against a hierarchy this module didn't itself create.
const BALANCE_QUERY: &str = "
    WITH RECURSIVE subtree(code, root) AS (
        SELECT code, code FROM accounts
        UNION ALL
        SELECT a.code, s.root FROM accounts a JOIN subtree s ON a.parent_code = s.code
    ) CYCLE code SET cycle_detected USING path
    SELECT s.root AS account,
           bool_or(s.cycle_detected) AS any_cycle,
           COALESCE(SUM(jl.debit - jl.credit), 0) AS balance
    FROM subtree s
    JOIN accounts leaf ON leaf.code = s.code
    LEFT JOIN accounts child ON child.parent_code = leaf.code
    LEFT JOIN journal_lines jl ON jl.account = s.code
    WHERE child.code IS NULL
    GROUP BY s.root
";

#[derive(sqlx::FromRow)]
struct BalanceRow {
    account: i64,
    any_cycle: Option<bool>,
    balance: Decimal,
}

/// Computes the full trial balance: every registered account's current balance.
///
/// # Errors
///
/// Returns [`AppError::Internal`] if the account hierarchy contains a cycle
/// (unreachable through [`register_account`]'s own constraints, but guarded
/// against regardless — see the module docs). Returns [`AppError::Database`]
/// if the query fails.
pub async fn trial_balance(pool: &PgPool) -> Result<HashMap<AccountCode, Dollar>, AppError> {
    let rows: Vec<BalanceRow> = sqlx::query_as(BALANCE_QUERY).fetch_all(pool).await?;
    let mut balances = HashMap::with_capacity(rows.len());
    for row in rows {
        if row.any_cycle.unwrap_or(false) {
            return Err(AppError::Internal(
                "cyclic account hierarchy detected while computing balances".to_string(),
            ));
        }
        balances.insert(code_from_i64(row.account)?, row.balance);
    }
    Ok(balances)
}

/// Computes one account's current balance (`0` if it has never been posted to).
///
/// # Errors
///
/// Returns [`AppError::NotFound`] if `code` is not registered. Returns
/// [`AppError::Internal`] if the account hierarchy contains a cycle. Returns
/// [`AppError::Database`] if the query fails.
pub async fn account_balance(pool: &PgPool, code: AccountCode) -> Result<Dollar, AppError> {
    if get_account(pool, code).await?.is_none() {
        return Err(AppError::NotFound(format!(
            "no account with code {}",
            code.0
        )));
    }
    let balances = trial_balance(pool).await?;
    Ok(balances.get(&code).copied().unwrap_or(Decimal::ZERO))
}

fn source_document_to_columns(doc: &SourceDocument) -> (&'static str, Option<Uuid>) {
    match doc {
        SourceDocument::ManualEntry => ("ManualEntry", None),
        SourceDocument::Invoice { id } => ("Invoice", Some(*id)),
        SourceDocument::Payment { id } => ("Payment", Some(*id)),
        SourceDocument::Receipt { id } => ("Receipt", Some(*id)),
        SourceDocument::Accrual => ("Accrual", None),
        SourceDocument::Consolidation => ("Consolidation", None),
    }
}

fn columns_to_source_document(kind: &str, id: Option<Uuid>) -> Result<SourceDocument, AppError> {
    match (kind, id) {
        ("ManualEntry", _) => Ok(SourceDocument::ManualEntry),
        ("Invoice", Some(id)) => Ok(SourceDocument::Invoice { id }),
        ("Payment", Some(id)) => Ok(SourceDocument::Payment { id }),
        ("Receipt", Some(id)) => Ok(SourceDocument::Receipt { id }),
        ("Accrual", _) => Ok(SourceDocument::Accrual),
        ("Consolidation", _) => Ok(SourceDocument::Consolidation),
        _ => Err(AppError::Internal(format!(
            "corrupt source document (kind {kind:?}, id {id:?}) in database"
        ))),
    }
}

fn period_month_to_i16(month: u32) -> Result<i16, AppError> {
    i16::try_from(month)
        .map_err(|_| AppError::Internal(format!("period month {month} out of range")))
}

fn period_month_from_i16(month: i16) -> Result<u32, AppError> {
    u32::try_from(month)
        .map_err(|_| AppError::Internal(format!("period month {month} out of range")))
}

/// Locks out concurrent posts/closes against the same fiscal period for the
/// remainder of the current transaction. A plain `SELECT ... FOR UPDATE`
/// can't do this on its own: `closed_periods` uses "row exists" to mean
/// closed, and `FOR UPDATE` only locks rows that already exist — it can't
/// prevent a concurrent transaction from also seeing "no row, so open" and
/// racing ahead. A transaction-scoped advisory lock, keyed by the period,
/// closes that gap regardless of whether the row exists yet.
async fn lock_period(
    tx: &mut Transaction<'_, Postgres>,
    period: FiscalPeriod,
) -> Result<(), AppError> {
    let key = format!("closed_period:{}:{}", period.year, period.month);
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
        .bind(key)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn period_is_closed(
    tx: &mut Transaction<'_, Postgres>,
    period: FiscalPeriod,
) -> Result<bool, AppError> {
    let month = period_month_to_i16(period.month)?;
    let row = sqlx::query("SELECT 1 FROM closed_periods WHERE year = $1 AND month = $2")
        .bind(period.year)
        .bind(month)
        .fetch_optional(&mut **tx)
        .await?;
    Ok(row.is_some())
}

/// Closes `period`: further postings to it are rejected. Mirrors
/// `casiros_erp::ledger::Ledger::close_period`. No HTTP route calls this
/// today (grepped both the routes and the frontend — there genuinely isn't
/// one), but the guarantee is preserved here regardless.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
pub async fn close_period(pool: &PgPool, period: FiscalPeriod) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    lock_period(&mut tx, period).await?;
    let month = period_month_to_i16(period.month)?;
    sqlx::query("INSERT INTO closed_periods (year, month) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(period.year)
        .bind(month)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// Posts `entry`: inserts it and every line in one transaction. `entry` is
/// assumed already validated (via [`JournalEntry::new`]/[`JournalLine::new`]
/// at the route layer) — this function only persists it.
///
/// # Errors
///
/// Returns [`ErpError::UnknownAccount`] if any line references an account
/// not in the chart. Returns [`ErpError::PeriodClosed`] if `entry.period`
/// has been closed. Returns [`AppError::Database`] if the query fails.
pub async fn post_entry(pool: &PgPool, entry: &JournalEntry) -> Result<(), AppError> {
    let mut tx = pool.begin().await?;
    lock_period(&mut tx, entry.period).await?;

    if period_is_closed(&mut tx, entry.period).await? {
        return Err(AppError::Erp(ErpError::PeriodClosed(entry.period)));
    }

    let referenced_codes: Vec<i64> = entry
        .lines
        .iter()
        .map(|line| code_to_i64(line.account))
        .collect();
    let existing: Vec<(i64,)> = sqlx::query_as("SELECT code FROM accounts WHERE code = ANY($1)")
        .bind(&referenced_codes)
        .fetch_all(&mut *tx)
        .await?;
    let existing: std::collections::HashSet<i64> =
        existing.into_iter().map(|(code,)| code).collect();
    if let Some(line) = entry
        .lines
        .iter()
        .find(|line| !existing.contains(&code_to_i64(line.account)))
    {
        return Err(AppError::Erp(ErpError::UnknownAccount(line.account)));
    }

    let (source_kind, source_id) = source_document_to_columns(&entry.source_document);
    let month = period_month_to_i16(entry.period.month)?;
    sqlx::query(
        "INSERT INTO journal_entries (id, date, description, causal_parent, source_kind, source_id, period_year, period_month)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(entry.id)
    .bind(entry.date)
    .bind(&entry.description)
    .bind(entry.causal_parent)
    .bind(source_kind)
    .bind(source_id)
    .bind(entry.period.year)
    .bind(month)
    .execute(&mut *tx)
    .await?;

    for (ordinal, line) in entry.lines.iter().enumerate() {
        let ordinal = i32::try_from(ordinal).map_err(|_| {
            AppError::Internal("journal entry has more lines than fit in i32".to_string())
        })?;
        sqlx::query(
            "INSERT INTO journal_lines (entry_id, line_ordinal, account, debit, credit, causal_formula)
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(entry.id)
        .bind(ordinal)
        .bind(code_to_i64(line.account))
        .bind(line.debit)
        .bind(line.credit)
        .bind(line.causal_formula.map(FormulaNode::name))
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

#[derive(sqlx::FromRow)]
struct EntryRow {
    id: Uuid,
    date: NaiveDate,
    description: String,
    causal_parent: Option<Uuid>,
    source_kind: String,
    source_id: Option<Uuid>,
    period_year: i32,
    period_month: i16,
}

#[derive(sqlx::FromRow)]
struct LineRow {
    entry_id: Uuid,
    account: i64,
    debit: Decimal,
    credit: Decimal,
    causal_formula: Option<String>,
}

/// Lists every journal entry posted so far, in posting order, with its lines.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails, or
/// [`AppError::Internal`] if a stored row is internally inconsistent (e.g. a
/// `causal_formula` value that no longer names a known formula).
pub async fn list_entries(pool: &PgPool) -> Result<Vec<JournalEntry>, AppError> {
    let entry_rows: Vec<EntryRow> = sqlx::query_as(
        "SELECT id, date, description, causal_parent, source_kind, source_id, period_year, period_month
         FROM journal_entries ORDER BY seq",
    )
    .fetch_all(pool)
    .await?;

    let line_rows: Vec<LineRow> = sqlx::query_as(
        "SELECT entry_id, account, debit, credit, causal_formula
         FROM journal_lines ORDER BY entry_id, line_ordinal",
    )
    .fetch_all(pool)
    .await?;

    let mut lines_by_entry: HashMap<Uuid, Vec<JournalLine>> = HashMap::new();
    for row in line_rows {
        let causal_formula = row
            .causal_formula
            .as_deref()
            .map(|name| {
                FormulaNode::from_name(name).ok_or_else(|| {
                    AppError::Internal(format!("corrupt causal_formula {name:?} in database"))
                })
            })
            .transpose()?;
        lines_by_entry
            .entry(row.entry_id)
            .or_default()
            .push(JournalLine {
                account: code_from_i64(row.account)?,
                debit: row.debit,
                credit: row.credit,
                causal_formula,
            });
    }

    entry_rows
        .into_iter()
        .map(|row| {
            let source_document = columns_to_source_document(&row.source_kind, row.source_id)?;
            let period = FiscalPeriod {
                year: row.period_year,
                month: period_month_from_i16(row.period_month)?,
            };
            Ok(JournalEntry {
                id: row.id,
                date: row.date,
                description: row.description,
                lines: lines_by_entry.remove(&row.id).unwrap_or_default(),
                causal_parent: row.causal_parent,
                source_document,
                period,
            })
        })
        .collect()
}
