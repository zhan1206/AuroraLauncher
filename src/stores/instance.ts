import { defineStore } from "pinia";
import { ref, type Ref } from "vue";
import { tauriCommand, type CommandError } from "@/composables/useTauriCommand";
import type {
  Instance,
  CreateInstanceRequest,
  UpdateInstanceRequest,
  InstanceStatus,
} from "@/types/instance";

/** Map of instance ID to its runtime status. */
const instanceStatusMap = new Map<string, InstanceStatus>();

export const useInstanceStore = defineStore("instance", () => {
  // ── State ───────────────────────────────────────────────
  const instances: Ref<Instance[]> = ref([]);
  const current: Ref<Instance | null> = ref(null);
  const loading: Ref<boolean> = ref(false);
  const error: Ref<string | null> = ref(null);

  // ── Getters (functions for non-reactive derived state) ──

  /** Get status of an instance by ID. */
  function getInstanceStatus(id: string): InstanceStatus {
    return instanceStatusMap.get(id) ?? 'idle';
  }

  /** Set the runtime status of an instance. */
  function setInstanceStatus(id: string, status: InstanceStatus): void {
    instanceStatusMap.set(id, status);
  }

  // ── Actions ────────────────────────────────────────────

  /** Load all instances from the backend. */
  async function loadInstances(): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      instances.value = await tauriCommand<Instance[]>("list_instances");
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
    } finally {
      loading.value = false;
    }
  }

  /** Select an instance by ID, fetching from backend. */
  async function selectInstance(id: string): Promise<void> {
    loading.value = true;
    error.value = null;
    try {
      current.value = await tauriCommand<Instance>("get_instance", { id });
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
    } finally {
      loading.value = false;
    }
  }

  /** Create a new instance. */
  async function createInstance(data: CreateInstanceRequest): Promise<Instance | null> {
    loading.value = true;
    error.value = null;
    try {
      const instance = await tauriCommand<Instance>("create_instance", { request: data });
      instances.value.unshift(instance);
      return instance;
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
      return null;
    } finally {
      loading.value = false;
    }
  }

  /** Update an existing instance. */
  async function updateInstance(id: string, data: Omit<UpdateInstanceRequest, 'id'>): Promise<Instance | null> {
    loading.value = true;
    error.value = null;
    try {
      const request: UpdateInstanceRequest = { id, ...data };
      const updated = await tauriCommand<Instance>("update_instance", { request });
      const idx = instances.value.findIndex((i) => i.id === updated.id);
      if (idx >= 0) {
        instances.value[idx] = updated;
      }
      if (current.value?.id === updated.id) {
        current.value = updated;
      }
      return updated;
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
      return null;
    } finally {
      loading.value = false;
    }
  }

  /** Delete an instance by ID. */
  async function deleteInstance(id: string): Promise<boolean> {
    loading.value = true;
    error.value = null;
    try {
      await tauriCommand<void>("delete_instance", { id });
      instances.value = instances.value.filter((i) => i.id !== id);
      if (current.value?.id === id) {
        current.value = null;
      }
      instanceStatusMap.delete(id);
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
    instances,
    current,
    loading,
    error,
    getInstanceStatus,
    setInstanceStatus,
    loadInstances,
    selectInstance,
    createInstance,
    updateInstance,
    deleteInstance,
  };
});
