import { defineStore } from "pinia";
import { ref, type Ref } from "vue";
import { tauriCommand, type CommandError } from "@/composables/useTauriCommand";
import type { AppSettings, DownloadMirror } from "@/types/settings";
import { DEFAULT_SETTINGS } from "@/types/settings";

export const useSettingsStore = defineStore("settings", () => {
  // ── State ───────────────────────────────────────────────
  const settings: Ref<AppSettings> = ref({ ...DEFAULT_SETTINGS });
  const loading: Ref<boolean> = ref(false);
  const error: Ref<string | null> = ref(null);

  // ── Computed-like helpers ───────────────────────────────

  function getMaxMemory(): number {
    return settings.value.default_max_memory;
  }

  function getMinMemory(): number {
    return settings.value.default_min_memory;
  }

  function getDownloadMirror(): DownloadMirror {
    return settings.value.download_mirror;
  }

  function getLanguage(): string {
    return settings.value.language;
  }

  function getJavaPath(): string | null {
    return settings.value.custom_java_path;
  }

  // ── Actions ────────────────────────────────────────────

  /** Load all settings from the backend. */
  async function fetchSettings(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      settings.value = await tauriCommand<AppSettings>("get_settings");
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
      // Fall back to defaults on error
      settings.value = { ...DEFAULT_SETTINGS };
    } finally {
      loading.value = false;
    }
  }

  /** Update application settings. */
  async function saveSettings(updates: Partial<AppSettings>): Promise<boolean> {
    loading.value = true;
    error.value = null;
    try {
      const merged: AppSettings = { ...settings.value, ...updates };
      await tauriCommand<void>("update_settings", { settings: merged });
      settings.value = merged;
      return true;
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
      return false;
    } finally {
      loading.value = false;
    }
  }

  /** Reset all settings to defaults. */
  async function resetSettings(): Promise<boolean> {
    loading.value = true;
    error.value = null;
    try {
      const defaults = await tauriCommand<AppSettings>("reset_settings");
      settings.value = defaults;
      return true;
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
      return false;
    } finally {
      loading.value = false;
    }
  }

  return {
    settings,
    loading,
    error,
    getMaxMemory,
    getMinMemory,
    getDownloadMirror,
    getLanguage,
    getJavaPath,
    fetchSettings,
    saveSettings,
    resetSettings,
  };
});
