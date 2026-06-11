<script setup lang="ts">
/**
 * AppSidebar — Application sidebar with logo, navigation, and account badge.
 */
import { useRoute } from 'vue-router';
import AccountBadge from '@/components/account/AccountBadge.vue';

const route = useRoute();

/** Navigation items. */
const navItems = [
  { path: '/', label: '首页', icon: '🏠' },
  { path: '/instances', label: '实例列表', icon: '📦' },
  { path: '/versions', label: '版本管理', icon: '📦' },
  { path: '/settings', label: '设置', icon: '⚙️' },
];

/** Check if a nav item is active based on the current route. */
function isActive(path: string): boolean {
  if (path === '/') {
    return route.path === '/';
  }
  return route.path.startsWith(path);
}
</script>

<template>
  <aside class="app-sidebar">
    <!-- Logo -->
    <div class="app-sidebar__logo">
      <div class="app-sidebar__logo-icon">🟩</div>
      <div class="app-sidebar__logo-text">
        <span class="app-sidebar__logo-title">AURORA</span>
        <span class="app-sidebar__logo-sub">Launcher</span>
      </div>
    </div>

    <!-- Navigation -->
    <nav class="app-sidebar__nav">
      <router-link
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="app-sidebar__nav-item"
        :class="{ 'app-sidebar__nav-item--active': isActive(item.path) }"
      >
        <span class="app-sidebar__nav-icon">{{ item.icon }}</span>
        <span class="app-sidebar__nav-label">{{ item.label }}</span>
      </router-link>
    </nav>

    <!-- Spacer -->
    <div class="app-sidebar__spacer" />

    <!-- Account badge at bottom -->
    <div class="app-sidebar__account">
      <AccountBadge />
    </div>
  </aside>
</template>

<style scoped>
.app-sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-surface);
  backdrop-filter: var(--color-surface-blur-value);
  -webkit-backdrop-filter: var(--color-surface-blur-value);
  border-right: var(--border-width) solid var(--color-border);
  overflow: hidden;
}

.app-sidebar__logo {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 20px 16px;
  border-bottom: var(--border-width) solid var(--color-border);
}

.app-sidebar__logo-icon {
  font-size: 28px;
  line-height: 1;
}

.app-sidebar__logo-text {
  display: flex;
  flex-direction: column;
}

.app-sidebar__logo-title {
  font-family: var(--font-display);
  font-size: 12px;
  color: var(--color-primary);
  text-shadow: 0 0 10px rgba(126, 200, 80, 0.5);
  letter-spacing: 2px;
}

.app-sidebar__logo-sub {
  font-family: var(--font-body);
  font-size: 10px;
  color: var(--color-text-muted);
  letter-spacing: 1px;
}

.app-sidebar__nav {
  display: flex;
  flex-direction: column;
  padding: 8px;
  gap: 2px;
}

.app-sidebar__nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  text-decoration: none;
  border-radius: var(--border-radius);
  border: 2px solid transparent;
  transition: all 0.15s ease;
  cursor: pointer;
}

.app-sidebar__nav-item:hover {
  background: var(--color-surface-hover);
  color: var(--color-text);
  border-color: var(--color-border);
}

.app-sidebar__nav-item--active {
  background: var(--color-surface-active);
  color: var(--color-primary);
  border-color: var(--color-primary);
  box-shadow: var(--shadow-pixel-sm);
}

.app-sidebar__nav-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
}

.app-sidebar__nav-label {
  flex: 1;
}

.app-sidebar__spacer {
  flex: 1;
}

.app-sidebar__account {
  padding: 12px 8px;
  border-top: var(--border-width) solid var(--color-border);
}
</style>
