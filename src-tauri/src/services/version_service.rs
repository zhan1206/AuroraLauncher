//! Version manifest service.
//!
//! Fetches the Minecraft version manifest from Mojang's Piston Meta API
//! (or the BMCLAPI mirror), caches it locally for 30 minutes, and provides
//! detailed version information parsing.

use crate::error::AppError;
use crate::models::version::{VersionDetail, VersionEntry, VersionManifest};
use crate::models::settings::DownloadMirror;
use crate::utils::http;
use crate::state::AppState;
use reqwest::Client;
use sqlx::SqlitePool;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Cache duration for the version manifest (30 minutes).
const CACHE_DURATION: Duration = Duration::from_secs(30 * 60);

/// Cached version manifest with its fetch timestamp.
struct CachedManifest {
    /// The cached manifest data.
    manifest: VersionManifest,
    /// When this manifest was fetched.
    fetched_at: Instant,
}

impl CachedManifest {
    /// Check whether the cached manifest has expired.
    fn is_expired(&self) -> bool {
        self.fetched_at.elapsed() > CACHE_DURATION
    }
}

/// Version service for fetching and caching Minecraft version data.
pub struct VersionService {
    /// Shared HTTP client.
    http_client: Client,
    /// Cached manifest, protected by an async RwLock.
    cache: Arc<RwLock<Option<CachedManifest>>>,
}

impl VersionService {
    /// Create a new version service with the given HTTP client.
    pub fn new(http_client: Client) -> Self {
        Self {
            http_client,
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the version manifest, using the cache if it's still fresh.
    ///
    /// If the cache has expired or doesn't exist, fetches a fresh manifest
    /// from the Mojang API (or BMCLAPI mirror).
    pub async fn get_manifest(&self, mirror: &DownloadMirror) -> Result<VersionManifest, AppError> {
        // Check cache first
        {
            let cache_read = self.cache.read().await;
            if let Some(cached) = cache_read.as_ref() {
                if !cached.is_expired() {
                    tracing::debug!("Using cached version manifest");
                    return Ok(cached.manifest.clone());
                }
            }
        }

        // Cache miss or expired — fetch from API
        tracing::info!("Fetching version manifest from API");
        let url = http::version_manifest_url(mirror);

        let response = http::retry_request(&self.http_client, self.http_client.get(&url)).await?;

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
        let cached = CachedManifest {
            manifest: manifest.clone(),
            fetched_at: Instant::now(),
        };
        {
            let mut cache_write = self.cache.write().await;
            *cache_write = Some(cached);
        }

        Ok(manifest)
    }

    /// Get the list of version entries, optionally filtered by type.
    pub async fn list_versions(
        &self,
        version_type: Option<&str>,
        mirror: &DownloadMirror,
    ) -> Result<Vec<VersionEntry>, AppError> {
        let manifest = self.get_manifest(mirror).await?;

        let versions = match version_type {
            Some(vt) => manifest
                .versions
                .into_iter()
                .filter(|v| v.version_type == vt)
                .collect(),
            None => manifest.versions,
        };

        Ok(versions)
    }

    /// Fetch the detailed version JSON for a specific version.
    ///
    /// The `version_url` comes from a [`VersionEntry`]'s `url` field.
    pub async fn get_version_detail(
        &self,
        version_url: &str,
        mirror: &DownloadMirror,
    ) -> Result<VersionDetail, AppError> {
        let url = http::replace_with_mirror(version_url, mirror);

        tracing::info!("Fetching version detail from: {}", url);

        let response = http::retry_request(&self.http_client, self.http_client.get(&url)).await?;

        if !response.status().is_success() {
            return Err(AppError::NetworkRequest(format!(
                "Failed to fetch version detail: HTTP {}",
                response.status()
            )));
        }

        let detail: VersionDetail = response
            .json()
            .await
            .map_err(|e| AppError::Serialization(e.to_string()))?;

        Ok(detail)
    }

    /// Find a specific version entry by ID.
    pub async fn find_version(
        &self,
        version_id: &str,
        mirror: &DownloadMirror,
    ) -> Result<Option<VersionEntry>, AppError> {
        let manifest = self.get_manifest(mirror).await?;
        Ok(manifest
            .versions
            .into_iter()
            .find(|v| v.id == version_id))
    }

    /// Invalidate the cached manifest, forcing a fresh fetch on next request.
    pub async fn invalidate_cache(&self) {
        let mut cache = self.cache.write().await;
        *cache = None;
        tracing::info!("Version manifest cache invalidated");
    }
}
