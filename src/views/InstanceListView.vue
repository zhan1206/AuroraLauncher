<script setup lang="ts">
/**
 * InstanceListView — Instance list with search, view toggle, and creation.
 */
import { onMounted, ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useInstanceStore } from '@/stores/instance';
import { useLaunchStore } from '@/stores/launch';
import InstanceCard from '@/components/instance/InstanceCard.vue';
import InstanceCreateDialog from '@/components/instance/InstanceCreateDialog.vue';
import PixelButton from '@/components/common/PixelButton.vue';
import PixelInput from '@/components/common/PixelInput.vue';
import AppHeader from '@/components/layout/AppHeader.vue';
import GlassPanel from '@/components/common/GlassPanel.vue';
import { formatRelativeTime, formatLoaderType } from '@/utils/formatters';
import { useToast } from '@/components/common/useToast';
import type { Instance } from '@/types/instance';

const router = useRouter();
const instanceStore = useInstanceStore();
const launchStore = useLaunchStore();
const toast = useToast();

// View state
const searchQuery = ref('');
const viewMode = ref<'grid' | 'list'>('grid');
const showCreateDialog = ref(false);

// Load instances
onMounted(() => {
  instanceStore.loadInstances();
});

/** Filtered instances based on search. */
const filteredInstances = computed(() => {
  if (!searchQuery.value.trim()) return instanceStore.instances;
  const query = searchQuery.value.toLowerCase().trim();
  return instanceStore.instances.filter(
    (i) =>
      i.name.toLowerCase().includes(query) ||
      i.version_id.toLowerCase().includes(query)
  );
});

/** Launch an instance. */
function handleLaunch(instance: Instance): void {
  launchStore.launch(instance.id);
  toast.info(`正在启动 ${instance.name}...`);
}

/** Edit an instance (navigate to detail). */
function handleEdit(instance: Instance): void {
  router.push(`/instances/${instance.id}`);
}

/** Delete an instance. */
async function handleDelete(instance: Instance): Promise<void> {
  const confirmed = window.confirm(`确定要删除实例 "${instance.name}" 吗？此操作不可撤销。`);
  if (confirmed) {
    const success = await instanceStore.deleteInstance(instance.id);
    if (success) {
      toast.success(`已删除实例 "${instance.name}"`);
    } else {
      toast.error('删除失败');
    }
  }
}

/** Handle instance creation. */
function handleCreated(): void {
  showCreateDialog.value = false;
  instanceStore.loadInstances();
}
</script>

