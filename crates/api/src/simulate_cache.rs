//! Redis-backed caching for `POST /api/v1/simulate`. Monte Carlo results are
//! deterministic for identical inputs (`MonteCarloConfig::seed` pins the
//! RNG), so an identical request is always safe to serve from cache rather
//! than recompute — genuinely useful given `config.iterations` can run up
//! to `max_iterations` (1,000,000 per `config/default.toml`). Only wraps
//! the synchronous REST endpoint, not `/ws/simulate` — streaming progress
//! is the point of that endpoint, not a pure cacheable call.
//!
//! Fail-open and timeout-bounded, exactly like `middleware::redis_rate_limit`
//! — see that module's docs for why the timeout specifically is load-bearing
//! (a `ConnectionManager` blocks in-flight commands while reconnecting, so
//! "fail open on error" alone doesn't prevent a stall). A Redis outage here
//! degrades to "always recompute," never a hang.

use crate::routes::simulate::{SimulateRequest, SimulateResponse};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use sha2::{Digest, Sha256};
use std::time::Duration;
use tracing::warn;

/// The maximum time a single cache read or write may take before it's
/// treated as a failure — see `middleware::redis_rate_limit::CHECK_TIMEOUT`
/// for the identical reasoning.
const CACHE_TIMEOUT: Duration = Duration::from_millis(250);

/// Caches `SimulateResponse`s in Redis, keyed by a hash of the request that produced them.
#[derive(Clone)]
pub struct SimulateCache {
    connection: ConnectionManager,
    ttl_secs: u64,
}

impl SimulateCache {
    /// Creates a cache backed by `connection`, with entries expiring after `ttl`.
    #[must_use]
    pub fn new(connection: ConnectionManager, ttl: Duration) -> Self {
        Self {
            connection,
            ttl_secs: ttl.as_secs(),
        }
    }

    /// `Universe`/`MonteCarloConfig` have no `HashMap` fields (checked when
    /// this cache was designed), so `serde_json`'s field-order-preserving
    /// serialization is stable across processes and restarts — no risk of
    /// the same logical request hashing differently after a redeploy.
    fn key(request: &SimulateRequest) -> String {
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_vec(request).unwrap_or_default());
        format!("casiros:simulate:{:x}", hasher.finalize())
    }

    /// Looks up a cached response for `request`. Returns `None` on a cache
    /// miss, a Redis error, or a timeout — the caller should recompute
    /// exactly as if this cache didn't exist.
    pub async fn get(&self, request: &SimulateRequest) -> Option<SimulateResponse> {
        match tokio::time::timeout(CACHE_TIMEOUT, self.get_inner(request)).await {
            Ok(Ok(response)) => response,
            Ok(Err(err)) => miss(&err.to_string()),
            Err(_elapsed) => miss("timed out"),
        }
    }

    async fn get_inner(
        &self,
        request: &SimulateRequest,
    ) -> Result<Option<SimulateResponse>, redis::RedisError> {
        let mut conn = self.connection.clone();
        let cached: Option<String> = conn.get(Self::key(request)).await?;
        Ok(cached.and_then(|json| serde_json::from_str(&json).ok()))
    }

    /// Stores `response` for `request`, best-effort — a failure to write is
    /// logged, never propagated (a cache write failing shouldn't turn a
    /// successful simulation into a failed request).
    pub async fn set(&self, request: &SimulateRequest, response: &SimulateResponse) {
        match tokio::time::timeout(CACHE_TIMEOUT, self.set_inner(request, response)).await {
            Ok(Ok(())) => {}
            Ok(Err(err)) => skip_write(&err.to_string()),
            Err(_elapsed) => skip_write("timed out"),
        }
    }

    async fn set_inner(
        &self,
        request: &SimulateRequest,
        response: &SimulateResponse,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.connection.clone();
        let payload = serde_json::to_string(response).unwrap_or_default();
        let _: () = conn
            .set_ex(Self::key(request), payload, self.ttl_secs)
            .await?;
        Ok(())
    }
}

