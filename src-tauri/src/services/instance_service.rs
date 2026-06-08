//! Instance service.
//!
//! Provides CRUD operations for game instances with fully isolated
//! `.minecraft` directories. Each instance's metadata is persisted both
//! in SQLite (for queries) and as a JSON sidecar file (for portability).

use crate::error::AppError;
use crate::models::instance::{
    CreateInstanceRequest, Instance, LaunchConfig, UpdateInstanceRequest,
};
use crate::utils::file;
use chrono::Utc;
use sqlx::SqlitePool;
use uuid::Uuid;

/// Create a new game instance.
///
/// This function:
/// 1. Generates a UUID for the instance
/// 2. Creates the isolated game directory
/// 3. Persists the instance metadata to both SQLite and a JSON sidecar
pub async fn create_instance(
    pool: &SqlitePool,
    request: &CreateInstanceRequest,
) -> Result<Instance, AppError> {
    // Check for duplicate name
    let existing: Option<Instance> = sqlx::query_as::<_, Instance>(
        "SELECT * FROM instances WHERE name = ?",
    )
    .bind(&request.name)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if existing.is_some() {
        return Err(AppError::InstanceExists(request.name.clone()));
    }

    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let loader_type = request
        .loader_type
        .as_deref()
        .unwrap_or("Vanilla")
        .to_string();
    let launch_config = request
        .launch_config
        .as_ref()
        .map(|lc| serde_json::to_string(lc).unwrap_or_else(|_| "{}".to_string()))
        .unwrap_or_else(|| serde_json::to_string(&LaunchConfig::default()).unwrap_or_else(|_| "{}".to_string()));

    let game_dir = file::instance_game_dir(&id)
        .to_string_lossy()
        .to_string();

    // Create the game directory
    file::ensure_dir(&file::instance_game_dir(&id)).await?;

    // Insert into database
    sqlx::query(
        r#"
        INSERT INTO instances (id, name, version_id, loader_type, loader_version, game_dir, java_id, launch_config, created_at, updated_at, icon, notes)
        VALUES (?, ?, ?, ?, ?, ?, NULL, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(&request.name)
    .bind(&request.version_id)
    .bind(&loader_type)
    .bind(&request.loader_version)
    .bind(&game_dir)
    .bind(&launch_config)
    .bind(&now)
    .bind(&now)
    .bind(&request.icon)
    .bind(&request.notes)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Fetch the created instance
    let instance = sqlx::query_as::<_, Instance>(
        "SELECT * FROM instances WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    // Write the sidecar JSON
    let metadata_path = file::instance_metadata_path(&id);
    let json_data = serde_json::to_string_pretty(&instance)
        .map_err(|e| AppError::Serialization(e.to_string()))?;
    file::write_file_with_dirs(&metadata_path, json_data.as_bytes()).await?;

    tracing::info!("Created instance: {} ({})", instance.name, instance.id);
    Ok(instance)
}

/// List all game instances, ordered by most recently updated.
pub async fn list_instances(pool: &SqlitePool) -> Result<Vec<Instance>, AppError> {
    let instances = sqlx::query_as::<_, Instance>(
        "SELECT * FROM instances ORDER BY updated_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(instances)
}

/// Get a single instance by ID.
pub async fn get_instance(pool: &SqlitePool, id: &str) -> Result<Instance, AppError> {
    let instance = sqlx::query_as::<_, Instance>(
        "SELECT * FROM instances WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?
    .ok_or_else(|| AppError::InstanceNotFound(id.to_string()))?;

    Ok(instance)
}

/// Update an existing instance.
pub async fn update_instance(
    pool: &SqlitePool,
    request: &UpdateInstanceRequest,
) -> Result<Instance, AppError> {
    // Verify the instance exists
    let existing = get_instance(pool, &request.id).await?;

    let now = Utc::now().to_rfc3339();
    let name = request.name.as_deref().unwrap_or(&existing.name);
    let version_id = request
        .version_id
        .as_deref()
        .unwrap_or(&existing.version_id);
    let loader_type = request
        .loader_type
        .as_deref()
        .unwrap_or(&existing.loader_type);
    let loader_version = request
        .loader_version
        .as_deref()
        .or(existing.loader_version.as_deref());
    // java_id: explicit Some(v) sets it, None keeps existing value
    let java_id = if request.java_id.is_some() {
        request.java_id.as_deref()
    } else {
        existing.java_id.as_deref()
    };
    let launch_config = request
        .launch_config
        .as_ref()
        .map(|lc| serde_json::to_string(lc).unwrap_or_else(|_| existing.launch_config.clone()))
        .unwrap_or(existing.launch_config.clone());
    let icon = request.icon.as_deref().or(existing.icon.as_deref());
    let notes = request.notes.as_deref().or(existing.notes.as_deref());

    sqlx::query(
        r#"
        UPDATE instances SET name=?, version_id=?, loader_type=?, loader_version=?,
        java_id=?, launch_config=?, updated_at=?, icon=?, notes=? WHERE id=?
        "#,
    )
    .bind(name)
    .bind(version_id)
    .bind(loader_type)
    .bind(loader_version)
    .bind(java_id)
    .bind(&launch_config)
    .bind(&now)
    .bind(icon)
    .bind(notes)
    .bind(&request.id)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let updated = get_instance(pool, &request.id).await?;

    // Update the sidecar JSON
    let metadata_path = file::instance_metadata_path(&request.id);
    let json_data = serde_json::to_string_pretty(&updated)
        .map_err(|e| AppError::Serialization(e.to_string()))?;
    file::write_file_with_dirs(&metadata_path, json_data.as_bytes()).await?;

    tracing::info!("Updated instance: {} ({})", updated.name, updated.id);
    Ok(updated)
}

/// Delete an instance by ID.
///
/// Removes the database record and optionally the instance's game directory.
pub async fn delete_instance(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM instances WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if result.rows_affected() == 0 {
        return Err(AppError::InstanceNotFound(id.to_string()));
    }

    // Remove the instance directory
    let instance_dir = file::instances_dir().join(id);
    file::delete_dir(&instance_dir).await?;

    tracing::info!("Deleted instance: {}", id);
    Ok(())
}

/// Parse the launch configuration from an instance's JSON string.
pub fn parse_launch_config(instance: &Instance) -> LaunchConfig {
    serde_json::from_str(&instance.launch_config).unwrap_or_default()
}
