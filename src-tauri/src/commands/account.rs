//! Account-related Tauri commands.
//!
//! 提供微软登录、离线登录、账号管理相关的 IPC 命令。

use crate::error::{CommandResponse, CommandResult};
use crate::models::account::Account;
use crate::services::account_service;
use crate::state::AppState;
use crate::utils::db_pool;
use tauri::State;

/// 微软登录（OAuth 2.0 设备代码流）。
///
/// 启动微软登录流程，完成后返回创建的账号信息。
#[tauri::command]
pub async fn login_microsoft(
    state: State<'_, AppState>,
) -> CommandResult<Account> {
    let account = account_service::login_microsoft(&state).await?;
    Ok(CommandResponse::ok(account))
}

/// 离线登录。
///
/// 使用自定义用户名创建离线账号，无需网络认证。
#[tauri::command]
pub async fn login_offline(
    state: State<'_, AppState>,
    username: String,
) -> CommandResult<Account> {
    let pool = db_pool::get_pool(&state)?;
    let account = account_service::login_offline(pool, &username).await?;
    Ok(CommandResponse::ok(account))
}

/// 获取所有账号列表。
///
/// 返回所有已登录的账号，按激活状态和创建时间排序。
#[tauri::command]
pub async fn get_accounts(
    state: State<'_, AppState>,
) -> CommandResult<Vec<Account>> {
    let pool = db_pool::get_pool(&state)?;
    let accounts = account_service::get_accounts(pool).await?;
    Ok(CommandResponse::ok(accounts))
}

/// 获取当前激活的账号。
///
/// 返回当前正在使用的账号信息。如果没有激活账号则返回错误。
#[tauri::command]
pub async fn get_active_account(
    state: State<'_, AppState>,
) -> CommandResult<Account> {
    let pool = db_pool::get_pool(&state)?;
    let account = account_service::get_active_account(pool).await?;
    Ok(CommandResponse::ok(account))
}

/// 设置激活账号。
///
/// 将指定 ID 的账号设为激活状态，其他账号自动取消激活。
#[tauri::command]
pub async fn set_active_account(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<Account> {
    let pool = db_pool::get_pool(&state)?;
    let account = account_service::set_active_account(pool, &id).await?;
    Ok(CommandResponse::ok(account))
}

/// 登出并删除账号。
///
/// 删除指定 ID 的账号，同时清除 OS 密钥链中的令牌。
#[tauri::command]
pub async fn logout(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<()> {
    let pool = db_pool::get_pool(&state)?;
    account_service::logout(pool, &id).await?;
    Ok(CommandResponse::ok(()))
}

/// 刷新账号令牌。
///
/// 仅支持微软账号。使用存储的 refresh token 重新认证整个链路。
#[tauri::command]
pub async fn refresh_account(
    state: State<'_, AppState>,
    id: String,
) -> CommandResult<Account> {
    let account = account_service::refresh_account(&state, &id).await?;
    Ok(CommandResponse::ok(account))
}
