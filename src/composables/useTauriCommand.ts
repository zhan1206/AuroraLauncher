import { invoke, type InvokeArgs } from "@tauri-apps/api/core";

/**
 * Standardized IPC response envelope matching the Rust backend's CommandResponse<T>.
 */
export interface CommandResponse<T> {
  code: number;
  data: T;
  message: string;
}

/**
 * Serialized error from the Rust backend's AppError type.
 */
export interface CommandError {
  code: number;
  message: string;
}

/**
 * Type-safe wrapper around Tauri's `invoke` function.
 *
 * All backend commands return `CommandResponse<T>` on success,
 * or throw an error whose `code` and `message` fields come from `AppError`.
 *
 * @param command - The Tauri command name (must match the Rust `#[tauri::command]` fn name).
 * @param args   - Optional arguments forwarded to the command.
 * @returns       The typed data payload on success.
 * @throws        An object with `code` and `message` on failure.
 */
export async function tauriCommand<T>(command: string, args?: InvokeArgs): Promise<T> {
  try {
    const response = await invoke<CommandResponse<T>>(command, args);

    // The backend always returns code=0 for success
    if (response.code !== 0) {
      throw {
        code: response.code,
        message: response.message,
      } as CommandError;
    }

    return response.data;
  } catch (error: unknown) {
    // Tauri serializes AppError as { code, message }
    if (typeof error === "object" && error !== null && "code" in error && "message" in error) {
      const cmdError = error as CommandError;
      console.error(`[Aurora] Command "${command}" failed (code ${cmdError.code}): ${cmdError.message}`);
      throw cmdError;
    }

    // Unexpected error shape
    const fallback: CommandError = {
      code: -1,
      message: String(error),
    };
    console.error(`[Aurora] Command "${command}" unexpected error:`, error);
    throw fallback;
  }
}

/**
 * Composable that provides the `tauriCommand` helper.
 *
 * Usage in a Vue component:
 * ```ts
 * const { command } = useTauriCommand();
 * const data = await command<MyType>("my_command", { arg: "value" });
 * ```
 */
export function useTauriCommand() {
  return {
    command: tauriCommand,
  };
}
