<script setup lang="ts">
/**
 * InstanceDetailView — Detailed view of a single instance with tabs.
 */
import { onMounted, computed, ref, watch, nextTick } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { useInstanceStore } from '@/stores/instance';
import { useLaunchStore } from '@/stores/launch';
import AppHeader from '@/components/layout/AppHeader.vue';
import LaunchButton from '@/components/launch/LaunchButton.vue';
import LaunchConfigPanel from '@/components/launch/LaunchConfigPanel.vue';
import VersionSelector from '@/components/version/VersionSelector.vue';
import PixelButton from '@/components/common/PixelButton.vue';
import GlassPanel from '@/components/common/GlassPanel.vue';
import { formatRelativeTime, formatLoaderType, formatDate } from '@/utils/formatters';
import { useToast } from '@/components/common/useToast';

const route = useRoute();
const router = useRouter();
const instanceStore = useInstanceStore();
const launchStore = useLaunchStore();
const toast = useToast();

const instanceId = computed(() => (route.params.id as string) || '');
const instance = computed(() => instanceStore.current);

// Tab state
const activeTab = ref<'overview' | 'version' | 'logs'>('overview');

// Log scroll container
const logContainer = ref<HTMLElement | null>(null);

// Load instance data
onMounted(() => {
  if (instanceId.value) {
    instanceStore.selectInstance(instanceId.value);
  }
});

// Watch for route changes
watch(instanceId, (newId) => {
  if (newId) {
    instanceStore.selectInstance(newId);
  }
});

// Auto-scroll logs to bottom
watch(
  () => launchStore.logs.length,
  async () => {
    await nextTick();
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight;
    }
  }
);

/** Whether this instance is currently running. */
const isRunning = computed(
  () => launchStore.currentInstanceId === instanceId.value && launchStore.isRunning
);

/** Delete the instance. */
async function handleDelete(): Promise<void> {
  if (!instance.value) return;
  const confirmed = window.confirm(`确定要删除实例 "${instance.value.name}" 吗？此操作不可撤销。`);
  if (confirmed) {
    const success = await instanceStore.deleteInstance(instanceId.value);
    if (success) {
      toast.success('实例已删除');
      router.push('/instances');
    } else {
      toast.error('删除失败');
    }
  }
}

/** Log level to CSS class. */
function logLevelClass(level: string): string {
  switch (level) {
    case 'ERROR':
    case 'FATAL':
      return 'log-line--error';
    case 'WARN':
      return 'log-line--warn';
    case 'DEBUG':
    case 'TRACE':
      return 'log-line--debug';
    default:
      return 'log-line--info';
  }
}
</script>

