//! End-to-end test of `GET /ws/simulate`: connects a real WebSocket client to
//! a real running server (via `actix-test`) and verifies the streamed
//! progress/final message sequence, rather than just exercising the handler
//! through `actix_web::test::call_service` (which can't drive a WebSocket
//! upgrade).
//!
//! `/ws/simulate` itself touches no `AppState` field, but `AppState`
//! construction still needs a real (Postgres-backed) pool as of Phase 9 —
//! see `tests/support`.

mod support;

use actix_web::{App, web};
use awc::ws;
use casiros_api::routes;
use casiros_api::state::AppState;
use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};

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
async fn ws_simulate_streams_progress_then_final() {
    let db = support::test_db().await;
    let pool = db.pool.clone();
    let mut srv = actix_test::start(move || {
        App::new()
            .app_data(web::Data::new(AppState::new(pool.clone())))
            .configure(routes::configure)
    });

    let mut connection = srv.ws_at("/ws/simulate").await.unwrap();

    let request = json!({
        "baseline": sample_universe(),
        "config": {
            "iterations": 250, "seed": 7, "track_convergence": false,
            "convergence_threshold": "0.0001", "convergence_batch_size": 1000
        }
    });
    connection
        .send(ws::Message::Text(request.to_string().into()))
        .await
        .unwrap();

    let mut saw_progress = false;
    let mut saw_final = false;
    while let Some(Ok(frame)) = connection.next().await {
        let ws::Frame::Text(bytes) = frame else {
            continue;
        };
        let message: Value = serde_json::from_slice(&bytes).unwrap();
        match message["type"].as_str() {
            Some("progress") => {
                saw_progress = true;
                assert!(message["completed"].is_number());
                assert_eq!(message["total"], 250);
            }
            Some("final") => {
                saw_final = true;
                assert!(message["metrics"]["wacc"]["sample_count"].is_number());
                break;
            }
            other => panic!("unexpected message type: {other:?}"),
        }
    }

    assert!(saw_progress, "expected at least one progress message");
    assert!(saw_final, "expected a final message");
}

#[actix_web::test]
async fn ws_simulate_reports_invalid_json_as_an_error_message() {
    let db = support::test_db().await;
    let pool = db.pool.clone();
    let mut srv = actix_test::start(move || {
        App::new()
            .app_data(web::Data::new(AppState::new(pool.clone())))
            .configure(routes::configure)
    });

    let mut connection = srv.ws_at("/ws/simulate").await.unwrap();
    connection
        .send(ws::Message::Text("not valid json".into()))
        .await
        .unwrap();

    let frame = connection.next().await.unwrap().unwrap();
    let ws::Frame::Text(bytes) = frame else {
        panic!("expected a text frame");
    };
    let message: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(message["type"], "error");
    assert!(message["message"].is_string());
}
