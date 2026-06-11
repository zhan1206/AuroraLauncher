//! Installation-related Tauri commands.

use crate::error::{CommandResponse, CommandResult};
use crate::models::settings::DownloadMirror;
use crate::services::install_service;
use crate::state::AppState;
use crate::utils::file;
use tauri::State;

/// Install a Minecraft version to the global data directory.
///
/// Downloads the client JAR, all libraries, the asset index, and all assets
/// into `<data_dir>/versions/`, `<data_dir>/libraries/`, and `<data_dir>/assets/`.
/// Progress is emitted via the `install:progress` event.
/// Blocks until installation is complete.
#[tauri::command]
pub async fn install_version(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    version_id: String,
) -> CommandResult<()> {
    let mirror = get_mirror(&state).await;
    let target_dir = file::data_dir();

    let pool = state.db_pool.get()
        .ok_or_else(|| crate::error::AppError::Database("Database not initialized".to_string()))?
        .clone();
    let http_client = state.http_client.clone();

    install_service::install_version(
        &pool,
        &http_client,
        &app_handle,
        &version_id,
        &target_dir,
        &mirror,
    )
    .await?;

    Ok(CommandResponse::ok(()))
}

/// Install a Minecraft version into the global data directory for a specific instance.
///
/// Version files are shared globally under `<data_dir>/versions/` etc.
/// The instance's own game directory is used only for runtime data (saves, configs, natives).
/// Blocks until installation is complete.
#[tauri::command]
pub async fn install_version_for_instance(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    instance_id: String,
    version_id: String,
) -> CommandResult<()> {
    let mirror = get_mirror(&state).await;
    let target_dir = file::data_dir();

    let pool = state.db_pool.get()
        .ok_or_else(|| crate::error::AppError::Database("Database not initialized".to_string()))?
        .clone();
    let http_client = state.http_client.clone();

    install_service::install_version(
        &pool,
        &http_client,
        &app_handle,
        &version_id,
        &target_dir,
        &mirror,
    )
    .await?;

    Ok(CommandResponse::ok(()))
}

/// Helper: get the current download mirror from settings.
async fn get_mirror(state: &AppState) -> DownloadMirror {
    let pool = match state.db_pool.get() {
        Some(p) => p,
        None => return DownloadMirror::default(),
    };

    crate::services::settings_service::get_raw(pool, "download_mirror")
        .await
        .ok()
        .flatten()
        .map(|v| DownloadMirror::from_str_lossy(&v))
        .unwrap_or_default()
}
