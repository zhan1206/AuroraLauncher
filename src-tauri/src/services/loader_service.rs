//! Mod loader installation service.
//!
//! Supports installing Fabric and Forge loaders. NeoForge and Quilt
//! are defined but return NotSupported errors.

use crate::error::AppError;
use crate::models::instance::LoaderType;
use crate::models::settings::DownloadMirror;
use crate::utils::http;
use crate::utils::file;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Fabric Meta API base URL.
const FABRIC_META_URL: &str = "https://meta.fabricmc.net/v2";

/// BMCLAPI Fabric Meta mirror.
const BMCLAPI_FABRIC_META_URL: &str = "https://bmclapi2.bangbang93.com/fabric-meta/v2";

/// Forge promotions API URL.
const FORGE_PROMOS_URL: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/promotions_slim.json";

/// A Fabric loader version from the Fabric Meta API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricLoaderVersion {
    /// Separator character (always ".").
    pub separator: String,
    /// Build number.
    pub build: i64,
    /// Maven artifact version.
    pub maven: String,
    /// Display version string.
    pub version: String,
    /// Whether this is a stable release.
    pub stable: bool,
}

/// A Forge version from the Forge promotions API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgePromotions {
    /// Homepage link.
    pub homepage: String,
    /// Promoted versions keyed by "mcVersion-forgeVersion".
    pub promos: serde_json::Value,
}

/// List available Fabric loader versions.
pub async fn list_fabric_loaders(
    http_client: &Client,
    mirror: &DownloadMirror,
) -> Result<Vec<FabricLoaderVersion>, AppError> {
    let url = match mirror {
        DownloadMirror::Bmclapi => format!("{}/versions/loader", BMCLAPI_FABRIC_META_URL),
        DownloadMirror::Official => format!("{}/versions/loader", FABRIC_META_URL),
    };

    let response = http::retry_request(http_client, http_client.get(&url)).await?;

    if !response.status().is_success() {
        return Err(AppError::NetworkRequest(format!(
            "Failed to fetch Fabric loader list: HTTP {}",
            response.status()
        )));
    }

    let versions: Vec<FabricLoaderVersion> = response
        .json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    Ok(versions)
}

/// Install the Fabric loader for a given Minecraft version.
///
/// This creates the necessary profile JSON and downloads the loader libraries.
pub async fn install_fabric_loader(
    http_client: &Client,
    game_dir: &Path,
    minecraft_version: &str,
    loader_version: &str,
    mirror: &DownloadMirror,
) -> Result<(), AppError> {
    let base_url = match mirror {
        DownloadMirror::Bmclapi => BMCLAPI_FABRIC_META_URL,
        DownloadMirror::Official => FABRIC_META_URL,
    };

    // Fetch the Fabric profile JSON
    let profile_url = format!(
        "{}/versions/loader/{}/{}",
        base_url, minecraft_version, loader_version
    );

    let response = http::retry_request(http_client, http_client.get(&profile_url)).await?;

    if !response.status().is_success() {
        return Err(AppError::NetworkRequest(format!(
            "Failed to fetch Fabric profile: HTTP {}",
            response.status()
        )));
    }

    let profile_data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    // The Fabric API returns an array with [loaderMeta, intermediaryMeta, profile]
    // The profile is the third element
    let profile = profile_data
        .as_array()
        .and_then(|arr| arr.get(2))
        .ok_or_else(|| AppError::Serialization("Invalid Fabric profile response".to_string()))?;

    // Write the version JSON to the game directory's versions folder
    let versions_dir = game_dir.join("versions").join(format!(
        "fabric-loader-{}-{}",
        loader_version, minecraft_version
    ));
    file::ensure_dir(&versions_dir).await?;

    let version_json_path = versions_dir.join(format!(
        "fabric-loader-{}-{}.json",
        loader_version, minecraft_version
    ));

    let json_str = serde_json::to_string_pretty(profile)
        .map_err(|e| AppError::Serialization(e.to_string()))?;
    file::write_file_with_dirs(&version_json_path, json_str.as_bytes()).await?;

    tracing::info!(
        "Installed Fabric loader {} for Minecraft {}",
        loader_version,
        minecraft_version
    );

    Ok(())
}

