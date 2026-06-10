//! Download-related Tauri commands.

use crate::error::{CommandResponse, CommandResult};
use crate::models::download::DownloadTask;
use crate::services::download_service;
use crate::state::AppState;
use crate::utils::db_pool;
use tauri::State;

/// Start a new download task.
///
/// Creates a task record in the database and begins the chunked download.
/// Progress is emitted via the `download:progress` event.
#[tauri::command]
pub async fn start_download(
    state: State<'_, AppState>,
    name: String,
    url: String,
    target_path: String,
    total_size: i64,
    sha256: Option<String>,
    concurrency: Option<i64>,
) -> CommandResult<DownloadTask> {
    let pool = db_pool::get_pool(&state)?;
    let concurrency = concurrency.unwrap_or(8);

    let task = download_service::create_download_task(
        pool,
        &name,
        &url,
        &target_path,
        total_size,
        sha256.as_deref(),
        concurrency,
    )
    .await?;

    // Start the download in the background
    let task_clone = task.clone();
    let pool_clone = pool.clone();
    let http_client = state.http_client.clone();
    let app_handle = state.app_handle.clone();

    tauri::async_runtime::spawn(async move {
        if let Err(e) = download_service::start_download(
            &pool_clone,
            &http_client,
            &task_clone,
            &app_handle,
        )
        .await
        {
            tracing::error!("Download failed: {} - {}", task_clone.name, e);
        }
    });

    Ok(CommandResponse::ok(task))
}

/// Pause an active download.
#[tauri::command]
pub async fn pause_download(
    state: State<'_, AppState>,
    task_id: String,
) -> CommandResult<()> {
    let pool = db_pool::get_pool(&state)?;
    download_service::pause_download(pool, &task_id).await?;
    Ok(CommandResponse::ok(()))
}

/// Resume a paused download.
#[tauri::command]
pub async fn resume_download(
    state: State<'_, AppState>,
    task_id: String,
) -> CommandResult<DownloadTask> {
    let pool = db_pool::get_pool(&state)?;
    let task = download_service::resume_download(pool, &task_id).await?;

    // Restart the download in the background
    let pool_clone = pool.clone();
    let http_client = state.http_client.clone();
    let app_handle = state.app_handle.clone();

    tauri::async_runtime::spawn(async move {
        if let Err(e) = download_service::start_download(
            &pool_clone,
            &http_client,
            &task,
            &app_handle,
        )
        .await
        {
            tracing::error!("Resume download failed: {}", e);
        }
    });

    // Return a fresh copy of the task
    let task = download_service::get_download_task(pool, &task_id).await?;
    Ok(CommandResponse::ok(task))
}

/// Cancel a download task.
#[tauri::command]
pub async fn cancel_download(
    state: State<'_, AppState>,
    task_id: String,
) -> CommandResult<()> {
    let pool = db_pool::get_pool(&state)?;
    download_service::cancel_download(pool, &task_id).await?;
    Ok(CommandResponse::ok(()))
}

/// List all download tasks.
#[tauri::command]
pub async fn list_download_tasks(
    state: State<'_, AppState>,
) -> CommandResult<Vec<DownloadTask>> {
    let pool = db_pool::get_pool(&state)?;
    let tasks = download_service::list_download_tasks(pool).await?;
    Ok(CommandResponse::ok(tasks))
}
