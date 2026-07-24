<script setup lang="ts">
/**
 * VersionListView — Minecraft version browser with download & install.
 *
 * Shows all available Minecraft versions, marks installed ones,
 * and provides one-click download for missing versions.
 */
import { onMounted, ref, computed } from 'vue';
import { useVersionStore } from '@/stores/version';
import { tauriCommand } from '@/composables/useTauriCommand';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { VersionEntry } from '@/types/version';

const versionStore = useVersionStore();

// ── Filter state ──────────────────────────────────────────────
const activeFilter = ref<'all' | 'release' | 'snapshot' | 'old_beta' | 'old_alpha'>('release');

// ── Install state per version ─────────────────────────────────
const installingMap = ref<Record<string, boolean>>({});
const installProgress = ref<Record<string, { pct: number; file: string; stage: string }>>({});
const installedVersions = ref<Set<string>>(new Set());
const checkingInstalled = ref(false);

// ── Unlisteners ───────────────────────────────────────────────
let progressUnlisten: UnlistenFn | null = null;

// ── Filtered & sorted versions ────────────────────────────────
const filteredVersions = computed<VersionEntry[]>(() => {
  if (!versionStore.manifest) return [];
  const all = versionStore.manifest.versions;
  if (activeFilter.value === 'all') return all;
  return all.filter((v) => v.type === activeFilter.value);
});

/** Count of versions per type for the filter tabs. */
const typeCounts = computed(() => {
  if (!versionStore.manifest) return { all: 0, release: 0, snapshot: 0, old_beta: 0, old_alpha: 0 };
  const vers = versionStore.manifest.versions;
  return {
    all: vers.length,
    release: vers.filter((v) => v.type === 'release').length,
    snapshot: vers.filter((v) => v.type === 'snapshot').length,
    old_beta: vers.filter((v) => v.type === 'old_beta').length,
    old_alpha: vers.filter((v) => v.type === 'old_alpha').length,
  };
});

const filterTabs = [
  { key: 'release' as const, label: '正式版' },
  { key: 'snapshot' as const, label: '快照版' },
  { key: 'old_beta' as const, label: '经典版' },
  { key: 'old_alpha' as const, label: '远古版' },
  { key: 'all' as const, label: '全部' },
];

// ── Lifecycle ─────────────────────────────────────────────────

onMounted(async () => {
  await versionStore.loadManifest();
  if (versionStore.manifest) {
    checkAllInstalled();
  }
  listenForProgress();
});

// ── Install progress listener (only set up once) ───────────────

async function listenForProgress() {
  if (progressUnlisten) return;
  const p = await listen<{
    version_id: string;
    total_files: number;
    completed_files: number;
    total_bytes: number;
    downloaded_bytes: number;
    current_file: string;
    stage: string;
  }>('install:progress', (event) => {
    const { version_id, downloaded_bytes, total_bytes, current_file, stage } = event.payload;
    const pct = total_bytes > 0 ? Math.round((downloaded_bytes / total_bytes) * 100) : 0;
    installProgress.value[version_id] = {
      pct,
      file: current_file,
      stage,
    };
    if (stage === 'completed') {
      installingMap.value[version_id] = false;
      installedVersions.value.add(version_id);
      delete installProgress.value[version_id];
    }
  });
  progressUnlisten = p;
}

// ── Check which versions are already installed ─────────────────

async function checkAllInstalled() {
  checkingInstalled.value = true;
  const vers = versionStore.manifest?.versions ?? [];
  // Check in batches to avoid blocking
  for (let i = 0; i < vers.length; i += 10) {
    const batch = vers.slice(i, i + 10);
    const results = await Promise.allSettled(
      batch.map((v) => tauriCommand<boolean>('check_version_installed', { versionId: v.id }))
    );
    results.forEach((r, j) => {
      if (r.status === 'fulfilled' && r.value) {
        installedVersions.value.add(batch[j].id);
      }
    });
  }
  checkingInstalled.value = false;
}

// ── Install a version ─────────────────────────────────────────

async function installVersion(versionId: string) {
  installingMap.value[versionId] = true;
  installProgress.value[versionId] = { pct: 0, file: '', stage: 'starting' };
  try {
    await tauriCommand<void>('install_version', { versionId });
    installedVersions.value.add(versionId);
  } catch (e: any) {
    console.error('Install failed:', e);
    alert(`安装失败: ${e.message ?? String(e)}`);
  } finally {
    installingMap.value[versionId] = false;
    delete installProgress.value[versionId];
  }
}

