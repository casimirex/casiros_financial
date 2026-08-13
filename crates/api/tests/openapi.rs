//! Integration tests for the generated `OpenAPI` document and Swagger UI.

use actix_web::{App, test};
use casiros_api::routes;
use serde_json::Value;

#[actix_web::test]
async fn openapi_document_lists_every_implemented_path() {
    let app = test::init_service(App::new().configure(routes::configure)).await;
    let req = test::TestRequest::get()
        .uri("/api-docs/openapi.json")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    let paths = body["paths"].as_object().expect("paths object");

    for expected in [
        "/healthz",
        "/api/v1/calculate/{formula}",
        "/api/v1/simulate",
        "/api/v1/narrative",
        "/api/v1/ledger/accounts",
        "/api/v1/ledger/accounts/{code}",
        "/api/v1/ledger/accounts/{code}/balance",
        "/api/v1/ledger/trial-balance",
        "/api/v1/journal/entries",
        "/api/v1/ap/suppliers",
        "/api/v1/ap/invoices",
        "/api/v1/ap/aging",
        "/api/v1/ap/payments/propose",
        "/api/v1/ar/customers",
        "/api/v1/ar/invoices",
        "/api/v1/ar/receipts/allocate",
        "/api/v1/treasury/cashflow/items",
        "/api/v1/treasury/cashflow/projection",
        "/api/v1/treasury/cashflow/shortfall",
        "/api/v1/treasury/fx/convert",
        "/api/v1/treasury/hedge/effectiveness",
    ] {
        assert!(paths.contains_key(expected), "missing path: {expected}");
    }

    // The WebSocket upgrade endpoint isn't representable in OpenAPI 3.0 and
    // is deliberately excluded from the document.
    assert!(!paths.contains_key("/ws/simulate"));
}

#[actix_web::test]
async fn swagger_ui_is_served() {
    let app = test::init_service(App::new().configure(routes::configure)).await;
    let req = test::TestRequest::get().uri("/swagger-ui/").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