<template>
  <div class="instance-detail-view animate-fade-in">
    <AppHeader title="实例详情" />

    <!-- Loading -->
    <div v-if="instanceStore.loading" class="glass-card animate-shimmer" style="height: 200px;" />

    <!-- Not found -->
    <GlassPanel v-else-if="!instance" padding="48px" style="text-align: center;">
      <p style="font-size: 36px; margin-bottom: 16px;">❓</p>
      <p style="font-family: var(--font-display); font-size: 12px; color: var(--color-text-secondary);">
        实例不存在或加载失败
      </p>
      <PixelButton style="margin-top: 16px;" @click="router.push('/instances')">返回实例列表</PixelButton>
    </GlassPanel>

    <!-- Instance detail -->
    <template v-else>
      <!-- Header card -->
      <div class="instance-detail__header glass-panel-grid" style="padding: 24px;">
        <div style="position: relative; z-index: 1;" class="instance-detail__header-content">
          <div class="instance-detail__header-info">
            <h3 class="instance-detail__name">{{ instance.name }}</h3>
            <div class="instance-detail__badges">
              <span class="tag-pixel">{{ instance.version_id }}</span>
              <span class="tag-pixel" style="color: var(--color-info); border-color: rgba(80,144,224,0.3); background: rgba(80,144,224,0.1);">
                {{ formatLoaderType(instance.loader_type) }}
              </span>
            </div>
          </div>
          <LaunchButton :instance-id="instance.id" :version-id="instance.version_id" />
        </div>
        <!-- Launch error banner -->
        <div
          v-if="launchStore.currentInstanceId === instance.id && launchStore.error"
          class="instance-detail__launch-error"
        >
          <span class="instance-detail__launch-error-icon">⚠️</span>
          <span class="instance-detail__launch-error-text">{{ launchStore.error }}</span>
        </div>

        <!-- Install progress banner -->
        <div
          v-if="launchStore.currentInstanceId === instance.id && launchStore.isInstalling && launchStore.currentInstallProgress"
          class="instance-detail__install-progress"
        >
          <span class="instance-detail__install-icon">⬇️</span>
          <div class="instance-detail__install-info">
            <span class="instance-detail__install-title">正在下载版本 {{ launchStore.currentInstallProgress.versionId }}</span>
            <div class="instance-detail__install-bar">
              <div
                class="instance-detail__install-bar-fill"
                :style="{ width: launchStore.currentInstallProgress.percent + '%' }"
              />
            </div>
            <span class="instance-detail__install-detail">
              {{ launchStore.currentInstallProgress.percent }}% —
              {{ launchStore.currentInstallProgress.completedFiles }}/{{ launchStore.currentInstallProgress.totalFiles }} 文件
              <span v-if="launchStore.currentInstallProgress.currentFile">
                — {{ launchStore.currentInstallProgress.currentFile }}
              </span>
            </span>
          </div>
        </div>
      </div>

      <!-- Tab navigation -->
      <div class="instance-detail__tabs">
        <button
          class="instance-detail__tab"
          :class="{ 'instance-detail__tab--active': activeTab === 'overview' }"
          @click="activeTab = 'overview'"
        >
          概览
        </button>
        <button
          class="instance-detail__tab"
          :class="{ 'instance-detail__tab--active': activeTab === 'version' }"
          @click="activeTab = 'version'"
        >
          版本
        </button>
        <button
          class="instance-detail__tab"
          :class="{ 'instance-detail__tab--active': activeTab === 'logs' }"
          @click="activeTab = 'logs'"
        >
          日志
        </button>
      </div>

      <!-- Tab: Overview -->
      <div v-if="activeTab === 'overview'" class="instance-detail__tab-content">
        <div class="instance-detail__overview-grid">
          <GlassPanel padding="20px">
            <h4 class="instance-detail__section-title">启动配置</h4>
            <LaunchConfigPanel :instance-id="instance.id" />
          </GlassPanel>

          <GlassPanel padding="20px">
            <h4 class="instance-detail__section-title">游戏统计</h4>
            <div class="instance-detail__stats">
              <div class="instance-detail__stat">
                <span class="instance-detail__stat-label">最后更新</span>
                <span class="instance-detail__stat-value">{{ formatRelativeTime(instance.updated_at) }}</span>
              </div>
              <div class="instance-detail__stat">
                <span class="instance-detail__stat-label">创建时间</span>
                <span class="instance-detail__stat-value">{{ formatDate(instance.created_at) }}</span>
              </div>
              <div class="instance-detail__stat">
                <span class="instance-detail__stat-label">游戏目录</span>
                <span class="instance-detail__stat-value instance-detail__stat-path">{{ instance.game_dir }}</span>
              </div>
            </div>
          </GlassPanel>
        </div>

        <!-- Delete button -->
        <div class="instance-detail__danger-zone">
          <PixelButton variant="danger" @click="handleDelete">
            🗑️ 删除实例
          </PixelButton>
        </div>
      </div>

      <!-- Tab: Version -->
      <div v-if="activeTab === 'version'" class="instance-detail__tab-content">
        <GlassPanel padding="20px">
          <h4 class="instance-detail__section-title">版本选择</h4>
          <VersionSelector :model-value="instance.version_id" />
        </GlassPanel>
      </div>

      <!-- Tab: Logs -->
      <div v-if="activeTab === 'logs'" class="instance-detail__tab-content">
        <GlassPanel padding="0" border>
          <div class="instance-detail__log-header">
            <h4 class="instance-detail__section-title" style="padding: 12px 16px; margin-bottom: 0; border-bottom: none;">
              启动日志
            </h4>
            <PixelButton size="sm" @click="launchStore.clearLogs()">清空</PixelButton>
          </div>
          <div ref="logContainer" class="instance-detail__log-container">
            <div v-if="launchStore.logs.length === 0" class="instance-detail__log-empty">
              {{ isRunning ? '等待日志输出...' : '启动游戏后日志将显示在这里' }}
            </div>
            <div
              v-for="(log, index) in launchStore.logs"
              :key="index"
              class="instance-detail__log-line"
              :class="logLevelClass(log.level)"
            >
              <span class="instance-detail__log-time">
                {{ new Date(log.timestamp).toLocaleTimeString('zh-CN') }}
              </span>
              <span class="instance-detail__log-level">[{{ log.level }}]</span>
              <span class="instance-detail__log-message">{{ log.message }}</span>
            </div>
          </div>
        </GlassPanel>
      </div>
    </template>
  </div>