// ── Format helpers ────────────────────────────────────────────

function formatDate(iso: string): string {
  if (!iso) return '';
  try {
    return new Date(iso).toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' });
  } catch {
    return iso;
  }
}

function typeLabel(type: string): string {
  const map: Record<string, string> = {
    release: '正式版',
    snapshot: '快照版',
    old_beta: '经典版',
    old_alpha: '远古版',
  };
  return map[type] ?? type;
}

function typeTagClass(type: string): string {
  const map: Record<string, string> = {
    release: 'tag-release',
    snapshot: 'tag-snapshot',
    old_beta: 'tag-beta',
    old_alpha: 'tag-alpha',
  };
  return map[type] ?? '';
}
</script>

<template>
  <div class="version-list-view animate-fade-in">
    <!-- Header -->
    <div class="section-header">
      <h2 class="section-title">📦 版本管理</h2>
      <button class="btn-pixel btn-pixel-secondary" @click="checkAllInstalled" :disabled="checkingInstalled">
        {{ checkingInstalled ? '检查中...' : '刷新已安装状态' }}
      </button>
    </div>

    <!-- Filter tabs -->
    <div class="version-filter-tabs">
      <button
        v-for="tab in filterTabs"
        :key="tab.key"
        class="version-filter-tab"
        :class="{ 'version-filter-tab--active': activeFilter === tab.key }"
        @click="activeFilter = tab.key"
      >
        <span>{{ tab.label }}</span>
        <span class="version-filter-tab__badge">{{ typeCounts[tab.key] }}</span>
      </button>
    </div>

    <!-- Loading state -->
    <div v-if="versionStore.loading" class="version-list-loading glass-panel-grid" style="padding: 32px; text-align: center;">
      <p class="text-muted">加载版本列表中...</p>
    </div>

    <!-- Error state -->
    <div v-else-if="versionStore.error" class="version-list-error glass-panel-grid" style="padding: 24px;">
      <span style="color: var(--color-danger);">加载失败: {{ versionStore.error }}</span>
      <button class="btn-pixel btn-pixel-secondary" @click="versionStore.loadManifest()">重试</button>
    </div>

    <!-- Version table -->
    <div v-else class="version-table-wrapper glass-panel-grid" style="padding: 0; overflow: hidden;">
      <table class="version-table">
        <thead>
          <tr>
            <th style="width: 40%;">版本</th>
            <th style="width: 12%;">类型</th>
            <th style="width: 15%;">发布日期</th>
            <th style="width: 15%;">状态</th>
            <th style="width: 18%;">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="ver in filteredVersions" :key="ver.id" class="version-row">
            <!-- Version ID -->
            <td class="version-cell-id">
              <span class="version-id">{{ ver.id }}</span>
            </td>

            <!-- Type badge -->
            <td>
              <span class="version-type-tag" :class="typeTagClass(ver.type)">
                {{ typeLabel(ver.type) }}
              </span>
            </td>

            <!-- Release date -->
            <td class="version-cell-date">{{ formatDate(ver.release_time) }}</td>

            <!-- Install status -->
            <td>
              <span v-if="installedVersions.has(ver.id)" class="version-status version-status--installed">
                ✅ 已安装
              </span>
              <span v-else-if="installingMap[ver.id]" class="version-status version-status--installing">
                ⏳ 下载中
                <span v-if="installProgress[ver.id]" class="install-pct">
                  {{ installProgress[ver.id].pct }}%
                </span>
              </span>
              <span v-else class="version-status version-status--not-installed">
                ⬜ 未安装
              </span>
            </td>

            <!-- Actions -->
            <td>
              <!-- Install progress bar when downloading -->
              <div v-if="installingMap[ver.id] && installProgress[ver.id]" class="install-bar-wrapper">
                <div class="install-bar">
                  <div
                    class="install-bar__fill"
                    :style="{ width: installProgress[ver.id].pct + '%' }"
                  />
                </div>
                <span class="install-bar__file" :title="installProgress[ver.id].file">
                  {{ installProgress[ver.id].file || '准备中...' }}
                </span>
              </div>
              <!-- Download button when not installed -->
              <button
                v-else-if="!installedVersions.has(ver.id)"
                class="btn-pixel btn-pixel-primary btn-sm"
                @click="installVersion(ver.id)"
                :disabled="installingMap[ver.id]"
              >
                ⬇ 下载
              </button>
              <!-- Already installed -->
              <span v-else class="text-success">
                ✅ 可用
              </span>
            </td>
          </tr>
        </tbody>
      </table>

      <!-- Empty state -->
      <div v-if="filteredVersions.length === 0 && !versionStore.loading" class="version-list-empty" style="padding: 32px; text-align: center;">
        <p class="text-muted">该分类下没有版本</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.version-list-view {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.version-filter-tabs {
  display: flex;
  gap: 4px;
  flex-wrap: wrap;
}

.version-filter-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--border-radius);
  cursor: pointer;
  transition: all 0.15s ease;
}

