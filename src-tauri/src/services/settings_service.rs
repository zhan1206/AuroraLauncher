//! Settings service.
//!
//! Provides typed read/write access to global application settings
//! stored as key-value pairs in the SQLite database.

use crate::error::AppError;
use crate::models::settings::{AppSettings, DownloadMirror};
use crate::utils::file;
use sqlx::SqlitePool;

/// Known settings keys.
mod keys {
    pub const DOWNLOAD_MIRROR: &str = "download_mirror";
    pub const DEFAULT_MAX_MEMORY: &str = "default_max_memory";
    pub const DEFAULT_MIN_MEMORY: &str = "default_min_memory";
    pub const DOWNLOAD_CONCURRENCY: &str = "download_concurrency";
    pub const CUSTOM_JAVA_PATH: &str = "custom_java_path";
    pub const WINDOW_WIDTH: &str = "window_width";
    pub const WINDOW_HEIGHT: &str = "window_height";
    pub const LANGUAGE: &str = "language";
}

/// Get a raw setting value by key.
pub async fn get_raw(pool: &SqlitePool, key: &str) -> Result<Option<String>, AppError> {
    let result = sqlx::query_as::<_, (String, String)>(
        "SELECT key, value FROM settings WHERE key = ?",
    )
    .bind(key)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(result.map(|(_, v)| v))
}

/// Set a raw setting value (upsert).
pub async fn set_raw(pool: &SqlitePool, key: &str, value: &str) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = ?",
    )
    .bind(key)
    .bind(value)
    .bind(value)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    tracing::debug!("Setting updated: {} = {}", key, value);
    Ok(())
}

/// Get all settings as key-value pairs.
pub async fn get_all(pool: &SqlitePool) -> Result<Vec<(String, String)>, AppError> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT key, value FROM settings ORDER BY key",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(rows)
}

/// Get the typed application settings, filling in defaults for missing keys.
pub async fn get_settings(pool: &SqlitePool) -> Result<AppSettings, AppError> {
    let mut settings = AppSettings::default();

    if let Some(val) = get_raw(pool, keys::DOWNLOAD_MIRROR).await? {
        settings.download_mirror = DownloadMirror::from_str_lossy(&val);
    }
    if let Some(val) = get_raw(pool, keys::DEFAULT_MAX_MEMORY).await? {
        settings.default_max_memory = val.parse().unwrap_or(2048);
    }
    if let Some(val) = get_raw(pool, keys::DEFAULT_MIN_MEMORY).await? {
        settings.default_min_memory = val.parse().unwrap_or(512);
    }
    if let Some(val) = get_raw(pool, keys::DOWNLOAD_CONCURRENCY).await? {
        settings.download_concurrency = val.parse().unwrap_or(8);
    }
    if let Some(val) = get_raw(pool, keys::CUSTOM_JAVA_PATH).await? {
        if !val.is_empty() {
            settings.custom_java_path = Some(val);
        }
    }
    if let Some(val) = get_raw(pool, keys::WINDOW_WIDTH).await? {
        settings.window_width = val.parse().unwrap_or(1280);
    }
    if let Some(val) = get_raw(pool, keys::WINDOW_HEIGHT).await? {
        settings.window_height = val.parse().unwrap_or(800);
    }
    if let Some(val) = get_raw(pool, keys::LANGUAGE).await? {
        settings.language = val;
    }

    Ok(settings)
}

/// Update application settings from a typed struct.
///
/// Only writes keys that differ from the current stored values.
pub async fn update_settings(pool: &SqlitePool, settings: &AppSettings) -> Result<(), AppError> {
    set_raw(pool, keys::DOWNLOAD_MIRROR, settings.download_mirror.as_str()).await?;
    set_raw(pool, keys::DEFAULT_MAX_MEMORY, &settings.default_max_memory.to_string()).await?;
    set_raw(pool, keys::DEFAULT_MIN_MEMORY, &settings.default_min_memory.to_string()).await?;
    set_raw(pool, keys::DOWNLOAD_CONCURRENCY, &settings.download_concurrency.to_string()).await?;
    set_raw(pool, keys::CUSTOM_JAVA_PATH, settings.custom_java_path.as_deref().unwrap_or("")).await?;
    set_raw(pool, keys::WINDOW_WIDTH, &settings.window_width.to_string()).await?;
    set_raw(pool, keys::WINDOW_HEIGHT, &settings.window_height.to_string()).await?;
    set_raw(pool, keys::LANGUAGE, &settings.language).await?;

    tracing::info!("Application settings updated");
    Ok(())
}

/// Reset all settings to defaults.
pub async fn reset_settings(pool: &SqlitePool) -> Result<AppSettings, AppError> {
    let defaults = AppSettings::default();
    update_settings(pool, &defaults).await?;
    Ok(defaults)
}

/// Get the data directory path.
pub fn get_data_dir() -> String {
    file::data_dir().to_string_lossy().to_string()
}
