//! Global application state.
//!
//! Shared state injected into all Tauri commands. Contains the HTTP client,
//! SQLite connection pool, download manager, version cache, and the Tauri app handle.

use reqwest::Client;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::Instant;
use tauri::AppHandle;
use tokio::sync::{Mutex, OnceCell, RwLock};

use crate::models::version::VersionManifest;
use crate::services::launch_service::GameProcess;

/// Cached version manifest with timestamp for TTL checking.
#[derive(Debug, Clone)]
pub struct CachedManifest {
    pub manifest: VersionManifest,
    pub fetched_at: Instant,
}

impl CachedManifest {
    /// Cache duration for the version manifest (30 minutes).
    const TTL: std::time::Duration = std::time::Duration::from_secs(30 * 60);

    pub fn is_expired(&self) -> bool {
        self.fetched_at.elapsed() > Self::TTL
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
    /// Whether the database has been initialized.
    pub db_initialized: Arc<RwLock<bool>>,
    /// Game process state, protected by a tokio async mutex.
    pub game_process: Arc<Mutex<GameProcess>>,
    /// Tauri app handle for emitting events.
    pub app_handle: AppHandle,
    /// Cached version manifest (30-min TTL), shared across all version commands.
    pub version_cache: Arc<RwLock<Option<CachedManifest>>>,
}

impl AppState {
    /// Create a new AppState with the given HTTP client and app handle.
    /// The database pool will be attached later during async init.
    pub fn new(http_client: Client, app_handle: AppHandle) -> Self {
        Self {
            http_client,
            db_pool: OnceCell::new(),
            db_initialized: Arc::new(RwLock::new(false)),
            game_process: Arc::new(Mutex::new(GameProcess::new())),
            app_handle,
            version_cache: Arc::new(RwLock::new(None)),
        }
    }
}
