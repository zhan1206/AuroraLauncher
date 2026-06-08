//! SQLite database initialization and migrations.
//!
//! The database file is stored at `{data_dir}/aurora.db` using WAL mode
//! for better concurrent read performance.

use crate::error::AppError;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::str::FromStr;

/// Initialize the SQLite database connection pool and run migrations.
///
/// The database file is stored at `{data_dir}/aurora.db`.
/// If the file does not exist, it will be created automatically.
pub async fn init_db(data_dir: &PathBuf) -> Result<SqlitePool, AppError> {
    // Ensure the data directory exists
    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir)
            .map_err(|e| AppError::DirectoryCreateFailed(e.to_string()))?;
    }

    let db_path = data_dir.join("aurora.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let options = SqliteConnectOptions::from_str(&db_url)
        .map_err(|e| AppError::Database(format!("Invalid database URL: {}", e)))?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .busy_timeout(std::time::Duration::from_secs(5));

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await
        .map_err(|e| AppError::Database(format!("Failed to connect to database: {}", e)))?;

    // Run migrations
    run_migrations(&pool).await?;

    tracing::info!("Database initialized at {}", db_path.display());
    Ok(pool)
}

/// Run database schema migrations.
async fn run_migrations(pool: &SqlitePool) -> Result<(), AppError> {
    // Create instances table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS instances (
            id              TEXT PRIMARY KEY,
            name            TEXT NOT NULL,
            version_id      TEXT NOT NULL,
            loader_type     TEXT NOT NULL DEFAULT 'Vanilla',
            loader_version  TEXT,
            game_dir        TEXT NOT NULL,
            java_id         TEXT,
            launch_config   TEXT NOT NULL,
            created_at      TEXT NOT NULL,
            updated_at      TEXT NOT NULL,
            icon            TEXT,
            notes           TEXT
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseMigration(format!("Failed to create instances table: {}", e)))?;

    // Create accounts table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS accounts (
            id            TEXT PRIMARY KEY,
            username      TEXT NOT NULL,
            display_name  TEXT,
            uuid          TEXT,
            account_type  TEXT NOT NULL DEFAULT 'Offline',
            is_active     INTEGER NOT NULL DEFAULT 0,
            created_at    TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseMigration(format!("Failed to create accounts table: {}", e)))?;

    // Create download_tasks table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS download_tasks (
            id           TEXT PRIMARY KEY,
            name         TEXT NOT NULL,
            url          TEXT NOT NULL,
            target_path  TEXT NOT NULL,
            total_size   INTEGER DEFAULT 0,
            downloaded   INTEGER DEFAULT 0,
            status       TEXT NOT NULL DEFAULT 'Pending',
            sha256       TEXT,
            concurrency  INTEGER DEFAULT 8,
            created_at   TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseMigration(format!("Failed to create download_tasks table: {}", e)))?;

    // Create settings table (key-value store)
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseMigration(format!("Failed to create settings table: {}", e)))?;

    // Create version_cache table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS version_cache (
            id           TEXT PRIMARY KEY,
            version_type TEXT NOT NULL,
            url          TEXT NOT NULL,
            release_time TEXT NOT NULL,
            cached_at    TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseMigration(format!("Failed to create version_cache table: {}", e)))?;

    // Insert default settings if they don't exist
    sqlx::query(
        r#"
        INSERT OR IGNORE INTO settings (key, value) VALUES
            ('download_mirror', 'Official'),
            ('default_max_memory', '2048'),
            ('default_min_memory', '512'),
            ('download_concurrency', '8'),
            ('language', 'zh-CN')
        "#,
    )
    .execute(pool)
    .await
    .map_err(|e| AppError::DatabaseMigration(format!("Failed to insert default settings: {}", e)))?;

    tracing::debug!("Database migrations completed");
    Ok(())
}
