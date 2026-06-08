<script setup lang="ts">
/**
 * DownloadQueue — Popup panel showing all download tasks.
 */
import { onMounted } from 'vue';
import { useDownloadStore } from '@/stores/download';
import DownloadProgress from '@/components/download/DownloadProgress.vue';

const downloadStore = useDownloadStore();

onMounted(() => {
  downloadStore.fetchTasks();
  downloadStore.startListening();
});
</script>

<template>
  <div class="download-queue">
    <div class="download-queue__header">
      <h3 class="download-queue__title">下载队列</h3>
      <span class="download-queue__count">
        {{ downloadStore.activeCount }} 个进行中
      </span>
    </div>

    <div v-if="downloadStore.loading" class="download-queue__loading">
      加载中...
    </div>

    <div v-else-if="downloadStore.tasks.length === 0" class="download-queue__empty">
      暂无下载任务
    </div>

    <div v-else class="download-queue__list">
      <DownloadProgress
        v-for="task in downloadStore.tasks"
        :key="task.id"
        :task="task"
      />
    </div>
  </div>
</template>

<style scoped>
.download-queue {
  width: 360px;
  max-height: 480px;
  display: flex;
  flex-direction: column;
  background: #1A1A2E;
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  box-shadow: var(--shadow-pixel-lg);
  overflow: hidden;
}

.download-queue__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: var(--border-width) solid var(--color-border);
}

.download-queue__title {
  font-family: var(--font-display);
  font-size: var(--font-size-sm);
  color: var(--color-primary);
  margin: 0;
}

.download-queue__count {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.download-queue__loading,
.download-queue__empty {
  padding: 32px 16px;
  text-align: center;
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
}

.download-queue__list {
  flex: 1;
  overflow-y: auto;
  padding: 0 12px;
}
</style>
