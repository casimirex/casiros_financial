//! A simple in-memory sliding-window rate limiter, keyed by client IP.
//!
//! No external rate-limiting crate is used here since none is listed in the
//! build prompt's `casiros-api` dependencies (section 7.1); a `HashMap` guarded
//! by a `Mutex` is a small enough amount of logic to own directly.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Tracks recent request timestamps per client IP and rejects requests once
/// the configured limit is exceeded within the configured window.
pub struct RateLimiter {
    max_requests: usize,
    window: Duration,
    requests: Mutex<HashMap<IpAddr, Vec<Instant>>>,
}

impl RateLimiter {
    /// Creates a limiter allowing at most `max_requests` per `window`, per client IP.
    #[must_use]
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            requests: Mutex::new(HashMap::new()),
        }
    }

    /// Records a request from `ip` and returns whether it is within the rate
    /// limit. Also prunes timestamps older than the window, so the map
    /// doesn't grow unboundedly for long-lived clients.
    #[must_use]
    pub fn check(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut requests = self
            .requests
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let timestamps = requests.entry(ip).or_default();
        timestamps.retain(|&t| now.duration_since(t) < self.window);
        if timestamps.len() >= self.max_requests {
            return false;
        }
        timestamps.push(now);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::RateLimiter;
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::Duration;

    fn localhost() -> IpAddr {
        IpAddr::V4(Ipv4Addr::LOCALHOST)
    }

    #[test]
    fn allows_requests_within_the_limit() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));
        let ip = localhost();
        assert!(limiter.check(ip));
        assert!(limiter.check(ip));
        assert!(limiter.check(ip));
    }

    #[test]
    fn rejects_requests_once_the_limit_is_exceeded() {
        let limiter = RateLimiter::new(2, Duration::from_secs(60));
        let ip = localhost();
        assert!(limiter.check(ip));
        assert!(limiter.check(ip));
        assert!(!limiter.check(ip));
    }

    #[test]
    fn tracks_each_ip_independently() {
        let limiter = RateLimiter::new(1, Duration::from_secs(60));
        let a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        assert!(limiter.check(a));
        assert!(!limiter.check(a));
        assert!(limiter.check(b));
    }

    #[test]
    fn old_timestamps_are_pruned_once_the_window_elapses() {
        let limiter = RateLimiter::new(1, Duration::from_millis(20));
        let ip = localhost();
        assert!(limiter.check(ip));
        assert!(!limiter.check(ip));
        std::thread::sleep(Duration::from_millis(30));
        assert!(limiter.check(ip));
    }
}
