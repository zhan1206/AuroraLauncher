//! Launch service — Minecraft game launch engine.
//!
//! Implements the full game launch pipeline:
//! 1. Load instance configuration
//! 2. Recursively resolve version.json chain (handles Fabric/Forge inheritance)
//! 3. Assemble classpath from all library JARs
//! 4. Extract native libraries for the current platform
//! 5. Build JVM arguments (-Xmx, -Xms, -Djava.library.path, etc.)
//! 6. Build game arguments (--username, --uuid, --accessToken, etc.)
//! 7. Spawn the Java process
//! 8. Capture stdout/stderr and emit log events
//! 9. Monitor process exit and report exit code

use crate::error::AppError;
use crate::models::instance::{Instance, LaunchConfig};
use crate::models::launch::LaunchCommand;
use crate::services::account_service;
use crate::services::instance_service;
use crate::services::java_service;
use crate::state::AppState;
use crate::utils::file;
use crate::utils::platform;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::Emitter;
use tokio::sync::Mutex;

/// Global game process state.
pub struct GameProcess {
    /// The running game process, if any.
    process: Option<tokio::process::Child>,
    /// The instance ID of the running game, if any.
    instance_id: Option<String>,
}

impl GameProcess {
    /// Create a new empty game process holder.
    pub fn new() -> Self {
        Self { process: None, instance_id: None }
    }
}

/// Current launch status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchStatus {
    /// Whether a game is currently running.
    pub is_running: bool,
    /// PID of the running game process, if any.
    pub pid: Option<u32>,
    /// The instance ID of the running game.
    pub instance_id: Option<String>,
}

// ── Version JSON Structures ──────────────────────────────────────────────────

/// A parsed version.json file (minimal fields needed for launch).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionJson {
    /// Version identifier.
    #[serde(default)]
    id: Option<String>,
    /// Parent version that this version inherits from.
    #[serde(default)]
    inherits_from: Option<String>,
    /// Main class for launching.
    #[serde(default)]
    main_class: Option<String>,
    /// Libraries required by this version.
    #[serde(default)]
    libraries: Vec<serde_json::Value>,
    /// Structured arguments (post-1.13 format).
    #[serde(default)]
    arguments: Option<VersionArguments>,
    /// Legacy minecraftArguments (pre-1.13 format).
    #[serde(default)]
    minecraft_arguments: Option<String>,
    /// Minimum Java version.
    #[serde(default)]
    java_version: Option<JavaVersionInfo>,
    /// Download information.
    #[serde(default)]
    downloads: Option<serde_json::Value>,
    /// Asset index reference.
    #[serde(default)]
    asset_index: Option<serde_json::Value>,
}

/// Structured launch arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VersionArguments {
    /// JVM arguments.
    #[serde(default)]
    jvm: Vec<serde_json::Value>,
    /// Game arguments.
    #[serde(default)]
    game: Vec<serde_json::Value>,
}

/// Java version requirement.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JavaVersionInfo {
    #[serde(default)]
    major_version: u32,
    #[serde(default)]
    component: String,
}

/// Resolved version information after merging the inheritance chain.
#[derive(Debug, Clone)]
struct ResolvedVersion {
    /// Main class to run.
    main_class: String,
    /// All libraries (merged from the entire chain).
    libraries: Vec<serde_json::Value>,
    /// JVM arguments (merged).
    jvm_args: Vec<serde_json::Value>,
    /// Game arguments (merged or legacy format).
    game_args: Vec<serde_json::Value>,
    /// Legacy minecraftArguments string (pre-1.13).
    legacy_args: Option<String>,
    /// Asset index ID.
    asset_index_id: String,
}

// ── Launch Pipeline ──────────────────────────────────────────────────────────

