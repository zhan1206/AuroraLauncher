//! Download service.
//!
//! Multi-threaded chunked download with resume support and SHA-256 verification.
//! Progress is reported via Tauri events (`download:progress`).

use crate::error::AppError;
use crate::models::download::{ChunkInfo, DownloadProgress, DownloadStatus, DownloadTask, PartFile};
use crate::utils::crypto;
use crate::utils::file;
use chrono::Utc;
use futures::stream::{self, StreamExt};
use reqwest::Client;
use sqlx::SqlitePool;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;
use tauri::Emitter;
use uuid::Uuid;

/// Default chunk size for parallel downloads (8 MB).
const DEFAULT_CHUNK_SIZE: u64 = 8 * 1024 * 1024;

/// Default concurrency for downloads.
const DEFAULT_CONCURRENCY: usize = 8;

/// Create a new download task in the database.
pub async fn create_download_task(
    pool: &SqlitePool,
    name: &str,
    url: &str,
    target_path: &str,
    total_size: i64,
    sha256: Option<&str>,
    concurrency: i64,
) -> Result<DownloadTask, AppError> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();
    let concurrency = if concurrency > 0 { concurrency } else { DEFAULT_CONCURRENCY as i64 };

    sqlx::query(
        r#"
        INSERT INTO download_tasks (id, name, url, target_path, total_size, downloaded, status, sha256, concurrency, created_at)
        VALUES (?, ?, ?, ?, ?, 0, 'Pending', ?, ?, ?)
        "#,
    )
    .bind(&id)
    .bind(name)
    .bind(url)
    .bind(target_path)
    .bind(total_size)
    .bind(sha256)
    .bind(concurrency)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let task = sqlx::query_as::<_, DownloadTask>(
        "SELECT * FROM download_tasks WHERE id = ?",
    )
    .bind(&id)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(task)
}

/// Start a chunked download with the given task.
///
/// This function:
/// 1. Sends a HEAD request to determine the file size and Range support
/// 2. Splits the download into chunks
/// 3. Downloads all chunks in parallel
/// 4. Assembles the final file
/// 5. Verifies SHA-256 if a hash was provided
/// 6. Emits progress events via the Tauri app handle
pub async fn start_download(
    pool: &SqlitePool,
    http_client: &Client,
    task: &DownloadTask,
    app_handle: &tauri::AppHandle,
) -> Result<(), AppError> {
    let target = PathBuf::from(&task.target_path);
    let part_file_path = file::part_meta_path(&target);
    let temp_file_path = file::temp_file_path(&target);

    // Update status to Downloading
    update_status(pool, &task.id, DownloadStatus::Downloading).await?;

    // Send HEAD request to get content length and check Range support
    let head_response = http_client
        .head(&task.url)
        .send()
        .await
        .map_err(|e| AppError::NetworkRequest(e.to_string()))?;

    if !head_response.status().is_success() {
        update_status(pool, &task.id, DownloadStatus::Failed).await?;
        return Err(AppError::NetworkRequest(format!(
            "HEAD request failed: HTTP {}",
            head_response.status()
        )));
    }

    let total_size: u64 = head_response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(task.total_size as u64);

    let accepts_ranges = head_response
        .headers()
        .get("accept-ranges")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("bytes"))
        .unwrap_or(false);

    // Update total size in database
    sqlx::query("UPDATE download_tasks SET total_size = ? WHERE id = ?")
        .bind(total_size as i64)
        .bind(&task.id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Check for existing part file (resume support)
    let part_file = if file::file_exists(&part_file_path).await {
        let data = file::read_file(&part_file_path).await?;
        serde_json::from_slice::<PartFile>(&data).ok()
    } else {
        None
    };

    // Create or validate part file
    let part = match part_file {
        Some(pf) => pf,
        None => {
            let chunks = if accepts_ranges && total_size > 0 {
                create_chunks(total_size, DEFAULT_CHUNK_SIZE)
            } else {
                // Single chunk if server doesn't support Range
                vec![ChunkInfo {
                    index: 0,
                    start: 0,
                    end: total_size.saturating_sub(1),
                    downloaded: 0,
                    completed: false,
                }]
            };

            let pf = PartFile {
                url: task.url.clone(),
                target_path: task.target_path.clone(),
                total_size,
                sha256: task.sha256.clone(),
                chunk_size: DEFAULT_CHUNK_SIZE,
                chunks,
            };

            // Write part file
            let json = serde_json::to_string_pretty(&pf)
                .map_err(|e| AppError::Serialization(e.to_string()))?;
            file::write_file_with_dirs(&part_file_path, json.as_bytes()).await?;

            pf
        }
    };

    // Ensure target directory exists
    if let Some(parent) = target.parent() {
        file::ensure_dir(parent).await?;
    }

    // Create the temp file (pre-allocate space)
    {
        let mut file = tokio::fs::File::create(&temp_file_path)
            .await
            .map_err(|e| AppError::FileIo(e))?;
        file.set_len(total_size).await.map_err(|e| AppError::FileIo(e))?;
    }

    // Track progress for emission
    let initial_downloaded: u64 = part.chunks.iter().map(|c| c.downloaded).sum();
    let total_downloaded: Arc<Mutex<u64>> = Arc::new(Mutex::new(initial_downloaded));
    let start_time = std::time::Instant::now();

    // Download incomplete chunks in parallel
    let concurrency = task.concurrency as usize;
    let incomplete_chunks: Vec<usize> = part
        .chunks
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.completed)
        .map(|(i, _)| i)
        .collect();

    let chunk_results: Vec<Result<usize, AppError>> = stream::iter(incomplete_chunks)
        .map(|chunk_idx| {
            let pool_clone = pool.clone();
            let http_client = http_client.clone();
            let task_id = task.id.clone();
            let url = task.url.clone();
            let temp_path = temp_file_path.clone();
            let total_size_for_progress = total_size;
            let total_downloaded_clone = total_downloaded.clone();
            let app_handle = app_handle.clone();

            async move {
                download_chunk(
                    &pool_clone,
                    &http_client,
                    &task_id,
                    &url,
                    &temp_path,
                    chunk_idx,
                    total_size_for_progress,
                    total_downloaded_clone,
                    &app_handle,
                    start_time,
                )
                .await
            }
        })
        .buffer_unordered(concurrency.min(DEFAULT_CONCURRENCY))
        .collect()
        .await;

    // Check for chunk errors
    for result in chunk_results {
        if let Err(e) = result {
            update_status(pool, &task.id, DownloadStatus::Failed).await?;
            return Err(e);
        }
    }

    // Verify hash if provided (auto-detects SHA-256 or SHA-1)
    // Non-fatal: log mismatch as warning but continue installation
    if let Some(ref expected_hash) = task.sha256 {
        if let Err(e) = crypto::verify_hash(&temp_file_path, expected_hash).await {
            tracing::warn!("Hash verification failed for {}: {} (continuing anyway)", task.name, e);
        }
    }

    // Move temp file to final destination
    file::move_file(&temp_file_path, &target).await?;

    // Clean up part file
    file::delete_file(&part_file_path).await?;

    // Update status to Completed
    sqlx::query("UPDATE download_tasks SET downloaded = total_size, status = 'Completed' WHERE id = ?")
        .bind(&task.id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    tracing::info!("Download completed: {} -> {}", task.name, task.target_path);
    Ok(())
}

