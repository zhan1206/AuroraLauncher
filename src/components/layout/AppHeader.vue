<script setup lang="ts">
/**
 * AppHeader — Page header with title, breadcrumbs, and utility actions.
 */
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { useDownloadStore } from '@/stores/download';

export interface AppHeaderProps {
  /** Page title text. */
  title: string;
}

defineProps<AppHeaderProps>();

const route = useRoute();
const downloadStore = useDownloadStore();

/** Breadcrumb segments derived from the route path. */
const breadcrumbs = computed(() => {
  const segments = route.path.split('/').filter(Boolean);
  return segments.map((segment, index) => {
    const path = '/' + segments.slice(0, index + 1).join('/');
    const label = segment.charAt(0).toUpperCase() + segment.slice(1);
    return { path, label };
  });
});

/** Whether there are active downloads. */
const hasDownloads = computed(() => downloadStore.activeCount > 0);
</script>

<template>
  <header class="app-header">
    <div class="app-header__left">
      <h1 class="app-header__title">{{ title }}</h1>
      <!-- Breadcrumbs -->
      <div v-if="breadcrumbs.length > 1" class="app-header__breadcrumbs">
        <router-link
          v-for="(crumb, index) in breadcrumbs"
          :key="crumb.path"
          :to="crumb.path"
          class="app-header__crumb"
          :class="{ 'app-header__crumb--current': index === breadcrumbs.length - 1 }"
        >
          <span v-if="index > 0" class="app-header__crumb-sep">/</span>
          {{ crumb.label }}
        </router-link>
      </div>
    </div>

    <div class="app-header__right">
      <!-- Download queue button -->
      <router-link to="/instances" class="app-header__action" title="下载队列">
        <span class="app-header__action-icon">⬇️</span>
        <span v-if="hasDownloads" class="app-header__badge">
          {{ downloadStore.activeCount }}
        </span>
      </router-link>

      <!-- Settings shortcut -->
      <router-link to="/settings" class="app-header__action" title="设置">
        <span class="app-header__action-icon">⚙️</span>
      </router-link>
    </div>
  </header>
</template>

<style scoped>
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: var(--space-md);
  margin-bottom: var(--space-md);
  border-bottom: var(--border-width) solid var(--color-border);
}

.app-header__left {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.app-header__title {
  font-family: var(--font-display);
  font-size: var(--font-size-xl);
  color: var(--color-primary);
  text-shadow: 0 0 10px rgba(126, 200, 80, 0.4);
  margin: 0;
}

.app-header__breadcrumbs {
  display: flex;
  align-items: center;
  gap: 4px;
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
}

.app-header__crumb {
  display: flex;
  align-items: center;
  gap: 4px;
  color: var(--color-text-muted);
  text-decoration: none;
  transition: color var(--transition-fast);
}

.app-header__crumb:hover {
  color: var(--color-text-secondary);
}

.app-header__crumb--current {
  color: var(--color-text-secondary);
}

.app-header__crumb-sep {
  color: var(--color-text-muted);
}

.app-header__right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.app-header__action {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: var(--border-radius);
  text-decoration: none;
  transition: all var(--transition-fast);
}

.app-header__action:hover {
  background: var(--color-surface-hover);
}

.app-header__action-icon {
  font-size: 18px;
}

.app-header__badge {
  position: absolute;
  top: 2px;
  right: 2px;
  min-width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 4px;
  font-family: var(--font-body);
  font-size: 9px;
  color: white;
  background: var(--color-danger);
  border-radius: 8px;
  font-weight: bold;
}
</style>
