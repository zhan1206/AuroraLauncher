<script setup lang="ts">
/**
 * SettingsView — Application settings with tabbed interface.
 */
import { onMounted, ref } from 'vue';
import { useSettingsStore } from '@/stores/settings';
import type { AppSettings, DownloadMirror } from '@/types/settings';
import AppHeader from '@/components/layout/AppHeader.vue';
import PixelInput from '@/components/common/PixelInput.vue';
import PixelButton from '@/components/common/PixelButton.vue';
import PixelSelect from '@/components/common/PixelSelect.vue';
import PixelSwitch from '@/components/common/PixelSwitch.vue';
import GlassPanel from '@/components/common/GlassPanel.vue';
import { useToast } from '@/components/common/useToast';

const settingsStore = useSettingsStore();
const toast = useToast();

// Tab state
const activeTab = ref<'general' | 'java' | 'download' | 'appearance'>('general');

// General settings
const language = ref('zh-CN');
const dataDir = ref('');

// Java settings
const javaPath = ref('');
const detectedJavaPaths = ref<string[]>([]);
const minMemory = ref(512);
const maxMemory = ref(2048);

// Download settings
const downloadMirror = ref<DownloadMirror>('Official');
const maxConcurrent = ref(8);

// Appearance settings
const animationEnabled = ref(true);

// Mirror options
const mirrorOptions = [
  { label: '官方源 (Mojang)', value: 'Official' },
  { label: 'BMCLAPI (国内镜像)', value: 'Bmclapi' },
];

// Concurrent options
const concurrentOptions = [
  { label: '1', value: 1 },
  { label: '2', value: 2 },
  { label: '4', value: 4 },
  { label: '8', value: 8 },
];

// Language options
const languageOptions = [
  { label: '简体中文', value: 'zh-CN' },
  { label: 'English', value: 'en-US' },
];

// Load settings from store
onMounted(async () => {
  await settingsStore.fetchSettings();
  const s = settingsStore.settings;
  javaPath.value = s.custom_java_path ?? '';
  minMemory.value = s.default_min_memory;
  maxMemory.value = s.default_max_memory;
  language.value = s.language;
  downloadMirror.value = s.download_mirror;
  maxConcurrent.value = s.download_concurrency;
});

/** Auto-detect Java via java list command. */
async function detectJava(): Promise<void> {
  try {
    const { tauriCommand } = await import('@/composables/useTauriCommand');
    const runtimes = await tauriCommand<Array<{ path: string; version: string }>>('list_java_runtimes');
    if (runtimes.length > 0) {
      const first = runtimes[0];
      javaPath.value = first.path;
      detectedJavaPaths.value = runtimes.map((r) => r.path);
      toast.info(`已检测到 Java ${first.version}`);
    } else {
      toast.warning('未检测到 Java 安装');
    }
  } catch {
    toast.warning('未检测到 Java 安装');
  }
}

/** Save all settings. */
async function saveSettings(): Promise<void> {
  const updates: Partial<AppSettings> = {
    custom_java_path: javaPath.value || null,
    default_min_memory: minMemory.value,
    default_max_memory: maxMemory.value,
    language: language.value,
    download_mirror: downloadMirror.value,
    download_concurrency: maxConcurrent.value,
  };
  const success = await settingsStore.saveSettings(updates);
  if (success) {
    toast.success('设置已保存');
  } else {
    toast.error('保存设置失败');
  }
}

/** Add a Java path manually. */
const newJavaPath = ref('');
function addJavaPath(): void {
  if (newJavaPath.value.trim()) {
    javaPath.value = newJavaPath.value.trim();
    if (!detectedJavaPaths.value.includes(newJavaPath.value.trim())) {
      detectedJavaPaths.value.push(newJavaPath.value.trim());
    }
    newJavaPath.value = '';
    toast.info('Java 路径已设置');
  }
}
</script>

