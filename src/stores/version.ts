import { defineStore } from "pinia";
import { ref, type Ref } from "vue";
import { tauriCommand, type CommandError } from "@/composables/useTauriCommand";
import type {
  VersionManifest,
  VersionEntry,
  VersionDetail,
  VersionTypeFilter,
} from "@/types/version";

export const useVersionStore = defineStore("version", () => {
  // ── State ───────────────────────────────────────────────
  const manifest: Ref<VersionManifest | null> = ref(null);
  const currentDetail: Ref<VersionDetail | null> = ref(null);
  const loading: Ref<boolean> = ref(false);
  const error: Ref<string | null> = ref(null);

  // ── Getters ─────────────────────────────────────────────

  /** Get filtered versions by type. */
  function getFilteredVersions(type: VersionTypeFilter = 'all'): VersionEntry[] {
    if (!manifest.value) return [];
    if (type === 'all') return manifest.value.versions;
    return manifest.value.versions.filter((v) => v.type === type);
  }

  /** Get the latest release version ID. */
  function getLatestRelease(): string {
    return manifest.value?.latest.release ?? '';
  }

  /** Get the latest snapshot version ID. */
  function getLatestSnapshot(): string {
    return manifest.value?.latest.snapshot ?? '';
  }

  // ── Actions ────────────────────────────────────────────

  /** Load the full version manifest from the backend. */
  async function loadManifest(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      manifest.value = await tauriCommand<VersionManifest>("get_version_manifest");
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
    } finally {
      loading.value = false;
    }
  }

  /** Load detailed version information by URL. */
  async function loadVersionDetail(versionUrl: string): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      currentDetail.value = await tauriCommand<VersionDetail>("get_version_detail", {
        versionUrl,
      });
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
    } finally {
      loading.value = false;
    }
  }

  return {
    manifest,
    currentDetail,
    loading,
    error,
    getFilteredVersions,
    getLatestRelease,
    getLatestSnapshot,
    loadManifest,
    loadVersionDetail,
  };
});
