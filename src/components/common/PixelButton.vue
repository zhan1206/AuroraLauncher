<script setup lang="ts">
/**
 * PixelButton — Pixel-style button with multiple variants.
 * Supports primary/secondary/danger styles, loading state, and icon slot.
 */
import { computed } from 'vue';

export interface PixelButtonProps {
  /** Button variant style. */
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  /** Whether the button is in a loading state. */
  loading?: boolean;
  /** Whether the button is disabled. */
  disabled?: boolean;
  /** Button size. */
  size?: 'sm' | 'md' | 'lg';
  /** HTML type attribute. */
  type?: 'button' | 'submit' | 'reset';
}

const props = withDefaults(defineProps<PixelButtonProps>(), {
  variant: 'secondary',
  loading: false,
  disabled: false,
  size: 'md',
  type: 'button',
});

const emit = defineEmits<{
  click: [event: MouseEvent];
}>();

const variantClass = computed(() => {
  const map: Record<string, string> = {
    primary: 'btn-pixel-primary',
    secondary: '',
    danger: 'btn-pixel-danger',
    ghost: 'btn-pixel-ghost',
  };
  return map[props.variant] ?? '';
});

const sizeClass = computed(() => {
  const map: Record<string, string> = {
    sm: 'btn-pixel-sm',
    md: '',
    lg: 'btn-pixel-lg',
  };
  return map[props.size] ?? '';
});

const isDisabled = computed(() => props.disabled || props.loading);

function handleClick(event: MouseEvent): void {
  if (isDisabled.value) return;
  emit('click', event);
}
</script>

<template>
  <button
    :type="type"
    class="btn-pixel"
    :class="[variantClass, sizeClass]"
    :disabled="isDisabled"
    @click="handleClick"
  >
    <span v-if="loading" class="btn-spinner">⏳</span>
    <slot v-else name="icon" />
    <span class="btn-label"><slot /></span>
  </button>
</template>

<style scoped>
.btn-pixel {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-xs);
  padding: 8px 16px;
  font-family: var(--font-body);
  font-size: var(--font-size-base);
  color: var(--color-text);
  background: var(--color-surface);
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  box-shadow: var(--shadow-pixel-sm);
  cursor: pointer;
  transition: all var(--transition-fast);
  user-select: none;
  line-height: 1;
}

.btn-pixel:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
  transform: translate(-1px, -1px);
  box-shadow: var(--shadow-pixel);
}

.btn-pixel:active {
  transform: translate(1px, 1px);
  box-shadow: none;
  background: var(--color-surface-active);
}

.btn-pixel:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.btn-pixel-primary {
  color: var(--color-text-inverse);
  background: var(--color-primary);
  border-color: var(--color-primary-dark);
  text-shadow: 0 1px 0 rgba(0, 0, 0, 0.3);
}

.btn-pixel-primary:hover {
  background: var(--color-primary-light);
  border-color: var(--color-primary);
  box-shadow: var(--shadow-pixel), var(--shadow-glow);
}

.btn-pixel-primary:active {
  background: var(--color-primary-dark);
}

.btn-pixel-danger {
  color: #fff;
  background: var(--color-danger);
  border-color: #c04040;
}

.btn-pixel-danger:hover {
  background: #e86060;
  box-shadow: var(--shadow-pixel);
}

.btn-pixel-ghost {
  background: transparent;
  border-color: transparent;
  box-shadow: none;
}

.btn-pixel-ghost:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border);
  box-shadow: none;
  transform: none;
}

.btn-pixel-ghost:active {
  background: var(--color-surface-active);
  transform: none;
}

.btn-pixel-sm {
  padding: 4px 10px;
  font-size: var(--font-size-sm);
}

.btn-pixel-lg {
  padding: 12px 28px;
  font-size: var(--font-size-lg);
}

.btn-spinner {
  display: inline-block;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.btn-label {
  display: inline-flex;
  align-items: center;
}
</style>
