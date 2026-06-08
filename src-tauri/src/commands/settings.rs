//! Settings-related Tauri commands.

use crate::error::{CommandResponse, CommandResult};
use crate::models::settings::AppSettings;
use crate::services::settings_service;
use crate::state::AppState;
use crate::utils::db_pool;
use tauri::State;

/// Get the typed application settings.
#[tauri::command]
pub async fn get_settings(
    state: State<'_, AppState>,
) -> CommandResult<AppSettings> {
    let pool = db_pool::get_pool(&state)?;
    let settings = settings_service::get_settings(pool).await?;
    Ok(CommandResponse::ok(settings))
}

/// Update application settings.
#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> CommandResult<()> {
    let pool = db_pool::get_pool(&state)?;
    settings_service::update_settings(pool, &settings).await?;
    Ok(CommandResponse::ok(()))
}

/// Reset all settings to defaults.
#[tauri::command]
pub async fn reset_settings(
    state: State<'_, AppState>,
) -> CommandResult<AppSettings> {
    let pool = db_pool::get_pool(&state)?;
    let defaults = settings_service::reset_settings(pool).await?;
    Ok(CommandResponse::ok(defaults))
}
