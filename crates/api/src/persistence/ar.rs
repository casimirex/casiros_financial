//! Postgres-backed customers and AR invoices.
//!
//! Storage-only, like `persistence::ap`: `allocate_receipt` (pure, mutates a
//! `&mut [ArInvoice]` in place) stays exactly as `casiros_erp` defines it —
//! this module wraps it in a transaction that fetches every invoice, runs
//! the same allocation, and writes every resulting balance back atomically.

use crate::error::AppError;
use casiros_erp::ap::supplier::PaymentTerms;
use casiros_erp::ar::customer::{Customer, CustomerId};
use casiros_erp::ar::invoice::{ArInvoice, ArInvoiceId, ArInvoiceStatus, RecognitionMethod};
use casiros_erp::ar::receipt::{Receipt, ReceiptAllocation};
use casiros_erp::ledger::account::AccountCode;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;

fn status_to_text(status: ArInvoiceStatus) -> &'static str {
    match status {
        ArInvoiceStatus::Open => "Open",
        ArInvoiceStatus::PartiallyCollected => "PartiallyCollected",
        ArInvoiceStatus::Collected => "Collected",
    }
}

fn status_from_text(text: &str) -> Result<ArInvoiceStatus, AppError> {
    match text {
        "Open" => Ok(ArInvoiceStatus::Open),
        "PartiallyCollected" => Ok(ArInvoiceStatus::PartiallyCollected),
        "Collected" => Ok(ArInvoiceStatus::Collected),
        other => Err(AppError::Internal(format!(
            "corrupt ar_invoices.status {other:?} in database"
        ))),
    }
}

fn net_days_to_i32(net_days: u32) -> Result<i32, AppError> {
    i32::try_from(net_days)
        .map_err(|_| AppError::Internal(format!("net_days {net_days} out of range")))
}

fn net_days_from_i32(value: i32) -> Result<u32, AppError> {
    u32::try_from(value).map_err(|_| AppError::Internal(format!("net_days {value} out of range")))
}

fn discount_days_to_i32(days: Option<u32>) -> Result<Option<i32>, AppError> {
    days.map(net_days_to_i32).transpose()
}

fn discount_days_from_i32(value: Option<i32>) -> Result<Option<u32>, AppError> {
    value.map(net_days_from_i32).transpose()
}

/// The four columns [`RecognitionMethod`] flattens into.
struct RecognitionColumns {
    kind: &'static str,
    date: Option<NaiveDate>,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
}

fn recognition_to_columns(method: RecognitionMethod) -> RecognitionColumns {
    match method {
        RecognitionMethod::PointInTime { recognition_date } => RecognitionColumns {
            kind: "PointInTime",
            date: Some(recognition_date),
            start: None,
            end: None,
        },
        RecognitionMethod::RatablyOverTime { start, end } => RecognitionColumns {
            kind: "RatablyOverTime",
            date: None,
            start: Some(start),
            end: Some(end),
        },
    }
}

fn columns_to_recognition(
    kind: &str,
    date: Option<NaiveDate>,
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
) -> Result<RecognitionMethod, AppError> {
    match (kind, date, start, end) {
        ("PointInTime", Some(recognition_date), None, None) => {
            Ok(RecognitionMethod::PointInTime { recognition_date })
        }
        ("RatablyOverTime", None, Some(start), Some(end)) => {
            Ok(RecognitionMethod::RatablyOverTime { start, end })
        }
        _ => Err(AppError::Internal(format!(
            "corrupt recognition method (kind {kind:?}, date {date:?}, start {start:?}, end {end:?}) in database"
        ))),
    }
}

#[derive(sqlx::FromRow)]
struct CustomerRow {
    id: Uuid,
    name: String,
    credit_limit: Decimal,
    net_days: i32,
    discount_percent: Option<Decimal>,
    discount_days: Option<i32>,
    receivable_account: i64,
}

impl TryFrom<CustomerRow> for Customer {
    type Error = AppError;

    fn try_from(row: CustomerRow) -> Result<Self, Self::Error> {
        Ok(Customer {
            id: CustomerId(row.id),
            name: row.name,
            credit_limit: row.credit_limit,
            payment_terms: PaymentTerms {
                net_days: net_days_from_i32(row.net_days)?,
                discount_percent: row.discount_percent,
                discount_days: discount_days_from_i32(row.discount_days)?,
            },
            receivable_account: AccountCode(u32::try_from(row.receivable_account).map_err(
                |_| AppError::Internal("receivable_account out of u32 range".to_string()),
            )?),
        })
    }
}

/// Registers a new customer.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
pub async fn register_customer(pool: &PgPool, customer: &Customer) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO customers (id, name, credit_limit, net_days, discount_percent, discount_days, receivable_account)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(customer.id.0)
    .bind(&customer.name)
    .bind(customer.credit_limit)
    .bind(net_days_to_i32(customer.payment_terms.net_days)?)
    .bind(customer.payment_terms.discount_percent)
    .bind(discount_days_to_i32(customer.payment_terms.discount_days)?)
    .bind(i64::from(customer.receivable_account.0))
    .execute(pool)
    .await?;
    Ok(())
}