.version-filter-tab:hover {
  color: var(--color-text);
  border-color: var(--color-primary);
}

.version-filter-tab--active {
  color: var(--color-primary);
  border-color: var(--color-primary);
  background: rgba(126, 200, 80, 0.1);
}

.version-filter-tab__badge {
  font-size: 10px;
  color: var(--color-text-muted);
  background: var(--color-surface-hover);
  padding: 1px 6px;
  border-radius: 10px;
}

.version-table-wrapper {
  overflow-x: auto;
}

.version-table {
  width: 100%;
  border-collapse: collapse;
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
}

.version-table thead {
  background: var(--color-surface-hover);
  border-bottom: 2px solid var(--color-border);
}

.version-table th {
  padding: 10px 14px;
  text-align: left;
  font-weight: bold;
  color: var(--color-text-secondary);
  font-size: var(--font-size-xs);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  white-space: nowrap;
}

.version-row {
  border-bottom: 1px solid var(--color-border);
  transition: background 0.1s ease;
}

.version-row:hover {
  background: var(--color-surface-hover);
}

.version-row td {
  padding: 10px 14px;
  vertical-align: middle;
}

.version-id {
  font-family: var(--font-display);
  font-weight: bold;
  color: var(--color-text);
}

.version-type-tag {
  font-size: var(--font-size-xs);
  padding: 2px 8px;
  border-radius: 3px;
  white-space: nowrap;
}

.tag-release {
  color: var(--color-primary);
  background: rgba(126, 200, 80, 0.1);
  border: 1px solid rgba(126, 200, 80, 0.3);
}

.tag-snapshot {
  color: var(--color-warning, #e0a040);
  background: rgba(224, 160, 64, 0.1);
  border: 1px solid rgba(224, 160, 64, 0.3);
}

.tag-beta {
  color: var(--color-info, #5090e0);
  background: rgba(80, 144, 224, 0.1);
  border: 1px solid rgba(80, 144, 224, 0.3);
}

.tag-alpha {
  color: var(--color-danger, #e04040);
  background: rgba(224, 64, 64, 0.1);
  border: 1px solid rgba(224, 64, 64, 0.3);
}

.version-cell-date {
  color: var(--color-text-muted);
  white-space: nowrap;
  font-size: var(--font-size-xs);
}

.version-status {
  font-size: var(--font-size-xs);
  white-space: nowrap;
}

.version-status--installed {
  color: var(--color-success, #40a040);
}

.version-status--installing {
  color: var(--color-warning, #e0a040);
}

.version-status--not-installed {
  color: var(--color-text-muted);
}

.install-pct {
  margin-left: 4px;
  font-weight: bold;
}

.install-bar-wrapper {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 120px;
}

.install-bar {
  height: 6px;
  background: var(--color-surface-hover);
  border-radius: 3px;
  overflow: hidden;
  border: 1px solid var(--color-border);
}

.install-bar__fill {
  height: 100%;
  background: var(--color-primary);
  border-radius: 3px;
  transition: width 0.3s ease;
}

.install-bar__file {
  font-size: 10px;
  color: var(--color-text-muted);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 180px;
}

.text-muted {
  color: var(--color-text-muted);
}

.text-success {
  color: var(--color-success, #40a040);
  font-size: var(--font-size-xs);
  white-space: nowrap;
}

.btn-sm {
  padding: 4px 12px;
  font-size: var(--font-size-xs);
}

/* Empty / loading states */
.version-list-loading,
.version-list-error,
.version-list-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}
</style>
