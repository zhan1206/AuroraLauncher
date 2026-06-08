<script setup lang="ts">
/**
 * AppLayout — Main application layout with sidebar and content area.
 */
import AppSidebar from './AppSidebar.vue';
</script>

<template>
  <div class="app-layout">
    <AppSidebar class="app-layout__sidebar" />
    <main class="app-layout__content">
      <router-view v-slot="{ Component }">
        <transition name="fade-slide" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: var(--color-bg);
}

.app-layout__sidebar {
  width: 240px;
  min-width: 240px;
  height: 100%;
  flex-shrink: 0;
}

.app-layout__content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-lg);
  background: var(--color-bg);
}

/* Route transition */
.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.2s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateY(8px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
