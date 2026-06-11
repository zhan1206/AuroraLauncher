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
/// - All others → Java 8
pub fn required_java_version(version_id: &str) -> u32 {
    // Parse the major.minor version from the version ID
    let parts: Vec<&str> = version_id.split('.').collect();
    if parts.len() >= 2 {
        if let Ok(major) = parts[0].parse::<u32>() {
            if let Ok(minor) = parts[1].parse::<u32>() {
                // MC 1.20.5+
                if major == 1 && minor > 20 {
                    return 21;
                }
                if major == 1 && minor == 20 {
                    // Check patch version for 1.20.x
                    if parts.len() >= 3 {
                        if let Ok(patch) = parts[2].parse::<u32>() {
                            if patch >= 5 {
                                return 21;
                            }
                        }
                    }
                }
                // MC 1.17+
                if major == 1 && minor >= 17 {
                    return 17;
                }
            }
        }
    }
    8
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

    // 1. Check JAVA_HOME
    if let Ok(java_home) = env::var("JAVA_HOME") {
        let java_bin = if cfg!(target_os = "windows") {
            std::path::PathBuf::from(&java_home).join("bin").join("javaw.exe")
        } else {
            std::path::PathBuf::from(&java_home).join("bin").join("java")
        };
        if java_bin.exists() {
            found.push(java_bin);
        }
    }

    // 2. Search PATH for java / javaw
    if let Ok(path_var) = env::var("PATH") {
        let exe_name = if cfg!(target_os = "windows") {
            "javaw.exe"
        } else {
            "java"
        };
        for dir in env::split_paths(&path_var) {
            let candidate = dir.join(exe_name);
            if candidate.exists() && !found.contains(&candidate) {
                found.push(candidate);
            }
            // On Windows also check java.exe
            if cfg!(target_os = "windows") {
                let java_exe = dir.join("java.exe");
                if java_exe.exists() && !found.contains(&java_exe) {
                    found.push(java_exe);
                }
            }
        }
    }

    // 3. Check common Windows installation directories
    if cfg!(target_os = "windows") {
        let program_files = env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".to_string());
        let program_files_x86 = env::var("ProgramFiles(x86)").unwrap_or_else(|_| r"C:\Program Files (x86)".to_string());

        for base in &[&program_files, &program_files_x86] {
            let java_dir = std::path::PathBuf::from(base).join("Java");
            scan_java_dir(&java_dir, &mut found);

            // Also check direct subdirs for Eclipse/Adoptium/OpenJDK
            if let Ok(entries) = std::fs::read_dir(base.as_str()) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    if name.contains("java") || name.contains("jdk") || name.contains("jre")
                        || name.contains("temurin") || name.contains("adoptium")
                        || name.contains("openjdk") || name.contains("zulu") {
                        scan_java_dir(&entry.path(), &mut found);
                    }
                }
            }
        }

        // 4. Check Scoop / Chocolatey / winget / SDKMAN-style locations
        if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
            for tool in &["scoop", "chocolatey", "Microsoft\\winget\\packages"] {
                let dir = std::path::PathBuf::from(&local_appdata).join(tool).join("apps");
                if dir.exists() {
                    if let Ok(entries) = std::fs::read_dir(&dir) {
                        for entry in entries.flatten() {
                            let name = entry.file_name().to_string_lossy().to_lowercase();
                            if name.contains("java") || name.contains("temurin") || name.contains("openjdk") {
                                scan_java_dir(&entry.path(), &mut found);
                            }
                        }
                    }
                }
            }
        }

        // 5. Check Eclipse Adoptium default install path
        let adoptium_path = std::path::PathBuf::from(&program_files).join("Eclipse Adoptium");
        if adoptium_path.exists() {
            scan_java_dir(&adoptium_path, &mut found);
        }
        let adoptium_x86 = std::path::PathBuf::from(&program_files_x86).join("Eclipse Adoptium");
        if adoptium_x86.exists() {
            scan_java_dir(&adoptium_x86, &mut found);
        }

        // 6. Check JetBrains bundled JDKs
        if let Ok(local_appdata) = env::var("LOCALAPPDATA") {
            let jb_dir = std::path::PathBuf::from(&local_appdata)
                .join("JetBrains")
                .join("Toolbox")
                .join("apps");
            if jb_dir.exists() {
                scan_java_dir(&jb_dir, &mut found);
            }
        }
    }

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
