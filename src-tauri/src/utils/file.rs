//! File utility functions.
//!
//! Helpers for common file and directory operations used throughout the launcher.

use crate::error::AppError;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Ensure a directory exists, creating it (and parents) if necessary.
pub async fn ensure_dir(path: &Path) -> Result<(), AppError> {
    if !path.exists() {
        tokio::fs::create_dir_all(path)
            .await
            .map_err(|e| AppError::DirectoryCreateFailed(e.to_string()))?;
    }
    Ok(())
}

/// Get the Aurora Launcher data directory for the current platform.
///
/// - Windows: `%APPDATA%/Aurora Launcher/`
/// - macOS: `~/Library/Application Support/Aurora Launcher/`
/// - Linux: `~/.local/share/Aurora Launcher/`
pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Aurora Launcher")
}

/// Get the instances root directory.
pub fn instances_dir() -> PathBuf {
    data_dir().join("instances")
}

/// Get the Java runtimes root directory.
pub fn java_dir() -> PathBuf {
    data_dir().join("java")
}

/// Get the game directory for a specific instance.
///
/// The game directory follows the pattern:
/// `{data_dir}/instances/{instance_id}/.minecraft/`
pub fn instance_game_dir(instance_id: &str) -> PathBuf {
    instances_dir()
        .join(instance_id)
        .join(".minecraft")
}

/// Get the metadata file path for a specific instance.
///
/// The metadata file is located at:
/// `{data_dir}/instances/{instance_id}/instance.json`
pub fn instance_metadata_path(instance_id: &str) -> PathBuf {
    instances_dir().join(instance_id).join("instance.json")
}

/// Write data to a file, creating parent directories as needed.
pub async fn write_file_with_dirs(path: &Path, data: &[u8]) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent).await?;
    }

    let mut file = tokio::fs::File::create(path)
        .await
        .map_err(|e| AppError::FileIo(e))?;

    file.write_all(data).await.map_err(|e| AppError::FileIo(e))?;

    Ok(())
}

/// Read a file to bytes.
pub async fn read_file(path: &Path) -> Result<Vec<u8>, AppError> {
    tokio::fs::read(path)
        .await
        .map_err(|e| AppError::FileIo(e))
}

/// Delete a file if it exists.
pub async fn delete_file(path: &Path) -> Result<(), AppError> {
    if path.exists() {
        tokio::fs::remove_file(path)
            .await
            .map_err(|e| AppError::FileIo(e))?;
    }
    Ok(())
}

/// Delete a directory and all its contents if it exists.
pub async fn delete_dir(path: &Path) -> Result<(), AppError> {
    if path.exists() {
        tokio::fs::remove_dir_all(path)
            .await
            .map_err(|e| AppError::FileIo(e))?;
    }
    Ok(())
}

/// Check if a file exists.
pub async fn file_exists(path: &Path) -> bool {
    tokio::fs::metadata(path).await.is_ok()
}

/// Get the file size in bytes, or 0 if the file doesn't exist.
pub async fn file_size(path: &Path) -> u64 {
    match tokio::fs::metadata(path).await {
        Ok(meta) => meta.len(),
        Err(_) => 0,
    }
}

/// Move a file from source to destination, creating parent directories.
pub async fn move_file(src: &Path, dest: &Path) -> Result<(), AppError> {
    if let Some(parent) = dest.parent() {
        ensure_dir(parent).await?;
    }
    tokio::fs::rename(src, dest)
        .await
        .map_err(|e| AppError::FileIo(e))
}

/// Create a temporary file path next to the target (used for partial downloads).
pub fn temp_file_path(target: &Path) -> PathBuf {
    let file_name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "download".to_string());
    target
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{}.part", file_name))
}

/// Create a part metadata file path for a download.
pub fn part_meta_path(target: &Path) -> PathBuf {
    let file_name = target
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "download".to_string());
    target
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(format!("{}.part.json", file_name))
}
