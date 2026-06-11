//! Java runtime service.
//!
//! Manages Java runtime detection, download, and version matching.
//! Uses Adoptium (Eclipse Temurin) as the JRE source.

use crate::error::AppError;
use crate::utils::file;
use crate::utils::platform;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Adoptium API base URL.
const ADOPTIUM_API: &str = "https://api.adoptium.net/v3";

/// A Java runtime installation managed by Aurora Launcher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaRuntime {
    /// Unique identifier for this Java installation.
    pub id: String,
    /// Major version (8, 17, 21).
    pub major_version: u32,
    /// Full version string (e.g. "17.0.9+9").
    pub version: String,
    /// Absolute path to the Java executable.
    pub path: String,
    /// Whether this is a managed (downloaded) or system Java.
    pub is_managed: bool,
}

/// Response from the Adoptium API for available JRE releases.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptiumRelease {
    /// Release metadata.
    pub release_name: String,
    /// Available binaries for this release.
    pub binaries: Vec<AdoptiumBinary>,
}

/// A binary package from Adoptium.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptiumBinary {
    /// Operating system.
    #[serde(rename = "os")]
    pub os_name: String,
    /// CPU architecture.
    pub architecture: String,
    /// Image type (jre or jdk).
    pub image_type: String,
    /// Download package information.
    pub package: AdoptiumPackage,
}

/// Download package information from Adoptium.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoptiumPackage {
    /// Download URL.
    pub link: String,
    /// Package name.
    pub name: String,
    /// File size in bytes.
    pub size: u64,
    /// SHA-256 checksum.
    pub checksum: String,
}

/// List available Java runtimes installed in the Aurora Launcher data directory.
pub async fn list_java_runtimes() -> Result<Vec<JavaRuntime>, AppError> {
    let java_dir = file::java_dir();
    let mut runtimes = Vec::new();

    if !java_dir.exists() {
        return Ok(runtimes);
    }

    let mut entries = tokio::fs::read_dir(&java_dir)
        .await
        .map_err(|e| AppError::FileIo(e))?;

    while let Some(entry) = entries.next_entry().await.map_err(|e| AppError::FileIo(e))? {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = entry.file_name().to_string_lossy().to_string();
            // Parse the version from directory name (e.g. "java-17")
            if let Some(version_str) = dir_name.strip_prefix("java-") {
                if let Ok(major_version) = version_str.parse::<u32>() {
                    let java_exec = if cfg!(target_os = "windows") {
                        path.join("bin").join("javaw.exe")
                    } else {
                        path.join("bin").join("java")
                    };

                    if java_exec.exists() {
                        let version = resolve_java_version(&java_exec).await;
                        runtimes.push(JavaRuntime {
                            id: format!("java-{}", major_version),
                            major_version,
                            version: version.unwrap_or_else(|| format!("{}.0.0", major_version)),
                            path: java_exec.to_string_lossy().to_string(),
                            is_managed: true,
                        });
                    }
                }
            }
        }
    }

    // Also detect system Java installations
    let system_javas = platform::find_system_java();
    for java_path in system_javas {
        if let Some(version) = resolve_java_version(&java_path).await {
            let major = parse_major_version(&version);
            runtimes.push(JavaRuntime {
                id: format!("system-{}", major),
                major_version: major,
                version,
                path: java_path.to_string_lossy().to_string(),
                is_managed: false,
            });
        }
    }

    Ok(runtimes)
}

