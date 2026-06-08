//! Global application state.
//!
//! Shared state injected into all Tauri commands. Contains the HTTP client,
//! SQLite connection pool, download manager, and the Tauri app handle.

use reqwest::Client;
use sqlx::SqlitePool;
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::{Mutex, OnceCell, RwLock};

use crate::services::launch_service::GameProcess;

/// Tracks the progress of an active download.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadProgress {
    /// Unique identifier for this download task.
    pub id: String,
    /// Human-readable filename being downloaded.
    pub filename: String,
    /// Total bytes expected (0 if unknown / streaming).
    pub total_bytes: u64,
    /// Bytes downloaded so far.
    pub downloaded_bytes: u64,
    /// Whether the download has completed.
    pub completed: bool,
    /// Error message if the download failed.
    pub error: Option<String>,
}

/// Manages the state of all active and recent downloads.
#[derive(Debug, Default)]
pub struct DownloadManager {
    /// Active downloads keyed by their task ID.
    downloads: Vec<DownloadProgress>,
}

impl DownloadManager {
    /// Create a new empty download manager.
    pub fn new() -> Self {
        Self {
            downloads: Vec::new(),
        }
    }

    /// Add a new download tracking entry.
    pub fn add_download(&mut self, progress: DownloadProgress) {
        self.downloads.push(progress);
    }

    /// Update an existing download by ID.
    pub fn update_download(&mut self, id: &str, downloaded: u64, completed: bool) {
        if let Some(entry) = self.downloads.iter_mut().find(|d| d.id == id) {
            entry.downloaded_bytes = downloaded;
            entry.completed = completed;
        }
    }

    /// Mark a download as failed.
    pub fn fail_download(&mut self, id: &str, error: String) {
        if let Some(entry) = self.downloads.iter_mut().find(|d| d.id == id) {
            entry.completed = true;
            entry.error = Some(error);
        }
    }

    /// Remove completed downloads older than a threshold.
    pub fn cleanup(&mut self) {
        self.downloads.retain(|d| !d.completed);
    }

    /// Get all downloads.
    pub fn get_all(&self) -> &[DownloadProgress] {
        &self.downloads
    }
}

/// Global application state injected into all Tauri commands.
///
/// Uses `OnceCell` for the database pool (set once during startup),
/// and `Arc<Mutex<>>` / `Arc<RwLock<>>` for other mutable state.
pub struct AppState {
    /// Shared HTTP client (read-only after construction, cheaply cloneable).
    pub http_client: Client,
    /// SQLite database pool, set once during async initialization.
    pub db_pool: OnceCell<SqlitePool>,
    /// Download manager state, protected by a tokio async mutex.
    pub download_manager: Arc<Mutex<DownloadManager>>,
    /// Whether the database has been initialized.
    pub db_initialized: Arc<RwLock<bool>>,
    /// Game process state, protected by a tokio async mutex.
    pub game_process: Arc<Mutex<GameProcess>>,
    /// Tauri app handle for emitting events.
    pub app_handle: AppHandle,
}

impl AppState {
    /// Create a new AppState with the given HTTP client and app handle.
    /// The database pool will be attached later during async init.
    pub fn new(http_client: Client, app_handle: AppHandle) -> Self {
        Self {
            http_client,
            db_pool: OnceCell::new(),
            download_manager: Arc::new(Mutex::new(DownloadManager::new())),
            db_initialized: Arc::new(RwLock::new(false)),
            game_process: Arc::new(Mutex::new(GameProcess::new())),
            app_handle,
        }
    }
}
