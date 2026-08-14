//! Postgres-backed suppliers and AP invoices.
//!
//! Storage-only: `aging_report`/`propose_payments` (pure functions over a
//! `&[ApInvoice]` slice) stay exactly as `casiros_erp` defines them — this
//! module's job is only to get that slice out of Postgres, not to
//! re-implement any of the business logic.

use crate::error::AppError;
use casiros_erp::ap::invoice::{ApInvoice, ApInvoiceId, ApInvoiceStatus};
use casiros_erp::ap::supplier::{PaymentTerms, Supplier, SupplierId};
use casiros_erp::ledger::account::AccountCode;
use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

fn status_to_text(status: ApInvoiceStatus) -> &'static str {
    match status {
        ApInvoiceStatus::Open => "Open",
        ApInvoiceStatus::PartiallyPaid => "PartiallyPaid",
        ApInvoiceStatus::Paid => "Paid",
    }
}

fn status_from_text(text: &str) -> Result<ApInvoiceStatus, AppError> {
    match text {
        "Open" => Ok(ApInvoiceStatus::Open),
        "PartiallyPaid" => Ok(ApInvoiceStatus::PartiallyPaid),
        "Paid" => Ok(ApInvoiceStatus::Paid),
        other => Err(AppError::Internal(format!(
            "corrupt ap_invoices.status {other:?} in database"
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

#[derive(sqlx::FromRow)]
struct SupplierRow {
    id: Uuid,
    name: String,
    net_days: i32,
    discount_percent: Option<Decimal>,
    discount_days: Option<i32>,
    payable_account: i64,
}

impl TryFrom<SupplierRow> for Supplier {
    type Error = AppError;

    fn try_from(row: SupplierRow) -> Result<Self, Self::Error> {
        Ok(Supplier {
            id: SupplierId(row.id),
            name: row.name,
            payment_terms: PaymentTerms {
                net_days: net_days_from_i32(row.net_days)?,
                discount_percent: row.discount_percent,
                discount_days: discount_days_from_i32(row.discount_days)?,
            },
            payable_account: AccountCode(
                u32::try_from(row.payable_account).map_err(|_| {
                    AppError::Internal("payable_account out of u32 range".to_string())
                })?,
            ),
        })
    }
}

/// Registers a new supplier.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
pub async fn register_supplier(pool: &PgPool, supplier: &Supplier) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO suppliers (id, name, net_days, discount_percent, discount_days, payable_account)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(supplier.id.0)
    .bind(&supplier.name)
    .bind(net_days_to_i32(supplier.payment_terms.net_days)?)
    .bind(supplier.payment_terms.discount_percent)
    .bind(discount_days_to_i32(supplier.payment_terms.discount_days)?)
    .bind(i64::from(supplier.payable_account.0))
    .execute(pool)
    .await?;
    Ok(())
}

/// Lists every registered supplier.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
pub async fn list_suppliers(pool: &PgPool) -> Result<Vec<Supplier>, AppError> {
    let rows: Vec<SupplierRow> = sqlx::query_as(
        "SELECT id, name, net_days, discount_percent, discount_days, payable_account FROM suppliers",
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(Supplier::try_from).collect()
}

#[derive(sqlx::FromRow)]
struct ApInvoiceRow {
    id: Uuid,
    supplier_id: Uuid,
    invoice_number: String,
    invoice_date: NaiveDate,
    amount: Decimal,
    net_days: i32,
    discount_percent: Option<Decimal>,
    discount_days: Option<i32>,
    amount_paid: Decimal,
    status: String,
}

impl TryFrom<ApInvoiceRow> for ApInvoice {
    type Error = AppError;

    fn try_from(row: ApInvoiceRow) -> Result<Self, Self::Error> {
        Ok(ApInvoice {
            id: ApInvoiceId(row.id),
            supplier: SupplierId(row.supplier_id),
            invoice_number: row.invoice_number,
            invoice_date: row.invoice_date,
            amount: row.amount,
            terms: PaymentTerms {
                net_days: net_days_from_i32(row.net_days)?,
                discount_percent: row.discount_percent,
                discount_days: discount_days_from_i32(row.discount_days)?,
            },
            amount_paid: row.amount_paid,
            status: status_from_text(&row.status)?,
        })
    }
}

/// Records a new AP invoice.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
pub async fn create_invoice(pool: &PgPool, invoice: &ApInvoice) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO ap_invoices
            (id, supplier_id, invoice_number, invoice_date, amount, net_days, discount_percent, discount_days, amount_paid, status)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(invoice.id.0)
    .bind(invoice.supplier.0)
    .bind(&invoice.invoice_number)
    .bind(invoice.invoice_date)
    .bind(invoice.amount)
    .bind(net_days_to_i32(invoice.terms.net_days)?)
    .bind(invoice.terms.discount_percent)
    .bind(discount_days_to_i32(invoice.terms.discount_days)?)
    .bind(invoice.amount_paid)
    .bind(status_to_text(invoice.status))
    .execute(pool)
    .await?;
    Ok(())
}

/// Lists every recorded AP invoice.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
pub async fn list_invoices(pool: &PgPool) -> Result<Vec<ApInvoice>, AppError> {
    let rows: Vec<ApInvoiceRow> = sqlx::query_as(
        "SELECT id, supplier_id, invoice_number, invoice_date, amount, net_days, discount_percent,
                discount_days, amount_paid, status
         FROM ap_invoices",
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter().map(ApInvoice::try_from).collect()
}
