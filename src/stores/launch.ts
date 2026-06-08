import { defineStore } from "pinia";
import { ref, computed, type Ref } from "vue";
import { tauriCommand, type CommandError } from "@/composables/useTauriCommand";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { LaunchStatus, GameLogEntry } from "@/types/launch";

// ── Rust 事件 payload 类型 ───────────────────────────────────

interface GameLogPayload {
  line: string;
  level: "info" | "error" | "warn";
}
interface LaunchStartedPayload {
  pid: number;
  instance_id: string;
}
interface LaunchExitedPayload {
  code: number;
  instance_id: string;
}

export const useLaunchStore = defineStore("launch", () => {
  // ── State ───────────────────────────────────────────────
  const status: Ref<LaunchStatus> = ref("idle");
  const currentInstanceId: Ref<string | null> = ref(null);
  const logs: Ref<GameLogEntry[]> = ref([]);
  const error: Ref<string | null> = ref(null);

  /** Unlisteners for Tauri events. */
  const unlisteners: UnlistenFn[] = [];

  // ── Getters ─────────────────────────────────────────────

  const isLaunching = computed(() => status.value === "launching" || status.value === "preparing");
  const isRunning = computed(() => status.value === "running");
  const isActive = computed(() => status.value !== "idle" && status.value !== "exited" && status.value !== "crashed");

  // ── Actions ────────────────────────────────────────────

  async function launch(instanceId: string): Promise<void> {
    status.value = "preparing";
    currentInstanceId.value = instanceId;
    error.value = null;
    logs.value = [];
    try {
      await tauriCommand<void>("launch_game", { instanceId });
      status.value = "launching";
      startListening();
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
      status.value = "crashed";
    }
  }

  async function kill(): Promise<void> {
    try {
      await tauriCommand<void>("kill_game");
      status.value = "exited";
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
    }
  }

  function clearLogs(): void {
    logs.value = [];
  }

  function resetState(): void {
    status.value = "idle";
    currentInstanceId.value = null;
    error.value = null;
    logs.value = [];
    stopListening();
  }

  function startListening(): void {
    // Rust: "game:log"
    const unlistenLog = listen<GameLogPayload>("game:log", (event) => {
      const levelMap: Record<string, GameLogEntry["level"]> = {
        info: "INFO", error: "ERROR", warn: "WARN",
      };
      const entry: GameLogEntry = {
        timestamp: new Date().toISOString(),
        level: levelMap[event.payload.level] ?? "INFO",
        source: "game",
        message: event.payload.line,
      };
      logs.value.push(entry);
      if (logs.value.length > 1000) {
        logs.value = logs.value.slice(-500);
      }
    });
    unlistenLog.then((u) => unlisteners.push(u));

    // Rust: "launch:started"
    const unlistenStarted = listen<LaunchStartedPayload>("launch:started", () => {
      status.value = "running";
    });
    unlistenStarted.then((u) => unlisteners.push(u));

    // Rust: "launch:exited"
    const unlistenExit = listen<LaunchExitedPayload>("launch:exited", (event) => {
      if (event.payload.code === 0) {
        status.value = "exited";
      } else {
        status.value = "crashed";
        error.value = `游戏异常退出，退出码: ${event.payload.code}`;
      }
      stopListening();
    });
    unlistenExit.then((u) => unlisteners.push(u));
  }

  function stopListening(): void {
    for (const unlisten of unlisteners) {
      unlisten();
    }
    unlisteners.length = 0;
  }

  return {
    status, currentInstanceId, logs, error,
    isLaunching, isRunning, isActive,
    launch, kill, clearLogs, resetState,
    startListening, stopListening,
  };
});
