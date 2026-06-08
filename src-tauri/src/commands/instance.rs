//! Instance-related Tauri commands.

use crate::error::{CommandResponse, CommandResult};
use crate::models::instance::{CreateInstanceRequest, Instance, UpdateInstanceRequest};
use crate::services::instance_service;
use crate::state::AppState;
use crate::utils::db_pool;
use tauri::State;

/// Create a new game instance.
///
/// The instance gets a fully isolated `.minecraft` directory and is
/// persisted to both SQLite and a JSON sidecar file.
#[tauri::command]
pub async fn create_instance(
    state: State<'_, AppState>,
    request: CreateInstanceRequest,
) -> CommandResult<Instance> {
    let pool = db_pool::get_pool(&state)?;
    let instance = instance_service::create_instance(pool, &request).await?;
    Ok(CommandResponse::ok(instance))
}

/// List all game instances, ordered by most recently updated.
#[tauri::command]
pub async fn list_instances(
    state: State<'_, AppState>,
) -> CommandResult<Vec<Instance>> {
    let pool = db_pool::get_pool(&state)?;
    let instances = instance_service::list_instances(pool).await?;
    Ok(CommandResponse::ok(instances))
}

/// Get a single instance by ID.
#[tauri::command]
pub async fn get_instance(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<Instance> {
    let pool = db_pool::get_pool(&state)?;
    let instance = instance_service::get_instance(pool, &id).await?;
    Ok(CommandResponse::ok(instance))
}

/// Update an existing instance.
#[tauri::command]
pub async fn update_instance(
    state: State<'_, AppState>,
    request: UpdateInstanceRequest,
) -> CommandResult<Instance> {
    let pool = db_pool::get_pool(&state)?;
    let instance = instance_service::update_instance(pool, &request).await?;
    Ok(CommandResponse::ok(instance))
}

/// Delete an instance by ID.
///
/// Removes the database record and the instance's game directory.
#[tauri::command]
pub async fn delete_instance(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<()> {
    let pool = db_pool::get_pool(&state)?;
    instance_service::delete_instance(pool, &id).await?;
    Ok(CommandResponse::ok(()))
}
