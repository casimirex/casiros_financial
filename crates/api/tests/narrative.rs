//! Integration tests for `POST /api/v1/narrative`.

use actix_web::{App, test};
use casiros_api::routes;
use serde_json::{Value, json};

#[actix_web::test]
async fn narrative_generates_a_memo_from_partial_metrics() {
    let app = test::init_service(App::new().configure(routes::configure)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/narrative")
        .set_json(json!({
            "company": "Acme Corp",
            "roe": "0.15",
            "debt_to_equity": "0.8",
            "current_ratio": "2.0"
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    let memo = body["memo"].as_str().unwrap();
    assert!(memo.starts_with("## Financial Analysis Memo: Acme Corp"));
    assert!(memo.contains("Return on Equity"));
    assert!(memo.contains("15.0%"));
    assert!(!memo.contains("Net Income"));
}

#[actix_web::test]
async fn narrative_with_no_metrics_is_just_the_header() {
    let app = test::init_service(App::new().configure(routes::configure)).await;
    let req = test::TestRequest::post()
        .uri("/api/v1/narrative")
        .set_json(json!({ "company": "Beta LLC" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: Value = test::read_body_json(resp).await;
    let memo = body["memo"].as_str().unwrap();
    assert_eq!(memo, "## Financial Analysis Memo: Beta LLC\n\n");
}
