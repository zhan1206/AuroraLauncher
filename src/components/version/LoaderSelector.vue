<script setup lang="ts">
/**
 * LoaderSelector — Minecraft loader type selector with icons and descriptions.
 */
import type { LoaderType } from '@/types/instance';

export interface LoaderSelectorProps {
  /** Selected loader type (v-model). */
  modelValue: LoaderType;
}

defineProps<LoaderSelectorProps>();

const emit = defineEmits<{
  'update:modelValue': [value: LoaderType];
}>();

/** Loader options with display metadata. */
const loaderOptions: { value: LoaderType; icon: string; label: string; description: string }[] = [
  {
    value: 'Vanilla',
    icon: '🟩',
    label: 'Vanilla',
    description: '原版 Minecraft，无模组加载器',
  },
  {
    value: 'Fabric',
    icon: '🧵',
    label: 'Fabric',
    description: '轻量级模组加载器，加载快',
  },
  {
    value: 'Forge',
    icon: '🔨',
    label: 'Forge',
    description: '经典模组加载器，兼容性广',
  },
];

/** Select a loader type. */
function selectLoader(loader: LoaderType): void {
  emit('update:modelValue', loader);
}
</script>

<template>
  <div class="loader-selector">
    <div
      v-for="option in loaderOptions"
      :key="option.value"
      class="loader-selector__option"
      :class="{ 'loader-selector__option--selected': option.value === modelValue }"
      @click="selectLoader(option.value)"
    >
      <span class="loader-selector__icon">{{ option.icon }}</span>
      <div class="loader-selector__info">
        <span class="loader-selector__label">{{ option.label }}</span>
        <span class="loader-selector__desc">{{ option.description }}</span>
      </div>
      <span v-if="option.value === modelValue" class="loader-selector__check">✓</span>
    </div>
  </div>
</template>

<style scoped>
.loader-selector {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.loader-selector__option {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  background: rgba(0, 0, 0, 0.2);
  border: 2px solid var(--color-border);
  border-radius: var(--border-radius);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.loader-selector__option:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
}

.loader-selector__option--selected {
  border-color: var(--color-primary);
  background: rgba(126, 200, 80, 0.08);
}

.loader-selector__icon {
  font-size: 24px;
  flex-shrink: 0;
}

.loader-selector__info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.loader-selector__label {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text);
  font-weight: bold;
}

.loader-selector__option--selected .loader-selector__label {
  color: var(--color-primary);
}

.loader-selector__desc {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.loader-selector__check {
  font-size: 16px;
  color: var(--color-primary);
  font-weight: bold;
}
</style>