<template>
  <div class="instance-list-view animate-fade-in">
    <AppHeader title="实例列表" />

    <!-- Toolbar -->
    <div class="instance-list__toolbar">
      <PixelInput
        v-model="searchQuery"
        placeholder="搜索实例..."
        style="max-width: 300px;"
      />

      <div class="instance-list__toolbar-right">
        <!-- View toggle -->
        <div class="instance-list__view-toggle">
          <button
            class="instance-list__view-btn"
            :class="{ 'instance-list__view-btn--active': viewMode === 'grid' }"
            @click="viewMode = 'grid'"
            title="网格视图"
          >
            ▦
          </button>
          <button
            class="instance-list__view-btn"
            :class="{ 'instance-list__view-btn--active': viewMode === 'list' }"
            @click="viewMode = 'list'"
            title="列表视图"
          >
            ☰
          </button>
        </div>

        <PixelButton variant="primary" @click="showCreateDialog = true">
          + 新建实例
        </PixelButton>
      </div>
    </div>

    <!-- Loading state -->
    <div v-if="instanceStore.loading" class="instance-list__loading">
      <div v-for="i in 3" :key="i" class="glass-card animate-shimmer" style="height: 80px;" />
    </div>

    <!-- Error state -->
    <GlassPanel v-else-if="instanceStore.error" padding="24px" style="text-align: center;">
      <p style="font-size: 24px; margin-bottom: 12px;">⚠️</p>
      <p style="font-family: var(--font-body); font-size: 14px; color: var(--color-danger);">
        {{ instanceStore.error }}
      </p>
      <PixelButton style="margin-top: 12px;" @click="instanceStore.loadInstances()">重试</PixelButton>
    </GlassPanel>

    <!-- Grid view -->
    <div v-else-if="filteredInstances.length > 0 && viewMode === 'grid'" class="instance-list__grid">
      <InstanceCard
        v-for="instance in filteredInstances"
        :key="instance.id"
        :instance="instance"
        @launch="handleLaunch"
        @edit="handleEdit"
        @delete="handleDelete"
      />
    </div>

    <!-- List view (table style) -->
    <div v-else-if="filteredInstances.length > 0 && viewMode === 'list'" class="instance-list__table">
      <div class="instance-list__table-header">
        <span>名称</span>
        <span>版本</span>
        <span>加载器</span>
        <span>最后游玩</span>
        <span>操作</span>
      </div>
      <div
        v-for="instance in filteredInstances"
        :key="instance.id"
        class="instance-list__table-row"
      >
        <span class="instance-list__table-name" @click="handleEdit(instance)">
          {{ instance.name }}
        </span>
        <span class="tag-pixel">{{ instance.version_id }}</span>
        <span class="tag-pixel" style="color: var(--color-info); border-color: rgba(80,144,224,0.3); background: rgba(80,144,224,0.1);">
          {{ formatLoaderType(instance.loader_type) }}
        </span>
        <span class="instance-list__table-meta">{{ formatRelativeTime(instance.updated_at) }}</span>
        <div class="instance-list__table-actions">
          <PixelButton size="sm" variant="primary" @click="handleLaunch(instance)">▶</PixelButton>
          <PixelButton size="sm" @click="handleEdit(instance)">✏️</PixelButton>
          <PixelButton size="sm" variant="danger" @click="handleDelete(instance)">🗑️</PixelButton>
        </div>
      </div>
    </div>

    <!-- Empty state -->
    <GlassPanel v-else padding="48px" style="text-align: center;">
      <p style="font-size: 36px; margin-bottom: 16px;">🏗️</p>
      <p style="font-family: var(--font-display); font-size: 12px; color: var(--color-text-secondary); margin-bottom: 16px;">
        {{ searchQuery ? '未找到匹配的实例' : '还没有任何实例' }}
      </p>
      <PixelButton variant="primary" @click="showCreateDialog = true">
        创建你的第一个实例
      </PixelButton>
    </GlassPanel>

    <!-- Create dialog -->
    <InstanceCreateDialog
      v-model="showCreateDialog"
      @created="handleCreated"
    />
  </div>
</template>

<style scoped>
.instance-list-view {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.instance-list__toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-md);
}

.instance-list__toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.instance-list__view-toggle {
  display: flex;
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  overflow: hidden;
}

.instance-list__view-btn {
  padding: 6px 10px;
  font-size: 16px;
  color: var(--color-text-secondary);
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all var(--transition-fast);
}

.instance-list__view-btn:hover {
  background: var(--color-surface-hover);
}

.instance-list__view-btn--active {
  background: var(--color-surface-active);
  color: var(--color-primary);
}

.instance-list__loading {
  display: grid;
  gap: 12px;
}

.instance-list__grid {
  display: grid;
  gap: 10px;
}

.instance-list__table {
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  overflow: hidden;
}

.instance-list__table-header {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr 1fr;
  gap: 8px;
  padding: 10px 16px;
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  background: rgba(0, 0, 0, 0.2);
  border-bottom: var(--border-width) solid var(--color-border);
}

.instance-list__table-row {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 1fr 1fr;
  gap: 8px;
  padding: 10px 16px;
  align-items: center;
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
  transition: background var(--transition-fast);
}

.instance-list__table-row:hover {
  background: var(--color-surface-hover);
}

.instance-list__table-name {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text);
  cursor: pointer;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.instance-list__table-name:hover {
  color: var(--color-primary);
}

.instance-list__table-meta {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.instance-list__table-actions {
  display: flex;
  gap: 4px;
}
</style>
