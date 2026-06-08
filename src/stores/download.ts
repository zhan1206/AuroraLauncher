import { defineStore } from "pinia";
import { ref, computed, type Ref } from "vue";
import { tauriCommand, type CommandError } from "@/composables/useTauriCommand";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { DownloadTask, DownloadProgress } from "@/types/download";

export const useDownloadStore = defineStore("download", () => {
  // ── State ───────────────────────────────────────────────
  const tasks: Ref<DownloadTask[]> = ref([]);
  const loading: Ref<boolean> = ref(false);
  const error: Ref<string | null> = ref(null);

  /** Map of task ID to live progress data. */
  const progressMap: Ref<Map<string, DownloadProgress>> = ref(new Map());

  /** Unlisteners for Tauri events. */
  const unlisteners: UnlistenFn[] = [];

  // ── Getters ─────────────────────────────────────────────

  /** Number of actively downloading tasks. */
  const activeCount = computed(() =>
    tasks.value.filter((t) => t.status === "Downloading").length
  );

  /** Number of pending tasks. */
  const pendingCount = computed(() =>
    tasks.value.filter((t) => t.status === "Pending").length
  );

  /** Get progress for a specific task. */
  function getProgress(taskId: string): DownloadProgress | null {
    return progressMap.value.get(taskId) ?? null;
  }

  // ── Actions ────────────────────────────────────────────

  /** Load all download tasks from the backend. */
  async function fetchTasks(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      tasks.value = await tauriCommand<DownloadTask[]>("list_download_tasks");
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
    } finally {
      loading.value = false;
    }
  }

  /** Pause a download task. */
  async function pauseTask(id: string): Promise<void> {
    try {
      await tauriCommand<void>("pause_download", { taskId: id });
      const task = tasks.value.find((t) => t.id === id);
      if (task) {
        task.status = "Paused";
      }
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
    }
  }

  /** Resume a paused download task. */
  async function resumeTask(id: string): Promise<void> {
    try {
      await tauriCommand<void>("resume_download", { taskId: id });
      const task = tasks.value.find((t) => t.id === id);
      if (task) {
        task.status = "Downloading";
      }
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
    }
  }

  /** Cancel a download task. */
  async function cancelTask(id: string): Promise<void> {
    try {
      await tauriCommand<void>("cancel_download", { taskId: id });
      const task = tasks.value.find((t) => t.id === id);
      if (task) {
        task.status = "Cancelled";
      }
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
    }
  }

  /** Start listening for download progress events from Tauri. */
  async function startListening(): Promise<void> {
    // 事件名与 Rust backend 的 app_handle.emit() 保持一致
    // Rust: "download:progress" — payload matches DownloadProgress struct
    const unlisten = await listen<DownloadProgress>("download:progress", (event) => {
      const payload = event.payload;
      progressMap.value.set(payload.task_id, payload);

      // Update the task's downloaded field in the tasks array
      const task = tasks.value.find((t) => t.id === payload.task_id);
      if (task) {
        task.downloaded = payload.downloaded;
        // If 100% complete, mark as Completed
        if (payload.percent >= 100.0) {
          task.status = "Completed";
        }
      }
    });
    unlisteners.push(unlisten);
  }

  /** Stop listening for events and clean up. */
  function stopListening(): void {
    for (const unlisten of unlisteners) {
      unlisten();
    }
    unlisteners.length = 0;
  }

  return {
    tasks,
    loading,
    error,
    progressMap,
    activeCount,
    pendingCount,
    getProgress,
    fetchTasks,
    pauseTask,
    resumeTask,
    cancelTask,
    startListening,
    stopListening,
  };
});