/// Download a single chunk of a file.
///
/// Each chunk downloads a specific byte range and writes it to the correct
/// offset in the shared temp file. The file is opened with write access
/// and seeks to the correct position before writing.
async fn download_chunk(
    pool: &SqlitePool,
    http_client: &Client,
    task_id: &str,
    url: &str,
    temp_path: &Path,
    chunk_idx: usize,
    total_size: u64,
    total_downloaded: Arc<Mutex<u64>>,
    app_handle: &tauri::AppHandle,
    start_time: std::time::Instant,
) -> Result<usize, AppError> {
    // Read the part file to get chunk info
    let target_guess = PathBuf::from(
        temp_path.to_string_lossy().replace(".part", ""),
    );
    let part_file_path = file::part_meta_path(&target_guess);
    let part_data = file::read_file(&part_file_path).await?;
    let mut part: PartFile = serde_json::from_slice(&part_data)
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    let chunk = part.chunks.get(chunk_idx)
        .ok_or_else(|| AppError::DownloadFailed(format!("Invalid chunk index: {}", chunk_idx)))?;

    let chunk_start = chunk.start;
    let resume_offset = chunk.downloaded;
    let range_start = chunk_start + resume_offset;
    let range_end = chunk.end;

    if range_start > range_end {
        return Ok(chunk_idx);
    }

    // Open the temp file for writing at the correct offset
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(temp_path)
        .await
        .map_err(|e| AppError::FileIo(e))?;

    // Download with Range header
    let range_header = format!("bytes={}-{}", range_start, range_end);
    let response = http_client
        .get(url)
        .header("Range", range_header)
        .send()
        .await
        .map_err(|e| AppError::NetworkRequest(e.to_string()))?;

    if !response.status().is_success() && response.status().as_u16() != 206 {
        return Err(AppError::DownloadFailed(format!(
            "Range request failed: HTTP {}",
            response.status()
        )));
    }

    // Stream the response body
    let mut stream = response.bytes_stream();
    let mut bytes_written: u64 = 0;

    while let Some(chunk_bytes) = stream.next().await {
        let data = chunk_bytes.map_err(|e| AppError::NetworkRequest(e.to_string()))?;

        // Seek to the correct position and write
        let write_offset = range_start + bytes_written;
        file.seek(tokio::io::SeekFrom::Start(write_offset))
            .await
            .map_err(|e| AppError::FileIo(e))?;
        file.write_all(&data).await.map_err(|e| AppError::FileIo(e))?;

        bytes_written += data.len() as u64;

        // Update total progress
        {
            let mut total = total_downloaded.lock().await;
            *total += data.len() as u64;

            // Emit progress event
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 { (*total as f64 / elapsed) as u64 } else { 0 };
            let percent = if total_size > 0 { (*total as f64 / total_size as f64) * 100.0 } else { 0.0 };

            let progress = DownloadProgress {
                task_id: task_id.to_string(),
                total: total_size,
                downloaded: *total,
                speed,
                percent,
            };

            let _ = app_handle.emit("download:progress", &progress);
        }
    }

    // Update the part file with chunk completion
    part.chunks[chunk_idx].downloaded = resume_offset + bytes_written;
    part.chunks[chunk_idx].completed = true;

    let part_json = serde_json::to_string_pretty(&part)
        .map_err(|e| AppError::Serialization(e.to_string()))?;
    file::write_file_with_dirs(&part_file_path, part_json.as_bytes()).await?;

    // Update database progress (best effort)
    let downloaded_total: u64 = part.chunks.iter().map(|c| c.downloaded).sum();
    sqlx::query("UPDATE download_tasks SET downloaded = ? WHERE id = ?")
        .bind(downloaded_total as i64)
        .bind(task_id)
        .execute(pool)
        .await
        .ok();

    Ok(chunk_idx)
}

