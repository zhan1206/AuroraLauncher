//! HTTP utility functions.
//!
//! Provides a configured reqwest client builder with consistent User-Agent,
//! retry logic, and mirror URL replacement for Chinese users.

use crate::error::AppError;
use crate::models::settings::DownloadMirror;
use reqwest::Client;
use std::time::Duration;

/// The User-Agent string sent with all HTTP requests.
const USER_AGENT: &str = "AuroraLauncher/0.1.0";

/// Maximum number of retry attempts for failed requests.
const MAX_RETRIES: u32 = 3;

/// Timeout for HTTP connections in seconds.
const CONNECT_TIMEOUT_SECS: u64 = 15;

/// Timeout for reading response bodies in seconds.
const READ_TIMEOUT_SECS: u64 = 120;

/// Build a configured reqwest HTTP client.
///
/// The client uses:
/// - A custom User-Agent header
/// - Connection and read timeouts
/// - rustls for TLS (no native TLS dependency)
/// - HTTP/2 support
pub fn build_http_client() -> Result<Client, AppError> {
    Client::builder()
        .user_agent(USER_AGENT)
        .connect_timeout(Duration::from_secs(CONNECT_TIMEOUT_SECS))
        .read_timeout(Duration::from_secs(READ_TIMEOUT_SECS))
        .use_rustls_tls()
        .build()
        .map_err(|e| AppError::NetworkRequest(format!("Failed to build HTTP client: {}", e)))
}

/// Replace Mojang official URLs with BMCLAPI mirror equivalents.
///
/// This is used when the user selects the BMCLAPI mirror for faster
/// downloads in mainland China.
pub fn replace_with_mirror(url: &str, mirror: &DownloadMirror) -> String {
    match mirror {
        DownloadMirror::Official => url.to_string(),
        DownloadMirror::Bmclapi => {
            // Version manifest
            if url.starts_with("https://piston-meta.mojang.com/mc/game/") {
                return url.replace(
                    "https://piston-meta.mojang.com/mc/game/",
                    "https://bmclapi2.bangbang93.com/mc/game/",
                );
            }
            // Launchermeta (version details)
            if url.starts_with("https://launchermeta.mojang.com/") {
                return url.replace(
                    "https://launchermeta.mojang.com/",
                    "https://bmclapi2.bangbang93.com/",
                );
            }
            // Minecraft resources (assets, libraries)
            if url.starts_with("https://resources.download.minecraft.net/") {
                return url.replace(
                    "https://resources.download.minecraft.net/",
                    "https://bmclapi2.bangbang93.com/resources/",
                );
            }
            // Libraries on Maven central / Minecraft libraries
            if url.starts_with("https://libraries.minecraft.net/") {
                return url.replace(
                    "https://libraries.minecraft.net/",
                    "https://bmclapi2.bangbang93.com/maven/",
                );
            }
            // Adoptium JRE downloads
            if url.starts_with("https://api.adoptium.net/") {
                return url.to_string(); // No BMCLAPI mirror for Adoptium
            }
            url.to_string()
        }
    }
}

/// Execute an HTTP request with automatic retries on transient failures.
///
/// Retries on connection errors and 5xx server errors up to `MAX_RETRIES` times
/// with exponential backoff.
pub async fn retry_request(
    client: &Client,
    request_builder: reqwest::RequestBuilder,
) -> Result<reqwest::Response, AppError> {
    let mut last_error = None;

    for attempt in 0..=MAX_RETRIES {
        let request = request_builder
            .try_clone()
            .ok_or_else(|| AppError::NetworkRequest("Cannot clone request for retry".to_string()))?
            .build()
            .map_err(|e| AppError::NetworkRequest(format!("Failed to build request: {}", e)))?;

        match client.execute(request).await {
            Ok(response) => {
                let status = response.status();
                if status.is_server_error() && attempt < MAX_RETRIES {
                    let delay = Duration::from_millis(500 * 2u64.pow(attempt));
                    tracing::warn!(
                        "Server error {} on attempt {}/{}, retrying in {:?}",
                        status,
                        attempt + 1,
                        MAX_RETRIES + 1,
                        delay
                    );
                    tokio::time::sleep(delay).await;
                    last_error = Some(AppError::NetworkRequest(format!(
                        "Server error: HTTP {}",
                        status
                    )));
                    continue;
                }
                return Ok(response);
            }
            Err(e) => {
                if attempt < MAX_RETRIES {
                    let delay = Duration::from_millis(500 * 2u64.pow(attempt));
                    tracing::warn!(
                        "Request error on attempt {}/{}: {}, retrying in {:?}",
                        attempt + 1,
                        MAX_RETRIES + 1,
                        e,
                        delay
                    );
                    tokio::time::sleep(delay).await;
                    last_error = Some(AppError::NetworkRequest(e.to_string()));
                    continue;
                }
                return Err(AppError::NetworkRequest(e.to_string()));
            }
        }
    }

    Err(last_error.unwrap_or_else(|| AppError::NetworkRequest("All retry attempts failed".to_string())))
}

/// Resolve the version manifest URL based on the selected mirror.
pub fn version_manifest_url(mirror: &DownloadMirror) -> String {
    replace_with_mirror(
        "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json",
        mirror,
    )
}

/// Fetch the version manifest with automatic mirror fallback.
///
/// First tries the official Mojang URL. If that fails, falls back to
/// BMCLAPI (commonly used in mainland China for faster access).
pub async fn fetch_version_manifest(
    client: &Client,
    mirror: &DownloadMirror,
) -> Result<reqwest::Response, AppError> {
    let primary_url = version_manifest_url(mirror);

    match retry_request(
        client,
        client.get(&primary_url)
    ).await {
        Ok(response) => Ok(response),
        Err(e) => {
            // If already using BMCLAPI, just propagate the error
            if matches!(mirror, DownloadMirror::Bmclapi) {
                return Err(e);
            }

            tracing::warn!(
                "Failed to fetch from official mirror: {}. Trying BMCLAPI fallback...",
                e
            );

            let fallback_url = version_manifest_url(&DownloadMirror::Bmclapi);
            retry_request(
                client,
                client.get(&fallback_url),
            ).await
        }
    }
}