</template>

<style scoped>
.instance-detail-view {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.instance-detail__header-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
}

.instance-detail__launch-error {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: 12px;
  padding: 10px 14px;
  background: rgba(224, 64, 64, 0.15);
  border: 1px solid rgba(224, 64, 64, 0.3);
  border-radius: var(--border-radius);
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-danger);
}

.instance-detail__launch-error-icon {
  flex-shrink: 0;
}

.instance-detail__launch-error-text {
  word-break: break-all;
}

.instance-detail__install-progress {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin-top: 12px;
  padding: 12px 14px;
  background: rgba(80, 144, 224, 0.1);
  border: 1px solid rgba(80, 144, 224, 0.3);
  border-radius: var(--border-radius);
}

.instance-detail__install-icon {
  font-size: 18px;
  flex-shrink: 0;
  margin-top: 1px;
}

.instance-detail__install-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 6px;
  min-width: 0;
}

.instance-detail__install-title {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-info);
  font-weight: bold;
}

.instance-detail__install-bar {
  height: 8px;
  background: var(--color-surface-hover);
  border-radius: 4px;
  overflow: hidden;
  border: 1px solid var(--color-border);
}

.instance-detail__install-bar-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--color-primary), var(--color-info));
  border-radius: 4px;
  transition: width 0.5s ease;
  min-width: 2px;
}

.instance-detail__install-detail {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.instance-detail__header-info {
  flex: 1;
  min-width: 0;
}

.instance-detail__name {
  font-family: var(--font-display);
  font-size: var(--font-size-lg);
  color: var(--color-primary);
  margin-bottom: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.instance-detail__badges {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.instance-detail__tabs {
  display: flex;
  gap: 2px;
  border-bottom: var(--border-width) solid var(--color-border);
}

.instance-detail__tab {
  padding: 8px 16px;
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  background: transparent;
  border: none;
  border-bottom: 2px solid transparent;
  cursor: pointer;
  transition: all var(--transition-fast);
  margin-bottom: -2px;
}

.instance-detail__tab:hover {
  color: var(--color-text);
}

.instance-detail__tab--active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.instance-detail__tab-content {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.instance-detail__overview-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-md);
}

.instance-detail__section-title {
  font-family: var(--font-display);
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  margin-bottom: 16px;
  padding-bottom: 8px;
  border-bottom: var(--border-width) solid var(--color-border);
}

.instance-detail__stats {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.instance-detail__stat {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.instance-detail__stat-label {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
}

.instance-detail__stat-value {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}

.instance-detail__stat-path {
  font-size: var(--font-size-xs);
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.instance-detail__danger-zone {
  display: flex;
  justify-content: flex-end;
  padding-top: var(--space-md);
  border-top: 1px dashed var(--color-border);
}

.instance-detail__log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-right: 12px;
  border-bottom: var(--border-width) solid var(--color-border);
}

.instance-detail__log-container {
  max-height: 400px;
  overflow-y: auto;
  padding: 8px;
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  background: rgba(0, 0, 0, 0.3);
}

.instance-detail__log-empty {
  padding: 24px;
  text-align: center;
  color: var(--color-text-muted);
}

.instance-detail__log-line {
  display: flex;
  gap: 8px;
  padding: 2px 4px;
  border-radius: 2px;
  line-height: 1.5;
}

.instance-detail__log-line:hover {
  background: rgba(255, 255, 255, 0.03);
}

.instance-detail__log-time {
  color: var(--color-text-muted);
  flex-shrink: 0;
}

.instance-detail__log-level {
  flex-shrink: 0;
  min-width: 50px;
}

.instance-detail__log-message {
  flex: 1;
  color: var(--color-text);
  word-break: break-all;
}

.log-line--error .instance-detail__log-level,
.log-line--error .instance-detail__log-message {
  color: var(--color-danger);
}

.log-line--warn .instance-detail__log-level,
.log-line--warn .instance-detail__log-message {
  color: var(--color-warning);
}

.log-line--debug .instance-detail__log-level,
.log-line--debug .instance-detail__log-message {
  color: var(--color-text-muted);
}

.log-line--info .instance-detail__log-level {
  color: var(--color-info);
}
</style>
