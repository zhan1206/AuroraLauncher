<script setup lang="ts">
/**
 * PixelSwitch — Pixel-styled toggle switch with animation.
 */
import { computed } from 'vue';

export interface PixelSwitchProps {
  /** Whether the switch is on (v-model). */
  modelValue: boolean;
  /** Label text displayed next to the switch. */
  label?: string;
  /** Whether the switch is disabled. */
  disabled?: boolean;
}

const props = withDefaults(defineProps<PixelSwitchProps>(), {
  label: '',
  disabled: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const isOn = computed({
  get: () => props.modelValue,
  set: (val: boolean) => emit('update:modelValue', val),
});

function toggle(): void {
  if (!props.disabled) {
    isOn.value = !isOn.value;
  }
}
</script>

<template>
  <div
    class="pixel-switch"
    :class="{ 'pixel-switch--on': isOn, 'pixel-switch--disabled': disabled }"
    @click="toggle"
  >
    <div class="pixel-switch__track">
      <div class="pixel-switch__thumb" />
    </div>
    <span v-if="label" class="pixel-switch__label">{{ label }}</span>
  </div>
</template>

<style scoped>
.pixel-switch {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  user-select: none;
}

.pixel-switch--disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.pixel-switch__track {
  position: relative;
  width: 40px;
  height: 20px;
  background: rgba(0, 0, 0, 0.4);
  border: 2px solid var(--color-border);
  border-radius: 2px;
  transition: all var(--transition-normal);
}

.pixel-switch--on .pixel-switch__track {
  background: rgba(126, 200, 80, 0.2);
  border-color: var(--color-primary);
}

.pixel-switch__thumb {
  position: absolute;
  top: 1px;
  left: 1px;
  width: 14px;
  height: 14px;
  background: var(--color-text-secondary);
  border-radius: 2px;
  transition: all var(--transition-normal);
}

.pixel-switch--on .pixel-switch__thumb {
  left: 21px;
  background: var(--color-primary);
  box-shadow: 0 0 8px rgba(126, 200, 80, 0.5);
}

.pixel-switch__label {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}
</style>