/// Download a JRE from Adoptium.
///
/// Downloads the JRE for the given major version and installs it to
/// `{data_dir}/java/java-{version}/`.
pub async fn download_java(
    http_client: &Client,
    major_version: u32,
) -> Result<JavaRuntime, AppError> {
    let platform_id = platform::adoptium_platform_id();
    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86_64") {
        "x64"
    } else {
        "x64"
    };

    let os_name = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else {
        "linux"
    };

    let url = format!(
        "{}/assets/latest/{}/hotspot?image_type=jre&os={}&architecture={}&vendor=eclipse",
        ADOPTIUM_API, major_version, os_name, arch
    );

    tracing::info!("Fetching Adoptium JRE info from: {}", url);

    let response = http_client
        .get(&url)
        .send()
        .await
        .map_err(|e| AppError::NetworkRequest(e.to_string()))?;

    if !response.status().is_success() {
        return Err(AppError::NetworkRequest(format!(
            "Failed to find JRE for Java {}: HTTP {}",
            major_version,
            response.status()
        )));
    }

    let releases: Vec<AdoptiumRelease> = response
        .json()
        .await
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    let release = releases
        .first()
        .ok_or_else(|| AppError::JavaNotFound(format!("No Adoptium release found for Java {}", major_version)))?;

    let binary = release
        .binaries
        .iter()
        .find(|b| {
            b.os_name == os_name
                && b.architecture == arch
                && b.image_type == "jre"
        })
        .or_else(|| release.binaries.first())
        .ok_or_else(|| AppError::JavaNotFound(format!("No binary found for Java {}", major_version)))?;

    let install_dir = platform::java_install_dir(major_version);
    file::ensure_dir(&install_dir).await?;

    // Download the archive
    let archive_url = &binary.package.link;
    let archive_name = &binary.package.name;

    tracing::info!("Downloading JRE: {} -> {}", archive_url, archive_name);

    // Download with timeout and better error reporting
    let archive_data = match http_client
        .get(archive_url)
        .send()
        .await
    {
        Ok(resp) => {
            if !resp.status().is_success() {
                return Err(AppError::NetworkRequest(format!(
                    "Failed to download JRE archive: HTTP {} from {}",
                    resp.status(), archive_url
                )));
            }
            resp.bytes()
                .await
                .map_err(|e| AppError::NetworkRequest(format!("Failed to read JRE archive bytes: {}", e)))?
        }
        Err(e) => {
            return Err(AppError::NetworkRequest(format!(
                "Failed to download JRE from {}: {}", archive_url, e
            )));
        }
    };

    tracing::info!("JRE archive downloaded: {} bytes", archive_data.len());

    // Verify checksum (non-fatal warning)
    let actual_hash = crate::utils::crypto::sha256_bytes(&archive_data);
    if !actual_hash.eq_ignore_ascii_case(&binary.package.checksum) {
        tracing::warn!(
            "JRE checksum mismatch (expected={}, actual={}), continuing anyway",
            binary.package.checksum, actual_hash
        );
    } else {
        tracing::info!("JRE checksum verified successfully");
    }

    // Extract the archive
    let temp_archive = install_dir.join(archive_name);
    file::write_file_with_dirs(&temp_archive, &archive_data).await?;

    extract_archive(&temp_archive, &install_dir).await?;

    // Clean up the archive
    file::delete_file(&temp_archive).await.ok();

    let java_exec = if cfg!(target_os = "windows") {
        find_java_exec(&install_dir).unwrap_or_else(|| install_dir.join("bin").join("javaw.exe"))
    } else {
        find_java_exec(&install_dir).unwrap_or_else(|| install_dir.join("bin").join("java"))
    };

    let version = resolve_java_version(&java_exec)
        .await
        .unwrap_or_else(|| format!("{}.0.0", major_version));

    Ok(JavaRuntime {
        id: format!("java-{}", major_version),
        major_version,
        version,
        path: java_exec.to_string_lossy().to_string(),
        is_managed: true,
    })
}

/// Resolve the best Java runtime for a given Minecraft version.
///
/// Rules:
/// - MC 1.20.5+ → Java 21
/// - MC 1.17+ → Java 17
/// - All others → Java 8
///
/// If no Java is found on the system, automatically downloads a JRE from Adoptium.
pub async fn resolve_java(version_id: &str) -> Result<JavaRuntime, AppError> {
    let required = platform::required_java_version(version_id);
    let runtimes = list_java_runtimes().await?;

    // Try to find an exact match
    if let Some(rt) = runtimes.iter().find(|r| r.major_version == required) {
        return Ok(rt.clone());
    }

    // Try to find a compatible runtime (higher version is OK)
    if let Some(rt) = runtimes.iter().filter(|r| r.major_version >= required).min_by_key(|r| r.major_version) {
        return Ok(rt.clone());
    }

    // Last resort: use any available Java (any version beats no version)
    if let Some(rt) = runtimes.first() {
        tracing::warn!(
            "No Java {} found for Minecraft {}; falling back to Java {} at {}",
            required, version_id, rt.major_version, rt.path
        );
        return Ok(rt.clone());
    }

    // No Java found at all — auto-download from Adoptium
    // Try required version first, then fall back to 21, then 17
    let fallback_versions = if required == 21 {
        vec![21, 17]
    } else if required == 17 {
        vec![17, 21]
    } else {
        vec![required, 21, 17]
    };

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .user_agent("AuroraLauncher/1.0")
        .build()
        .map_err(|e| AppError::NetworkRequest(format!("Failed to create HTTP client: {}", e)))?;

    for &version in &fallback_versions {
        tracing::info!("Auto-downloading Java {} from Adoptium for Minecraft {}", version, version_id);
        match download_java(&client, version).await {
            Ok(rt) => return Ok(rt),
            Err(e) => {
                tracing::warn!("Failed to download Java {}: {}; trying next version...", version, e);
            }
        }
    }

    Err(AppError::JavaNotFound(format!(
        "No Java runtime found and auto-download failed. Please install Java {} or later manually.",
        required
    )))
}