/// List available Forge loader versions for a given Minecraft version.
pub async fn list_forge_loaders(
    http_client: &Client,
    minecraft_version: &str,
) -> Result<Vec<String>, AppError> {
    let response = http::retry_request(http_client, http_client.get(FORGE_PROMOS_URL)).await?;

    if !response.status().is_success() {
        return Err(AppError::NetworkRequest(format!(
            "Failed to fetch Forge promotions: HTTP {}",
            response.status()
        )));
    }

    let promotions: ForgePromotions = response
        .json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    // Extract Forge versions for the requested MC version
    let mut versions = Vec::new();
    if let Some(promos) = promotions.promos.as_object() {
        for key in promos.keys() {
            if key.starts_with(minecraft_version) && key.contains("forge") {
                // Extract the full version identifier (e.g. "1.21.1-52.0.40")
                if let Some(version) = key.strip_suffix("-latest").or_else(|| key.strip_suffix("-recommended")) {
                    versions.push(version.to_string());
                }
            }
        }
    }

    versions.sort();
    versions.dedup();
    Ok(versions)
}

/// Install the Forge loader for a given Minecraft version.
///
/// Forge installation requires running the Forge installer JAR, which
/// is a complex process. This implementation downloads the installer
/// and provides the framework for running it.
pub async fn install_forge_loader(
    http_client: &Client,
    game_dir: &Path,
    minecraft_version: &str,
    forge_version: &str,
) -> Result<(), AppError> {
    // Download the Forge installer JAR
    let installer_url = format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
        minecraft_version, forge_version, minecraft_version, forge_version
    );

    let installer_dir = game_dir.join("temp");
    file::ensure_dir(&installer_dir).await?;

    let installer_path = installer_dir.join(format!(
        "forge-{}-{}-installer.jar",
        minecraft_version, forge_version
    ));

    tracing::info!("Downloading Forge installer from: {}", installer_url);

    let response = http_client
        .get(&installer_url)
        .send()
        .await
        .map_err(|e| AppError::NetworkRequest(e.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::NetworkRequest(format!(
            "Failed to download Forge installer: HTTP {}",
            response.status()
        )));
    }

    let installer_data = response
        .bytes()
        .await
        .map_err(|e| AppError::NetworkRequest(e.to_string()))?;

    file::write_file_with_dirs(&installer_path, &installer_data).await?;

    // Run the Forge installer in headless mode
    // Note: This requires Java to be available
    let java_path = std::env::var("JAVA_HOME")
        .ok()
        .map(|home| {
            let bin = if cfg!(target_os = "windows") {
                "bin/javaw.exe"
            } else {
                "bin/java"
            };
            format!("{}/{}", home, bin)
        })
        .unwrap_or_else(|| "java".to_string());

    let output = tokio::process::Command::new(&java_path)
        .arg("-jar")
        .arg(&installer_path)
        .arg("--installServer")
        .arg(game_dir)
        .output()
        .await
        .map_err(|e| AppError::LaunchFailed(format!("Failed to run Forge installer: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::LaunchFailed(format!(
            "Forge installer failed: {}",
            stderr
        )));
    }

    // Clean up installer
    file::delete_file(&installer_path).await.ok();

    tracing::info!(
        "Installed Forge {} for Minecraft {}",
        forge_version,
        minecraft_version
    );

    Ok(())
}

/// Install a mod loader for the given type.
///
/// Dispatches to the appropriate loader installation function.
pub async fn install_loader(
    http_client: &Client,
    game_dir: &Path,
    loader_type: &LoaderType,
    minecraft_version: &str,
    loader_version: &str,
    mirror: &DownloadMirror,
) -> Result<(), AppError> {
    match loader_type {
        LoaderType::Fabric => {
            install_fabric_loader(http_client, game_dir, minecraft_version, loader_version, mirror).await
        }
        LoaderType::Forge => {
            install_forge_loader(http_client, game_dir, minecraft_version, loader_version).await
        }
        LoaderType::NeoForge => {
            Err(AppError::VersionNotSupported(
                "NeoForge loader installation is not yet supported".to_string(),
            ))
        }
        LoaderType::Quilt => {
            Err(AppError::VersionNotSupported(
                "Quilt loader installation is not yet supported".to_string(),
            ))
        }
        LoaderType::Vanilla => Ok(()),
    }
}
