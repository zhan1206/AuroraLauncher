//! Platform detection utilities.
//!
//! Provides helpers for detecting the current operating system, architecture,
//! and platform-specific Minecraft identifiers.

use serde::{Deserialize, Serialize};
use std::env;

/// The current platform as recognized by Minecraft's metadata system.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum McPlatform {
    /// Windows (x86_64 or x86)
    Windows,
    /// macOS
    MacOS,
    /// Linux
    Linux,
}

/// The current CPU architecture.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum McArch {
    /// x86_64 / AMD64
    X86_64,
    /// x86 / i686
    X86,
    /// ARM64 / AArch64
    Arm64,
}

/// Detected platform information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// The operating system.
    pub os: McPlatform,
    /// The CPU architecture.
    pub arch: McArch,
    /// The Minecraft-native classifier string (e.g. "natives-windows").
    pub natives_classifier: String,
    /// File extension for executables on this platform.
    pub exe_suffix: String,
    /// Path separator character.
    pub path_separator: String,
    /// Classpath separator character.
    pub classpath_separator: String,
}

/// Detect the current platform information.
pub fn detect_platform() -> PlatformInfo {
    let os = if cfg!(target_os = "windows") {
        McPlatform::Windows
    } else if cfg!(target_os = "macos") {
        McPlatform::MacOS
    } else {
        McPlatform::Linux
    };

    let arch = if cfg!(target_arch = "aarch64") {
        McArch::Arm64
    } else if cfg!(target_arch = "x86_64") {
        McArch::X86_64
    } else {
        McArch::X86
    };

    let natives_classifier = match (&os, &arch) {
        (McPlatform::Windows, McArch::X86_64) => "natives-windows".to_string(),
        (McPlatform::Windows, _) => "natives-windows".to_string(),
        (McPlatform::Linux, McArch::Arm64) => "natives-linux-arm64".to_string(),
        (McPlatform::Linux, _) => "natives-linux".to_string(),
        (McPlatform::MacOS, McArch::Arm64) => "natives-macos-arm64".to_string(),
        (McPlatform::MacOS, _) => "natives-macos".to_string(),
    };

    let exe_suffix = match os {
        McPlatform::Windows => ".exe".to_string(),
        _ => String::new(),
    };

    let path_separator = match os {
        McPlatform::Windows => ";".to_string(),
        _ => ":".to_string(),
    };

    let classpath_separator = match os {
        McPlatform::Windows => ";".to_string(),
        _ => ":".to_string(),
    };

    PlatformInfo {
        os,
        arch,
        natives_classifier,
        exe_suffix,
        path_separator,
        classpath_separator,
    }
}

/// Get the Java executable name for the current platform.
pub fn java_executable_name() -> String {
    if cfg!(target_os = "windows") {
        "javaw.exe".to_string()
    } else {
        "java".to_string()
    }
}

/// Get the platform-specific Adoptium JRE download identifier.
///
/// Returns the OS/arch tuple used in Adoptium API URLs.
pub fn adoptium_platform_id() -> &'static str {
    if cfg!(target_os = "windows") && cfg!(target_arch = "x86_64") {
        "windows-x64"
    } else if cfg!(target_os = "windows") && cfg!(target_arch = "x86") {
        "windows-x86"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "aarch64") {
        "macos-arm64"
    } else if cfg!(target_os = "macos") && cfg!(target_arch = "x86_64") {
        "macos-x64"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "x86_64") {
        "linux-x64"
    } else if cfg!(target_os = "linux") && cfg!(target_arch = "aarch64") {
        "linux-arm64"
    } else {
        "linux-x64"
    }
}

/// Determine the required Java major version based on the Minecraft version ID.
///
/// Rules:
/// - MC 1.20.5+ → Java 21
/// - MC 1.17+ → Java 17
/// - All others → Java 21 (default to latest stable)
pub fn required_java_version(version_id: &str) -> u32 {
    // Parse the major.minor version from the version ID
    let parts: Vec<&str> = version_id.split('.').collect();
    if parts.len() >= 2 {
        if let Ok(major) = parts[0].parse::<u32>() {
            if let Ok(minor) = parts[1].parse::<u32>() {
                // Standard Minecraft format: 1.x.x
                if major == 1 {
                    // MC 1.20.5+
                    if minor > 20 {
                        return 21;
                    }
                    if minor == 20 {
                        // Check patch version for 1.20.x
                        if parts.len() >= 3 {
                            if let Ok(patch) = parts[2].parse::<u32>() {
                                if patch >= 5 {
                                    return 21;
                                }
                            }
                        }
                        return 17;
                    }
                    // MC 1.17 - 1.19.x → Java 17
                    if minor >= 17 {
                        return 17;
                    }
                }

                // Non-1.x versions or unrecognizable format → default to Java 21
                // Modern mod loaders and custom versions typically need Java 17+
                return 21;
            }
        }
    }

    // Unrecognizable format → default to Java 21 (widely compatible)
    21
}

/// Get the platform-specific data directory for Adoptium JRE installations.
pub fn java_install_dir(version: u32) -> std::path::PathBuf {
    crate::utils::file::java_dir().join(format!("java-{}", version))
}

