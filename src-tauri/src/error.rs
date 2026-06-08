//! Unified error types and command response envelope for Aurora Launcher.
//!
//! Error code convention:
//! - 1xxxx: Network errors
//! - 2xxxx: File I/O errors
//! - 3xxxx: Authentication errors
//! - 4xxxx: Launch errors
//! - 5xxxx: Configuration errors
//! - 6xxxx: Database errors
//! - 7xxxx: Serialization errors

use serde::Serialize;
use thiserror::Error;

/// Unified error type for the entire Aurora Launcher application.
#[derive(Debug, Error)]
pub enum AppError {
    // ── Network Errors (1xxxx) ──────────────────────────────
    #[error("网络请求失败: {0}")]
    NetworkRequest(String),

    #[error("网络超时: {0}")]
    NetworkTimeout(String),

    #[error("下载失败: {0}")]
    DownloadFailed(String),

    #[error("哈希校验失败: 期望 {expected}, 实际 {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("URL 无效: {0}")]
    InvalidUrl(String),

    // ── File I/O Errors (2xxxx) ─────────────────────────────
    #[error("文件未找到: {0}")]
    FileNotFound(String),

    #[error("文件读写失败: {0}")]
    FileIo(#[from] std::io::Error),

    #[error("解压缩失败: {0}")]
    DecompressionFailed(String),

    #[error("目录创建失败: {0}")]
    DirectoryCreateFailed(String),

    #[error("文件已存在: {0}")]
    FileExists(String),

    // ── Authentication Errors (3xxxx) ───────────────────────
    #[error("微软登录失败: {0}")]
    MicrosoftLoginFailed(String),

    #[error("令牌刷新失败: {0}")]
    TokenRefreshFailed(String),

    #[error("认证已过期: {0}")]
    AuthExpired(String),

    #[error("离线模式不可用: {0}")]
    OfflineUnavailable(String),

    // ── Launch Errors (4xxxx) ───────────────────────────────
    #[error("Java 未找到: {0}")]
    JavaNotFound(String),

    #[error("版本不支持: {0}")]
    VersionNotSupported(String),

    #[error("启动失败: {0}")]
    LaunchFailed(String),

    #[error("进程已退出: 退出码 {0}")]
    ProcessExited(i32),

    #[error("加载器安装失败: {0}")]
    LoaderInstallFailed(String),

    // ── Configuration Errors (5xxxx) ────────────────────────
    #[error("配置无效: {0}")]
    InvalidConfig(String),

    #[error("实例已存在: {0}")]
    InstanceExists(String),

    #[error("实例未找到: {0}")]
    InstanceNotFound(String),

    #[error("下载任务未找到: {0}")]
    DownloadTaskNotFound(String),

    // ── Database Errors (6xxxx) ─────────────────────────────
    #[error("数据库错误: {0}")]
    Database(String),

    #[error("数据库迁移失败: {0}")]
    DatabaseMigration(String),

    // ── Serialization Errors (7xxxx) ────────────────────────
    #[error("序列化错误: {0}")]
    Serialization(String),
}

impl AppError {
    /// Returns the numeric error code following the convention.
    pub fn code(&self) -> i32 {
        match self {
            // Network
            AppError::NetworkRequest(_) => 10001,
            AppError::NetworkTimeout(_) => 10002,
            AppError::DownloadFailed(_) => 10003,
            AppError::HashMismatch { .. } => 10004,
            AppError::InvalidUrl(_) => 10005,
            // File
            AppError::FileNotFound(_) => 20001,
            AppError::FileIo(_) => 20002,
            AppError::DecompressionFailed(_) => 20003,
            AppError::DirectoryCreateFailed(_) => 20004,
            AppError::FileExists(_) => 20005,
            // Auth
            AppError::MicrosoftLoginFailed(_) => 30001,
            AppError::TokenRefreshFailed(_) => 30002,
            AppError::AuthExpired(_) => 30003,
            AppError::OfflineUnavailable(_) => 30004,
            // Launch
            AppError::JavaNotFound(_) => 40001,
            AppError::VersionNotSupported(_) => 40002,
            AppError::LaunchFailed(_) => 40003,
            AppError::ProcessExited(_) => 40004,
            AppError::LoaderInstallFailed(_) => 40005,
            // Config
            AppError::InvalidConfig(_) => 50001,
            AppError::InstanceExists(_) => 50002,
            AppError::InstanceNotFound(_) => 50003,
            AppError::DownloadTaskNotFound(_) => 50004,
            // Database
            AppError::Database(_) => 60001,
            AppError::DatabaseMigration(_) => 60002,
            // Serialization
            AppError::Serialization(_) => 70001,
        }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Serialization(err.to_string())
    }
}

impl From<url::ParseError> for AppError {
    fn from(err: url::ParseError) -> Self {
        AppError::InvalidUrl(err.to_string())
    }
}

/// The standardized IPC response envelope.
/// All Tauri commands return `CommandResult<T>` which serializes as this shape.
#[derive(Debug, Serialize)]
pub struct CommandResponse<T: Serialize> {
    pub code: i32,
    pub data: T,
    pub message: String,
}

impl<T: Serialize> CommandResponse<T> {
    /// Create a successful response.
    pub fn ok(data: T) -> Self {
        Self {
            code: 0,
            data,
            message: "success".to_string(),
        }
    }

    /// Create a successful response with a custom message.
    pub fn ok_with_message(data: T, message: String) -> Self {
        Self {
            code: 0,
            data,
            message,
        }
    }
}

/// Type alias for command results that follow the IPC envelope convention.
pub type CommandResult<T> = Result<CommandResponse<T>, AppError>;

/// Make AppError serializable so Tauri can send it to the frontend.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct ErrorPayload {
            code: i32,
            message: String,
        }
        let payload = ErrorPayload {
            code: self.code(),
            message: self.to_string(),
        };
        payload.serialize(serializer)
    }
}