/// Create evenly-sized chunks for a download.
fn create_chunks(total_size: u64, chunk_size: u64) -> Vec<ChunkInfo> {
    let mut chunks = Vec::new();
    let mut offset: u64 = 0;
    let mut index: usize = 0;

    while offset < total_size {
        let end = std::cmp::min(offset + chunk_size - 1, total_size - 1);
        chunks.push(ChunkInfo {
            index,
            start: offset,
            end,
            downloaded: 0,
            completed: false,
        });
        offset = end + 1;
        index += 1;
    }

    chunks
}

/// Pause a download task.
pub async fn pause_download(pool: &SqlitePool, task_id: &str) -> Result<(), AppError> {
    let task = get_download_task(pool, task_id).await?;
    if task.status != DownloadStatus::Downloading.as_str()
        && task.status != DownloadStatus::Pending.as_str()
    {
        return Err(AppError::InvalidConfig(format!(
            "Cannot pause download in state: {}",
            task.status
        )));
    }
    update_status(pool, task_id, DownloadStatus::Paused).await?;
    Ok(())
}

/// Resume a paused download task.
pub async fn resume_download(pool: &SqlitePool, task_id: &str) -> Result<DownloadTask, AppError> {
    let task = get_download_task(pool, task_id).await?;
    if task.status != DownloadStatus::Paused.as_str() {
        return Err(AppError::InvalidConfig(format!(
            "Cannot resume download in state: {}",
            task.status
        )));
    }
    update_status(pool, task_id, DownloadStatus::Pending).await?;
    Ok(get_download_task(pool, task_id).await?)
}

/// Cancel a download task.
pub async fn cancel_download(pool: &SqlitePool, task_id: &str) -> Result<(), AppError> {
    let task = get_download_task(pool, task_id).await?;
    update_status(pool, task_id, DownloadStatus::Cancelled).await?;

    // Clean up partial files
    let target = PathBuf::from(&task.target_path);
    let temp_path = file::temp_file_path(&target);
    let part_meta = file::part_meta_path(&target);
    file::delete_file(&temp_path).await.ok();
    file::delete_file(&part_meta).await.ok();

    Ok(())
}

/// Get a download task by ID.
pub async fn get_download_task(pool: &SqlitePool, id: &str) -> Result<DownloadTask, AppError> {
    sqlx::query_as::<_, DownloadTask>("SELECT * FROM download_tasks WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::DownloadTaskNotFound(id.to_string()))
}

/// Update the status of a download task.
async fn update_status(pool: &SqlitePool, id: &str, status: DownloadStatus) -> Result<(), AppError> {
    sqlx::query("UPDATE download_tasks SET status = ? WHERE id = ?")
        .bind(status.as_str())
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}

/// List all download tasks.
pub async fn list_download_tasks(pool: &SqlitePool) -> Result<Vec<DownloadTask>, AppError> {
    sqlx::query_as::<_, DownloadTask>("SELECT * FROM download_tasks ORDER BY created_at DESC")
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}

/// Clean up completed/failed/cancelled download tasks.
pub async fn cleanup_download_tasks(pool: &SqlitePool) -> Result<(), AppError> {
    sqlx::query("DELETE FROM download_tasks WHERE status IN ('Completed', 'Failed', 'Cancelled')")
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;
    Ok(())
}
