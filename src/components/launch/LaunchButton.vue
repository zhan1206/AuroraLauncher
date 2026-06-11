<script setup lang="ts">
/**
 * LaunchButton — Large launch/stop button with animated states.
 * Automatically prompts to download missing version files before launching.
 */
import { computed, ref } from 'vue';
import { useLaunchStore } from '@/stores/launch';
import { tauriCommand } from '@/composables/useTauriCommand';

export interface LaunchButtonProps {
  /** The instance ID to launch. */
  instanceId: string;
  /** The version ID needed by this instance. */
  versionId?: string;
  /** Whether the button is disabled. */
  disabled?: boolean;
}

const props = withDefaults(defineProps<LaunchButtonProps>(), {
  versionId: '',
  disabled: false,
});

const launchStore = useLaunchStore();
const checking = ref(false);

/** Current launch status */
const status = computed(() => {
  if (launchStore.currentInstanceId !== props.instanceId) return 'idle';
  return launchStore.status;
});

/** Whether the button is in launching state. */
const isLaunching = computed(() => status.value === 'launching' || status.value === 'preparing' || status.value === 'installing');

/** Whether the game is running. */
const isRunning = computed(() => status.value === 'running');

/** Whether we're checking install status. */
const isChecking = computed(() => checking.value);

/** Whether we're installing (download in progress). */
const isInstalling = computed(() => status.value === 'installing');

/** Button label text. */
const buttonText = computed(() => {
  if (isChecking.value) return '检查版本...';
  if (isInstalling.value) {
    const pct = launchStore.currentInstallProgress?.percent ?? 0;
    return `下载中 ${pct}%...`;
  }
  if (isLaunching.value) return '启动中...';
  if (isRunning.value) return '停止游戏';
  if (status.value === 'crashed') return '游戏崩溃 - 重试';
  return '▶ 启动游戏';
});

/** Error tooltip text when crashed. */
const errorTooltip = computed(() => {
  if (status.value === 'crashed' && launchStore.error) {
    return launchStore.error;
  }
  return undefined;
});

/** Check if version is installed, prompt to download if not. */
async function checkAndLaunch(): Promise<void> {
  const vid = props.versionId;
  if (!vid) {
    await launchStore.launch(props.instanceId);
    return;
  }

  checking.value = true;
  try {
    const installed = await tauriCommand<boolean>('check_version_installed', { versionId: vid });
    if (!installed) {
      const confirmed = window.confirm(
        `版本 ${vid} 尚未下载，是否立即下载并安装？\n\n点击"确定"开始下载，下载完成后将自动启动游戏。`
      );
      if (confirmed) {
        await launchStore.installAndLaunch(props.instanceId, vid);
        return;
      }
      return;
    }
    await launchStore.launch(props.instanceId);
  } catch (e) {
    await launchStore.launch(props.instanceId);
  } finally {
    checking.value = false;
  }
}

/** Launch or kill the game. */
function handleLaunch(): void {
  if (isRunning.value) {
    launchStore.kill();
  } else {
    checkAndLaunch();
  }
}
</script>

  <template>
  <div class="launch-button-wrapper">
    <button
      class="launch-button"
      :class="{
        'launch-button--launching': isLaunching || isChecking,
        'launch-button--installing': isInstalling,
        'launch-button--running': isRunning,
        'launch-button--crashed': status === 'crashed',
        'launch-button--disabled': disabled,
      }"
      :disabled="(disabled && !isRunning) || isChecking || isInstalling"
      @click="handleLaunch"
    >
      <!-- Checking or Launching spinner -->
      <span v-if="isChecking || isLaunching" class="launch-button__spinner" />

      <span class="launch-button__text">{{ buttonText }}</span>
    </button>

    <!-- Error message display -->
    <p v-if="status === 'crashed' && launchStore.error" class="launch-button__error">
      ⚠️ {{ launchStore.error }}
    </p>
  </div>
</template>

<style scoped>
.launch-button {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 14px 40px;
  font-family: var(--font-display);
  font-size: var(--font-size-base);
  color: var(--color-text-inverse);
  background: var(--color-primary);
  border: var(--border-width) solid var(--color-primary-dark);
  border-radius: var(--border-radius);
  box-shadow: var(--shadow-pixel), var(--shadow-glow);
  cursor: pointer;
  transition: all var(--transition-normal);
  text-shadow: 0 1px 0 rgba(0, 0, 0, 0.3);
  user-select: none;
  position: relative;
  overflow: hidden;
}

.launch-button:hover {
  background: var(--color-primary-light);
  border-color: var(--color-primary);
  box-shadow: var(--shadow-pixel-lg), var(--shadow-glow-strong);
  transform: translate(-1px, -2px);
}

.launch-button:active {
  transform: translate(1px, 1px);
  box-shadow: var(--shadow-pixel-sm);
}

.launch-button--running {
  background: var(--color-danger);
  border-color: #c04040;
  box-shadow: var(--shadow-pixel);
}

.launch-button--running:hover {
  background: #e86060;
  border-color: var(--color-danger);
  box-shadow: var(--shadow-pixel);
}

.launch-button--crashed {
  background: var(--color-warning);
  border-color: #c0a030;
  animation: glowPulse 2s ease-in-out infinite alternate;
}

.launch-button--disabled {
  opacity: 0.4;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.launch-button__spinner {
  display: inline-block;
  width: 18px;
  height: 18px;
  border: 3px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

.launch-button--launching {
  animation: glowPulse 1.5s ease-in-out infinite alternate;
}

.launch-button--installing {
  background: var(--color-info);
  border-color: #3070c0;
  animation: glowPulse 1.5s ease-in-out infinite alternate;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.launch-button__text {
  display: inline-flex;
  align-items: center;
}
</style>
