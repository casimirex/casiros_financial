//! Integration tests for the `/api/v1/{ledger,journal,ap,ar,treasury}/*`
//! routes, against the real `casiros_api::routes::configure` app assembly.
//!
//! Unlike `tests/integration.rs`'s stateless `calculate`/`simulate` checks,
//! every route here reads or writes `AppState`, so each test builds its own
//! app with a fresh `AppState` registered — mirroring exactly what `main.rs`
//! does, since `routes::configure` deliberately does not create one itself.

use actix_web::{App, test, web};
use casiros_api::routes;
use casiros_api::state::AppState;
use serde_json::{Value, json};

fn app_state() -> web::Data<AppState> {
    web::Data::new(AppState::new())
}

#[actix_web::test]
async fn ledger_register_and_list_accounts() {
    let state = app_state();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/ledger/accounts")
        .set_json(json!({ "code": 1000, "name": "Cash", "account_type": "Asset", "parent": null }))
        .to_request();
    assert_eq!(test::call_service(&app, req).await.status(), 201);

    let req = test::TestRequest::post()
        .uri("/api/v1/ledger/accounts")
        .set_json(
            json!({ "code": 3000, "name": "Owner Equity", "account_type": "Equity", "parent": null }),
        )
        .to_request();
    assert_eq!(test::call_service(&app, req).await.status(), 201);

    // Duplicate registration is rejected.
    let req = test::TestRequest::post()
        .uri("/api/v1/ledger/accounts")
        .set_json(json!({ "code": 1000, "name": "Cash", "account_type": "Asset", "parent": null }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);

    let req = test::TestRequest::get()
        .uri("/api/v1/ledger/accounts")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 2);

    let req = test::TestRequest::get()
        .uri("/api/v1/ledger/accounts/9999")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn journal_posting_updates_balances_and_trial_balance() {
    let state = app_state();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/ledger/accounts")
        .set_json(json!({ "code": 1000, "name": "Cash", "account_type": "Asset", "parent": null }))
        .to_request();
    assert_eq!(test::call_service(&app, req).await.status(), 201);

    let req = test::TestRequest::post()
        .uri("/api/v1/ledger/accounts")
        .set_json(
            json!({ "code": 3000, "name": "Owner Equity", "account_type": "Equity", "parent": null }),
        )
        .to_request();
    assert_eq!(test::call_service(&app, req).await.status(), 201);

    let req = test::TestRequest::post()
        .uri("/api/v1/journal/entries")
        .set_json(json!({
            "date": "2026-08-13",
            "description": "Initial capital",
            "lines": [
                { "account": 1000, "debit": "1000.00", "credit": "0", "causal_formula": null },
                { "account": 3000, "debit": "0", "credit": "1000.00", "causal_formula": null }
            ],
            "causal_parent": null,
            "source_document": "ManualEntry",
            "period": { "year": 2026, "month": 8, "closed": false }
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let req = test::TestRequest::get()
        .uri("/api/v1/journal/entries")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    let req = test::TestRequest::get()
        .uri("/api/v1/ledger/accounts/1000/balance")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["balance"], "1000.00");

    let req = test::TestRequest::get()
        .uri("/api/v1/ledger/trial-balance")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    let entries = body.as_array().unwrap();
    assert_eq!(entries.len(), 2);
}

#[actix_web::test]
async fn journal_unbalanced_entry_is_rejected() {
    let state = app_state();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/ledger/accounts")
        .set_json(json!({ "code": 1000, "name": "Cash", "account_type": "Asset", "parent": null }))
        .to_request();
    assert_eq!(test::call_service(&app, req).await.status(), 201);

    let req = test::TestRequest::post()
        .uri("/api/v1/journal/entries")
        .set_json(json!({
            "date": "2026-08-13",
            "description": "Unbalanced",
            "lines": [
                { "account": 1000, "debit": "1000.00", "credit": "0", "causal_formula": null }
            ],
            "causal_parent": null,
            "source_document": "ManualEntry",
            "period": { "year": 2026, "month": 8, "closed": false }
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn ap_supplier_invoice_and_aging_round_trip() {
    let state = app_state();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/ap/suppliers")
        .set_json(json!({
            "name": "Acme Corp",
            "payment_terms": { "net_days": 30, "discount_percent": null, "discount_days": null },
            "payable_account": 3000
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let supplier: Value = test::read_body_json(resp).await;
    let supplier_id = supplier["id"].as_str().unwrap();

    let req = test::TestRequest::get()
        .uri("/api/v1/ap/suppliers")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    let req = test::TestRequest::post()
        .uri("/api/v1/ap/invoices")
        .set_json(json!({
            "supplier": supplier_id,
            "invoice_number": "INV-001",
            "invoice_date": "2026-08-01",
            "amount": "500.00",
            "terms": { "net_days": 30, "discount_percent": null, "discount_days": null }
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let req = test::TestRequest::get()
        .uri("/api/v1/ap/invoices")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    let req = test::TestRequest::get()
        .uri("/api/v1/ap/aging?as_of=2026-08-13")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["current"], "500.00");

    let req = test::TestRequest::post()
        .uri("/api/v1/ap/payments/propose")
        .set_json(json!({
            "as_of": "2026-08-13",
            "available_cash": "1000.00",
            "current_liabilities": "500.00"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn ap_invoice_with_non_positive_amount_is_422() {
    let state = app_state();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/ap/suppliers")
        .set_json(json!({
            "name": "Acme Corp",
            "payment_terms": { "net_days": 30, "discount_percent": null, "discount_days": null },
            "payable_account": 3000
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let supplier: Value = test::read_body_json(resp).await;
    let supplier_id = supplier["id"].as_str().unwrap();

    let req = test::TestRequest::post()
        .uri("/api/v1/ap/invoices")
        .set_json(json!({
            "supplier": supplier_id,
            "invoice_number": "INV-002",
            "invoice_date": "2026-08-01",
            "amount": "0.00",
            "terms": { "net_days": 30, "discount_percent": null, "discount_days": null }
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 422);
}

#[actix_web::test]
async fn ar_customer_invoice_and_receipt_allocation_round_trip() {
    let state = app_state();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/ar/customers")
        .set_json(json!({
            "name": "Beta LLC",
            "credit_limit": "10000.00",
            "payment_terms": { "net_days": 30, "discount_percent": null, "discount_days": null },
            "receivable_account": 1000
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let customer: Value = test::read_body_json(resp).await;
    let customer_id = customer["id"].as_str().unwrap();

    let req = test::TestRequest::get()
        .uri("/api/v1/ar/customers")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    let req = test::TestRequest::post()
        .uri("/api/v1/ar/invoices")
        .set_json(json!({
            "customer": customer_id,
            "invoice_number": "AR-001",
            "invoice_date": "2026-08-01",
            "amount": "300.00",
            "terms": { "net_days": 30, "discount_percent": null, "discount_days": null },
            "recognition_method": { "PointInTime": { "recognition_date": "2026-08-01" } }
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);
    let invoice: Value = test::read_body_json(resp).await;
    let invoice_id = invoice["id"].as_str().unwrap();

    let req = test::TestRequest::get()
        .uri("/api/v1/ar/invoices")
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);

    let req = test::TestRequest::post()
        .uri("/api/v1/ar/receipts/allocate")
        .set_json(json!({ "customer": customer_id, "amount": "300.00", "date": "2026-08-13" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let allocations: Value = test::read_body_json(resp).await;
    let allocations = allocations.as_array().unwrap();
    assert_eq!(allocations.len(), 1);
    assert_eq!(allocations[0]["invoice"], invoice_id);
    assert_eq!(allocations[0]["amount_applied"], "300.00");
}

#[actix_web::test]
async fn treasury_cashflow_projection_and_shortfall() {
    let state = app_state();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/treasury/cashflow/items")
        .set_json(json!({
            "date": "2026-08-20",
            "amount": "-1500.00",
            "category": "Operating",
            "description": "Payroll"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let req = test::TestRequest::get()
        .uri("/api/v1/treasury/cashflow/projection?opening_balance=1000.00&as_of=2026-09-01")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["balance"], "-500.00");

    let req = test::TestRequest::get()
        .uri("/api/v1/treasury/cashflow/shortfall?opening_balance=1000.00&as_of=2026-09-01")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn treasury_fx_convert_is_stateless() {
    let state = app_state();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/treasury/fx/convert")
        .set_json(json!({
            "exposure": { "currency": [69, 85, 82], "amount": "100.00" },
            "rate": { "from": [69, 85, 82], "to": [85, 83, 68], "rate": "1.10", "as_of": "2026-08-13" }
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["converted"], "110.0000");
}

#[actix_web::test]
async fn treasury_hedge_effectiveness_is_stateless() {
    let state = app_state();
    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/api/v1/treasury/hedge/effectiveness")
        .set_json(json!({ "hedge_gain_loss": "-95.00", "exposure_gain_loss": "100.00" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["effectiveness"], "0.95");
    assert_eq!(body["highly_effective"], true);
}
