//! Shared database pool accessor.
//!
//! Provides a single `get_pool` function used by all command modules
//! to retrieve the SQLite connection pool from the application state.

use crate::error::AppError;
use crate::state::AppState;
use sqlx::SqlitePool;

/// Retrieve the database connection pool from the application state.
///
/// Returns an error if the database has not been initialized yet.
pub fn get_pool(state: &AppState) -> Result<&SqlitePool, AppError> {
    state.db_pool.get().ok_or_else(|| {
        AppError::Database("Database not initialized".to_string())
    })
}
