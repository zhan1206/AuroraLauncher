//! Minecraft version manifest and detail models.
//!
//! These structs map to the official Mojang Piston Meta API responses:
//! - `https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`
//! - Individual version JSON files referenced by the manifest.

use serde::{Deserialize, Serialize};

/// The top-level version manifest returned by Mojang's piston-meta API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    /// Pointers to the latest release and snapshot versions.
    pub latest: LatestVersions,
    /// All known Minecraft versions.
    pub versions: Vec<VersionEntry>,
}

/// Latest version pointers from the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersions {
    /// The ID of the latest stable release (e.g. "1.21.4").
    pub release: String,
    /// The ID of the latest snapshot.
    pub snapshot: String,
}

/// A single version entry in the manifest list.
///
/// Each entry provides just enough metadata to display the version in a list
/// and to fetch its full detail JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    /// Version identifier (e.g. "1.21.4", "24w46a").
    pub id: String,
    /// Version type: "release", "snapshot", "old_beta", "old_alpha".
    #[serde(rename = "type")]
    pub version_type: String,
    /// Absolute URL to the version detail JSON.
    pub url: String,
    /// ISO-8601 timestamp of when this version was released.
    pub release_time: String,
    /// SHA-1 hash of the version detail JSON (for cache validation).
    #[serde(default)]
    pub sha1: Option<String>,
}

/// Full detail for a specific Minecraft version.
///
/// This is parsed from the individual version JSON file whose URL comes from
/// a [`VersionEntry`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDetail {
    /// Version identifier.
    pub id: String,
    /// Version type.
    #[serde(rename = "type")]
    pub version_type: String,
    /// The main class used to launch the game (e.g. "net.minecraft.client.main.Main").
    pub main_class: String,
    /// Libraries required by this version.
    pub libraries: Vec<Library>,
    /// Downloadable assets for this version (client, server, server_mappings, client_mappings).
    pub downloads: VersionDownloads,
    /// Asset index metadata.
    pub asset_index: AssetIndexRef,
    /// Arguments for launching the game (newer format, split into game and JVM args).
    #[serde(default)]
    pub arguments: Option<VersionArguments>,
    /// Legacy `minecraftArguments` field (used before 1.13).
    #[serde(default)]
    pub minecraft_arguments: Option<String>,
    /// Minimum Java version required.
    #[serde(default)]
    pub java_version: Option<JavaVersion>,
    /// ISO-8601 timestamp of when this version was released.
    pub release_time: String,
}

/// A library dependency for a Minecraft version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    /// Maven-style artifact name (e.g. "com.mojang:brigadier:1.0.18").
    pub name: String,
    /// Download information for this library.
    #[serde(default)]
    pub downloads: Option<LibraryDownloads>,
    /// Rules determining whether this library applies (OS, features, etc.).
    #[serde(default)]
    pub rules: Option<Vec<serde_json::Value>>,
    /// Native libraries for specific platforms.
    #[serde(default)]
    pub natives: Option<serde_json::Value>,
    /// Extract rules for native libraries.
    #[serde(default)]
    pub extract: Option<serde_json::Value>,
}

/// Download information for a library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownloads {
    /// The artifact (main JAR) for this library.
    pub artifact: Option<LibraryArtifact>,
    /// Native libraries keyed by platform classifier.
    #[serde(default)]
    pub classifiers: Option<serde_json::Value>,
}

/// A single downloadable artifact (JAR or native).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryArtifact {
    /// Download URL.
    pub url: String,
    /// SHA-1 hash.
    pub sha1: String,
    /// File size in bytes.
    pub size: u64,
    /// Relative path within the libraries directory.
    pub path: String,
}

/// Downloadable files for a version (client JAR, server JAR, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDownloads {
    /// Client download.
    pub client: Option<DownloadInfo>,
    /// Server download.
    pub server: Option<DownloadInfo>,
    /// Client mappings.
    pub client_mappings: Option<DownloadInfo>,
    /// Server mappings.
    pub server_mappings: Option<DownloadInfo>,
}

/// Information about a downloadable file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    /// Download URL.
    pub url: String,
    /// SHA-1 hash.
    pub sha1: String,
    /// File size in bytes.
    pub size: u64,
}

/// Reference to the asset index for a version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndexRef {
    /// Asset index ID (e.g. "18").
    pub id: String,
    /// SHA-1 hash of the asset index JSON.
    pub sha1: String,
    /// File size in bytes.
    pub size: u64,
    /// Total number of assets.
    #[serde(default)]
    pub total_size: Option<u64>,
    /// Download URL for the asset index JSON.
    pub url: String,
}

/// Structured launch arguments (post-1.13 format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionArguments {
    /// JVM arguments (e.g. "-Xmx2G", "-Djava.library.path=...").
    pub jvm: Vec<serde_json::Value>,
    /// Game arguments (e.g. "--username", "--version").
    pub game: Vec<serde_json::Value>,
}

/// Java version requirement for a Minecraft version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersion {
    /// Major version number (e.g. 17, 21).
    pub major_version: u32,
    /// Component name (e.g. "java-runtime-gamma").
    pub component: String,
}
