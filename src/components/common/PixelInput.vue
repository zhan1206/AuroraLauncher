<script setup lang="ts">
/**
 * PixelInput — Pixel-styled text input with label and error state.
 */
import { computed } from 'vue';

export interface PixelInputProps {
  /** Input value (v-model). */
  modelValue: string;
  /** Label text displayed above the input. */
  label?: string;
  /** Placeholder text. */
  placeholder?: string;
  /** Error message displayed below the input. */
  error?: string;
  /** Input type attribute. */
  type?: string;
  /** Whether the input is disabled. */
  disabled?: boolean;
}

const props = withDefaults(defineProps<PixelInputProps>(), {
  label: '',
  placeholder: '',
  error: '',
  type: 'text',
  disabled: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const inputValue = computed({
  get: () => props.modelValue,
  set: (val: string) => emit('update:modelValue', val),
});

const hasError = computed(() => !!props.error);
</script>

<template>
  <div class="pixel-input-wrapper">
    <label v-if="label" class="pixel-input__label">{{ label }}</label>
    <input
      v-model="inputValue"
      :type="type"
      :placeholder="placeholder"
      :disabled="disabled"
      class="input-pixel"
      :class="{ 'pixel-input--error': hasError }"
    />
    <p v-if="hasError" class="pixel-input__error">{{ error }}</p>
  </div>
</template>

<style scoped>
.pixel-input-wrapper {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.pixel-input__label {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  margin-bottom: 2px;
}

.pixel-input--error {
  border-color: var(--color-danger) !important;
  box-shadow: 0 0 0 1px var(--color-danger) !important;
}

.pixel-input__error {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-danger);
  margin-top: 2px;
}
</style>
