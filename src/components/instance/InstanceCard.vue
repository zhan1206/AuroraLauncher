<script setup lang="ts">
/**
 * InstanceCard — Card displaying a game instance with hover actions.
 */
import { computed } from 'vue';
import type { Instance } from '@/types/instance';
import { formatRelativeTime, formatLoaderType } from '@/utils/formatters';

export interface InstanceCardProps {
  /** The instance to display. */
  instance: Instance;
}

const props = defineProps<InstanceCardProps>();

const emit = defineEmits<{
  launch: [instance: Instance];
  edit: [instance: Instance];
  delete: [instance: Instance];
}>();

/** Display text for last updated time (used as proxy for last played). */
const updatedText = computed(() => formatRelativeTime(props.instance.updated_at));

/** Loader type badge text. */
const loaderText = computed(() => formatLoaderType(props.instance.loader_type));
</script>

<template>
  <div class="instance-card glass-card" @click="emit('edit', instance)">
    <!-- Instance info -->
    <div class="instance-card__info">
      <h3 class="instance-card__name">{{ instance.name }}</h3>
      <div class="instance-card__badges">
        <span class="tag-pixel">{{ instance.version_id }}</span>
        <span
          v-if="instance.loader_type !== 'Vanilla'"
          class="tag-pixel"
          style="color: var(--color-info); border-color: rgba(80,144,224,0.3); background: rgba(80,144,224,0.1);"
        >
          {{ loaderText }}
        </span>
      </div>
      <p class="instance-card__meta">
        <span>更新于: {{ updatedText }}</span>
      </p>
    </div>

    <!-- Hover actions -->
    <div class="instance-card__actions">
      <button
        class="btn-pixel btn-pixel-primary instance-card__launch-btn"
        @click.stop="emit('launch', instance)"
      >
        ▶ 启动
      </button>
      <button
        class="btn-pixel instance-card__edit-btn"
        @click.stop="emit('edit', instance)"
      >
        ✏️
      </button>
      <button
        class="btn-pixel btn-pixel-danger instance-card__delete-btn"
        @click.stop="emit('delete', instance)"
      >
        🗑️
      </button>
    </div>
  </div>
</template>

<style scoped>
.instance-card {
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;
  transition: all var(--transition-normal);
  position: relative;
  overflow: hidden;
}

.instance-card:hover {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-pixel-sm), var(--shadow-glow);
}

.instance-card__info {
  flex: 1;
  min-width: 0;
}

.instance-card__name {
  font-family: var(--font-display);
  font-size: var(--font-size-sm);
  color: var(--color-text);
  margin-bottom: 6px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.instance-card__badges {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
  margin-bottom: 6px;
}

.instance-card__meta {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.instance-card__actions {
  display: flex;
  align-items: center;
  gap: 6px;
  opacity: 0;
  transform: translateX(8px);
  transition: all var(--transition-normal);
}

.instance-card:hover .instance-card__actions {
  opacity: 1;
  transform: translateX(0);
}

.instance-card__launch-btn {
  padding: 6px 16px;
  font-size: var(--font-size-sm);
}

.instance-card__edit-btn,
.instance-card__delete-btn {
  padding: 6px 8px;
  font-size: var(--font-size-sm);
}
</style>
