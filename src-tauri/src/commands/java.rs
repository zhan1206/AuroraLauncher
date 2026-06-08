//! Java-related Tauri commands.

use crate::error::{CommandResponse, CommandResult};
use crate::services::java_service;
use crate::services::java_service::JavaRuntime;
use crate::state::AppState;
use tauri::State;

/// List all available Java runtimes (both managed and system).
#[tauri::command]
pub async fn list_java_runtimes() -> CommandResult<Vec<JavaRuntime>> {
    let runtimes = java_service::list_java_runtimes().await?;
    Ok(CommandResponse::ok(runtimes))
}

/// Download and install a JRE from Adoptium.
///
/// `major_version` should be 8, 17, or 21.
#[tauri::command]
pub async fn download_java(
    state: State<'_, AppState>,
    major_version: u32,
) -> CommandResult<JavaRuntime> {
    let runtime = java_service::download_java(&state.http_client, major_version).await?;
    Ok(CommandResponse::ok(runtime))
}

/// Resolve the best Java runtime for a given Minecraft version.
///
/// Automatically selects Java 8, 17, or 21 based on the version ID.
#[tauri::command]
pub async fn resolve_java(
    version_id: String,
) -> CommandResult<JavaRuntime> {
    let runtime = java_service::resolve_java(&version_id).await?;
    Ok(CommandResponse::ok(runtime))
}