/// Logs why a cache lookup is being treated as a miss, and returns `None` —
/// the shared fail-open path for [`SimulateCache::get`].
fn miss(reason: &str) -> Option<SimulateResponse> {
    warn!(
        reason,
        "simulate cache: get unavailable, treating as a miss"
    );
    None
}

/// Logs why a cache write is being skipped — the shared fail-open path for
/// [`SimulateCache::set`].
fn skip_write(reason: &str) {
    warn!(
        reason,
        "simulate cache: set unavailable, skipping cache write"
    );
}

#[cfg(test)]
mod tests {
    use super::{Duration, SimulateCache, SimulateRequest, SimulateResponse};
    use std::collections::HashMap;
    use testcontainers_modules::redis::Redis;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;

    fn sample_request(seed: u64) -> SimulateRequest {
        let json = serde_json::json!({
            "baseline": {
                "risk_free_rate": "0.03", "inflation_rate": "0.02", "market_return": "0.08",
                "portfolio_return": "0.10", "return_std_dev": "0.15",
                "revenue": "1000000.0", "cogs": "600000.0", "operating_expenses": "200000.0",
                "interest_expense": "50000.0", "tax_rate": "0.25", "beta": "1.2",
                "cost_of_equity": "0.11", "cost_of_debt": "0.06",
                "total_assets": "1500000.0", "current_assets": "400000.0", "inventory": "100000.0",
                "current_liabilities": "200000.0", "total_liabilities": "750000.0", "total_equity": "750000.0",
                "share_price": "50.0", "shares_outstanding": "20000.0"
            },
            "config": {
                "iterations": 10, "seed": seed, "track_convergence": false,
                "convergence_threshold": "0.0001", "convergence_batch_size": 5
            }
        });
        serde_json::from_value(json).expect("valid SimulateRequest")
    }

    fn sample_response() -> SimulateResponse {
        SimulateResponse {
            scenarios_requested: 10,
            scenarios_evaluated: 10,
            scenarios_failed: 0,
            metrics: HashMap::new(),
        }
    }

    /// One Redis container per test — same reasoning as
    /// `middleware::redis_rate_limit`'s tests (a `ConnectionManager` spawns
    /// background tasks tied to the runtime that created it).
    async fn test_cache() -> (SimulateCache, ContainerAsync<Redis>) {
        let container = Redis::default()
            .start()
            .await
            .expect("start redis container");
        let port = container
            .get_host_port_ipv4(6379)
            .await
            .expect("get mapped port");
        let client =
            redis::Client::open(format!("redis://127.0.0.1:{port}/")).expect("open redis client");
        let connection = client
            .get_connection_manager()
            .await
            .expect("connection manager");
        (
            SimulateCache::new(connection, Duration::from_secs(60)),
            container,
        )
    }

    #[tokio::test]
    async fn miss_on_empty_cache() {
        let (cache, _container) = test_cache().await;
        assert!(cache.get(&sample_request(1)).await.is_none());
    }

    #[tokio::test]
    async fn set_then_get_round_trips() {
        let (cache, _container) = test_cache().await;
        let request = sample_request(2);
        let response = sample_response();
        cache.set(&request, &response).await;

        let cached = cache.get(&request).await.expect("cache hit");
        assert_eq!(cached.scenarios_requested, response.scenarios_requested);
        assert_eq!(cached.scenarios_evaluated, response.scenarios_evaluated);
        assert_eq!(cached.scenarios_failed, response.scenarios_failed);
    }

    #[tokio::test]
    async fn different_requests_do_not_collide() {
        let (cache, _container) = test_cache().await;
        cache.set(&sample_request(3), &sample_response()).await;
        assert!(cache.get(&sample_request(4)).await.is_none());
    }
}
