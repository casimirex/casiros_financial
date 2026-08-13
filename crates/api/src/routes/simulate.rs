//! `POST /api/v1/simulate` (synchronous) and `GET /ws/simulate` (streaming).

use crate::error::AppError;
use actix_web::{HttpRequest, HttpResponse, web};
use actix_ws::{Message, Session};
use casiros_simulator::aggregation::{SimulationResults, aggregate};
use casiros_simulator::monte_carlo::{MonteCarloConfig, generate_scenarios, run_simulation};
use casiros_simulator::universe::{Universe, UniverseMetrics};
use futures_util::StreamExt;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, instrument};

/// The request body for both the synchronous and streaming simulate endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct SimulateRequest {
    /// The baseline scenario to perturb.
    pub baseline: Universe,
    /// The Monte Carlo run configuration.
    pub config: MonteCarloConfig,
}

/// The response body for `POST /api/v1/simulate`.
#[derive(Debug, Serialize)]
pub struct SimulateResponse {
    /// How many scenarios were requested.
    pub scenarios_requested: u32,
    /// How many scenarios evaluated without error.
    pub scenarios_evaluated: usize,
    /// How many scenarios failed to evaluate (e.g. a perturbed denominator hit zero).
    pub scenarios_failed: usize,
    /// One [`SimulationResults`] per [`UniverseMetrics`] field, keyed by field name.
    pub metrics: HashMap<String, SimulationResults>,
}

/// Extracts one [`UniverseMetrics`] field from every successful scenario.
fn extract(
    successes: &[UniverseMetrics],
    selector: impl Fn(&UniverseMetrics) -> Decimal,
) -> Vec<Decimal> {
    successes.iter().map(selector).collect()
}

/// Aggregates every [`UniverseMetrics`] field across `successes` into a named
/// [`SimulationResults`] map. Returns an empty map if `successes` is empty.
fn aggregate_all_metrics(
    successes: &[UniverseMetrics],
) -> Result<HashMap<String, SimulationResults>, AppError> {
    if successes.is_empty() {
        return Ok(HashMap::new());
    }
    let mut metrics = HashMap::new();
    metrics.insert(
        "ebit".to_string(),
        aggregate(&extract(successes, |m| m.ebit))?,
    );
    metrics.insert(
        "net_income".to_string(),
        aggregate(&extract(successes, |m| m.net_income))?,
    );
    metrics.insert(
        "profit_margin".to_string(),
        aggregate(&extract(successes, |m| m.profit_margin))?,
    );
    metrics.insert(
        "return_on_equity".to_string(),
        aggregate(&extract(successes, |m| m.return_on_equity))?,
    );
    metrics.insert(
        "return_on_assets".to_string(),
        aggregate(&extract(successes, |m| m.return_on_assets))?,
    );
    metrics.insert(
        "current_ratio".to_string(),
        aggregate(&extract(successes, |m| m.current_ratio))?,
    );
    metrics.insert(
        "quick_ratio".to_string(),
        aggregate(&extract(successes, |m| m.quick_ratio))?,
    );
    metrics.insert(
        "debt_to_equity".to_string(),
        aggregate(&extract(successes, |m| m.debt_to_equity))?,
    );
    metrics.insert(
        "interest_coverage".to_string(),
        aggregate(&extract(successes, |m| m.interest_coverage))?,
    );
    metrics.insert(
        "asset_turnover".to_string(),
        aggregate(&extract(successes, |m| m.asset_turnover))?,
    );
    metrics.insert(
        "wacc".to_string(),
        aggregate(&extract(successes, |m| m.wacc))?,
    );
    metrics.insert(
        "sharpe_ratio".to_string(),
        aggregate(&extract(successes, |m| m.sharpe_ratio))?,
    );
    Ok(metrics)
}

/// Runs a full Monte Carlo simulation and aggregates every metric. Pure
/// (besides being CPU-bound), so it is safe to run on a blocking thread.
fn run_simulation_and_aggregate(
    baseline: &Universe,
    config: &MonteCarloConfig,
) -> Result<SimulateResponse, AppError> {
    let scenarios = generate_scenarios(baseline, config)?;
    let results = run_simulation(&scenarios, config);
    let scenarios_evaluated = results.iter().filter(|r| r.is_ok()).count();
    let scenarios_failed = results.len() - scenarios_evaluated;
    let successes: Vec<UniverseMetrics> = results.into_iter().filter_map(Result::ok).collect();
    let metrics = aggregate_all_metrics(&successes)?;
    Ok(SimulateResponse {
        scenarios_requested: config.iterations,
        scenarios_evaluated,
        scenarios_failed,
        metrics,
    })
}

