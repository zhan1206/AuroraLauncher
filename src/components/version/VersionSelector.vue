<script setup lang="ts">
/**
 * VersionSelector — Version selection component with filtering and search.
 */
import { ref, computed, onMounted } from 'vue';
import PixelInput from '@/components/common/PixelInput.vue';
import { useVersionStore } from '@/stores/version';
import { formatVersionType } from '@/utils/formatters';
import type { VersionTypeFilter } from '@/types/version';

export interface VersionSelectorProps {
  /** Selected version ID (v-model). */
  modelValue: string;
  /** Optional loader type to filter compatible versions. */
  loaderType?: string;
}

withDefaults(defineProps<VersionSelectorProps>(), {
  loaderType: '',
});

const emit = defineEmits<{
  'update:modelValue': [value: string];
}>();

const versionStore = useVersionStore();

// Filter state
const searchQuery = ref('');
const activeFilter = ref<VersionTypeFilter>('all');

// Load the version manifest on mount
onMounted(() => {
  if (!versionStore.manifest) {
    versionStore.loadManifest();
  }
});

/** Filtered versions based on search and type filter. */
const filteredVersions = computed(() => {
  let versions = versionStore.getFilteredVersions(activeFilter.value);

  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase().trim();
    versions = versions.filter((v) => v.id.toLowerCase().includes(query));
  }

  return versions;
});

/** Select a version. */
function selectVersion(versionId: string): void {
  emit('update:modelValue', versionId);
}

/** Filter tabs. */
const filterTabs: { key: VersionTypeFilter; label: string }[] = [
  { key: 'all', label: '全部' },
  { key: 'release', label: '正式版' },
  { key: 'snapshot', label: '快照' },
  { key: 'old_alpha', label: '远古版' },
];
</script>

<template>
  <div class="version-selector">
    <!-- Search -->
    <PixelInput
      v-model="searchQuery"
      placeholder="搜索版本..."
      style="margin-bottom: 8px;"
    />

    <!-- Filter tabs -->
    <div class="version-selector__filters">
      <button
        v-for="tab in filterTabs"
        :key="tab.key"
        class="version-selector__filter-btn"
        :class="{ 'version-selector__filter-btn--active': activeFilter === tab.key }"
        @click="activeFilter = tab.key"
      >
        {{ tab.label }}
      </button>
    </div>

    <!-- Version list -->
    <div class="version-selector__list">
      <div v-if="versionStore.loading" class="version-selector__loading">
        加载版本列表中...
      </div>

      <div v-else-if="versionStore.error" class="version-selector__error">
        {{ versionStore.error }}
      </div>

      <div v-else-if="filteredVersions.length === 0" class="version-selector__empty">
        未找到匹配的版本
      </div>

      <div
        v-for="version in filteredVersions"
        :key="version.id"
        class="version-selector__item"
        :class="{ 'version-selector__item--selected': version.id === modelValue }"
        @click="selectVersion(version.id)"
      >
        <span class="version-selector__item-id">{{ version.id }}</span>
        <span class="version-selector__item-type tag-pixel">{{ formatVersionType(version.type) }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.version-selector {
  display: flex;
  flex-direction: column;
}

.version-selector__filters {
  display: flex;
  gap: 4px;
  margin-bottom: 8px;
}

.version-selector__filter-btn {
  padding: 4px 10px;
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  background: transparent;
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.version-selector__filter-btn:hover {
  background: var(--color-surface-hover);
  border-color: var(--color-border-hover);
}

.version-selector__filter-btn--active {
  color: var(--color-primary);
  background: rgba(126, 200, 80, 0.1);
  border-color: var(--color-primary);
}

.version-selector__list {
  max-height: 240px;
  overflow-y: auto;
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  background: rgba(0, 0, 0, 0.2);
}

.version-selector__item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  cursor: pointer;
  transition: all var(--transition-fast);
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
}

.version-selector__item:hover {
  background: var(--color-surface-hover);
}

.version-selector__item--selected {
  background: rgba(126, 200, 80, 0.1);
  border-left: 2px solid var(--color-primary);
}

.version-selector__item-id {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text);
}

.version-selector__item--selected .version-selector__item-id {
  color: var(--color-primary);
}

.version-selector__loading,
.version-selector__empty {
  padding: 24px;
  text-align: center;
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
}

.version-selector__error {
  padding: 24px;
  text-align: center;
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-danger);
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
