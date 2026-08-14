//! `/api/v1/ap/*` — suppliers, AP invoices, aging, and payment proposals.

use crate::error::AppError;
use crate::persistence::ap;
use crate::state::AppState;
use actix_web::{HttpResponse, web};
use casiros_core::types::Dollar;
use casiros_erp::ap::invoice::{AgingReport, ApInvoice, aging_report};
use casiros_erp::ap::payment::{PaymentProposal, propose_payments};
use casiros_erp::ap::supplier::{PaymentTerms, Supplier, SupplierId};
use casiros_erp::ledger::account::AccountCode;
use chrono::NaiveDate;
use serde::Deserialize;
use tracing::{info, instrument};
use utoipa::{IntoParams, ToSchema};

/// The request body for `POST /api/v1/ap/suppliers`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSupplierRequest {
    /// The supplier's name.
    pub name: String,
    /// The supplier's standard payment terms.
    pub payment_terms: PaymentTerms,
    /// The chart-of-accounts liability account this supplier's payables post to.
    pub payable_account: AccountCode,
}

/// Registers a new supplier.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
#[utoipa::path(
    post,
    path = "/api/v1/ap/suppliers",
    request_body = CreateSupplierRequest,
    responses((status = 201, description = "Supplier registered", body = Supplier)),
    tag = "ap"
)]
#[instrument(name = "POST /ap/suppliers", skip(state, request))]
pub async fn create_supplier(
    state: web::Data<AppState>,
    request: web::Json<CreateSupplierRequest>,
) -> Result<HttpResponse, AppError> {
    let request = request.into_inner();
    let supplier = Supplier {
        id: SupplierId::new(),
        name: request.name,
        payment_terms: request.payment_terms,
        payable_account: request.payable_account,
    };
    ap::register_supplier(&state.pool, &supplier).await?;
    info!(supplier_id = ?supplier.id, "supplier registered");
    Ok(HttpResponse::Created().json(supplier))
}

/// Lists every registered supplier.
///
/// # Errors
///
/// Returns [`AppError::Database`] if the query fails.
#[utoipa::path(
    get,
    path = "/api/v1/ap/suppliers",
    responses((status = 200, description = "Every registered supplier", body = Vec<Supplier>)),
    tag = "ap"
)]
#[instrument(name = "GET /ap/suppliers", skip(state))]
pub async fn list_suppliers(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let suppliers = ap::list_suppliers(&state.pool).await?;
    info!(count = suppliers.len(), "listed suppliers");
    Ok(HttpResponse::Ok().json(suppliers))
}

/// The request body for `POST /api/v1/ap/invoices`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApInvoiceRequest {
    /// The supplier that issued this invoice.
    pub supplier: SupplierId,
    /// The supplier's own invoice number.
    pub invoice_number: String,
    /// The date the invoice was issued.
    pub invoice_date: NaiveDate,
    /// The invoice amount.
    #[schema(value_type = Decimal)]
    pub amount: Dollar,
    /// The payment terms governing this invoice.
    pub terms: PaymentTerms,
}

/// Records a new AP invoice.
///
/// # Errors
///
/// Returns [`AppError::Calculation`] (422) if `amount` is not strictly positive.
#[utoipa::path(
    post,
    path = "/api/v1/ap/invoices",
    request_body = CreateApInvoiceRequest,
    responses(
        (status = 201, description = "Invoice recorded", body = ApInvoice),
        (status = 422, description = "Amount is not strictly positive"),
    ),
    tag = "ap"
)]
#[instrument(name = "POST /ap/invoices", skip(state, request))]
pub async fn create_invoice(
    state: web::Data<AppState>,
    request: web::Json<CreateApInvoiceRequest>,
) -> Result<HttpResponse, AppError> {
    let request = request.into_inner();
    let invoice = ApInvoice::new(
        request.supplier,
        request.invoice_number,
        request.invoice_date,
        request.amount,
        request.terms,
    )?;
    ap::create_invoice(&state.pool, &invoice).await?;
    info!(invoice_id = ?invoice.id, "AP invoice recorded");
    Ok(HttpResponse::Created().json(invoice))
}

/// Lists every recorded AP invoice.
///
/// # Errors
///
/// This handler is infallible; it always returns `Ok`.
#[utoipa::path(
    get,
    path = "/api/v1/ap/invoices",
    responses((status = 200, description = "Every recorded AP invoice", body = Vec<ApInvoice>)),
    tag = "ap"
)]
#[instrument(name = "GET /ap/invoices", skip(state))]
pub async fn list_invoices(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let invoices = ap::list_invoices(&state.pool).await?;
    info!(count = invoices.len(), "listed AP invoices");
    Ok(HttpResponse::Ok().json(invoices))
}

/// A date to evaluate aging or liquidity as of.
#[derive(Debug, Deserialize, IntoParams)]
pub struct AsOfQuery {
    /// The evaluation date.
    pub as_of: NaiveDate,
}

/// Builds an AP aging report as of `as_of`.
///
/// # Errors
///
/// Returns [`AppError::Calculation`] (422) if a date computation overflows.
#[utoipa::path(
    get,
    path = "/api/v1/ap/aging",
    params(AsOfQuery),
    responses((status = 200, description = "The aging report", body = AgingReport)),
    tag = "ap"
)]
#[instrument(name = "GET /ap/aging", skip(state))]
pub async fn aging(
    state: web::Data<AppState>,
    query: web::Query<AsOfQuery>,
) -> Result<HttpResponse, AppError> {
    let invoices = ap::list_invoices(&state.pool).await?;
    let report = aging_report(&invoices, query.as_of)?;
    Ok(HttpResponse::Ok().json(report))
}

/// The request body for `POST /api/v1/ap/payments/propose`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ProposePaymentsRequest {
    /// The date to evaluate proposals as of.
    pub as_of: NaiveDate,
    /// Cash currently available.
    #[schema(value_type = Decimal)]
    pub available_cash: Dollar,
    /// The entity's current liabilities.
    #[schema(value_type = Decimal)]
    pub current_liabilities: Dollar,
}

/// Proposes which open AP invoices to pay, per [`casiros_erp::ap::payment::propose_payments`].
///
/// # Errors
///
/// Returns [`AppError::Erp`] if a running total or the liquidity check fails
/// for a degenerate input (e.g. zero `current_liabilities`).
#[utoipa::path(
    post,
    path = "/api/v1/ap/payments/propose",
    request_body = ProposePaymentsRequest,
    responses((status = 200, description = "Proposed payments, grouped by supplier", body = Vec<PaymentProposal>)),
    tag = "ap"
)]
#[instrument(name = "POST /ap/payments/propose", skip(state, request))]
pub async fn propose(
    state: web::Data<AppState>,
    request: web::Json<ProposePaymentsRequest>,
) -> Result<HttpResponse, AppError> {
    let request = request.into_inner();
    let invoices = ap::list_invoices(&state.pool).await?;
    let proposals = propose_payments(
        &invoices,
        request.as_of,
        request.available_cash,
        request.current_liabilities,
    )?;
    info!(proposal_count = proposals.len(), "payments proposed");
    Ok(HttpResponse::Ok().json(proposals))
}
