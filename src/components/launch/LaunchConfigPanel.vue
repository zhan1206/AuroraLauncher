<script setup lang="ts">
/**
 * LaunchConfigPanel — Configuration panel for instance launch settings.
 * Memory/JVM settings are read from and written to instance.launch_config (JSON string).
 */
import { ref, watch, computed } from 'vue';
import PixelInput from '@/components/common/PixelInput.vue';
import PixelButton from '@/components/common/PixelButton.vue';
import { useInstanceStore } from '@/stores/instance';
import { formatMemory } from '@/utils/formatters';
import { validateMemoryPair, validateJvmArgs } from '@/utils/validators';
import { useToast } from '@/components/common/useToast';
import type { LaunchConfig } from '@/types/instance';

export interface LaunchConfigPanelProps {
  /** The instance ID to configure. */
  instanceId: string;
}

const props = defineProps<LaunchConfigPanelProps>();

const instanceStore = useInstanceStore();
const toast = useToast();

/** Parse launch_config JSON or return defaults. */
function parseLaunchConfig(raw: string): LaunchConfig {
  try {
    return JSON.parse(raw) as LaunchConfig;
  } catch {
    return { min_memory: 512, max_memory: 2048, jvm_args: [], game_args: [], fullscreen: false, width: 854, height: 480 };
  }
}

// Form state
const minMemory = ref(512);
const maxMemory = ref(2048);
const jvmArgsStr = ref('');
const javaId = ref<string | null>(null);
const saving = ref(false);

// Initialize from instance data
watch(
  () => instanceStore.current,
  (instance) => {
    if (instance && instance.id === props.instanceId) {
      const cfg = parseLaunchConfig(instance.launch_config);
      minMemory.value = cfg.min_memory;
      maxMemory.value = cfg.max_memory;
      jvmArgsStr.value = cfg.jvm_args.join(' ');
      javaId.value = instance.java_id;
    }
  },
  { immediate: true }
);

// Validation
const memoryError = computed(() => {
  const result = validateMemoryPair(minMemory.value, maxMemory.value);
  return result.valid ? '' : result.message;
});

const jvmError = computed(() => {
  if (!jvmArgsStr.value) return '';
  const result = validateJvmArgs(jvmArgsStr.value);
  return result.valid ? '' : result.message;
});

/** Build updated LaunchConfig from form fields. */
function buildConfig(): LaunchConfig {
  const instance = instanceStore.current;
  const base = instance ? parseLaunchConfig(instance.launch_config) : {} as LaunchConfig;
  return {
    ...base,
    min_memory: minMemory.value,
    max_memory: maxMemory.value,
    jvm_args: jvmArgsStr.value ? jvmArgsStr.value.split(/\s+/).filter(Boolean) : [],
  };
}

/** Save the launch config. */
async function handleSave(): Promise<void> {
  saving.value = true;
  try {
    await instanceStore.updateInstance(props.instanceId, {
      launch_config: buildConfig(),
    });
    toast.success('启动配置已保存');
  } catch (e) {
    toast.error('保存失败');
  } finally {
    saving.value = false;
  }
}

/** Auto-detect Java via list_java_runtimes command. */
async function detectJava(): Promise<void> {
  try {
    const { tauriCommand } = await import('@/composables/useTauriCommand');
    const runtimes = await tauriCommand<Array<{ id: string; path: string; version: string }>>('list_java_runtimes');
    if (runtimes.length > 0) {
      javaId.value = runtimes[0].id;
      toast.info(`已检测到 Java ${runtimes[0].version}`);
      await instanceStore.updateInstance(props.instanceId, { java_id: runtimes[0].id });
    } else {
      toast.warning('未检测到 Java，请手动指定');
    }
  } catch {
    toast.warning('未检测到 Java，请手动指定');
  }
}
</script>

<template>
  <div class="launch-config-panel">
    <!-- Memory config -->
    <div class="launch-config-panel__section">
      <label class="launch-config-panel__label">内存配置</label>
      <div class="launch-config-panel__memory">
        <div class="launch-config-panel__memory-field">
          <label class="launch-config-panel__sublabel">最小内存 (MB)</label>
          <input
            v-model.number="minMemory"
            type="number"
            class="input-pixel"
            min="128"
            step="128"
          />
          <span class="launch-config-panel__unit">{{ formatMemory(minMemory) }}</span>
        </div>
        <span class="launch-config-panel__range-sep">—</span>
        <div class="launch-config-panel__memory-field">
          <label class="launch-config-panel__sublabel">最大内存 (MB)</label>
          <input
            v-model.number="maxMemory"
            type="number"
            class="input-pixel"
            min="128"
            step="128"
          />
          <span class="launch-config-panel__unit">{{ formatMemory(maxMemory) }}</span>
        </div>
      </div>
      <p v-if="memoryError" class="launch-config-panel__error">{{ memoryError }}</p>
    </div>

    <!-- Java selection -->
    <div class="launch-config-panel__section">
      <label class="launch-config-panel__label">Java 运行时</label>
      <div class="launch-config-panel__java-row">
        <span class="launch-config-panel__java-id input-pixel" style="flex:1; display:flex; align-items:center; padding: 6px 10px;">
          {{ javaId ?? '未指定（将自动选择）' }}
        </span>
        <PixelButton size="sm" @click="detectJava">自动检测</PixelButton>
      </div>
    </div>

    <!-- JVM arguments -->
    <div class="launch-config-panel__section">
      <PixelInput
        v-model="jvmArgsStr"
        label="JVM 额外参数"
        placeholder="例: -XX:+UseG1GC -XX:ParallelGCThreads=4"
        :error="jvmError"
      />
    </div>

    <!-- Save button -->
    <div class="launch-config-panel__actions">
      <PixelButton
        variant="primary"
        :loading="saving"
        :disabled="!!memoryError || !!jvmError"
        @click="handleSave"
      >
        💾 保存配置
      </PixelButton>
    </div>
  </div>
</template>

<style scoped>
.launch-config-panel {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.launch-config-panel__section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.launch-config-panel__label {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  font-weight: bold;
}

.launch-config-panel__memory {
  display: flex;
  align-items: flex-end;
  gap: 12px;
}

.launch-config-panel__memory-field {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.launch-config-panel__sublabel {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.launch-config-panel__unit {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.launch-config-panel__range-sep {
  font-size: var(--font-size-lg);
  color: var(--color-text-muted);
  padding-bottom: 8px;
}

.launch-config-panel__java-row {
  display: flex;
  gap: 8px;
  align-items: center;
}

.launch-config-panel__error {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-danger);
}

.launch-config-panel__actions {
  display: flex;
  justify-content: flex-end;
  margin-top: 8px;
}
</style>
