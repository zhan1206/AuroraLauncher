//! Minecraft version installation service.
//!
//! Orchestrates the full version download pipeline:
//! client JAR, libraries, asset index, and asset objects.

use crate::error::AppError;
use crate::models::download::DownloadTask;
use crate::models::settings::DownloadMirror;
use crate::models::version::{Library, VersionDetail, VersionEntry};
use crate::services::download_service;
use crate::services::version_service::VersionService;
use crate::utils::{file, http};
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::AppHandle;
use tauri::Emitter;

/// Represents one file that needs to be downloaded as part of a version install.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallFile {
    pub name: String,
    pub url: String,
    pub path: String,
    pub size: u64,
    pub sha1: Option<String>,
    pub required: bool,
}

/// Tracks the progress of a version installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallProgress {
    pub version_id: String,
    pub total_files: usize,
    pub completed_files: usize,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub current_file: String,
    pub stage: String,
}

/// Install a Minecraft version into the given directory.
///
/// Steps:
/// 1. Fetch version detail JSON
/// 2. Download client JAR
/// 3. Download all matching library files
/// 4. Download asset index
/// 5. Download all asset objects
pub async fn install_version(
    pool: &SqlitePool,
    http_client: &reqwest::Client,
    app_handle: &AppHandle,
    version_id: &str,
    target_dir: &PathBuf,
    mirror: &DownloadMirror,
) -> Result<(), AppError> {
    tracing::info!("Installing Minecraft version: {}", version_id);

    // Step 1: Get version manifest and find the version entry
    emit_progress(app_handle, version_id, 0, 0, 0, 0, "", "fetching_manifest");

    let version_service = VersionService::new(http_client.clone());
    let entry = version_service
        .find_version(version_id, mirror)
        .await?
        .ok_or_else(|| AppError::InvalidConfig(format!("Version not found: {}", version_id)))?;

    // Step 2: Fetch version detail
    emit_progress(app_handle, version_id, 0, 0, 0, 0, &entry.id, "fetching_detail");

    let detail = version_service
        .get_version_detail(&entry.url, mirror)
        .await?;

    // Save version detail JSON so the launcher can read it later
    let version_json_path = target_dir
        .join("versions")
        .join(&detail.id)
        .join(format!("{}.json", detail.id));
    let detail_json = serde_json::to_string_pretty(&detail)
        .map_err(|e| AppError::Serialization(e.to_string()))?;
    file::write_file_with_dirs(&version_json_path, detail_json.as_bytes()).await?;
    tracing::info!("Saved version JSON to {}", version_json_path.display());

    // Step 3: Build the list of files to download
    let files = build_install_file_list(&detail, target_dir, mirror);

    let total_bytes: u64 = files.iter().map(|f| f.size).sum();
    let total_files = files.len();

    tracing::info!(
        "Version {} requires {} files ({:.1} MB)",
        version_id,
        total_files,
        total_bytes as f64 / 1_048_576.0
    );

    let mut downloaded_bytes: u64 = 0;
    let mut completed = 0usize;

    // Step 4: Download each file
    for file in &files {
        if !file.required && file.size == 0 {
            completed += 1;
            continue;
        }

        emit_progress(
            app_handle,
            version_id,
            total_files,
            completed,
            total_bytes,
            downloaded_bytes,
            &file.name,
            "downloading_files",
        );

        match download_single_file(pool, http_client, app_handle, file, mirror).await {
            Ok(_) => {
                downloaded_bytes += file.size;
                completed += 1;
            }
            Err(e) => {
                tracing::error!("Failed to download {}: {}", file.name, e);
                if file.required {
                    return Err(e);
                }
                // Skip optional files that fail
                completed += 1;
                tracing::warn!("Skipping optional file: {}", file.name);
            }
        }
    }

    emit_progress(
        app_handle,
        version_id,
        total_files,
        completed,
        total_bytes,
        downloaded_bytes,
        "",
        "completed",
    );

    tracing::info!(
        "Version {} installed successfully ({} files, {:.1} MB)",
        version_id,
        completed,
        downloaded_bytes as f64 / 1_048_576.0
    );

    Ok(())
}

