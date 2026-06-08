<script setup lang="ts">
/**
 * PixelCard — Frosted glass card with pixel border.
 * Supports hoverable and selected states.
 */
import { computed } from 'vue';

export interface PixelCardProps {
  /** Whether the card responds to hover with lift effect. */
  hoverable?: boolean;
  /** Whether the card is in a selected state. */
  selected?: boolean;
  /** Custom padding value. */
  padding?: string;
}

const props = withDefaults(defineProps<PixelCardProps>(), {
  hoverable: false,
  selected: false,
  padding: '16px',
});

const cardClass = computed(() => {
  return {
    'pixel-card': true,
    'pixel-card--hoverable': props.hoverable,
    'pixel-card--selected': props.selected,
  };
});
</script>

<template>
  <div :class="cardClass" :style="{ padding }">
    <slot />
  </div>
</template>

<style scoped>
.pixel-card {
  background: var(--color-surface);
  backdrop-filter: var(--color-surface-blur-value);
  -webkit-backdrop-filter: var(--color-surface-blur-value);
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  box-shadow: var(--shadow-pixel-sm);
  transition: all var(--transition-normal);
}

.pixel-card--hoverable {
  cursor: pointer;
}

.pixel-card--hoverable:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
  transform: translate(-1px, -3px);
  box-shadow: var(--shadow-pixel), 0 4px 12px rgba(0, 0, 0, 0.2);
}

.pixel-card--hoverable:active {
  transform: translate(1px, 1px);
  box-shadow: var(--shadow-pixel-sm);
}

.pixel-card--selected {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-pixel-sm), var(--shadow-glow);
}

.pixel-card--selected:hover {
  border-color: var(--color-primary-light);
}
</style>