<template>
  <div class="settings-view animate-fade-in">
    <AppHeader title="设置" />

    <!-- Tab navigation -->
    <div class="settings__tabs">
      <button
        class="settings__tab"
        :class="{ 'settings__tab--active': activeTab === 'general' }"
        @click="activeTab = 'general'"
      >
        通用
      </button>
      <button
        class="settings__tab"
        :class="{ 'settings__tab--active': activeTab === 'java' }"
        @click="activeTab = 'java'"
      >
        Java
      </button>
      <button
        class="settings__tab"
        :class="{ 'settings__tab--active': activeTab === 'download' }"
        @click="activeTab = 'download'"
      >
        下载
      </button>
      <button
        class="settings__tab"
        :class="{ 'settings__tab--active': activeTab === 'appearance' }"
        @click="activeTab = 'appearance'"
      >
        外观
      </button>
    </div>

    <!-- Tab: General -->
    <div v-if="activeTab === 'general'" class="settings__content">
      <GlassPanel padding="20px">
        <h3 class="settings__section-title">通用设置</h3>

        <div class="settings__field">
          <PixelSelect
            v-model="language"
            label="语言"
            :options="languageOptions"
          />
        </div>

        <div class="settings__field">
          <PixelInput
            v-model="dataDir"
            label="数据目录"
            placeholder="使用默认路径"
          />
        </div>
      </GlassPanel>
    </div>

    <!-- Tab: Java -->
    <div v-if="activeTab === 'java'" class="settings__content">
      <GlassPanel padding="20px">
        <h3 class="settings__section-title">Java 配置</h3>

        <div class="settings__field">
          <PixelInput
            v-model="javaPath"
            label="当前 Java 路径"
            placeholder="自动检测"
          />
          <div class="settings__field-actions">
            <PixelButton size="sm" @click="detectJava">自动检测</PixelButton>
          </div>
        </div>

        <!-- Detected Java paths -->
        <div v-if="detectedJavaPaths.length > 0" class="settings__field">
          <label class="settings__label">已检测到的 Java</label>
          <div v-for="path in detectedJavaPaths" :key="path" class="settings__java-item">
            <span class="settings__java-path">{{ path }}</span>
            <PixelButton size="sm" @click="javaPath = path">使用</PixelButton>
          </div>
        </div>

        <!-- Add Java path manually -->
        <div class="settings__field">
          <PixelInput
            v-model="newJavaPath"
            label="手动添加 Java 路径"
            placeholder="C:\Program Files\Java\jdk-17\bin\javaw.exe"
          />
          <div class="settings__field-actions">
            <PixelButton size="sm" @click="addJavaPath">添加</PixelButton>
          </div>
        </div>

        <!-- Memory settings -->
        <div class="settings__field">
          <label class="settings__label">默认内存配置 (MB)</label>
          <div class="settings__memory-row">
            <div class="settings__memory-field">
              <label class="settings__sublabel">最小</label>
              <input v-model.number="minMemory" type="number" class="input-pixel" min="128" step="128" />
            </div>
            <span class="settings__range-sep">—</span>
            <div class="settings__memory-field">
              <label class="settings__sublabel">最大</label>
              <input v-model.number="maxMemory" type="number" class="input-pixel" min="128" step="128" />
            </div>
          </div>
        </div>
      </GlassPanel>
    </div>

    <!-- Tab: Download -->
    <div v-if="activeTab === 'download'" class="settings__content">
      <GlassPanel padding="20px">
        <h3 class="settings__section-title">下载设置</h3>

        <div class="settings__field">
          <PixelSelect
            v-model="downloadMirror"
            label="下载源"
            :options="mirrorOptions"
          />
          <p class="settings__hint">国内用户建议使用 BMCLAPI 镜像以加快下载速度</p>
        </div>

        <div class="settings__field">
          <PixelSelect
            v-model="maxConcurrent"
            label="最大并发下载数"
            :options="concurrentOptions"
          />
        </div>
      </GlassPanel>
    </div>

    <!-- Tab: Appearance -->
    <div v-if="activeTab === 'appearance'" class="settings__content">
      <GlassPanel padding="20px">
        <h3 class="settings__section-title">外观设置</h3>

        <div class="settings__field">
          <div class="settings__switch-row">
            <PixelSwitch v-model="animationEnabled" label="启用动画效果" />
          </div>
        </div>

        <div class="settings__field">
          <label class="settings__label">主题色（即将推出）</label>
          <div class="settings__theme-preview">
            <div class="settings__theme-color" style="background: var(--color-primary);" title="草方块绿" />
            <div class="settings__theme-color" style="background: #5CE1E6;" title="钻石蓝" />
            <div class="settings__theme-color" style="background: #A855F7;" title="末影紫" />
          </div>
          <p class="settings__hint">自定义主题色功能将在后续版本推出</p>
        </div>
      </GlassPanel>
    </div>

    <!-- Save button -->
    <div class="settings__save">
      <PixelButton variant="primary" @click="saveSettings">
        💾 保存设置
      </PixelButton>
    </div>
  </div>
</template>

<style scoped>
.settings-view {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.settings__tabs {
  display: flex;
  gap: 2px;
  border-bottom: var(--border-width) solid var(--color-border);
}

.settings__tab {
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

.settings__tab:hover {
  color: var(--color-text);
}

.settings__tab--active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.settings__content {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.settings__section-title {
  font-family: var(--font-display);
  font-size: var(--font-size-xs);
  color: var(--color-primary);
  margin-bottom: 20px;
  padding-bottom: 8px;
  border-bottom: var(--border-width) solid var(--color-border);
}

.settings__field {
  margin-bottom: 16px;
}

.settings__field-actions {
  margin-top: 6px;
  display: flex;
  gap: 6px;
}

.settings__label {
  display: block;
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  margin-bottom: 6px;
}

.settings__sublabel {
  display: block;
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  margin-bottom: 4px;
}

.settings__hint {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  margin-top: 4px;
}

.settings__memory-row {
  display: flex;
  align-items: flex-end;
  gap: 12px;
}

.settings__memory-field {
  flex: 1;
}

.settings__range-sep {
  font-size: var(--font-size-lg);
  color: var(--color-text-muted);
  padding-bottom: 8px;
}

.settings__java-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  background: rgba(0, 0, 0, 0.2);
  border-radius: var(--border-radius);
  margin-bottom: 4px;
}

.settings__java-path {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex: 1;
  margin-right: 8px;
}

.settings__switch-row {
  display: flex;
  align-items: center;
  padding: 8px 0;
}

.settings__theme-preview {
  display: flex;
  gap: 8px;
  margin-top: 4px;
}

.settings__theme-color {
  width: 32px;
  height: 32px;
  border-radius: var(--border-radius);
  border: 2px solid var(--color-border);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.settings__theme-color:hover {
  transform: scale(1.1);
  box-shadow: var(--shadow-glow);
}

.settings__save {
  display: flex;
  justify-content: flex-end;
  padding-top: var(--space-md);
  border-top: 1px dashed var(--color-border);
}
</style>
