//! Account service — Microsoft OAuth 2.0 device flow and offline account management.
//!
//! Implements the full Microsoft → Xbox Live → XSTS → Minecraft authentication chain:
//! 1. Request device code from Microsoft
//! 2. Poll for user authorization (device code flow)
//! 3. Exchange Microsoft token for Xbox Live token
//! 4. Exchange XBL token for XSTS token
//! 5. Exchange XSTS token for Minecraft access token
//! 6. Fetch Minecraft profile (UUID + username)
//!
//! Tokens are persisted to the OS keychain via the `keyring` crate.
//! Account metadata is stored in SQLite for multi-account support.

use crate::error::AppError;
use crate::models::account::{Account, AccountType};
use crate::state::AppState;
use crate::utils::file;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use tauri::Emitter;
use uuid::Uuid;

// ── OAuth Constants ──────────────────────────────────────────────────────────

/// Microsoft OAuth client ID (public, same as HMCL and other open-source launchers).
const MS_CLIENT_ID: &str = "00000000402b5328";

/// OAuth scope for Xbox Live sign-in with offline access.
const MS_SCOPE: &str = "XboxLive.signin offline_access";

/// Microsoft device code endpoint.
const MS_DEVICE_CODE_URL: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";

/// Microsoft token endpoint.
const MS_TOKEN_URL: &str =
    "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";

/// Xbox Live authentication endpoint.
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";

/// XSTS authorization endpoint.
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

/// Minecraft authentication endpoint (login with Xbox).
const MC_AUTH_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";

/// Minecraft profile endpoint.
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";

/// Keyring service name for token storage.
const KEYRING_SERVICE: &str = "Aurora Launcher";

/// Polling interval for device code token request (seconds).
const DEVICE_CODE_POLL_INTERVAL_SECS: u64 = 5;

/// Maximum polling duration for device code (5 minutes).
const DEVICE_CODE_MAX_POLL_SECS: u64 = 300;

// ── OAuth Response Types ─────────────────────────────────────────────────────

/// Response from the Microsoft device code request.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeviceCodeResponse {
    /// The device code used for polling.
    device_code: String,
    /// The code the user must enter at the verification URL.
    user_code: String,
    /// The URL where the user enters the user_code.
    verification_uri: String,
    /// Seconds between poll requests.
    interval: Option<u64>,
    /// Seconds until the device code expires.
    expires_in: Option<u64>,
}

/// Response from the Microsoft token endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MsaTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    token_type: Option<String>,
}

/// Response from Xbox Live authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct XblAuthResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: XblDisplayClaims,
}

/// Display claims from XBL authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct XblDisplayClaims {
    xui: Vec<XblUserInfo>,
}

/// User info from XBL display claims.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct XblUserInfo {
    uhs: String,
}

/// Response from XSTS authorization.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct XstsAuthResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: XblDisplayClaims,
}

/// Response from Minecraft authentication (login with Xbox).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct McAuthResponse {
    access_token: String,
    expires_in: Option<u64>,
}

/// Minecraft profile from the Mojang API.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct McProfile {
    id: String,
    name: String,
}

/// Token data stored in the OS keychain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTokens {
    /// Microsoft access token.
    pub ms_access_token: String,
    /// Microsoft refresh token.
    pub ms_refresh_token: String,
    /// Minecraft access token.
    pub mc_access_token: String,
    /// Expiration timestamp (Unix epoch seconds).
    pub expires_at: i64,
}

// ── Microsoft Login Flow ─────────────────────────────────────────────────────

