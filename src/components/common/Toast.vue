<script setup lang="ts">
/**
 * Toast — Global toast notification renderer.
 * Displays stacked notifications in the top-right corner.
 * Must be mounted once in the app root.
 */
import { useToast, type ToastType } from './useToast';

const { toasts, remove } = useToast();

/** Map of toast types to icons. */
const iconMap: Record<ToastType, string> = {
  success: '✓',
  error: '✕',
  info: 'ℹ',
  warning: '⚠',
};

/** Map of toast types to CSS class names. */
const typeClassMap: Record<ToastType, string> = {
  success: 'toast-item--success',
  error: 'toast-item--error',
  info: 'toast-item--info',
  warning: 'toast-item--warning',
};
</script>

<template>
  <div class="toast-container">
    <TransitionGroup name="toast">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast-item"
        :class="typeClassMap[toast.type]"
      >
        <span class="toast-item__icon">{{ iconMap[toast.type] }}</span>
        <span class="toast-item__message">{{ toast.message }}</span>
        <button class="toast-item__close" @click="remove(toast.id)">✕</button>
      </div>
    </TransitionGroup>
  </div>
</template>

<style scoped>
.toast-container {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
  max-width: 400px;
}

.toast-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  background: #1A1A2E;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 2px solid var(--color-border);
  border-radius: var(--border-radius);
  box-shadow: var(--shadow-pixel);
  pointer-events: auto;
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text);
  min-width: 280px;
}

.toast-item--success {
  border-color: var(--color-success);
}

.toast-item--success .toast-item__icon {
  color: var(--color-success);
}

.toast-item--error {
  border-color: var(--color-danger);
}

.toast-item--error .toast-item__icon {
  color: var(--color-danger);
}

.toast-item--info {
  border-color: var(--color-info);
}

.toast-item--info .toast-item__icon {
  color: var(--color-info);
}

.toast-item--warning {
  border-color: var(--color-warning);
}

.toast-item--warning .toast-item__icon {
  color: var(--color-warning);
}

.toast-item__icon {
  font-size: 16px;
  font-weight: bold;
  flex-shrink: 0;
}

.toast-item__message {
  flex: 1;
  line-height: 1.4;
}

.toast-item__close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  background: transparent;
  border: none;
  color: var(--color-text-muted);
  font-size: 12px;
  cursor: pointer;
  flex-shrink: 0;
  transition: color var(--transition-fast);
}

.toast-item__close:hover {
  color: var(--color-text);
}

/* Toast transition */
.toast-enter-active {
  transition: all 0.3s ease-out;
}

.toast-leave-active {
  transition: all 0.2s ease-in;
}

.toast-enter-from {
  opacity: 0;
  transform: translateX(40px);
}

.toast-leave-to {
  opacity: 0;
  transform: translateX(40px);
}

.toast-move {
  transition: transform 0.2s ease;
}
</style>
