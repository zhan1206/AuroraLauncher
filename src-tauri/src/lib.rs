//! Aurora Launcher — Tauri backend library.
//!
//! This module wires up the application state, database initialization,
//! and all IPC command handlers.

mod commands;
mod db;
mod error;
mod models;
mod services;
mod state;
mod utils;

use state::AppState;
use tauri::Manager;
use utils::file;

/// Initialize the application state and run the Tauri app.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            // Initialize tracing subscriber
            tracing_subscriber::fmt::init();

            // Build the shared HTTP client
            let http_client = utils::http::build_http_client()
                .expect("Failed to build HTTP client");

            // Get the app handle for state
            let app_handle = app.handle().clone();

            // Manage the global application state
            let app_state = AppState::new(http_client, app_handle);
            app.manage(app_state);

            // Initialize the database asynchronously
            let app_handle_clone = app.handle().clone();
            tokio::spawn(async move {
                let data_dir = file::data_dir();
                match db::init_db(&data_dir).await {
                    Ok(pool) => {
                        // Attach the pool to the app state
                        if let Some(state) = app_handle_clone.try_state::<AppState>() {
                            let _ = state.db_pool.set(pool);
                            *state.db_initialized.write().await = true;
                        }
                        tracing::info!("Database initialized successfully");
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize database: {}", e);
                    }
                }
            });

            tracing::info!("Aurora Launcher initialized successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            // ── Version Commands ──
            commands::version::get_version_manifest,
            commands::version::get_version_detail,
            // ── Instance Commands ──
            commands::instance::create_instance,
            commands::instance::list_instances,
            commands::instance::get_instance,
            commands::instance::update_instance,
            commands::instance::delete_instance,
            // ── Download Commands ──
            commands::download::start_download,
            commands::download::pause_download,
            commands::download::resume_download,
            commands::download::cancel_download,
            commands::download::list_download_tasks,
            // ── Java Commands ──
            commands::java::list_java_runtimes,
            commands::java::download_java,
            commands::java::resolve_java,
            // ── Settings Commands ──
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::settings::reset_settings,
            // ── Account Commands ──
            commands::account::login_microsoft,
            commands::account::login_offline,
            commands::account::get_accounts,
            commands::account::get_active_account,
            commands::account::set_active_account,
            commands::account::logout,
            commands::account::refresh_account,
            // ── Launch Commands ──
            commands::launch::launch_game,
            commands::launch::kill_game,
            commands::launch::get_launch_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// A simple greeting command for testing the Tauri IPC bridge.
#[tauri::command]
fn greet(name: &str) -> error::CommandResult<String> {
    Ok(error::CommandResponse::ok(format!(
        "你好，{}！欢迎使用 Aurora Launcher ✨",
        name
    )))
}
