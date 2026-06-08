<script setup lang="ts">
/**
 * DownloadProgress — Progress display for a single download task.
 */
import { computed } from 'vue';
import type { DownloadTask } from '@/types/download';
import { formatFileSize, formatSpeed } from '@/utils/formatters';
import { useDownloadStore } from '@/stores/download';
import PixelButton from '@/components/common/PixelButton.vue';

export interface DownloadProgressProps {
  /** The download task to display. */
  task: DownloadTask;
}

const props = defineProps<DownloadProgressProps>();

const downloadStore = useDownloadStore();

/** Live progress data for this task. */
const progress = computed(() => downloadStore.getProgress(props.task.id));

/** Download percentage (0-100). */
const percent = computed(() => {
  if (progress.value) return progress.value.percent;
  if (props.task.total_size > 0) {
    return Math.round((props.task.downloaded / props.task.total_size) * 100);
  }
  return 0;
});

/** Download speed string. */
const speed = computed(() => {
  if (progress.value) return formatSpeed(progress.value.speed);
  return '';
});

/** Is the task paused. */
const isPaused = computed(() => props.task.status === 'Paused');

/** Is the task downloading. */
const isDownloading = computed(() => props.task.status === 'Downloading');

/** Is the task completed. */
const isCompleted = computed(() => props.task.status === 'Completed');

/** Is the task failed. */
const isFailed = computed(() => props.task.status === 'Failed');
</script>

<template>
  <div class="download-progress" :class="{ 'download-progress--failed': isFailed }">
    <div class="download-progress__header">
      <span class="download-progress__name">{{ task.name }}</span>
      <span class="download-progress__status">
        <template v-if="isCompleted">✓ 完成</template>
        <template v-else-if="isFailed">✕ 失败</template>
        <template v-else-if="isPaused">⏸ 暂停</template>
        <template v-else-if="isDownloading">{{ percent }}%</template>
        <template v-else>等待中</template>
      </span>
    </div>

    <!-- Progress bar -->
    <div class="progress-pixel">
      <div
        class="progress-pixel-fill"
        :style="{ width: `${percent}%` }"
        :class="{
          'progress-pixel-fill--complete': isCompleted,
          'progress-pixel-fill--failed': isFailed,
          'progress-pixel-fill--paused': isPaused
        }"
      />
    </div>

    <!-- Info row -->
    <div class="download-progress__info">
      <span class="download-progress__size">
        {{ formatFileSize(task.downloaded) }} / {{ formatFileSize(task.total_size) }}
      </span>
      <span v-if="speed" class="download-progress__speed">{{ speed }}</span>
    </div>

    <!-- Action buttons -->
    <div class="download-progress__actions">
      <PixelButton
        v-if="isPaused"
        size="sm"
        @click="downloadStore.resumeTask(task.id)"
      >
        ▶ 继续
      </PixelButton>
      <PixelButton
        v-if="isDownloading"
        size="sm"
        @click="downloadStore.pauseTask(task.id)"
      >
        ⏸ 暂停
      </PixelButton>
      <PixelButton
        v-if="isDownloading || isPaused"
        size="sm"
        variant="danger"
        @click="downloadStore.cancelTask(task.id)"
      >
        ✕ 取消
      </PixelButton>
    </div>
  </div>
</template>

<style scoped>
.download-progress {
  padding: 10px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
}

.download-progress--failed {
  opacity: 0.8;
}

.download-progress__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.download-progress__name {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  margin-right: 8px;
}

.download-progress__status {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  flex-shrink: 0;
}

.download-progress__info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-top: 6px;
}

.download-progress__size {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.download-progress__speed {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-primary);
}

.download-progress__actions {
  display: flex;
  gap: 4px;
  margin-top: 6px;
}

.progress-pixel-fill--complete {
  background: var(--color-success);
}

.progress-pixel-fill--failed {
  background: var(--color-danger);
}

.progress-pixel-fill--paused {
  background: var(--color-warning);
}
</style>
