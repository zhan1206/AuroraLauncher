//! Application settings model.
//!
//! Global settings persisted as key-value pairs in the SQLite database,
//! with typed accessors for known configuration keys.

use serde::{Deserialize, Serialize};

/// Download mirror preference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum DownloadMirror {
    /// Official Mojang / Adoptium sources.
    Official,
    /// BMCLAPI mirror (faster in mainland China).
    Bmclapi,
}

impl Default for DownloadMirror {
    fn default() -> Self {
        DownloadMirror::Official
    }
}

impl DownloadMirror {
    /// Convert to the string representation stored in the database.
    pub fn as_str(&self) -> &str {
        match self {
            DownloadMirror::Official => "Official",
            DownloadMirror::Bmclapi => "Bmclapi",
        }
    }

    /// Parse from the database string representation.
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "Official" => DownloadMirror::Official,
            "Bmclapi" => DownloadMirror::Bmclapi,
            _ => DownloadMirror::Official,
        }
    }
}

/// Typed application settings with defaults.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    /// Default download mirror.
    #[serde(default)]
    pub download_mirror: DownloadMirror,
    /// Default maximum memory for new instances in MB.
    #[serde(default = "default_max_memory")]
    pub default_max_memory: u64,
    /// Default minimum memory for new instances in MB.
    #[serde(default = "default_min_memory")]
    pub default_min_memory: u64,
    /// Download concurrency (number of parallel chunks).
    #[serde(default = "default_concurrency")]
    pub download_concurrency: u32,
    /// Custom Java path override. If set, used instead of auto-detection.
    #[serde(default)]
    pub custom_java_path: Option<String>,
    /// Window width.
    #[serde(default = "default_window_width")]
    pub window_width: u32,
    /// Window height.
    #[serde(default = "default_window_height")]
    pub window_height: u32,
    /// Language preference (e.g. "zh-CN", "en-US").
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_max_memory() -> u64 {
    2048
}
fn default_min_memory() -> u64 {
    512
}
fn default_concurrency() -> u32 {
    8
}
fn default_window_width() -> u32 {
    1280
}
fn default_window_height() -> u32 {
    800
}
fn default_language() -> String {
    "zh-CN".to_string()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            download_mirror: DownloadMirror::default(),
            default_max_memory: default_max_memory(),
            default_min_memory: default_min_memory(),
            download_concurrency: default_concurrency(),
            custom_java_path: None,
            window_width: default_window_width(),
            window_height: default_window_height(),
            language: default_language(),
        }
    }
}