/// Start the Microsoft OAuth 2.0 device code flow.
///
/// This function:
/// 1. Requests a device code from Microsoft
/// 2. Emits the verification URL and user code to the frontend
/// 3. Polls for user authorization
/// 4. Completes the XBL → XSTS → MC auth chain
/// 5. Persists the account and tokens
///
/// Returns the created Account on success.
pub async fn login_microsoft(state: &AppState) -> Result<Account, AppError> {
    let http_client = &state.http_client;
    let app_handle = &state.app_handle;

    // Step 1: Request device code
    let device_code = request_device_code(http_client).await?;

    // Emit verification info to the frontend
    let _ = app_handle.emit(
        "account:device_code",
        serde_json::json!({
            "user_code": device_code.user_code,
            "verification_uri": device_code.verification_uri,
        }),
    );

    tracing::info!(
        "Microsoft device code: {} — verify at {}",
        device_code.user_code,
        device_code.verification_uri
    );

    // Step 2: Poll for user authorization
    let msa_token = poll_for_token(
        http_client,
        &device_code.device_code,
        device_code.interval.unwrap_or(DEVICE_CODE_POLL_INTERVAL_SECS),
    )
    .await?;

    // Step 3: Xbox Live authentication
    let (xbl_token, user_hash) = authenticate_xbl(http_client, &msa_token.access_token).await?;

    // Step 4: XSTS authorization
    let (xsts_token, user_hash) = authenticate_xsts(http_client, &xbl_token).await?;

    // Step 5: Minecraft authentication
    let mc_token = authenticate_minecraft(http_client, &user_hash, &xsts_token).await?;

    // Step 6: Fetch Minecraft profile
    let profile = fetch_minecraft_profile(http_client, &mc_token.access_token).await?;

    // Persist the account to the database
    let pool = state
        .db_pool
        .get()
        .ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;

    let account = upsert_microsoft_account(
        pool,
        &profile.name,
        &profile.id,
        &msa_token,
        &mc_token,
    )
    .await?;

    // Persist tokens to OS keychain
    save_tokens_to_keychain(
        &account.id,
        &msa_token.access_token,
        msa_token.refresh_token.as_deref().unwrap_or(""),
        &mc_token.access_token,
        msa_token.expires_in.unwrap_or(3600),
    )?;

    let _ = app_handle.emit("account:login_success", &account);

    tracing::info!(
        "Microsoft login successful: {} ({})",
        account.username,
        account.id
    );

    Ok(account)
}

/// Request a device code from Microsoft's OAuth endpoint.
async fn request_device_code(client: &Client) -> Result<DeviceCodeResponse, AppError> {
    let params = [
        ("client_id", MS_CLIENT_ID),
        ("scope", MS_SCOPE),
    ];

    let response = client
        .post(MS_DEVICE_CODE_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::MicrosoftLoginFailed(format!("Device code request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::MicrosoftLoginFailed(format!(
            "Device code request failed: HTTP {} — {}",
            status, body
        )));
    }

    response
        .json::<DeviceCodeResponse>()
        .await
        .map_err(|e| AppError::MicrosoftLoginFailed(format!("Failed to parse device code response: {}", e)))
}

/// Poll the Microsoft token endpoint until the user authorizes the device code.
async fn poll_for_token(
    client: &Client,
    device_code: &str,
    interval_secs: u64,
) -> Result<MsaTokenResponse, AppError> {
    let start = std::time::Instant::now();
    let max_duration = std::time::Duration::from_secs(DEVICE_CODE_MAX_POLL_SECS);

    loop {
        // Check if we've exceeded the maximum polling duration
        if start.elapsed() > max_duration {
            return Err(AppError::MicrosoftLoginFailed(
                "Device code authorization timed out".to_string(),
            ));
        }

        // Wait before polling
        tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;

        let params = [
            ("client_id", MS_CLIENT_ID),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", device_code),
        ];

        let response = client
            .post(MS_TOKEN_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::MicrosoftLoginFailed(format!("Token poll request failed: {}", e)))?;

        let status = response.status();
        let body = response.text().await.map_err(|e| {
            AppError::MicrosoftLoginFailed(format!("Failed to read token response body: {}", e))
        })?;

        // Parse the response — it could be a token or an error
        if let Ok(token) = serde_json::from_str::<MsaTokenResponse>(&body) {
            return Ok(token);
        }

        // Parse the error response
        if let Ok(error_resp) = serde_json::from_str::<serde_json::Value>(&body) {
            let error_code = error_resp
                .get("error")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown_error");

            match error_code {
                "authorization_pending" => {
                    // User hasn't authorized yet — keep polling
                    tracing::debug!("Authorization pending, continuing to poll...");
                    continue;
                }
                "slow_down" => {
                    // We're polling too fast — wait longer
                    tracing::warn!("Polling too fast, backing off...");
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    continue;
                }
                "expired_token" => {
                    return Err(AppError::MicrosoftLoginFailed(
                        "Device code expired. Please try again.".to_string(),
                    ));
                }
                "declined" => {
                    return Err(AppError::MicrosoftLoginFailed(
                        "Authorization was declined by the user.".to_string(),
                    ));
                }
                _ => {
                    return Err(AppError::MicrosoftLoginFailed(format!(
                        "Token request error: {} — HTTP {}",
                        error_code, status
                    )));
                }
            }
        }

        return Err(AppError::MicrosoftLoginFailed(format!(
            "Unexpected token response: HTTP {} — {}",
            status, body
        )));
    }
}