/// Build the complete list of files needed to install a version.
fn build_install_file_list(
    detail: &VersionDetail,
    target_dir: &PathBuf,
    mirror: &DownloadMirror,
) -> Vec<InstallFile> {
    let mut files = Vec::new();

    // 1. Client JAR
    if let Some(ref client) = detail.downloads.client {
        let client_jar_path = target_dir
            .join("versions")
            .join(&detail.id)
            .join(format!("{}.jar", detail.id));
        files.push(InstallFile {
            name: format!("{}.jar", detail.id),
            url: http::replace_with_mirror(&client.url, mirror),
            path: client_jar_path.to_string_lossy().to_string(),
            size: client.size,
            sha1: Some(client.sha1.clone()),
            required: true,
        });
    }

    // 2. Save version detail JSON
    let version_json_path = target_dir
        .join("versions")
        .join(&detail.id)
        .join(format!("{}.json", detail.id));
    // We'll save this separately, not as a download

    // 3. Libraries - filter by OS rules
    let platform_info = crate::utils::platform::detect_platform();
    let os_name = match platform_info.os {
        crate::utils::platform::McPlatform::Windows => "windows",
        crate::utils::platform::McPlatform::Linux => "linux",
        crate::utils::platform::McPlatform::MacOS => "osx",
    };

    for lib in &detail.libraries {
        // Check rules to see if this library should be downloaded for the current OS
        if !library_applies_to_os(lib, os_name) {
            continue;
        }

        // Get the artifact download info
        if let Some(ref downloads) = lib.downloads {
            if let Some(ref artifact) = downloads.artifact {
                let lib_path = target_dir
                    .join("libraries")
                    .join(&artifact.path);
                files.push(InstallFile {
                    name: lib.name.clone(),
                    url: http::replace_with_mirror(&artifact.url, mirror),
                    path: lib_path.to_string_lossy().to_string(),
                    size: artifact.size,
                    sha1: Some(artifact.sha1.clone()),
                    required: true,
                });
            }

            // Native libraries for current OS
            if let Some(ref natives) = lib.natives {
                if let Some(os_classifier) = natives.get(os_name) {
                    if let Some(classifier_str) = os_classifier.as_str() {
                        if let Some(ref classifiers) = downloads.classifiers {
                            let native_name = classifier_str.replace(
                                "${arch}",
                                match platform_info.arch {
                                    crate::utils::platform::McArch::X86_64 => "64",
                                    crate::utils::platform::McArch::X86 => "32",
                                    crate::utils::platform::McArch::Arm64 => "arm64",
                                },
                            );
                            if let Some(native_artifact) = classifiers.get(&native_name) {
                                let native_url = native_artifact["url"].as_str().unwrap_or("");
                                let native_size = native_artifact["size"].as_u64().unwrap_or(0);
                                let native_sha1 = native_artifact["sha1"].as_str().map(|s| s.to_string());
                                let native_path = native_artifact["path"].as_str().unwrap_or("");

                                let native_lib_path = target_dir
                                    .join("libraries")
                                    .join(native_path);

                                files.push(InstallFile {
                                    name: format!("{}-native-{}", lib.name, os_name),
                                    url: http::replace_with_mirror(native_url, mirror),
                                    path: native_lib_path.to_string_lossy().to_string(),
                                    size: native_size,
                                    sha1: native_sha1,
                                    required: true,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // 4. Asset index
    let asset_index_path = target_dir
        .join("assets")
        .join("indexes")
        .join(format!("{}.json", detail.asset_index.id));
    files.push(InstallFile {
        name: format!("asset-index-{}", detail.asset_index.id),
        url: http::replace_with_mirror(&detail.asset_index.url, mirror),
        path: asset_index_path.to_string_lossy().to_string(),
        size: detail.asset_index.size,
        sha1: Some(detail.asset_index.sha1.clone()),
        required: true,
    });

    files
}

/// Check if a library should be downloaded for the current OS.
///
/// Library rules work as: if the first matching rule is "allow", download it.
/// If rules is None, the library applies to all platforms.
fn library_applies_to_os(lib: &Library, os_name: &str) -> bool {
    let rules = match &lib.rules {
        Some(r) => r,
        None => return true, // No rules = applies to all platforms
    };

    for rule in rules {
        let action = rule.get("action").and_then(|a| a.as_str()).unwrap_or("allow");
        let action_is_allow = action == "allow";

        if let Some(os_obj) = rule.get("os") {
            if let Some(rule_os_name) = os_obj.get("name").and_then(|n| n.as_str()) {
                if rule_os_name == os_name {
                    return action_is_allow;
                }
            }
        } else {
            // Rule without "os" key - applies to all platforms
            return action_is_allow;
        }
    }

    // If no rule matched, default to not downloading
    false
}

/// Download a single install file using the download service.
async fn download_single_file(
    pool: &SqlitePool,
    http_client: &reqwest::Client,
    app_handle: &AppHandle,
    file: &InstallFile,
    mirror: &DownloadMirror,
) -> Result<(), AppError> {
    // Skip if file already exists with correct size
    if let Ok(meta) = std::fs::metadata(&file.path) {
        if meta.len() == file.size && file.size > 0 {
            tracing::debug!("Skipping existing file: {}", file.name);
            return Ok(());
        }
    }

    // Create download task
    let task = download_service::create_download_task(
        pool,
        &file.name,
        &file.url,
        &file.path,
        file.size as i64,
        file.sha1.as_deref(),
        4, // concurrency
    )
    .await?;

    // Execute download
    download_service::start_download(pool, http_client, &task, app_handle).await
}

/// Emit an install progress event to the frontend.
fn emit_progress(
    app_handle: &AppHandle,
    version_id: &str,
    total_files: usize,
    completed_files: usize,
    total_bytes: u64,
    downloaded_bytes: u64,
    current_file: &str,
    stage: &str,
) {
    let progress = InstallProgress {
        version_id: version_id.to_string(),
        total_files,
        completed_files,
        total_bytes,
        downloaded_bytes,
        current_file: current_file.to_string(),
        stage: stage.to_string(),
    };
    let _ = app_handle.emit("install:progress", &progress);
}
