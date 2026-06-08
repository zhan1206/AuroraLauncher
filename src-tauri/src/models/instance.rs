//! Game instance model.
//!
//! An instance represents an isolated Minecraft installation with its own
//! game directory, version, loader configuration, and launch settings.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Loader type for a game instance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum LoaderType {
    Vanilla,
    Forge,
    Fabric,
    NeoForge,
    Quilt,
}

impl Default for LoaderType {
    fn default() -> Self {
        LoaderType::Vanilla
    }
}

impl LoaderType {
    /// Convert to the string representation stored in the database.
    pub fn as_str(&self) -> &str {
        match self {
            LoaderType::Vanilla => "Vanilla",
            LoaderType::Forge => "Forge",
            LoaderType::Fabric => "Fabric",
            LoaderType::NeoForge => "NeoForge",
            LoaderType::Quilt => "Quilt",
        }
    }

    /// Parse from the database string representation.
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "Vanilla" => LoaderType::Vanilla,
            "Forge" => LoaderType::Forge,
            "Fabric" => LoaderType::Fabric,
            "NeoForge" => LoaderType::NeoForge,
            "Quilt" => LoaderType::Quilt,
            _ => LoaderType::Vanilla,
        }
    }
}

/// A game instance record.
///
/// Each instance has a fully isolated `.minecraft` directory under
/// `{data_dir}/instances/{id}/.minecraft/`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Instance {
    /// Unique identifier (UUID v4).
    pub id: String,
    /// Human-readable instance name.
    pub name: String,
    /// Minecraft version ID this instance runs (e.g. "1.21.4").
    pub version_id: String,
    /// Loader type (Vanilla, Forge, Fabric, etc.).
    pub loader_type: String,
    /// Loader version (e.g. "0.16.9" for Fabric). NULL for Vanilla.
    pub loader_version: Option<String>,
    /// Absolute path to the instance's game directory.
    pub game_dir: String,
    /// Java runtime identifier (references a managed JRE).
    pub java_id: Option<String>,
    /// Serialized launch configuration JSON.
    pub launch_config: String,
    /// ISO-8601 timestamp of creation.
    pub created_at: String,
    /// ISO-8601 timestamp of last modification.
    pub updated_at: String,
    /// Optional icon identifier or path.
    pub icon: Option<String>,
    /// Optional user notes.
    pub notes: Option<String>,
}

/// Launch configuration embedded in an instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    /// Minimum memory allocation in MB.
    #[serde(default = "default_min_memory")]
    pub min_memory: u64,
    /// Maximum memory allocation in MB.
    #[serde(default = "default_max_memory")]
    pub max_memory: u64,
    /// Custom JVM arguments.
    #[serde(default)]
    pub jvm_args: Vec<String>,
    /// Custom game arguments.
    #[serde(default)]
    pub game_args: Vec<String>,
    /// Fullscreen mode.
    #[serde(default)]
    pub fullscreen: bool,
    /// Window width.
    #[serde(default = "default_width")]
    pub width: u32,
    /// Window height.
    #[serde(default = "default_height")]
    pub height: u32,
}

fn default_min_memory() -> u64 {
    512
}
fn default_max_memory() -> u64 {
    2048
}
fn default_width() -> u32 {
    854
}
fn default_height() -> u32 {
    480
}

impl Default for LaunchConfig {
    fn default() -> Self {
        Self {
            min_memory: default_min_memory(),
            max_memory: default_max_memory(),
            jvm_args: Vec::new(),
            game_args: Vec::new(),
            fullscreen: false,
            width: default_width(),
            height: default_height(),
        }
    }
}

/// Request body for creating a new instance.
#[derive(Debug, Deserialize)]
pub struct CreateInstanceRequest {
    /// Instance display name.
    pub name: String,
    /// Minecraft version ID.
    pub version_id: String,
    /// Loader type. Defaults to Vanilla.
    pub loader_type: Option<String>,
    /// Loader version. Required if loader_type is not Vanilla.
    pub loader_version: Option<String>,
    /// Custom launch configuration.
    pub launch_config: Option<LaunchConfig>,
    /// Optional icon.
    pub icon: Option<String>,
    /// Optional notes.
    pub notes: Option<String>,
}

/// Request body for updating an existing instance.
#[derive(Debug, Deserialize)]
pub struct UpdateInstanceRequest {
    /// Instance ID to update.
    pub id: String,
    /// New display name.
    pub name: Option<String>,
    /// New version ID.
    pub version_id: Option<String>,
    /// New loader type.
    pub loader_type: Option<String>,
    /// New loader version.
    pub loader_version: Option<String>,
    /// New Java runtime ID. Use `null` to clear.
    pub java_id: Option<String>,
    /// New launch configuration.
    pub launch_config: Option<LaunchConfig>,
    /// New icon.
    pub icon: Option<String>,
    /// New notes.
    pub notes: Option<String>,
}
