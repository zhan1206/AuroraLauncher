//! Launch configuration models.
//!
//! These structs define how a Minecraft instance is launched, including
//! JVM arguments, memory settings, and game arguments.

use serde::{Deserialize, Serialize};

/// Resolved JVM arguments for launching Minecraft.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JvmArgs {
    /// Path to the Java executable.
    pub java_path: String,
    /// Minimum heap size in MB (e.g. "-Xms512m").
    pub min_memory: u64,
    /// Maximum heap size in MB (e.g. "-Xmx2048m").
    pub max_memory: u64,
    /// Additional JVM flags (e.g. "-XX:+UseG1GC").
    pub extra_args: Vec<String>,
    /// Classpath separator for the current OS.
    pub classpath_separator: String,
}

/// The resolved launch command and environment for a Minecraft instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchCommand {
    /// Full command-line arguments (including java executable as argv[0]).
    pub command: Vec<String>,
    /// Working directory (the instance's .minecraft directory).
    pub working_dir: String,
    /// Environment variables to set.
    pub env_vars: Vec<(String, String)>,
    /// Classpath string.
    pub classpath: String,
    /// Main class to run.
    pub main_class: String,
    /// Game arguments.
    pub game_args: Vec<String>,
    /// JVM arguments.
    pub jvm_args: Vec<String>,
}
