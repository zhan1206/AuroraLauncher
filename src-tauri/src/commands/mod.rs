//! Tauri command modules.
//!
//! Each module exposes IPC commands that the frontend can invoke.
//! All commands follow the `State<'_, AppState> -> CommandResult<T>` convention.

pub mod version;
pub mod instance;
pub mod download;
pub mod java;
pub mod settings;
pub mod account;
pub mod launch;