/// Lists every registered customer.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
pub async fn list_customers(pool: &PgPool) -> Result<Vec<Customer>, AppError> {
    let rows: Vec<CustomerRow> = sqlx::query_as(
        "SELECT id, name, credit_limit, net_days, discount_percent, discount_days, receivable_account
         FROM customers",
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(Customer::try_from).collect()
}

#[derive(sqlx::FromRow)]
struct ArInvoiceRow {
    id: Uuid,
    customer_id: Uuid,
    invoice_number: String,
    invoice_date: NaiveDate,
    amount: Decimal,
    net_days: i32,
    discount_percent: Option<Decimal>,
    discount_days: Option<i32>,
    recognition_kind: String,
    recognition_date: Option<NaiveDate>,
    recognition_start: Option<NaiveDate>,
    recognition_end: Option<NaiveDate>,
    amount_received: Decimal,
    status: String,
}

impl TryFrom<ArInvoiceRow> for ArInvoice {
    type Error = AppError;

    fn try_from(row: ArInvoiceRow) -> Result<Self, Self::Error> {
        Ok(ArInvoice {
            id: ArInvoiceId(row.id),
            customer: CustomerId(row.customer_id),
            invoice_number: row.invoice_number,
            invoice_date: row.invoice_date,
            amount: row.amount,
            terms: PaymentTerms {
                net_days: net_days_from_i32(row.net_days)?,
                discount_percent: row.discount_percent,
                discount_days: discount_days_from_i32(row.discount_days)?,
            },
            recognition_method: columns_to_recognition(
                &row.recognition_kind,
                row.recognition_date,
                row.recognition_start,
                row.recognition_end,
            )?,
            amount_received: row.amount_received,
            status: status_from_text(&row.status)?,
        })
    }
}

const SELECT_INVOICES: &str = "
    SELECT id, customer_id, invoice_number, invoice_date, amount, net_days, discount_percent,
           discount_days, recognition_kind, recognition_date, recognition_start, recognition_end,
           amount_received, status
    FROM ar_invoices
";

async fn fetch_all_invoices<'e, E>(executor: E) -> Result<Vec<ArInvoice>, AppError>
where
    E: PgExecutor<'e>,
{
    let rows: Vec<ArInvoiceRow> = sqlx::query_as(SELECT_INVOICES).fetch_all(executor).await?;
    rows.into_iter().map(ArInvoice::try_from).collect()
}

/// Records a new AR invoice.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
pub async fn create_invoice(pool: &PgPool, invoice: &ArInvoice) -> Result<(), AppError> {
    let recognition = recognition_to_columns(invoice.recognition_method);
    sqlx::query(
        "INSERT INTO ar_invoices
            (id, customer_id, invoice_number, invoice_date, amount, net_days, discount_percent,
             discount_days, recognition_kind, recognition_date, recognition_start, recognition_end,
             amount_received, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
    )
    .bind(invoice.id.0)
    .bind(invoice.customer.0)
    .bind(&invoice.invoice_number)
    .bind(invoice.invoice_date)
    .bind(invoice.amount)
    .bind(net_days_to_i32(invoice.terms.net_days)?)
    .bind(invoice.terms.discount_percent)
    .bind(discount_days_to_i32(invoice.terms.discount_days)?)
    .bind(recognition.kind)
    .bind(recognition.date)
    .bind(recognition.start)
    .bind(recognition.end)
    .bind(invoice.amount_received)
    .bind(status_to_text(invoice.status))
    .execute(pool)
    .await?;
    Ok(())
}

/// Lists every recorded AR invoice.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
pub async fn list_invoices(pool: &PgPool) -> Result<Vec<ArInvoice>, AppError> {
    fetch_all_invoices(pool).await
}

/// Allocates `receipt` across every open AR invoice for its customer (via
/// `casiros_erp::ar::receipt::allocate_receipt`, unchanged), then writes
/// every invoice's updated `amount_received`/`status` back in one
/// transaction — the whole allocation succeeds or none of it does.
///
/// # Errors
///
/// Returns [`AppError::Erp`] if `receipt.amount` is not strictly positive,
/// or if a balance computation overflows. Returns [`AppError::Database`] if
/// the query fails.
pub async fn allocate_receipt(
    pool: &PgPool,
    receipt: &Receipt,
) -> Result<Vec<ReceiptAllocation>, AppError> {
    let mut tx = pool.begin().await?;
    let mut invoices = fetch_all_invoices(&mut *tx).await?;
    let allocations = casiros_erp::ar::receipt::allocate_receipt(receipt, &mut invoices)?;

    for invoice in &invoices {
        sqlx::query("UPDATE ar_invoices SET amount_received = $1, status = $2 WHERE id = $3")
            .bind(invoice.amount_received)
            .bind(status_to_text(invoice.status))
            .bind(invoice.id.0)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(allocations)
}
