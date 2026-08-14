//! Postgres connection setup and migration running.

use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Runs every pending migration under `migrations/` (relative to this
/// crate) against `pool`. Idempotent: already-applied migrations are
/// skipped via sqlx's own migration-history table, and the migration runner
/// takes a Postgres advisory lock for the duration, so concurrent replicas
/// starting up at once can't race each other.
///
/// # Errors
///
/// Returns an error if any migration fails to apply.
pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Connects to `database_url` and runs every pending migration before
/// returning the pool, so the server never starts up against a
/// not-yet-migrated schema.
///
/// # Errors
///
/// Returns [`sqlx::Error`] if the connection fails or a migration fails to apply.
pub async fn connect_and_migrate(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    migrate(&pool)
        .await
        .map_err(|err| sqlx::Error::Migrate(Box::new(err)))?;
    Ok(pool)
}
