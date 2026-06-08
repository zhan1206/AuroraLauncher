//! Version-related Tauri commands.

use crate::error::{CommandResponse, CommandResult};
use crate::models::version::{VersionDetail, VersionManifest};
use crate::models::settings::DownloadMirror;
use crate::services::version_service::VersionService;
use crate::state::AppState;
use tauri::State;

/// Fetch the Minecraft version manifest.
///
/// Returns the full manifest including latest version pointers and all version entries.
/// Results are cached for 30 minutes.
#[tauri::command]
pub async fn get_version_manifest(
    state: State<'_, AppState>,
) -> CommandResult<VersionManifest> {
    let mirror = get_mirror(&state).await;
    let service = VersionService::new(state.http_client.clone());
    let manifest = service.get_manifest(&mirror).await?;
    Ok(CommandResponse::ok(manifest))
}

/// List Minecraft versions, optionally filtered by type.
///
/// `version_type` can be "release", "snapshot", "old_beta", or "old_alpha".
#[tauri::command]
pub async fn get_version_detail(
    state: State<'_, AppState>,
    version_url: String,
) -> CommandResult<VersionDetail> {
    let mirror = get_mirror(&state).await;
    let service = VersionService::new(state.http_client.clone());
    let detail = service.get_version_detail(&version_url, &mirror).await?;
    Ok(CommandResponse::ok(detail))
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