/// Check if a Java executable at the given path is valid.
pub async fn is_valid_java(path: &std::path::Path) -> bool {
    if !path.exists() {
        return false;
    }

    // Try running `java -version` to verify the executable works
    match tokio::process::Command::new(path)
        .arg("-version")
        .output()
        .await
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

/// Search for Java installations in standard system locations.
pub fn find_system_java() -> Vec<std::path::PathBuf> {
    let mut found = Vec::new();

    // 1. Use Windows `where` command to search entire system PATH (most reliable)
    #[cfg(target_os = "windows")]
    {
        for exe in &["javaw.exe", "java.exe"] {
            if let Ok(output) = std::process::Command::new("cmd")
                .args(["/c", "where", exe])
                .output()
            {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        let path = line.trim();
                        if !path.is_empty() {
                            let pb = std::path::PathBuf::from(path);
                            if pb.exists() && !found.contains(&pb) {
                                found.push(pb);
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Check JAVA_HOME
    if let Ok(java_home) = env::var("JAVA_HOME") {
        let java_bin = if cfg!(target_os = "windows") {
            std::path::PathBuf::from(&java_home).join("bin").join("javaw.exe")
        } else {
            std::path::PathBuf::from(&java_home).join("bin").join("java")
        };
        if java_bin.exists() && !found.contains(&java_bin) {
            found.push(java_bin);
        }
    }

    // 3. Search PATH manually as backup
    if let Ok(path_var) = env::var("PATH") {
        let exe_name = if cfg!(target_os = "windows") { "javaw.exe" } else { "java" };
        for dir in env::split_paths(&path_var) {
            let candidate = dir.join(exe_name);
            if candidate.exists() && !found.contains(&candidate) {
                found.push(candidate);
            }
            if cfg!(target_os = "windows") {
                let java_exe = dir.join("java.exe");
                if java_exe.exists() && !found.contains(&java_exe) {
                    found.push(java_exe);
                }
            }
        }
    }

    // 4. Common installation directories
    if cfg!(target_os = "windows") {
        let program_files = env::var("ProgramFiles").unwrap_or_default();
        let pf_x86 = env::var("ProgramFiles(x86)").unwrap_or_default();

        for base in [program_files, pf_x86] {
            if base.is_empty() { continue; }
            let base_path = std::path::PathBuf::from(&base);

            // Direct Java/ directory
            scan_java_dir(&base_path.join("Java"), &mut found);

            // Scan all subdirectories with java-related names
            if let Ok(entries) = std::fs::read_dir(&base_path) {
                for entry in entries.flatten() {
                    let name_lower = entry.file_name().to_string_lossy().to_lowercase();
                    let is_java_related = name_lower.contains("java")
                        || name_lower.contains("jdk")
                        || name_lower.contains("jre")
                        || name_lower.contains("temurin")
                        || name_lower.contains("adoptium")
                        || name_lower.contains("openjdk")
                        || name_lower.contains("zulu");
                    if is_java_related {
                        scan_java_dir(&entry.path(), &mut found);
                    }
                }
            }
        }

        // Scoop/Chocolatey/winget
        if let Ok(localapp) = env::var("LOCALAPPDATA") {
            for tool in ["scoop\\apps", "chocolatey"] {
                let dir = std::path::PathBuf::from(&localapp).join(tool);
                if dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(&dir) {
                        for e in entries.flatten() {
                            let n = e.file_name().to_string_lossy().to_lowercase();
                            if n.contains("java") || n.contains("temurin") || n.contains("openjdk") {
                                scan_java_dir(&e.path(), &mut found);
                            }
                        }
                    }
                }
            }
        }

        // JetBrains Toolbox
        if let Ok(localapp) = env::var("LOCALAPPDATA") {
            let jb = std::path::PathBuf::from(&localapp)
                .join("Jetbrains").join("Toolbox").join("apps");
            if jb.exists() { scan_java_dir(&jb, &mut found); }
        }
    }

    tracing::info!("find_system_java found {} Java runtime(s)", found.len());
    found
}

/// Recursively scan a directory for javaw.exe / java executables (max 2 levels deep).
fn scan_java_dir(dir: &std::path::Path, found: &mut Vec<std::path::PathBuf>) {
    if !dir.exists() {
        return;
    }
    let exe_name = if cfg!(target_os = "windows") {
        "javaw.exe"
    } else {
        "java"
    };

    // Direct bin/ subdirectory
    let direct = dir.join("bin").join(exe_name);
    if direct.exists() && !found.contains(&direct) {
        found.push(direct);
    }
    // Also direct java.exe on Windows
    if cfg!(target_os = "windows") {
        let java_exe = dir.join("bin").join("java.exe");
        if java_exe.exists() && !found.contains(&java_exe) {
            found.push(java_exe);
        }
    }

    // One level deeper (e.g. Eclipse Adoptium/jdk-17.0.9+9/bin/javaw.exe)
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let nested = path.join("bin").join(exe_name);
                if nested.exists() && !found.contains(&nested) {
                    found.push(nested);
                }
                if cfg!(target_os = "windows") {
                    let java_exe = path.join("bin").join("java.exe");
                    if java_exe.exists() && !found.contains(&java_exe) {
                        found.push(java_exe);
                    }
                }
            }
        }
    }
}
