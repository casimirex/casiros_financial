//! Integration tests exercising `casiros-api`'s HTTP routes end-to-end,
//! against the real `casiros_api::routes::configure` app assembly.

use actix_web::{App, test};
use casiros_api::routes;
use serde_json::{Value, json};

#[actix_web::test]
async fn healthz_reports_ok() {
    let app = test::init_service(App::new().configure(routes::configure)).await;
    let req = test::TestRequest::get().uri("/healthz").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["status"], "ok");
}

#[actix_web::test]
async fn calculate_future_value_matches_hand_computed_result() {
    let app = test::init_service(App::new().configure(routes::configure)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/calculate/future_value")
        .set_json(json!({ "params": { "pv": "1000.0", "rate": "0.05", "periods": "10" } }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["formula"], "future_value");
    // Cross-checked against Phase 1's own doc-tested value: 1000*(1.05)^10.
    assert_eq!(body["result"], "1628.894626777441406250000");
}

#[actix_web::test]
async fn calculate_unknown_formula_is_404() {
    let app = test::init_service(App::new().configure(routes::configure)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/calculate/not_a_real_formula")
        .set_json(json!({ "params": {} }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn calculate_missing_parameter_is_422() {
    let app = test::init_service(App::new().configure(routes::configure)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/calculate/current_ratio")
        .set_json(json!({ "params": { "current_assets": "300000.0" } }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 422);

    let body: Value = test::read_body_json(resp).await;
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("current_liabilities")
    );
}

#[actix_web::test]
async fn calculate_invalid_decimal_is_400() {
    let app = test::init_service(App::new().configure(routes::configure)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/calculate/future_value")
        .set_json(json!({ "params": { "pv": "not-a-number", "rate": "0.05", "periods": "10" } }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn calculate_division_by_zero_is_422() {
    let app = test::init_service(App::new().configure(routes::configure)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/calculate/current_ratio")
        .set_json(
            json!({ "params": { "current_assets": "300000.0", "current_liabilities": "0.0" } }),
        )
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 422);
}

#[actix_web::test]
async fn calculate_series_formula_is_rejected_as_bad_request() {
    let app = test::init_service(App::new().configure(routes::configure)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/calculate/discounted_cash_flow")
        .set_json(json!({ "params": { "rate": "0.10" } }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

fn sample_universe() -> Value {
    json!({
        "risk_free_rate": "0.03", "inflation_rate": "0.02", "market_return": "0.08",
        "portfolio_return": "0.10", "return_std_dev": "0.15",
        "revenue": "1000000.0", "cogs": "600000.0", "operating_expenses": "200000.0",
        "interest_expense": "50000.0", "tax_rate": "0.25", "beta": "1.2",
        "cost_of_equity": "0.11", "cost_of_debt": "0.06",
        "total_assets": "1500000.0", "current_assets": "400000.0", "inventory": "100000.0",
        "current_liabilities": "200000.0", "total_liabilities": "750000.0", "total_equity": "750000.0",
        "share_price": "50.0", "shares_outstanding": "20000.0"
    })
}

#[actix_web::test]
async fn simulate_runs_and_aggregates_every_metric() {
    let app = test::init_service(App::new().configure(routes::configure)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/simulate")
        .set_json(json!({
            "baseline": sample_universe(),
            "config": {
                "iterations": 50, "seed": 42, "track_convergence": false,
                "convergence_threshold": "0.0001", "convergence_batch_size": 20
            }
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    assert_eq!(body["scenarios_requested"], 50);
    assert_eq!(
        body["scenarios_evaluated"].as_u64().unwrap() + body["scenarios_failed"].as_u64().unwrap(),
        50
    );
    assert!(body["metrics"]["return_on_equity"]["sample_count"].is_number());
    assert!(body["metrics"].get("wacc").is_some());
}