/// Resolve the Java version string by running `java -version`.
async fn resolve_java_version(java_path: &Path) -> Option<String> {
    let output = tokio::process::Command::new(java_path)
        .arg("-version")
        .output()
        .await
        .ok()?;

    // Java prints version to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    // Parse version from output like: openjdk version "17.0.9" 2024-04-16
    for line in stderr.lines() {
        if line.contains("version") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    return Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    None
}

/// Parse the major version from a Java version string.
fn parse_major_version(version: &str) -> u32 {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.is_empty() {
        return 8;
    }
    // Java 9+ uses single number (e.g. "17.0.9")
    // Java 8 uses "1.8.x"
    if parts[0] == "1" && parts.len() > 1 {
        parts[1].parse().unwrap_or(8)
    } else {
        parts[0].parse().unwrap_or(8)
    }
}

/// Find the Java executable within an extracted Adoptium directory.
fn find_java_exec(install_dir: &Path) -> Option<PathBuf> {
    // Adoptium archives typically extract to a subdirectory like
    // jdk-17.0.9+9-jre/ or temurin-17.0.9+9/
    if let Ok(entries) = std::fs::read_dir(install_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let exec_name = if cfg!(target_os = "windows") {
                    "javaw.exe"
                } else {
                    "java"
                };
                let java_bin = path.join("bin").join(exec_name);
                if java_bin.exists() {
                    return Some(java_bin);
                }
                // Check nested directories
                if let Ok(nested) = std::fs::read_dir(&path) {
                    for nested_entry in nested.flatten() {
                        let nested_path = nested_entry.path();
                        if nested_path.is_dir() {
                            let nested_bin = nested_path.join("bin").join(exec_name);
                            if nested_bin.exists() {
                                return Some(nested_bin);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract a downloaded archive (zip, tar.gz) to the install directory.
async fn extract_archive(archive_path: &Path, install_dir: &Path) -> Result<(), AppError> {
    let extension = archive_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("");

    match extension {
        "zip" => {
            let file = std::fs::File::open(archive_path)
                .map_err(|e| AppError::FileIo(e))?;
            let mut archive = zip::ZipArchive::new(file)
                .map_err(|e| AppError::DecompressionFailed(e.to_string()))?;

            for i in 0..archive.len() {
                let mut entry = archive
                    .by_index(i)
                    .map_err(|e| AppError::DecompressionFailed(e.to_string()))?;
                    let out_path = match entry.enclosed_name() {
                        Some(path) => install_dir.join(path),
                        None => continue,
                    };

                if entry.is_dir() {
                    std::fs::create_dir_all(&out_path)
                        .map_err(|e| AppError::DirectoryCreateFailed(e.to_string()))?;
                } else {
                    if let Some(parent) = out_path.parent() {
                        std::fs::create_dir_all(parent)
                            .map_err(|e| AppError::DirectoryCreateFailed(e.to_string()))?;
                    }
                    let mut outfile = std::fs::File::create(&out_path)
                        .map_err(|e| AppError::FileIo(e))?;
                    std::io::copy(&mut entry, &mut outfile)
                        .map_err(|e| AppError::FileIo(e))?;
                }
            }
        }
        "gz" => {
            // .tar.gz — decompress the gzip layer then extract tar
            let file = std::fs::File::open(archive_path)
                .map_err(|e| AppError::FileIo(e))?;
            let gz_decoder = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(gz_decoder);
            archive.unpack(install_dir)
                .map_err(|e| AppError::DecompressionFailed(e.to_string()))?;
        }
        _ => {
            return Err(AppError::DecompressionFailed(format!(
                "Unsupported archive format: {}",
                extension
            )));
        }
    }

    Ok(())
}
