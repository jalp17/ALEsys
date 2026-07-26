//! Database connection helper
//!
//! Shared DB pool creation logic used by both the API and CLI.

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Build a DATABASE_URL from PG* environment variables if DATABASE_URL is not set.
pub fn resolve_database_url() -> String {
    if let Ok(url) = std::env::var("DATABASE_URL") {
        return url;
    }
    let host = std::env::var("PGHOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("PGPORT").unwrap_or_else(|_| "5432".to_string());
    let user = std::env::var("PGUSER").unwrap_or_else(|_| "alesys".to_string());
    let password = std::env::var("PGPASSWORD").unwrap_or_else(|_| "alesys".to_string());
    let dbname = std::env::var("PGDATABASE").unwrap_or_else(|_| "alesys".to_string());
    format!("postgres://{}:{}@{}:{}/{}", user, password, host, port, dbname)
}

/// Create a connection pool to PostgreSQL.
///
/// Reads configuration from environment variables:
/// - `DATABASE_URL` (or `PGHOST`, `PGPORT`, `PGUSER`, `PGPASSWORD`, `PGDATABASE`)
/// - `DB_MAX_CONNECTIONS` (default: 5)
/// - `DB_MIN_CONNECTIONS` (default: 1)
pub async fn create_db_pool() -> crate::Result<PgPool> {
    let database_url = resolve_database_url();

    let max_connections = std::env::var("DB_MAX_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5u32);

    let min_connections = std::env::var("DB_MIN_CONNECTIONS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1u32);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .connect(&database_url)
        .await
        .map_err(|e| crate::AlesysError::Database(e))?;

    Ok(pool)
}

/// Execute raw SQL against the database.
pub async fn execute_sql(pool: &PgPool, sql: &str) -> crate::Result<()> {
    // Split by semicolons and execute each statement
    for statement in sql.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }
        sqlx::query(trimmed)
            .execute(pool)
            .await
            .map_err(|e| crate::AlesysError::Database(e))?;
    }
    Ok(())
}

/// Check if the database is reachable.
pub async fn check_database(pool: &PgPool) -> bool {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .is_ok()
}
