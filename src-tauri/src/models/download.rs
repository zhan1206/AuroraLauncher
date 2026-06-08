//! Download task model.
//!
//! Tracks the state and progress of file downloads used for game assets,
//! version JARs, libraries, and Java runtimes.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// The current status of a download task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

impl Default for DownloadStatus {
    fn default() -> Self {
        DownloadStatus::Pending
    }
}

impl DownloadStatus {
    /// Convert to the string representation stored in the database.
    pub fn as_str(&self) -> &str {
        match self {
            DownloadStatus::Pending => "Pending",
            DownloadStatus::Downloading => "Downloading",
            DownloadStatus::Paused => "Paused",
            DownloadStatus::Completed => "Completed",
            DownloadStatus::Failed => "Failed",
            DownloadStatus::Cancelled => "Cancelled",
        }
    }

    /// Parse from the database string representation.
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "Pending" => DownloadStatus::Pending,
            "Downloading" => DownloadStatus::Downloading,
            "Paused" => DownloadStatus::Paused,
            "Completed" => DownloadStatus::Completed,
            "Failed" => DownloadStatus::Failed,
            "Cancelled" => DownloadStatus::Cancelled,
            _ => DownloadStatus::Pending,
        }
    }
}

/// A download task record stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DownloadTask {
    /// Unique identifier (UUID v4).
    pub id: String,
    /// Human-readable name for the download.
    pub name: String,
    /// Source URL.
    pub url: String,
    /// Local target file path.
    pub target_path: String,
    /// Total file size in bytes (0 if unknown).
    pub total_size: i64,
    /// Bytes downloaded so far.
    pub downloaded: i64,
    /// Current status.
    pub status: String,
    /// Expected SHA-256 hash for verification.
    pub sha256: Option<String>,
    /// Number of concurrent download chunks.
    pub concurrency: i64,
    /// ISO-8601 timestamp of creation.
    pub created_at: String,
}

/// Real-time download progress payload emitted via Tauri events.
///
/// Sent through `app.emit("download:progress", payload)` to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    /// The download task ID.
    pub task_id: String,
    /// Total bytes to download.
    pub total: u64,
    /// Bytes downloaded so far.
    pub downloaded: u64,
    /// Current download speed in bytes/second.
    pub speed: u64,
    /// Download progress as a percentage (0-100).
    pub percent: f64,
}

/// Metadata for a single chunk in a chunked download.
///
/// Stored in a `.part` file alongside the target for resume support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    /// Zero-based chunk index.
    pub index: usize,
    /// Start byte offset (inclusive).
    pub start: u64,
    /// End byte offset (inclusive).
    pub end: u64,
    /// Bytes downloaded for this chunk.
    pub downloaded: u64,
    /// Whether this chunk is complete.
    pub completed: bool,
}

/// The metadata file stored alongside a partial download.
///
/// This file enables resume support by tracking which chunks have been
/// downloaded and their byte ranges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartFile {
    /// Source URL.
    pub url: String,
    /// Target file path.
    pub target_path: String,
    /// Total file size in bytes.
    pub total_size: u64,
    /// Expected SHA-256 hash.
    pub sha256: Option<String>,
    /// Chunk size in bytes.
    pub chunk_size: u64,
    /// Status of each chunk.
    pub chunks: Vec<ChunkInfo>,
}
