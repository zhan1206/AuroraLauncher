/**
 * Toast notification system for Aurora Launcher.
 * Provides a global toast singleton with success/error/info/warning methods.
 */
import { ref, type Ref } from 'vue';

/** Toast severity levels. */
export type ToastType = 'success' | 'error' | 'info' | 'warning';

/** A single toast notification. */
export interface ToastItem {
  id: number;
  type: ToastType;
  message: string;
  duration: number;
  createdAt: number;
}

/** Reactive toast list. */
const toasts: Ref<ToastItem[]> = ref([]);

/** Auto-incrementing ID counter. */
let nextId = 0;

/** Remove a toast by ID. */
function removeToast(id: number): void {
  const idx = toasts.value.findIndex((t) => t.id === id);
  if (idx >= 0) {
    toasts.value.splice(idx, 1);
  }
}

/** Add a toast notification. */
function addToast(type: ToastType, message: string, duration: number = 3000): void {
  const id = nextId++;
  const toast: ToastItem = {
    id,
    type,
    message,
    duration,
    createdAt: Date.now(),
  };
  toasts.value.push(toast);

  // Auto-dismiss after duration
  if (duration > 0) {
    setTimeout(() => {
      removeToast(id);
    }, duration);
  }
}

/** Toast API. */
export const useToast = () => ({
  toasts,
  success: (message: string, duration?: number) => addToast('success', message, duration),
  error: (message: string, duration?: number) => addToast('error', message, duration),
  info: (message: string, duration?: number) => addToast('info', message, duration),
  warning: (message: string, duration?: number) => addToast('warning', message, duration),
  remove: removeToast,
});
