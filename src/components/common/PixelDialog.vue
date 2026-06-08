<script setup lang="ts">
/**
 * PixelDialog — Modal dialog with pixel-glass styling.
 * Supports v-model for open/close state, custom width, and footer slot.
 */
import { computed } from 'vue';

export interface PixelDialogProps {
  /** Whether the dialog is visible (v-model). */
  modelValue: boolean;
  /** Dialog title text. */
  title: string;
  /** Custom width for the dialog. */
  width?: string;
}

const props = withDefaults(defineProps<PixelDialogProps>(), {
  width: '480px',
});

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const isVisible = computed({
  get: () => props.modelValue,
  set: (val: boolean) => emit('update:modelValue', val),
});

/** Close the dialog when clicking the overlay. */
function onOverlayClick(): void {
  isVisible.value = false;
}

/** Prevent clicks inside the dialog from closing it. */
function onDialogClick(event: MouseEvent): void {
  event.stopPropagation();
}

/** Close the dialog. */
function close(): void {
  isVisible.value = false;
}

/** Expose close method for parent components. */
defineExpose({ close });
</script>

<template>
  <Teleport to="body">
    <Transition name="dialog-fade">
      <div v-if="isVisible" class="pixel-dialog-overlay" @click="onOverlayClick">
        <div
          class="pixel-dialog"
          :style="{ maxWidth: width }"
          @click="onDialogClick"
        >
          <!-- Header -->
          <div class="pixel-dialog__header">
            <h3 class="pixel-dialog__title">{{ title }}</h3>
            <button class="pixel-dialog__close" @click="close">✕</button>
          </div>

          <!-- Body -->
          <div class="pixel-dialog__body">
            <slot />
          </div>

          <!-- Footer -->
          <div v-if="$slots.footer" class="pixel-dialog__footer">
            <slot name="footer" />
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.pixel-dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  -webkit-backdrop-filter: blur(4px);
}

.pixel-dialog {
  width: 90%;
  background: #1A1A2E;
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  box-shadow: var(--shadow-pixel-lg);
  overflow: hidden;
}

.pixel-dialog__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: var(--border-width) solid var(--color-border);
}

.pixel-dialog__title {
  font-family: var(--font-display);
  font-size: var(--font-size-sm);
  color: var(--color-primary);
  text-shadow: 0 0 10px rgba(126, 200, 80, 0.3);
  margin: 0;
}

.pixel-dialog__close {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  background: transparent;
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius);
  color: var(--color-text-secondary);
  font-size: 14px;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.pixel-dialog__close:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-danger);
  color: var(--color-danger);
}

.pixel-dialog__body {
  padding: 20px;
  max-height: 60vh;
  overflow-y: auto;
}

.pixel-dialog__footer {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--space-sm);
  padding: 12px 20px;
  border-top: var(--border-width) solid var(--color-border);
}

/* Dialog transition */
.dialog-fade-enter-active {
  transition: all 0.25s ease-out;
}

.dialog-fade-leave-active {
  transition: all 0.15s ease-in;
}

.dialog-fade-enter-from {
  opacity: 0;
}

.dialog-fade-enter-from .pixel-dialog {
  transform: translateY(20px) scale(0.95);
}

.dialog-fade-leave-to {
  opacity: 0;
}

.dialog-fade-enter-to .pixel-dialog {
  transform: translateY(0) scale(1);
}
</style>