/// Launch a game instance.
///
/// This is the main entry point for launching Minecraft. It:
/// 1. Loads the instance and account data
/// 2. Resolves the version.json chain
/// 3. Assembles the classpath and arguments
/// 4. Spawns the game process
/// 5. Starts log capture and process monitoring
pub async fn launch_game(
    state: &AppState,
    instance_id: &str,
    game_process: &Arc<Mutex<GameProcess>>,
) -> Result<LaunchCommand, AppError> {
    let pool = state
        .db_pool
        .get()
        .ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;

    // Check if a game is already running
    {
        let gp = game_process.lock().await;
        if gp.process.is_some() {
            return Err(AppError::LaunchFailed(
                "A game is already running. Please stop it first.".to_string(),
            ));
        }
    }

    // Load the instance
    let instance = instance_service::get_instance(pool, instance_id).await?;
    let launch_config = instance_service::parse_launch_config(&instance);

    // Resolve the Java runtime
    let java_runtime = java_service::resolve_java(&instance.version_id).await?;

    // Get account information
    let (username, uuid, access_token) = account_service::get_mc_access_token(state).await?;

    // Resolve the version.json chain
    let resolved_version = resolve_version_chain(&instance.version_id).await?;

    // Assemble the classpath
    let (classpath, native_dirs) =
        assemble_classpath(&instance, &resolved_version).await?;

    // Extract natives
    extract_natives(&instance, &resolved_version, &native_dirs).await?;

    // Build the full launch command
    let launch_command = build_launch_command(
        &instance,
        &launch_config,
        &java_runtime.path,
        &resolved_version,
        &classpath,
        &native_dirs,
        &username,
        &uuid,
        &access_token,
    )?;

    // Spawn the game process
    let mut command = tokio::process::Command::new(&launch_command.command[0]);
    command
        .args(&launch_command.command[1..])
        .current_dir(&launch_command.working_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    // Set environment variables
    for (key, value) in &launch_command.env_vars {
        command.env(key, value);
    }

    let mut child = command
        .spawn()
        .map_err(|e| AppError::LaunchFailed(format!("Failed to spawn game process: {}", e)))?;

    let pid = child.id().unwrap_or(0);

    // Store the process
    {
        let mut gp = game_process.lock().await;
        gp.process = Some(child);
        gp.instance_id = Some(instance_id.to_string());
    }

    // Emit launch started event
    let _ = state.app_handle.emit(
        "launch:started",
        serde_json::json!({
            "pid": pid,
            "instance_id": instance_id,
        }),
    );

    tracing::info!("Game launched: PID={}, Instance={}", pid, instance_id);

    // Start log capture and process monitoring in a background task
    let app_handle = state.app_handle.clone();
    let instance_id_owned = instance_id.to_string();
    // Safe: Arc::clone increments the reference count, keeping the Mutex alive
    // for the entire lifetime of the spawned task.
    let game_process_arc = Arc::clone(game_process);

    tokio::spawn(async move {
        // Get the stdout and stderr handles from the process
        let (stdout, stderr) = {
            let mut gp = game_process_arc.lock().await;
            if let Some(ref mut child) = gp.process {
                let stdout = child.stdout.take();
                let stderr = child.stderr.take();
                (stdout, stderr)
            } else {
                return;
            }
        };

        // Spawn tasks to read stdout and stderr
        let handle_out = app_handle.clone();
        let stdout_task = tokio::spawn(async move {
            if let Some(stdout) = stdout {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = handle_out.emit(
                        "game:log",
                        serde_json::json!({
                            "line": line,
                            "level": "info",
                        }),
                    );
                }
            }
        });

        let handle_err = app_handle.clone();
        let stderr_task = tokio::spawn(async move {
            if let Some(stderr) = stderr {
                use tokio::io::{AsyncBufReadExt, BufReader};
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = handle_err.emit(
                        "game:log",
                        serde_json::json!({
                            "line": line,
                            "level": "error",
                        }),
                    );
                }
            }
        });

        // Wait for both log readers to finish
        let _ = stdout_task.await;
        let _ = stderr_task.await;

        // Wait for the process to exit
        let exit_code = {
            let mut gp = game_process_arc.lock().await;
            if let Some(ref mut child) = gp.process {
                match child.wait().await {
                    Ok(status) => status.code().unwrap_or(-1),
                    Err(_) => -1,
                }
            } else {
                -1
            }
        };

        // Clear the process reference
        {
            let mut gp = game_process_arc.lock().await;
            gp.process = None;
            gp.instance_id = None;
        }

        // Emit launch exited event
        let _ = app_handle.emit(
            "launch:exited",
            serde_json::json!({
                "code": exit_code,
                "instance_id": instance_id_owned,
            }),
        );

        tracing::info!(
            "Game exited: code={}, instance={}",
            exit_code,
            instance_id_owned
        );
    });

    Ok(launch_command)
}

