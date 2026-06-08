//! 游戏启动相关 Tauri 命令。
//!
//! 提供游戏启动、终止和状态查询的 IPC 命令。

use crate::error::{CommandResponse, CommandResult};
use crate::models::launch::LaunchCommand;
use crate::services::launch_service::{self, LaunchStatus};
use crate::state::AppState;
use tauri::State;

/// 启动游戏实例。
///
/// 根据指定实例 ID 加载配置、解析版本链、组装 classpath 并启动 Java 进程。
/// 启动成功后在后台监控进程输出，并通过 `game:log` / `launch:exited` 事件推送给前端。
#[tauri::command]
pub async fn launch_game(
    instance_id: String,
    state: State<'_, AppState>,
) -> CommandResult<LaunchCommand> {
    let cmd = launch_service::launch_game(&state, &instance_id, &state.game_process).await?;
    Ok(CommandResponse::ok(cmd))
}

/// 强制终止当前运行的游戏进程。
///
/// 如果没有正在运行的进程则返回错误。
#[tauri::command]
pub async fn kill_game(state: State<'_, AppState>) -> CommandResult<()> {
    launch_service::kill_game(&state.game_process).await?;
    Ok(CommandResponse::ok(()))
}

/// 查询当前游戏启动状态。
///
/// 返回是否有进程正在运行、PID 和实例 ID。
#[tauri::command]
pub async fn get_launch_status(state: State<'_, AppState>) -> CommandResult<LaunchStatus> {
    let status = launch_service::get_launch_status(&state.game_process).await;
    Ok(CommandResponse::ok(status))
}
