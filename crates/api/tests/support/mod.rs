//! Shared test-only helper for spinning up a real, migrated Postgres via
//! testcontainers. Lives under `tests/support/` (not a top-level
//! `tests/*.rs` file) so it's a plain module included by `mod support;`,
//! not a separate test binary of its own.
//!
//! One container per test function, not shared across tests: a `PgPool`
//! spawns background tasks onto the Tokio runtime it was created in, and
//! `#[tokio::test]`/`#[actix_web::test]` give each test function its own
//! runtime — sharing a pool across them breaks it (`PoolTimedOut`, since the
//! pool's connection-management tasks die with the runtime that spawned
//! them). Per-test containers cost a little startup time but keep every
//! test independent and parallelizable, and Docker layer caching keeps that
//! cost small after the first run.

#![allow(dead_code)] // not every test binary that includes this module uses every item

use redis::aio::ConnectionManager;
use sqlx::PgPool;
use testcontainers_modules::postgres::Postgres;
use testcontainers_modules::redis::Redis;
use testcontainers_modules::testcontainers::ContainerAsync;
use testcontainers_modules::testcontainers::ImageExt;
use testcontainers_modules::testcontainers::runners::AsyncRunner;

/// A migrated Postgres, plus the container it lives in — keep this alive
/// for the duration of a test; it's torn down (and the container removed)
/// when dropped.
pub(crate) struct TestDb {
    pub(crate) pool: PgPool,
    _container: ContainerAsync<Postgres>,
}

/// Starts a fresh "16"-tagged Postgres container (matching
/// `docker-compose.yml`'s `db` service — the crate's own default tag,
/// `11-alpine`, predates the recursive-CTE `CYCLE` clause the ledger
/// persistence module relies on) and runs every migration against it.
pub(crate) async fn test_db() -> TestDb {
    let container = Postgres::default()
        .with_tag("16")
        .start()
        .await
        .expect("start postgres container");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("get mapped port");
    let database_url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
    let pool = casiros_api::persistence::db::connect_and_migrate(&database_url)
        .await
        .expect("connect and migrate");
    TestDb {
        pool,
        _container: container,
    }
}

/// A Redis `ConnectionManager`, plus the container it lives in — same
/// per-test-container reasoning as [`TestDb`] (a `ConnectionManager` also
/// spawns background tasks tied to the runtime that created it).
pub(crate) struct TestRedis {
    pub(crate) connection: ConnectionManager,
    _container: ContainerAsync<Redis>,
}

/// Starts a fresh Redis container and returns a connected, multiplexed
/// `ConnectionManager` — matches what `main.rs` builds in production.
pub(crate) async fn test_redis() -> TestRedis {
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
    TestRedis {
        connection,
        _container: container,
    }
}