/// Kill the running game process.
pub async fn kill_game(game_process: &Arc<Mutex<GameProcess>>) -> Result<(), AppError> {
    let mut gp = game_process.lock().await;
    if let Some(ref mut child) = gp.process {
        child.kill().await.map_err(|e| {
            AppError::LaunchFailed(format!("Failed to kill game process: {}", e))
        })?;
        gp.process = None;
        gp.instance_id = None;
        tracing::info!("Game process killed");
        Ok(())
    } else {
        Err(AppError::LaunchFailed("No game is running".to_string()))
    }
}

/// Get the current launch status.
pub async fn get_launch_status(
    game_process: &Arc<Mutex<GameProcess>>,
) -> LaunchStatus {
    let gp = game_process.lock().await;
    match &gp.process {
        Some(child) => LaunchStatus {
            is_running: true,
            pid: child.id(),
            instance_id: gp.instance_id.clone(),
        },
        None => LaunchStatus {
            is_running: false,
            pid: None,
            instance_id: None,
        },
    }
}

// ── Version Chain Resolution ─────────────────────────────────────────────────

/// Recursively resolve the version.json chain.
///
/// Minecraft versions can inherit from a parent version via `inheritsFrom`.
/// For example, Fabric's version.json inherits from the vanilla version.json.
/// This function loads and merges all version.json files in the chain.
async fn resolve_version_chain(version_id: &str) -> Result<ResolvedVersion, AppError> {
    let data_dir = file::data_dir();
    let versions_dir = data_dir.join("versions");

    let mut chain: Vec<VersionJson> = Vec::new();
    let mut current_id = version_id.to_string();
    let mut visited = HashSet::new();

    // Load version.json files, following the inheritsFrom chain
    loop {
        if visited.contains(&current_id) {
            return Err(AppError::LaunchFailed(format!(
                "Circular version inheritance detected at: {}",
                current_id
            )));
        }
        visited.insert(current_id.clone());

        let version_json_path = versions_dir.join(&current_id).join(format!("{}.json", current_id));
        if !version_json_path.exists() {
            return Err(AppError::LaunchFailed(format!(
                "Version JSON not found: {}",
                version_json_path.display()
            )));
        }

        let content = tokio::fs::read_to_string(&version_json_path)
            .await
            .map_err(|e| AppError::FileIo(e))?;

        let version: VersionJson = serde_json::from_str(&content)
            .map_err(|e| AppError::Serialization(format!(
                "Failed to parse version JSON for {}: {}", current_id, e
            )))?;

        // Check if this version inherits from another
        match &version.inherits_from {
            Some(parent) => {
                chain.push(version);
                current_id = parent.clone();
            }
            None => {
                chain.push(version);
                break;
            }
        }
    }

    // Merge the chain (child overrides parent)
    // The chain is ordered [child, parent, grandparent, ...]
    // We process from the root (last) to the child (first) to build the merged result
    let root = chain.last().ok_or_else(|| {
        AppError::LaunchFailed("Empty version chain".to_string())
    })?;

    // Collect main_class from the first version that specifies it
    let main_class = chain
        .iter()
        .find_map(|v| v.main_class.clone())
        .unwrap_or_else(|| "net.minecraft.client.main.Main".to_string());

    // Merge libraries (child first, then parent — dedup by name)
    let mut all_libraries: Vec<serde_json::Value> = Vec::new();
    let mut seen_names: HashSet<String> = HashSet::new();
    for version in &chain {
        if let Some(libs) = version.libraries.as_array() {
            for lib in libs {
                if let Some(name) = lib.get("name").and_then(|n| n.as_str()) {
                    let name_key = name.to_string();
                    if !seen_names.contains(&name_key) {
                        seen_names.insert(name_key);
                        all_libraries.push(lib.clone());
                    }
                }
            }
        }
    }

    // Merge JVM arguments
    let mut all_jvm_args: Vec<serde_json::Value> = Vec::new();
    for version in chain.iter().rev() {
        if let Some(ref args) = version.arguments {
            all_jvm_args.extend(args.jvm.clone());
        }
    }

    // Merge game arguments
    let mut all_game_args: Vec<serde_json::Value> = Vec::new();
    let mut legacy_args: Option<String> = None;
    for version in chain.iter().rev() {
        if let Some(ref args) = version.arguments {
            all_game_args.extend(args.game.clone());
        }
        if legacy_args.is_none() {
            legacy_args = version.minecraft_arguments.clone();
        }
    }

    // Get asset index ID
    let asset_index_id = chain
        .iter()
        .find_map(|v| {
            v.asset_index
                .as_ref()
                .and_then(|ai| ai.get("id"))
                .and_then(|id| id.as_str())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| version_id.to_string());

    Ok(ResolvedVersion {
        main_class,
        libraries: all_libraries,
        jvm_args: all_jvm_args,
        game_args: all_game_args,
        legacy_args,
        asset_index_id,
    })
}

// ── Classpath Assembly ───────────────────────────────────────────────────────

/// Assemble the classpath from all libraries in the resolved version.
///
/// Returns the classpath string and a list of native library directories to extract.
async fn assemble_classpath(
    instance: &Instance,
    resolved: &ResolvedVersion,
) -> Result<(String, Vec<PathBuf>), AppError> {
    let data_dir = file::data_dir();
    let libraries_dir = data_dir.join("libraries");
    let platform_info = platform::detect_platform();

    let mut classpath_entries: Vec<String> = Vec::new();
    let mut native_dirs: Vec<PathBuf> = Vec::new();

    for lib in &resolved.libraries {
        // Check if the library applies to this platform (rules)
        if !should_include_library(lib, &platform_info) {
            continue;
        }

        // Handle artifact (main JAR)
        if let Some(downloads) = lib.get("downloads") {
            if let Some(artifact) = downloads.get("artifact") {
                if let Some(path) = artifact.get("path").and_then(|p| p.as_str()) {
                    let jar_path = libraries_dir.join(path);
                    if jar_path.exists() {
                        classpath_entries.push(jar_path.to_string_lossy().to_string());
                    }
                }
            }
        }

        // Handle natives
        if let Some(natives) = lib.get("natives") {
            let native_key = &platform_info.natives_classifier;
            // Try the exact native classifier, then without arch suffix
            let native_classifier = if let Some(classifier) = natives.get(native_key) {
                classifier.as_str().map(|s| s.to_string())
            } else {
                // Try without arch suffix (e.g. "natives-windows" instead of "natives-windows-arm64")
                let base_key = native_key
                    .rsplitn(2, '-')
                    .last()
                    .unwrap_or(native_key);
                let fallback_key = format!("natives-{}", base_key);
                natives
                    .get(&fallback_key)
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string())
            };

            if let Some(classifier) = native_classifier {
                if let Some(downloads) = lib.get("downloads") {
                    if let Some(classifiers) = downloads.get("classifiers") {
                        if let Some(native_artifact) = classifiers.get(&classifier) {
                            if let Some(path) = native_artifact.get("path").and_then(|p| p.as_str())
                            {
                                let native_jar = libraries_dir.join(path);
                                if native_jar.exists() {
                                    // We'll extract this native later
                                    let native_dir = file::instance_game_dir(&instance.id)
                                        .join("natives");
                                    native_dirs.push(native_dir);
                                    // Also add the native jar path for extraction reference
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Add the version's client JAR to the classpath
    let client_jar = data_dir
        .join("versions")
        .join(&instance.version_id)
        .join(format!("{}.jar", instance.version_id));
    if client_jar.exists() {
        classpath_entries.push(client_jar.to_string_lossy().to_string());
    }

    let classpath = classpath_entries.join(&platform_info.classpath_separator);
    Ok((classpath, native_dirs))
}

/// Extract native libraries for the current platform.
///
/// Native libraries are JAR files containing .dll (Windows), .so (Linux),
/// or .dylib (macOS) files. They need to be extracted to the instance's
/// natives directory so the JVM can find them via -Djava.library.path.
async fn extract_natives(
    instance: &Instance,
    resolved: &ResolvedVersion,
    _native_dirs: &[PathBuf],
) -> Result<(), AppError> {
    let data_dir = file::data_dir();
    let libraries_dir = data_dir.join("libraries");
    let platform_info = platform::detect_platform();
    let natives_target_dir = file::instance_game_dir(&instance.id).join("natives");

    file::ensure_dir(&natives_target_dir).await?;

    // Platform-specific native file extensions
    let native_extensions: &[&str] = match platform_info.os {
        platform::McPlatform::Windows => &[".dll"],
        platform::McPlatform::Linux => &[".so"],
        platform::McPlatform::MacOS => &[".dylib", ".jnilib"],
    };

    // Determine which files to exclude (based on extract rules)
    let default_exclude = vec!["META-INF/"];

    for lib in &resolved.libraries {
        if !should_include_library(lib, &platform_info) {
            continue;
        }

        // Get the native classifier for this platform
        if let Some(natives) = lib.get("natives") {
            let native_key = &platform_info.natives_classifier;
            let native_classifier = if let Some(classifier) = natives.get(native_key) {
                classifier.as_str().map(|s| s.to_string())
            } else {
                let base_key = native_key
                    .rsplitn(2, '-')
                    .last()
                    .unwrap_or(native_key);
                let fallback_key = format!("natives-{}", base_key);
                natives
                    .get(&fallback_key)
                    .and_then(|c| c.as_str())
                    .map(|s| s.to_string())
            };

            if let Some(classifier) = native_classifier {
                if let Some(downloads) = lib.get("downloads") {
                    if let Some(classifiers) = downloads.get("classifiers") {
                        if let Some(native_artifact) = classifiers.get(&classifier) {
                            if let Some(path) =
                                native_artifact.get("path").and_then(|p| p.as_str())
                            {
                                let native_jar = libraries_dir.join(path);
                                if native_jar.exists() {
                                    // Get extract exclusion rules
                                    let exclude = lib
                                        .get("extract")
                                        .and_then(|e| e.get("exclude"))
                                        .and_then(|e| e.as_array())
                                        .map(|arr| {
                                            arr.iter()
                                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                                .collect::<Vec<_>>()
                                        })
                                        .unwrap_or_else(|| {
                                            default_exclude
                                                .iter()
                                                .map(|s| s.to_string())
                                                .collect()
                                        });

                                    extract_native_jar(&native_jar, &natives_target_dir, native_extensions, &exclude)?;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Extract native files from a JAR (which is a ZIP archive).
fn extract_native_jar(
    jar_path: &Path,
    target_dir: &Path,
    extensions: &[&str],
    exclude: &[String],
) -> Result<(), AppError> {
    let file = std::fs::File::open(jar_path).map_err(|e| AppError::FileIo(e))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::DecompressionFailed(format!("Failed to open native JAR: {}", e)))?;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| AppError::DecompressionFailed(format!("Failed to read JAR entry: {}", e)))?;

        let entry_name = entry.name().to_string();

        // Check exclusion rules
        let should_exclude = exclude.iter().any(|pattern| entry_name.starts_with(pattern.as_str()));
        if should_exclude {
            continue;
        }

        // Check if this file has a native extension
        let is_native = extensions.iter().any(|ext| entry_name.ends_with(ext));
        if !is_native && !entry.is_dir() {
            continue;
        }

        // Build the output path (use just the filename, not the full path inside the JAR)
        if entry.is_dir() {
            continue;
        }

        let file_name = Path::new(&entry_name)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| continue);

        let out_path = target_dir.join(&file_name);

        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::DirectoryCreateFailed(e.to_string()))?;
        }

        let mut outfile = std::fs::File::create(&out_path)
            .map_err(|e| AppError::FileIo(e))?;
        std::io::copy(&mut entry, &mut outfile)
            .map_err(|e| AppError::FileIo(e))?;
    }

    Ok(())
}

// ── Command Building ─────────────────────────────────────────────────────────

/// Build the full launch command for the game.
#[allow(clippy::too_many_arguments)]
fn build_launch_command(
    instance: &Instance,
    launch_config: &LaunchConfig,
    java_path: &str,
    resolved: &ResolvedVersion,
    classpath: &str,
    _native_dirs: &[PathBuf],
    username: &str,
    uuid: &str,
    access_token: &str,
) -> Result<LaunchCommand, AppError> {
    let platform_info = platform::detect_platform();
    let data_dir = file::data_dir();

    let game_dir = PathBuf::from(&instance.game_dir);
    let assets_dir = data_dir.join("assets");
    let natives_dir = game_dir.join("natives");
    let versions_dir = data_dir.join("versions");
    let version_jar = versions_dir
        .join(&instance.version_id)
        .join(format!("{}.jar", instance.version_id));

    // ── JVM Arguments ────────────────────────────────
    let mut jvm_args: Vec<String> = Vec::new();

    // 内存设置
    jvm_args.push(format!("-Xms{}m", launch_config.min_memory));
    jvm_args.push(format!("-Xmx{}m", launch_config.max_memory));

    // Native 库路径
    jvm_args.push(format!(
        "-Djava.library.path={}",
        natives_dir.to_string_lossy()
    ));

    // 启动器标识
    jvm_args.push("-Dminecraft.launcher.brand=Aurora".to_string());
    jvm_args.push("-Dminecraft.launcher.version=1.0.0".to_string());

    // 解析条件 JVM 参数（version.json 中的 arguments.jvm 字段）
    for arg in &resolved.jvm_args {
        let resolved_args = resolve_argument(arg, &platform_info);
        jvm_args.extend(resolved_args);
    }

    // 用户自定义 JVM 参数
    jvm_args.extend(launch_config.jvm_args.iter().cloned());

    // Classpath
    jvm_args.push("-cp".to_string());
    jvm_args.push(classpath.to_string());

    // ── Game Arguments ───────────────────────────────
    let mut game_args: Vec<String> = Vec::new();

    // 主类
    let main_class = resolved.main_class.clone();

    // 标准游戏参数
    game_args.push("--username".to_string());
    game_args.push(username.to_string());

    game_args.push("--version".to_string());
    game_args.push(instance.version_id.clone());

    game_args.push("--gameDir".to_string());
    game_args.push(game_dir.to_string_lossy().to_string());

    game_args.push("--assetsDir".to_string());
    game_args.push(assets_dir.to_string_lossy().to_string());

    game_args.push("--assetIndex".to_string());
    game_args.push(resolved.asset_index_id.clone());

    game_args.push("--uuid".to_string());
    game_args.push(uuid.to_string());

    game_args.push("--accessToken".to_string());
    game_args.push(access_token.to_string());

    game_args.push("--userType".to_string());
    game_args.push("msa".to_string());

    game_args.push("--versionType".to_string());
    game_args.push("Aurora".to_string());

    // 解析条件游戏参数（version.json 中的 arguments.game 字段或 legacy 格式）
    if let Some(ref legacy) = resolved.legacy_args {
        // 旧版格式（1.13 之前）：minecraftArguments 是一个空格分隔的字符串
        let legacy_parts: Vec<String> = legacy
            .split_whitespace()
            .map(|s| substitute_placeholders(s, &instance, &resolved, username, uuid, access_token, &game_dir, &assets_dir))
            .collect();
        game_args.extend(legacy_parts);
    } else {
        // 新版格式：arguments.game 是一个条件参数数组
        for arg in &resolved.game_args {
            let resolved_args = resolve_argument(arg, &platform_info);
            let substituted: Vec<String> = resolved_args
                .iter()
                .map(|s| substitute_placeholders(s, &instance, resolved, username, uuid, access_token, &game_dir, &assets_dir))
                .collect();
            game_args.extend(substituted);
        }
    }

    // 用户自定义游戏参数
    game_args.extend(launch_config.game_args.iter().cloned());

    // 全屏模式
    if launch_config.fullscreen {
        game_args.push("--fullscreen".to_string());
    }

    // 窗口尺寸
    game_args.push("--width".to_string());
    game_args.push(launch_config.width.to_string());
    game_args.push("--height".to_string());
    game_args.push(launch_config.height.to_string());

    // ── 组装完整命令 ────────────────────────────────
    let mut command: Vec<String> = Vec::new();
    command.push(java_path.to_string());
    command.extend(jvm_args.clone());
    command.push(main_class.clone());
    command.extend(game_args.clone());

    // 环境变量
    let env_vars: Vec<(String, String)> = Vec::new();

    Ok(LaunchCommand {
        command,
        working_dir: game_dir.to_string_lossy().to_string(),
        env_vars,
        classpath: classpath.to_string(),
        main_class,
        game_args,
        jvm_args,
    })
}

/// 判断一个库是否应包含在当前平台上。
///
/// 检查库的 `rules` 字段，根据当前平台和特性决定是否应包含该库。
/// 如果没有 rules 字段，默认包含。
fn should_include_library(
    lib: &serde_json::Value,
    platform_info: &platform::PlatformInfo,
) -> bool {
    // 如果没有 rules，默认包含
    let rules = match lib.get("rules") {
        Some(rules) => rules,
        None => return true,
    };

    let rules_arr = match rules.as_array() {
        Some(arr) => arr,
        None => return true,
    };

    evaluate_rules(rules_arr, platform_info)
}

/// 评估一组规则，决定是否包含某个项目。
///
/// 规则逻辑：
/// - 如果存在 `action=allow` 的规则且匹配，则包含
/// - 如果存在 `action=disallow` 的规则且匹配，则排除
/// - 如果没有规则匹配，默认不包含（当存在规则时）
fn evaluate_rules(
    rules: &[serde_json::Value],
    platform_info: &platform::PlatformInfo,
) -> bool {
    let mut allowed = false;
    let mut disallowed = false;

    for rule in rules {
        let action = rule
            .get("action")
            .and_then(|a| a.as_str())
            .unwrap_or("");

        let matches = rule_matches_os(rule, platform_info)
            && rule_matches_features(rule);

        if matches {
            match action {
                "allow" => allowed = true,
                "disallow" => disallowed = true,
                _ => {}
            }
        }
    }

    allowed && !disallowed
}

/// 检查规则的 os 条件是否匹配当前平台。
fn rule_matches_os(
    rule: &serde_json::Value,
    platform_info: &platform::PlatformInfo,
) -> bool {
    let os_val = match rule.get("os") {
        Some(os) => os,
        None => return true, // 没有 os 条件则默认匹配
    };

    let os_name = os_val.get("name").and_then(|n| n.as_str()).unwrap_or("");

    let _current_os_name = match platform_info.os {
        platform::McPlatform::Windows => "windows",
        platform::McPlatform::Linux => "linux",
        platform::McPlatform::MacOS => "osx",
    };

    match os_name {
        "windows" => platform_info.os == platform::McPlatform::Windows,
        "linux" => platform_info.os == platform::McPlatform::Linux,
        "osx" => platform_info.os == platform::McPlatform::MacOS,
        _ => true,
    }
    && if let Some(arch) = os_val.get("arch").and_then(|a| a.as_str()) {
        let current_arch = match platform_info.arch {
            platform::McArch::X86_64 => "x86_64",
            platform::McArch::X86 => "x86",
            platform::McArch::Arm64 => "arm64",
        };
        arch == current_arch
    } else {
        true
    }
}

/// 检查规则的 features 条件是否匹配。
///
/// 目前支持 `is_demo_user` 和 `has_custom_resolution` 特性。
fn rule_matches_features(rule: &serde_json::Value) -> bool {
    let features = match rule.get("features") {
        Some(f) => f,
        None => return true, // 没有 features 条件则默认匹配
    };

    // is_demo_user：我们不支持演示模式，所以如果要求 is_demo_user=true 则不匹配
    if features
        .get("is_demo_user")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return false;
    }

    // has_custom_resolution：我们总是有自定义分辨率（使用窗口大小），所以匹配
    if features
        .get("has_custom_resolution")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return true;
    }

    true
}

/// 解析 version.json 中的条件参数。
///
/// 参数可以是：
/// 1. 普通字符串：直接返回
/// 2. 带规则的对象：根据 rules 决定是否包含 value
fn resolve_argument(
    arg: &serde_json::Value,
    platform_info: &platform::PlatformInfo,
) -> Vec<String> {
    // 情况1：普通字符串
    if let Some(s) = arg.as_str() {
        return vec![s.to_string()];
    }

    // 情况2：带规则的对象
    if let Some(obj) = arg.as_object() {
        // 检查规则
        let rules_match = if let Some(rules) = obj.get("rules") {
            if let Some(rules_arr) = rules.as_array() {
                evaluate_rules(rules_arr, platform_info)
            } else {
                true
            }
        } else {
            true
        };

        if !rules_match {
            return Vec::new();
        }

        // 获取值
        if let Some(value) = obj.get("value") {
            if let Some(s) = value.as_str() {
                return vec![s.to_string()];
            } else if let Some(arr) = value.as_array() {
                return arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
            }
        }
    }

    Vec::new()
}

/// 替换参数中的占位符。
///
/// Minecraft 的 version.json 中使用 `${...}` 格式的占位符，
/// 需要在运行时替换为实际值。
fn substitute_placeholders(
    s: &str,
    instance: &Instance,
    resolved: &ResolvedVersion,
    username: &str,
    uuid: &str,
    access_token: &str,
    game_dir: &Path,
    assets_dir: &Path,
) -> String {
    let data_dir = file::data_dir();
    let natives_dir = game_dir.join("natives");
    let versions_dir = data_dir.join("versions");
    let version_jar = versions_dir
        .join(&instance.version_id)
        .join(format!("{}.jar", instance.version_id));

    let mut result = s.to_string();

    // 替换常见的占位符
    result = result.replace("${auth_player_name}", username);
    result = result.replace("${version_name}", &instance.version_id);
    result = result.replace("${game_directory}", &game_dir.to_string_lossy());
    result = result.replace("${assets_root}", &assets_dir.to_string_lossy());
    result = result.replace("${assets_index_name}", &resolved.asset_index_id);
    result = result.replace("${auth_uuid}", uuid);
    result = result.replace("${auth_access_token}", access_token);
    result = result.replace("${user_type}", "msa");
    result = result.replace("${version_type}", "Aurora");
    result = result.replace("${natives_directory}", &natives_dir.to_string_lossy());
    result = result.replace("${launcher_name}", "Aurora");
    result = result.replace("${launcher_version}", "1.0.0");
    result = result.replace("${classpath}", ""); // classpath 在 -cp 参数中已处理
    result = result.replace(
        "${library_directory}",
        &data_dir.join("libraries").to_string_lossy(),
    );
    result = result.replace(
        "${classpath_separator}",
        &platform::detect_platform().classpath_separator,
    );

    result
}