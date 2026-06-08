<script setup lang="ts">
/**
 * GlassPanel — Simple glass-panel container.
 * Provides frosted glass background with optional border.
 */
export interface GlassPanelProps {
  /** Custom padding value. */
  padding?: string;
  /** Whether to show the border. */
  border?: boolean;
}

withDefaults(defineProps<GlassPanelProps>(), {
  padding: '16px',
  border: true,
});
</script>

<template>
  <div
    class="glass-panel-container"
    :class="{ 'glass-panel-container--border': border }"
    :style="{ padding }"
  >
    <slot />
  </div>
</template>

<style scoped>
.glass-panel-container {
  background: var(--color-surface);
  backdrop-filter: var(--color-surface-blur-value);
  -webkit-backdrop-filter: var(--color-surface-blur-value);
  border-radius: var(--border-radius);
}

.glass-panel-container--border {
  border: var(--border-width) solid var(--color-border);
}
</style>
