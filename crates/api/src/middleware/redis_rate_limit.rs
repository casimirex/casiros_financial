//! A Redis-backed sliding-window rate limiter, keyed by client IP.
//!
//! Replaces the in-process `Mutex<HashMap<IpAddr, Vec<Instant>>>` limiter
//! (`rate_limit.rs`, now deleted) with the standard Redis sorted-set
//! sliding-window pattern: `ZADD` a per-request member scored by its
//! timestamp, `ZREMRANGEBYSCORE` to prune anything older than the window,
//! `ZCARD` to count what's left. Semantically identical to the old
//! in-process version — same per-IP sliding window, same limit — just
//! backed by a store every worker (and, if this ever runs multi-instance,
//! every instance) shares.
//!
//! Fail-open on a Redis error: logged via `tracing::warn!` and treated as
//! "allow," matching `state.rs`'s `lock()` helper's own reasoning ("a single
//! failed request should not permanently wedge shared state for every
//! subsequent request") applied to an external dependency instead of a
//! local mutex — rate limiting is defense-in-depth here, not a hard
//! security boundary, and an outage in a cache/rate-limit store shouldn't
//! take a financial API down.
//!
//! That fail-open promise only holds if a Redis outage actually produces an
//! *error* quickly rather than *hanging* — verified this the hard way:
//! `ConnectionManager`'s default reconnect behavior blocks in-flight
//! commands while it retries, so with no timeout at all a stopped Redis
//! container made every request hang for 15+ seconds instead of failing
//! open. [`CHECK_TIMEOUT`] bounds every check so a Redis outage degrades to
//! "unlimited," not "the whole API stalls."

use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use std::net::IpAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tracing::warn;
use uuid::Uuid;

/// The maximum time a single rate-limit check may take before it's treated
/// as a failure (and, per the fail-open policy, the request is allowed
/// through). Generous relative to Redis's normal sub-millisecond latency,
/// tight relative to how long a caller should ever wait on rate limiting.
const CHECK_TIMEOUT: Duration = Duration::from_millis(250);

/// Tracks recent request timestamps per client IP in Redis and rejects
/// requests once the configured limit is exceeded within the configured window.
#[derive(Clone)]
pub struct RedisRateLimiter {
    connection: ConnectionManager,
    max_requests: usize,
    window: Duration,
}

impl RedisRateLimiter {
    /// Creates a limiter allowing at most `max_requests` per `window`, per
    /// client IP, backed by `connection`.
    #[must_use]
    pub fn new(connection: ConnectionManager, max_requests: usize, window: Duration) -> Self {
        Self {
            connection,
            max_requests,
            window,
        }
    }

    /// Records a request from `ip` and returns whether it is within the
    /// rate limit. Never returns an error to the caller, and never blocks
    /// longer than [`CHECK_TIMEOUT`]: a slow or failed Redis is logged and
    /// treated as "allow" (see the module docs).
    pub async fn check(&self, ip: IpAddr) -> bool {
        match tokio::time::timeout(CHECK_TIMEOUT, self.check_inner(ip)).await {
            Ok(Ok(allowed)) => allowed,
            Ok(Err(err)) => fail_open(ip, &err.to_string()),
            Err(_elapsed) => fail_open(ip, "check timed out"),
        }
    }

    async fn check_inner(&self, ip: IpAddr) -> Result<bool, redis::RedisError> {
        let mut conn = self.connection.clone();
        let key = format!("casiros:ratelimit:{ip}");

        let now_millis = now_millis()?;
        let window_millis = i64::try_from(self.window.as_millis()).unwrap_or(i64::MAX);
        let cutoff = now_millis.saturating_sub(window_millis);

        let _pruned: i64 = conn.zrembyscore(&key, i64::MIN, cutoff).await?;
        let count: usize = conn.zcard(&key).await?;
        if count >= self.max_requests {
            return Ok(false);
        }

        // A per-request-unique member: two requests landing in the same
        // millisecond must still count as two entries, not collapse into one.
        let member = format!("{now_millis}-{}", Uuid::new_v4());
        let _: () = conn.zadd(&key, member, now_millis).await?;
        // Millisecond-precision PEXPIRE, not EXPIRE: EXPIRE's whole-second
        // truncation would round any sub-second window down to 0 and delete
        // the key — including the member this call just added — immediately.
        let _: () = conn.pexpire(&key, window_millis).await?;
        Ok(true)
    }
}

/// Logs `reason` and returns `true` — the shared fail-open path for both a
/// Redis error and a check that ran past [`CHECK_TIMEOUT`].
fn fail_open(ip: IpAddr, reason: &str) -> bool {
    warn!(%ip, reason, "rate limiter: redis unavailable, failing open");
    true
}

fn now_millis() -> Result<i64, redis::RedisError> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    i64::try_from(elapsed.as_millis()).map_err(|_| {
        redis::RedisError::from((
            redis::ErrorKind::TypeError,
            "system clock too far in the future to fit in an i64 millisecond timestamp",
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::RedisRateLimiter;
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::Duration;
    use testcontainers_modules::redis::Redis;
    use testcontainers_modules::testcontainers::ContainerAsync;
    use testcontainers_modules::testcontainers::runners::AsyncRunner;

    /// One Redis container per test — a `ConnectionManager` spawns
    /// background tasks tied to the Tokio runtime it was created in, and
    /// `#[tokio::test]` gives each test its own runtime, so sharing one
    /// across tests would break it the same way a shared `PgPool` does (see
    /// `tests/support/mod.rs`'s doc comment).
    async fn test_limiter(
        max_requests: usize,
        window: Duration,
    ) -> (RedisRateLimiter, ContainerAsync<Redis>) {
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
            RedisRateLimiter::new(connection, max_requests, window),
            container,
        )
    }

    fn localhost() -> IpAddr {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    }

    #[tokio::test]
    async fn allows_requests_within_the_limit() {
        let (limiter, _container) = test_limiter(3, Duration::from_secs(60)).await;
        let ip = localhost();
        assert!(limiter.check(ip).await);
        assert!(limiter.check(ip).await);
        assert!(limiter.check(ip).await);
    }

    #[tokio::test]
    async fn rejects_requests_once_the_limit_is_exceeded() {
        let (limiter, _container) = test_limiter(2, Duration::from_secs(60)).await;
        let ip = localhost();
        assert!(limiter.check(ip).await);
        assert!(limiter.check(ip).await);
        assert!(!limiter.check(ip).await);
    }

    #[tokio::test]
    async fn tracks_each_ip_independently() {
        let (limiter, _container) = test_limiter(1, Duration::from_secs(60)).await;
        let a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        assert!(limiter.check(a).await);
        assert!(!limiter.check(a).await);
        assert!(limiter.check(b).await);
    }

    #[tokio::test]
    async fn old_timestamps_are_pruned_once_the_window_elapses() {
        let (limiter, _container) = test_limiter(1, Duration::from_millis(200)).await;
        let ip = localhost();
        assert!(limiter.check(ip).await);
        assert!(!limiter.check(ip).await);
        tokio::time::sleep(Duration::from_millis(300)).await;
        assert!(limiter.check(ip).await);
    }
}