/// Runs a Monte Carlo simulation synchronously and returns the full aggregated result.
///
/// # Errors
///
/// Returns [`AppError::Calculation`] if scenario generation or aggregation fails.
/// Returns [`AppError::Internal`] if the blocking task itself fails to run.
#[instrument(name = "POST /simulate", skip(req))]
pub async fn handle_simulate(req: web::Json<SimulateRequest>) -> Result<HttpResponse, AppError> {
    let SimulateRequest { baseline, config } = req.into_inner();
    let block_result = web::block(move || run_simulation_and_aggregate(&baseline, &config))
        .await
        .map_err(|err| {
            error!(error = %err, "simulation blocking task failed");
            AppError::Internal(format!("simulation task failed: {err}"))
        })?;
    let response = block_result.map_err(|err| {
        error!(error = %err, "simulation failed");
        err
    })?;
    info!(
        scenarios_evaluated = response.scenarios_evaluated,
        scenarios_failed = response.scenarios_failed,
        "simulation succeeded"
    );
    Ok(HttpResponse::Ok().json(response))
}

/// An outgoing WebSocket message from `/ws/simulate`.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum WsOutgoing {
    /// Progress after evaluating one batch of scenarios.
    Progress {
        /// Scenarios evaluated so far.
        completed: usize,
        /// Total scenarios requested.
        total: usize,
    },
    /// The final aggregated result, sent once and followed by connection close.
    Final {
        /// One [`SimulationResults`] per [`UniverseMetrics`] field.
        metrics: HashMap<String, SimulationResults>,
    },
    /// Something went wrong; the connection closes after this message.
    Error {
        /// A human-readable description of the failure.
        message: String,
    },
}

/// The number of scenarios evaluated per progress update.
const WS_BATCH_SIZE: usize = 100;

async fn send_json(session: &mut Session, value: &WsOutgoing) -> bool {
    let Ok(text) = serde_json::to_string(value) else {
        return false;
    };
    session.text(text).await.is_ok()
}

/// Runs a full simulation over an already-open WebSocket session, streaming a
/// progress update every [`WS_BATCH_SIZE`] scenarios and finishing with the
/// aggregated result.
async fn run_streamed_simulation(raw_message: &str, session: &mut Session) {
    let Ok(SimulateRequest { baseline, config }) =
        serde_json::from_str::<SimulateRequest>(raw_message)
    else {
        send_json(
            session,
            &WsOutgoing::Error {
                message: "invalid request body".to_string(),
            },
        )
        .await;
        return;
    };

    let scenarios = match generate_scenarios(&baseline, &config) {
        Ok(scenarios) => scenarios,
        Err(err) => {
            send_json(
                session,
                &WsOutgoing::Error {
                    message: err.to_string(),
                },
            )
            .await;
            return;
        }
    };

    let mut all_metrics: Vec<UniverseMetrics> = Vec::with_capacity(scenarios.len());
    for batch in scenarios.chunks(WS_BATCH_SIZE) {
        let batch_results = run_simulation(batch, &config);
        all_metrics.extend(batch_results.into_iter().filter_map(Result::ok));

        let progress = WsOutgoing::Progress {
            completed: all_metrics.len().min(scenarios.len()),
            total: scenarios.len(),
        };
        if !send_json(session, &progress).await {
            return;
        }
    }

    match aggregate_all_metrics(&all_metrics) {
        Ok(metrics) => {
            send_json(session, &WsOutgoing::Final { metrics }).await;
        }
        Err(err) => {
            send_json(
                session,
                &WsOutgoing::Error {
                    message: err.to_string(),
                },
            )
            .await;
        }
    }
    let _: Result<(), actix_ws::Closed> = session.clone().close(None).await;
}

/// Upgrades the connection to a WebSocket and streams a Monte Carlo
/// simulation's progress. The client sends a [`SimulateRequest`] as its first
/// text message; the server streams [`WsOutgoing::Progress`] updates and
/// finishes with a single [`WsOutgoing::Final`] message before closing.
///
/// # Errors
///
/// Returns [`AppError::Internal`] if the WebSocket handshake itself fails.
#[instrument(name = "GET /ws/simulate", skip(req, stream))]
pub async fn ws_simulate(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, AppError> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)
        .map_err(|err| AppError::Internal(format!("websocket handshake failed: {err}")))?;

    actix_web::rt::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Text(text) => run_streamed_simulation(&text, &mut session).await,
                Message::Close(reason) => {
                    let _: Result<(), actix_ws::Closed> = session.close(reason).await;
                    break;
                }
                _ => {}
            }
        }
    });

    info!("websocket simulation session started");
    Ok(response)
}