/// Authenticate with Xbox Live using the Microsoft access token.
async fn authenticate_xbl(
    client: &Client,
    ms_access_token: &str,
) -> Result<(String, String), AppError> {
    let body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={}", ms_access_token)
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });

    let response = client
        .post(XBL_AUTH_URL)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::MicrosoftLoginFailed(format!("XBL auth request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::MicrosoftLoginFailed(format!(
            "XBL authentication failed: HTTP {} — {}",
            status, body
        )));
    }

    let xbl_resp: XblAuthResponse = response.json().await.map_err(|e| {
        AppError::MicrosoftLoginFailed(format!("Failed to parse XBL response: {}", e))
    })?;

    let user_hash = xbl_resp
        .display_claims
        .xui
        .first()
        .map(|u| u.uhs.clone())
        .ok_or_else(|| AppError::MicrosoftLoginFailed("XBL response missing user hash".to_string()))?;

    Ok((xbl_resp.token, user_hash))
}

/// Authenticate with XSTS using the Xbox Live token.
async fn authenticate_xsts(
    client: &Client,
    xbl_token: &str,
) -> Result<(String, String), AppError> {
    let body = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
        "TokenType": "JWT"
    });

    let response = client
        .post(XSTS_AUTH_URL)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::MicrosoftLoginFailed(format!("XSTS auth request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::MicrosoftLoginFailed(format!(
            "XSTS authorization failed: HTTP {} — {}",
            status, body
        )));
    }

    let xsts_resp: XstsAuthResponse = response.json().await.map_err(|e| {
        AppError::MicrosoftLoginFailed(format!("Failed to parse XSTS response: {}", e))
    })?;

    let user_hash = xsts_resp
        .display_claims
        .xui
        .first()
        .map(|u| u.uhs.clone())
        .ok_or_else(|| AppError::MicrosoftLoginFailed("XSTS response missing user hash".to_string()))?;

    Ok((xsts_resp.token, user_hash))
}

/// Authenticate with Minecraft services using the XSTS token and user hash.
async fn authenticate_minecraft(
    client: &Client,
    user_hash: &str,
    xsts_token: &str,
) -> Result<McAuthResponse, AppError> {
    let body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={};{}", user_hash, xsts_token)
    });

    let response = client
        .post(MC_AUTH_URL)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::MicrosoftLoginFailed(format!("MC auth request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::MicrosoftLoginFailed(format!(
            "Minecraft authentication failed: HTTP {} — {}",
            status, body
        )));
    }

    response.json::<McAuthResponse>().await.map_err(|e| {
        AppError::MicrosoftLoginFailed(format!("Failed to parse MC auth response: {}", e))
    })
}

/// Fetch the Minecraft profile using the MC access token.
async fn fetch_minecraft_profile(
    client: &Client,
    mc_access_token: &str,
) -> Result<McProfile, AppError> {
    let response = client
        .get(MC_PROFILE_URL)
        .header("Authorization", format!("Bearer {}", mc_access_token))
        .send()
        .await
        .map_err(|e| AppError::MicrosoftLoginFailed(format!("MC profile request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::MicrosoftLoginFailed(format!(
            "Failed to fetch MC profile: HTTP {} — {}",
            status, body
        )));
    }

    response.json::<McProfile>().await.map_err(|e| {
        AppError::MicrosoftLoginFailed(format!("Failed to parse MC profile: {}", e))
    })
}

