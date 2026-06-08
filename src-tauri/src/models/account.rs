//! Account model.
//!
//! Supports both offline and Microsoft (online) account types.

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// The type of authentication for an account.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum AccountType {
    Offline,
    Microsoft,
}

impl Default for AccountType {
    fn default() -> Self {
        AccountType::Offline
    }
}

impl AccountType {
    /// Convert to the string representation stored in the database.
    pub fn as_str(&self) -> &str {
        match self {
            AccountType::Offline => "Offline",
            AccountType::Microsoft => "Microsoft",
        }
    }

    /// Parse from the database string representation.
    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "Offline" => AccountType::Offline,
            "Microsoft" => AccountType::Microsoft,
            _ => AccountType::Offline,
        }
    }
}

/// An account record stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    /// Unique identifier (UUID v4).
    pub id: String,
    /// Login username.
    pub username: String,
    /// Display name (in-game name for Microsoft accounts).
    pub display_name: Option<String>,
    /// Minecraft UUID.
    pub uuid: Option<String>,
    /// Account type (Offline or Microsoft).
    pub account_type: String,
    /// Whether this is the currently active account.
    pub is_active: bool,
    /// ISO-8601 timestamp of creation.
    pub created_at: String,
}
