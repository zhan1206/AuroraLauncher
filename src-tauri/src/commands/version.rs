//! Version-related Tauri commands.

use crate::error::{AppError, CommandResponse, CommandResult};
use crate::models::settings::DownloadMirror;
use crate::models::version::{VersionDetail, VersionEntry, VersionManifest};
use crate::services::version_service;
use crate::state::{AppState, CachedManifest};
use std::time::Instant;
use tauri::State;

/// Fetch the Minecraft version manifest, using the 30-minute cache.
#[tauri::command]
pub async fn get_version_manifest(
    state: State<'_, AppState>,
) -> CommandResult<VersionManifest> {
    let mirror = get_mirror(&state).await;

    // Check cache
    {
        let cache = state.version_cache.read().await;
        if let Some(ref cached) = *cache {
            if !cached.is_expired() {
                tracing::debug!("Using cached version manifest");
                return Ok(CommandResponse::ok(cached.manifest.clone()));
            }
        }
    }

    // Fetch fresh manifest with mirror fallback
    let response = crate::utils::http::fetch_version_manifest(
        &state.http_client,
        &mirror,
    )
    .await?;

    if !response.status().is_success() {
        return Err(AppError::NetworkRequest(format!(
            "Failed to fetch version manifest: HTTP {}",
            response.status()
        )));
    }

    let manifest: VersionManifest = response
        .json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    tracing::info!(
        "Fetched version manifest: {} versions available",
        manifest.versions.len()
    );

    // Update cache
    {
        let mut cache = state.version_cache.write().await;
        *cache = Some(CachedManifest {
            manifest: manifest.clone(),
            fetched_at: Instant::now(),
        });
    }

    Ok(CommandResponse::ok(manifest))
}

/// List Minecraft versions, optionally filtered by type.
#[tauri::command]
pub async fn list_versions(
    state: State<'_, AppState>,
    version_type: Option<String>,
) -> CommandResult<Vec<VersionEntry>> {
    let mirror = get_mirror(&state).await;
    let service = version_service::VersionService::new(state.http_client.clone());
    let versions = service.list_versions(version_type.as_deref(), &mirror).await?;
    Ok(CommandResponse::ok(versions))
}

/// Fetch the detailed version JSON for a specific version.
#[tauri::command]
pub async fn get_version_detail(
    state: State<'_, AppState>,
    version_url: String,
) -> CommandResult<VersionDetail> {
    let mirror = get_mirror(&state).await;
    let service = version_service::VersionService::new(state.http_client.clone());
    let detail = service.get_version_detail(&version_url, &mirror).await?;
    Ok(CommandResponse::ok(detail))
}

/// Check whether a specific version is installed (version.json and client jar exist).
#[tauri::command]
pub async fn check_version_installed(
    version_id: String,
) -> CommandResult<bool> {
    let data_dir = crate::utils::file::data_dir();
    let version_dir = data_dir.join("versions").join(&version_id);
    let version_json = version_dir.join(format!("{}.json", version_id));
    let client_jar = version_dir.join(format!("{}.jar", version_id));

    let installed = version_json.exists() && client_jar.exists();
    Ok(CommandResponse::ok(installed))
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
