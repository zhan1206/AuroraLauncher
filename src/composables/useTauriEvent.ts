import { onUnmounted, type Ref, ref } from "vue";
import { listen, type UnlistenFn, type EventCallback } from "@tauri-apps/api/event";

/**
 * Composable for listening to Tauri backend events within a Vue component.
 *
 * Automatically unregisters the listener when the component is unmounted.
 *
 * Usage:
 * ```ts
 * const { onEvent } = useTauriEvent();
 * onEvent<DownloadProgress>("download:progress", (payload) => {
 *   console.log(payload.downloaded_bytes);
 * });
 * ```
 */
export function useTauriEvent() {
  const unlisteners: UnlistenFn[] = [];

  /**
   * Register a listener for a Tauri event.
   *
   * @param event   - The event name (must match the Rust-side `app_handle.emit()` name).
   * @param handler - Callback invoked with the event payload.
   */
  async function onEvent<T>(event: string, handler: EventCallback<T>): Promise<void> {
    const unlisten = await listen<T>(event, handler);
    unlisteners.push(unlisten);
  }

  /**
   * Register a listener and bind the latest event payload to a reactive ref.
   *
   * @param event       - The event name.
   * @param initialValue - The initial value for the ref before any event is received.
   * @returns           A reactive ref that updates whenever a new event payload arrives.
   */
  async function bindEvent<T>(event: string, initialValue: T): Promise<Ref<T>> {
    const data = ref(initialValue) as Ref<T>;
    const unlisten = await listen<T>(event, (e) => {
      data.value = e.payload;
    });
    unlisteners.push(unlisten);
    return data;
  }

  // Auto-cleanup on component unmount
  onUnmounted(() => {
    for (const unlisten of unlisteners) {
      unlisten();
    }
    unlisteners.length = 0;
  });

  return {
    onEvent,
    bindEvent,
  };
}