// ── Account Database Operations ──────────────────────────────────────────────

/// Create or update a Microsoft account in the database.
async fn upsert_microsoft_account(
    pool: &SqlitePool,
    username: &str,
    uuid: &str,
    msa_token: &MsaTokenResponse,
    mc_token: &McAuthResponse,
) -> Result<Account, AppError> {
    let now = Utc::now().to_rfc3339();

    // Check if an account with this UUID already exists
    let existing = sqlx::query_as::<_, Account>(
        "SELECT * FROM accounts WHERE uuid = ? AND account_type = 'Microsoft'",
    )
    .bind(uuid)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    if let Some(account) = existing {
        // Update existing account
        sqlx::query(
            "UPDATE accounts SET username = ?, display_name = ?, is_active = 1 WHERE id = ?",
        )
        .bind(username)
        .bind(username)
        .bind(&account.id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        // Deactivate other accounts
        sqlx::query("UPDATE accounts SET is_active = 0 WHERE id != ?")
            .bind(&account.id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        // Fetch the updated account
        let updated = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
            .bind(&account.id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(updated)
    } else {
        // Create new Microsoft account
        let id = Uuid::new_v4().to_string();

        // Deactivate all existing accounts
        sqlx::query("UPDATE accounts SET is_active = 0")
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        sqlx::query(
            r#"
            INSERT INTO accounts (id, username, display_name, uuid, account_type, is_active, created_at)
            VALUES (?, ?, ?, ?, 'Microsoft', 1, ?)
            "#,
        )
        .bind(&id)
        .bind(username)
        .bind(username)
        .bind(uuid)
        .bind(&now)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
            .bind(&id)
            .fetch_one(pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(account)
    }
}

/// Create an offline account with a custom username.
pub async fn login_offline(pool: &SqlitePool, username: &str) -> Result<Account, AppError> {
    if username.trim().is_empty() {
        return Err(AppError::OfflineUnavailable(
            "Username cannot be empty".to_string(),
        ));
    }

    if username.len() > 16 {
        return Err(AppError::OfflineUnavailable(
            "Username must be 16 characters or less".to_string(),
        ));
    }

    let now = Utc::now().to_rfc3339();
    let id = Uuid::new_v4().to_string();

    // Generate a deterministic offline UUID from the username
    // Use version 3 UUID (MD5) with a namespace for offline mode
    let offline_uuid = generate_offline_uuid(username);

    // Deactivate all existing accounts
    sqlx::query("UPDATE accounts SET is_active = 0")
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    sqlx::query(
        r#"
        INSERT INTO accounts (id, username, display_name, uuid, account_type, is_active, created_at)
        VALUES (?, ?, ?, ?, 'Offline', 1, ?)
        "#,
    )
    .bind(&id)
    .bind(username)
    .bind(username)
    .bind(&offline_uuid)
    .bind(&now)
    .execute(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
        .bind(&id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    tracing::info!("Offline account created: {} ({})", username, id);
    Ok(account)
}

/// Generate an offline UUID from a username.
///
/// Uses the same algorithm as the vanilla Minecraft launcher:
/// UUID v3 with namespace "OfflinePlayer" + username.
fn generate_offline_uuid(username: &str) -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(b"OfflinePlayer:");
    hasher.update(username.as_bytes());
    let hash = hasher.finalize();

    // Format as UUID (take first 16 bytes and set version/variant bits)
    let bytes = &hash[..16];
    let mut uuid_bytes = [0u8; 16];
    uuid_bytes.copy_from_slice(bytes);

    // Set version to 3 (name-based)
    uuid_bytes[6] = (uuid_bytes[6] & 0x0f) | 0x30;
    // Set variant to RFC 4122
    uuid_bytes[8] = (uuid_bytes[8] & 0x3f) | 0x80;

    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        u32::from_be_bytes([uuid_bytes[0], uuid_bytes[1], uuid_bytes[2], uuid_bytes[3]]),
        u16::from_be_bytes([uuid_bytes[4], uuid_bytes[5]]),
        u16::from_be_bytes([uuid_bytes[6], uuid_bytes[7]]),
        u16::from_be_bytes([uuid_bytes[8], uuid_bytes[9]]),
        u64::from_be_bytes([
            uuid_bytes[10], uuid_bytes[11], uuid_bytes[12], uuid_bytes[13], uuid_bytes[14],
            uuid_bytes[15], 0, 0
        ]) >> 16,
    )
}

/// List all accounts from the database.
pub async fn get_accounts(pool: &SqlitePool) -> Result<Vec<Account>, AppError> {
    let accounts = sqlx::query_as::<_, Account>(
        "SELECT * FROM accounts ORDER BY is_active DESC, created_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(accounts)
}

/// Get the currently active account.
pub async fn get_active_account(pool: &SqlitePool) -> Result<Account, AppError> {
    let account = sqlx::query_as::<_, Account>(
        "SELECT * FROM accounts WHERE is_active = 1 LIMIT 1",
    )
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.to_string()))?;

    account.ok_or_else(|| AppError::OfflineUnavailable("No active account".to_string()))
}

/// Set the active account by ID.
pub async fn set_active_account(pool: &SqlitePool, id: &str) -> Result<Account, AppError> {
    // Verify the account exists
    let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::OfflineUnavailable(format!("Account not found: {}", id)))?;

    // Deactivate all accounts
    sqlx::query("UPDATE accounts SET is_active = 0")
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // Activate the selected account
    sqlx::query("UPDATE accounts SET is_active = 1 WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    tracing::info!("Active account set to: {} ({})", account.username, id);
    Ok(account)
}

/// Log out (delete) an account by ID.
pub async fn logout(pool: &SqlitePool, id: &str) -> Result<(), AppError> {
    let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::OfflineUnavailable(format!("Account not found: {}", id)))?;

    // Remove tokens from keychain (ignore errors if not found)
    let _ = delete_tokens_from_keychain(id);

    // Delete the account from the database
    sqlx::query("DELETE FROM accounts WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    // If the deleted account was active, activate the most recent one
    if account.is_active {
        let _ = sqlx::query("UPDATE accounts SET is_active = 1 WHERE id = (SELECT id FROM accounts ORDER BY created_at DESC LIMIT 1)")
            .execute(pool)
            .await;
    }

    tracing::info!("Account logged out: {} ({})", account.username, id);
    Ok(())
}

// ── Token Refresh ────────────────────────────────────────────────────────────

/// Refresh a Microsoft account's tokens.
///
/// Uses the stored refresh token to obtain new access tokens
/// and re-authenticates the entire chain.
pub async fn refresh_account(state: &AppState, id: &str) -> Result<Account, AppError> {
    let pool = state
        .db_pool
        .get()
        .ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;

    // Fetch the account
    let account = sqlx::query_as::<_, Account>("SELECT * FROM accounts WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::OfflineUnavailable(format!("Account not found: {}", id)))?;

    // Only Microsoft accounts can be refreshed
    if account.account_type != AccountType::Microsoft.as_str() {
        return Err(AppError::TokenRefreshFailed(
            "Only Microsoft accounts can be refreshed".to_string(),
        ));
    }

    // Load tokens from keychain
    let tokens = load_tokens_from_keychain(id)?;

    if tokens.ms_refresh_token.is_empty() {
        return Err(AppError::TokenRefreshFailed(
            "No refresh token found. Please log in again.".to_string(),
        ));
    }

    let http_client = &state.http_client;

    // Refresh the Microsoft access token
    let params = [
        ("client_id", MS_CLIENT_ID),
        ("grant_type", "refresh_token"),
        ("refresh_token", &tokens.ms_refresh_token),
        ("scope", MS_SCOPE),
    ];

    let response = http_client
        .post(MS_TOKEN_URL)
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::TokenRefreshFailed(format!("Token refresh request failed: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::TokenRefreshFailed(format!(
            "Token refresh failed: HTTP {} — {}",
            status, body
        )));
    }

    let msa_token: MsaTokenResponse = response.json().await.map_err(|e| {
        AppError::TokenRefreshFailed(format!("Failed to parse refreshed token: {}", e))
    })?;

    // Re-authenticate the entire chain
    let (xbl_token, user_hash) = authenticate_xbl(http_client, &msa_token.access_token).await?;
    let (xsts_token, user_hash) = authenticate_xsts(http_client, &xbl_token).await?;
    let mc_token = authenticate_minecraft(http_client, &user_hash, &xsts_token).await?;

    // Update tokens in keychain
    save_tokens_to_keychain(
        id,
        &msa_token.access_token,
        msa_token.refresh_token.as_deref().unwrap_or(&tokens.ms_refresh_token),
        &mc_token.access_token,
        msa_token.expires_in.unwrap_or(3600),
    )?;

    tracing::info!("Account tokens refreshed: {} ({})", account.username, id);
    Ok(account)
}

/// Get the Minecraft access token for the active account.
///
/// If the token is expired, attempts to refresh it automatically.
pub async fn get_mc_access_token(state: &AppState) -> Result<(String, String, String), AppError> {
    let pool = state
        .db_pool
        .get()
        .ok_or_else(|| AppError::Database("Database not initialized".to_string()))?;

    let account = get_active_account(pool).await?;

    if account.account_type == AccountType::Offline.as_str() {
        // Offline mode: return the offline UUID
        let uuid = account.uuid.unwrap_or_else(|| generate_offline_uuid(&account.username));
        return Ok((account.username.clone(), uuid, "offline_token".to_string()));
    }

    // Load tokens from keychain
    let tokens = load_tokens_from_keychain(&account.id)?;

    // Check if token is expired (with 60 second buffer)
    let now = Utc::now().timestamp();
    if tokens.expires_at - now < 60 {
        // Token is expired or about to expire — refresh it
        let _ = refresh_account(state, &account.id).await;
        let tokens = load_tokens_from_keychain(&account.id)?;
        let uuid = account.uuid.unwrap_or_default();
        return Ok((account.username.clone(), uuid, tokens.mc_access_token));
    }

    let uuid = account.uuid.unwrap_or_default();
    Ok((account.username.clone(), uuid, tokens.mc_access_token))
}

// ── Keychain Operations ──────────────────────────────────────────────────────

/// Save account tokens to the OS keychain.
fn save_tokens_to_keychain(
    account_id: &str,
    ms_access_token: &str,
    ms_refresh_token: &str,
    mc_access_token: &str,
    expires_in_secs: u64,
) -> Result<(), AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, &format!("account_{}", account_id))
        .map_err(|e| AppError::AuthExpired(format!("Failed to create keyring entry: {}", e)))?;

    let expires_at = Utc::now().timestamp() + expires_in_secs as i64;

    let tokens = AccountTokens {
        ms_access_token: ms_access_token.to_string(),
        ms_refresh_token: ms_refresh_token.to_string(),
        mc_access_token: mc_access_token.to_string(),
        expires_at,
    };

    let json = serde_json::to_string(&tokens)
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    entry
        .set_password(&json)
        .map_err(|e| AppError::AuthExpired(format!("Failed to save tokens to keychain: {}", e)))?;

    Ok(())
}

/// Load account tokens from the OS keychain.
fn load_tokens_from_keychain(account_id: &str) -> Result<AccountTokens, AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, &format!("account_{}", account_id))
        .map_err(|e| AppError::AuthExpired(format!("Failed to create keyring entry: {}", e)))?;

    let json = entry
        .get_password()
        .map_err(|e| AppError::AuthExpired(format!("Failed to load tokens from keychain: {}", e)))?;

    let tokens: AccountTokens = serde_json::from_str(&json)
        .map_err(|e| AppError::Serialization(format!("Failed to parse keychain tokens: {}", e)))?;

    Ok(tokens)
}

/// Delete account tokens from the OS keychain.
fn delete_tokens_from_keychain(account_id: &str) -> Result<(), AppError> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, &format!("account_{}", account_id))
        .map_err(|e| AppError::AuthExpired(format!("Failed to create keyring entry: {}", e)))?;

    entry
        .delete_credential()
        .map_err(|e| AppError::AuthExpired(format!("Failed to delete tokens from keychain: {}", e)))?;

    Ok(())
}
