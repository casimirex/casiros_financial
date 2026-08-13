//! CASIROS API server entry point. A thin binary wrapper around the
//! `casiros_api` library, which holds the actual application logic (see
//! `lib.rs` for why the split exists).

use actix_cors::Cors;
use actix_web::dev::Service;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::{App, HttpServer, web};
use casiros_api::middleware::rate_limit::RateLimiter;
use casiros_api::middleware::tracing::{REQUEST_ID_HEADER, new_request_id};
use casiros_api::routes;
use casiros_api::state::AppState;
use std::time::Duration;
use tracing::Instrument;
use tracing_subscriber::EnvFilter;

/// Requests allowed per client IP within [`RATE_LIMIT_WINDOW`].
const RATE_LIMIT_MAX_REQUESTS: usize = 100;
/// The rate-limiting sliding window.
const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);
/// The default bind address, overridable via `CASIROS_SERVER__BIND_ADDR`
/// (matching the `CASIROS_<SECTION>__<KEY>` env override convention from
/// `CASIROS_BUILD_PROMPT.md` section 12).
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8080";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();

    let bind_addr = std::env::var("CASIROS_SERVER__BIND_ADDR")
        .unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());
    tracing::info!(bind_addr = %bind_addr, "starting CASIROS API server");

    let rate_limiter = web::Data::new(RateLimiter::new(RATE_LIMIT_MAX_REQUESTS, RATE_LIMIT_WINDOW));
    // Created once, outside the HttpServer factory closure, and shared (via
    // web::Data's Arc) across every worker thread — see routes::configure's
    // doc comment for why this can't be created inside `configure` itself.
    let app_state = web::Data::new(AppState::new());

    HttpServer::new(move || {
        App::new()
            .app_data(rate_limiter.clone())
            .app_data(app_state.clone())
            .wrap(Cors::permissive())
            .wrap_fn(|req, srv| {
                let request_id = new_request_id();
                let method = req.method().clone();
                let path = req.path().to_string();
                let span =
                    tracing::info_span!("http_request", request_id = %request_id, %method, %path);
                let fut = srv.call(req);
                async move {
                    let mut res = fut.await?;
                    let header_value = HeaderValue::from_str(&request_id)
                        .unwrap_or_else(|_| HeaderValue::from_static("invalid"));
                    res.headers_mut()
                        .insert(HeaderName::from_static(REQUEST_ID_HEADER), header_value);
                    Ok(res)
                }
                .instrument(span)
            })
            .wrap_fn(|req, srv| {
                let limiter = req.app_data::<web::Data<RateLimiter>>().cloned();
                let ip = req.peer_addr().map(|addr| addr.ip());
                let allowed = match (limiter, ip) {
                    (Some(limiter), Some(ip)) => limiter.check(ip),
                    // No peer address (common in tests) or no limiter configured: allow.
                    _ => true,
                };
                let fut = srv.call(req);
                async move {
                    if !allowed {
                        return Err(actix_web::error::ErrorTooManyRequests(
                            "rate limit exceeded",
                        ));
                    }
                    fut.await
                }
            })
            .configure(routes::configure)
    })
    .bind(&bind_addr)?
    .run()
    .await
}
