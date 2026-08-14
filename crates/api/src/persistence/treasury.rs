//! Postgres-backed cash flow forecast items.
//!
//! Storage-only: `CashForecast::projected_balance`/`first_shortfall_date`
//! (pure methods over its own internally-held items) stay exactly as
//! `casiros_erp` defines them — this module fetches the stored items and
//! rebuilds an ephemeral in-memory `CashForecast` to run them against
//! (`CashForecast` has no bulk constructor, only `new()` + `add()`).

use crate::error::AppError;
use casiros_erp::treasury::cashflow::{CashFlowCategory, CashFlowItem, CashForecast};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;

fn category_to_text(category: CashFlowCategory) -> &'static str {
    match category {
        CashFlowCategory::Operating => "Operating",
        CashFlowCategory::Investing => "Investing",
        CashFlowCategory::Financing => "Financing",
    }
}

fn category_from_text(text: &str) -> Result<CashFlowCategory, AppError> {
    match text {
        "Operating" => Ok(CashFlowCategory::Operating),
        "Investing" => Ok(CashFlowCategory::Investing),
        "Financing" => Ok(CashFlowCategory::Financing),
        other => Err(AppError::Internal(format!(
            "corrupt cash_flow_items.category {other:?} in database"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct CashFlowItemRow {
    category: String,
    description: String,
    amount: Decimal,
    date: NaiveDate,
}

impl TryFrom<CashFlowItemRow> for CashFlowItem {
    type Error = AppError;

    fn try_from(row: CashFlowItemRow) -> Result<Self, Self::Error> {
        Ok(CashFlowItem {
            category: category_from_text(&row.category)?,
            description: row.description,
            amount: row.amount,
            date: row.date,
        })
    }
}

/// Adds an item to the cash flow forecast.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
pub async fn add_item(pool: &PgPool, item: &CashFlowItem) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO cash_flow_items (category, description, amount, date) VALUES ($1, $2, $3, $4)",
    )
    .bind(category_to_text(item.category))
    .bind(&item.description)
    .bind(item.amount)
    .bind(item.date)
    .execute(pool)
    .await?;
    Ok(())
}

/// Loads every stored item into a fresh, in-memory [`CashForecast`] — the
/// insertion order (`id`, `BIGSERIAL`) is preserved, matching what
/// `CashForecast::first_shortfall_date`'s stable sort relies on to
/// tie-break same-day items exactly as the in-memory version did.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
pub async fn load_forecast(pool: &PgPool) -> Result<CashForecast, AppError> {
    let rows: Vec<CashFlowItemRow> = sqlx::query_as(
        "SELECT category, description, amount, date FROM cash_flow_items ORDER BY id",
    )
    .fetch_all(pool)
    .await?;
    let mut forecast = CashForecast::new();
    for row in rows {
        forecast.add(CashFlowItem::try_from(row)?);
    }
    Ok(forecast)
}
