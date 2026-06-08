<script setup lang="ts">
/**
 * InstanceEditForm — Form for editing instance configuration.
 * Memory and JVM settings are stored in instance.launch_config (JSON string).
 */
import { ref, watch, computed } from 'vue';
import PixelInput from '@/components/common/PixelInput.vue';
import PixelButton from '@/components/common/PixelButton.vue';
import { useInstanceStore } from '@/stores/instance';
import { validateInstanceName, validateMemoryPair, validateJvmArgs } from '@/utils/validators';
import { formatMemory } from '@/utils/formatters';
import { useToast } from '@/components/common/useToast';
import type { Instance, LaunchConfig } from '@/types/instance';

export interface InstanceEditFormProps {
  /** The instance to edit. */
  instance: Instance;
}

const props = defineProps<InstanceEditFormProps>();

const emit = defineEmits<{
  updated: [];
  cancel: [];
}>();

const instanceStore = useInstanceStore();
const toast = useToast();

/** Parse launch_config JSON or return defaults. */
function parseLaunchConfig(instance: Instance): LaunchConfig {
  try {
    return JSON.parse(instance.launch_config) as LaunchConfig;
  } catch {
    return { min_memory: 512, max_memory: 2048, jvm_args: [], game_args: [], fullscreen: false, width: 854, height: 480 };
  }
}

// Form state
const name = ref(props.instance.name);
const launchCfg = ref<LaunchConfig>(parseLaunchConfig(props.instance));
const jvmArgsStr = ref(launchCfg.value.jvm_args.join(' '));
const saving = ref(false);

// Sync form values when instance changes
watch(
  () => props.instance,
  (inst) => {
    name.value = inst.name;
    launchCfg.value = parseLaunchConfig(inst);
    jvmArgsStr.value = launchCfg.value.jvm_args.join(' ');
  }
);

// Validation
const nameError = computed(() => {
  if (!name.value) return '';
  const result = validateInstanceName(name.value);
  return result.valid ? '' : result.message;
});

const memoryError = computed(() => {
  const result = validateMemoryPair(launchCfg.value.min_memory, launchCfg.value.max_memory);
  return result.valid ? '' : result.message;
});

const jvmError = computed(() => {
  if (!jvmArgsStr.value) return '';
  const result = validateJvmArgs(jvmArgsStr.value);
  return result.valid ? '' : result.message;
});

const canSave = computed(() =>
  name.value.trim().length > 0 &&
  !nameError.value &&
  !memoryError.value &&
  !jvmError.value &&
  !saving.value
);

/** Save the edited instance. */
async function handleSave(): Promise<void> {
  if (!canSave.value) return;

  saving.value = true;
  try {
    const newCfg: LaunchConfig = {
      ...launchCfg.value,
      jvm_args: jvmArgsStr.value
        ? jvmArgsStr.value.split(/\s+/).filter(Boolean)
        : [],
    };

    const result = await instanceStore.updateInstance(props.instance.id, {
      name: name.value.trim(),
      launch_config: newCfg,
    });

    if (result) {
      toast.success('实例配置已保存');
      emit('updated');
    }
  } catch (e) {
    toast.error('保存失败');
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="instance-edit-form">
    <PixelInput
      v-model="name"
      label="实例名称"
      placeholder="输入实例名称..."
      :error="nameError"
    />

    <div class="instance-edit-form__section">
      <label class="instance-edit-form__label">内存配置</label>
      <div class="instance-edit-form__memory-row">
        <div class="instance-edit-form__memory-field">
          <label class="instance-edit-form__sublabel">最小内存</label>
          <input
            v-model.number="launchCfg.min_memory"
            type="number"
            class="input-pixel"
            min="128"
            step="128"
          />
          <span class="instance-edit-form__unit">{{ formatMemory(launchCfg.min_memory) }}</span>
        </div>
        <span class="instance-edit-form__range-sep">—</span>
        <div class="instance-edit-form__memory-field">
          <label class="instance-edit-form__sublabel">最大内存</label>
          <input
            v-model.number="launchCfg.max_memory"
            type="number"
            class="input-pixel"
            min="128"
            step="128"
          />
          <span class="instance-edit-form__unit">{{ formatMemory(launchCfg.max_memory) }}</span>
        </div>
      </div>
      <p v-if="memoryError" class="instance-edit-form__error">{{ memoryError }}</p>
    </div>

    <PixelInput
      v-model="jvmArgsStr"
      label="JVM 参数"
      placeholder="例: -XX:+UseG1GC"
      :error="jvmError"
    />

    <div class="instance-edit-form__actions">
      <PixelButton variant="ghost" @click="emit('cancel')">取消</PixelButton>
      <PixelButton
        variant="primary"
        :loading="saving"
        :disabled="!canSave"
        @click="handleSave"
      >
        保存修改
      </PixelButton>
    </div>
  </div>
</template>

<style scoped>
.instance-edit-form {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.instance-edit-form__section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.instance-edit-form__label {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}

.instance-edit-form__memory-row {
  display: flex;
  align-items: flex-end;
  gap: 12px;
}

.instance-edit-form__memory-field {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.instance-edit-form__sublabel {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.instance-edit-form__unit {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.instance-edit-form__range-sep {
  font-size: var(--font-size-lg);
  color: var(--color-text-muted);
  padding-bottom: 8px;
}

.instance-edit-form__error {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-danger);
}

.instance-edit-form__actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 8px;
}
</style>
