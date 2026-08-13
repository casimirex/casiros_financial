//! `/api/v1/ar/*` — customers, AR invoices, and cash receipt allocation.

use crate::error::AppError;
use crate::state::{AppState, lock};
use actix_web::{HttpResponse, web};
use casiros_core::types::Dollar;
use casiros_erp::ap::supplier::PaymentTerms;
use casiros_erp::ar::customer::{Customer, CustomerId};
use casiros_erp::ar::invoice::{ArInvoice, RecognitionMethod};
use casiros_erp::ar::receipt::{Receipt, ReceiptAllocation, allocate_receipt};
use casiros_erp::ledger::account::AccountCode;
use chrono::NaiveDate;
use serde::Deserialize;
use tracing::{info, instrument};
use utoipa::ToSchema;

/// The request body for `POST /api/v1/ar/customers`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCustomerRequest {
    /// The customer's name.
    pub name: String,
    /// The maximum outstanding receivable balance this customer may carry.
    #[schema(value_type = Decimal)]
    pub credit_limit: Dollar,
    /// The customer's standard payment terms.
    pub payment_terms: PaymentTerms,
    /// The chart-of-accounts asset account this customer's receivables post to.
    pub receivable_account: AccountCode,
}

/// Registers a new customer.
///
/// # Errors
///
/// This handler is infallible; it always returns `Ok`.
#[utoipa::path(
    post,
    path = "/api/v1/ar/customers",
    request_body = CreateCustomerRequest,
    responses((status = 201, description = "Customer registered", body = Customer)),
    tag = "ar"
)]
#[instrument(name = "POST /ar/customers", skip(state, request))]
pub async fn create_customer(
    state: web::Data<AppState>,
    request: web::Json<CreateCustomerRequest>,
) -> Result<HttpResponse, AppError> {
    let request = request.into_inner();
    let customer = Customer {
        id: CustomerId::new(),
        name: request.name,
        credit_limit: request.credit_limit,
        payment_terms: request.payment_terms,
        receivable_account: request.receivable_account,
    };
    lock(&state.customers).insert(customer.id, customer.clone());
    info!(customer_id = ?customer.id, "customer registered");
    Ok(HttpResponse::Created().json(customer))
}

/// Lists every registered customer.
///
/// # Errors
///
/// This handler is infallible; it always returns `Ok`.
#[utoipa::path(
    get,
    path = "/api/v1/ar/customers",
    responses((status = 200, description = "Every registered customer", body = Vec<Customer>)),
    tag = "ar"
)]
#[instrument(name = "GET /ar/customers", skip(state))]
pub async fn list_customers(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let customers: Vec<Customer> = lock(&state.customers).values().cloned().collect();
    info!(count = customers.len(), "listed customers");
    Ok(HttpResponse::Ok().json(customers))
}

/// The request body for `POST /api/v1/ar/invoices`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateArInvoiceRequest {
    /// The customer this invoice was issued to.
    pub customer: CustomerId,
    /// This entity's own invoice number.
    pub invoice_number: String,
    /// The date the invoice was issued.
    pub invoice_date: NaiveDate,
    /// The total invoice amount.
    #[schema(value_type = Decimal)]
    pub amount: Dollar,
    /// The payment terms governing this invoice's due date.
    pub terms: PaymentTerms,
    /// How the underlying performance obligation is recognized under ASC 606.
    pub recognition_method: RecognitionMethod,
}

/// Records a new AR invoice.
///
/// # Errors
///
/// Returns [`AppError::Calculation`] (422) if `amount` is not strictly
/// positive. Returns [`AppError::Erp`] (400) if `recognition_method` is
/// [`RecognitionMethod::RatablyOverTime`] with an end not after its start.
#[utoipa::path(
    post,
    path = "/api/v1/ar/invoices",
    request_body = CreateArInvoiceRequest,
    responses(
        (status = 201, description = "Invoice recorded", body = ArInvoice),
        (status = 400, description = "Invalid recognition period"),
        (status = 422, description = "Amount is not strictly positive"),
    ),
    tag = "ar"
)]
#[instrument(name = "POST /ar/invoices", skip(state, request))]
pub async fn create_invoice(
    state: web::Data<AppState>,
    request: web::Json<CreateArInvoiceRequest>,
) -> Result<HttpResponse, AppError> {
    let request = request.into_inner();
    let invoice = ArInvoice::new(
        request.customer,
        request.invoice_number,
        request.invoice_date,
        request.amount,
        request.terms,
        request.recognition_method,
    )?;
    lock(&state.ar_invoices).insert(invoice.id, invoice.clone());
    info!(invoice_id = ?invoice.id, "AR invoice recorded");
    Ok(HttpResponse::Created().json(invoice))
}

/// Lists every recorded AR invoice.
///
/// # Errors
///
/// This handler is infallible; it always returns `Ok`.
#[utoipa::path(
    get,
    path = "/api/v1/ar/invoices",
    responses((status = 200, description = "Every recorded AR invoice", body = Vec<ArInvoice>)),
    tag = "ar"
)]
#[instrument(name = "GET /ar/invoices", skip(state))]
pub async fn list_invoices(state: web::Data<AppState>) -> Result<HttpResponse, AppError> {
    let invoices: Vec<ArInvoice> = lock(&state.ar_invoices).values().cloned().collect();
    info!(count = invoices.len(), "listed AR invoices");
    Ok(HttpResponse::Ok().json(invoices))
}

/// The request body for `POST /api/v1/ar/receipts/allocate`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct AllocateReceiptRequest {
    /// The customer this cash was received from.
    pub customer: CustomerId,
    /// The amount received.
    #[schema(value_type = Decimal)]
    pub amount: Dollar,
    /// The date the cash was received.
    pub date: NaiveDate,
}

/// Allocates an incoming cash receipt across the customer's open invoices,
/// oldest due date first, and persists the updated invoice balances.
///
/// # Errors
///
/// Returns [`AppError::Calculation`] (422) if `amount` is not strictly
/// positive, or if a date/balance computation overflows.
#[utoipa::path(
    post,
    path = "/api/v1/ar/receipts/allocate",
    request_body = AllocateReceiptRequest,
    responses((status = 200, description = "How the receipt was allocated across invoices", body = Vec<ReceiptAllocation>)),
    tag = "ar"
)]
#[instrument(name = "POST /ar/receipts/allocate", skip(state, request))]
pub async fn allocate(
    state: web::Data<AppState>,
    request: web::Json<AllocateReceiptRequest>,
) -> Result<HttpResponse, AppError> {
    let request = request.into_inner();
    let receipt = Receipt::new(request.customer, request.amount, request.date)?;

    let mut invoices_guard = lock(&state.ar_invoices);
    let mut invoices: Vec<ArInvoice> = invoices_guard.values().cloned().collect();
    let allocations = allocate_receipt(&receipt, &mut invoices)?;
    for invoice in invoices {
        invoices_guard.insert(invoice.id, invoice);
    }
    drop(invoices_guard);

    info!(allocation_count = allocations.len(), "receipt allocated");
    Ok(HttpResponse::Ok().json(allocations))
}
