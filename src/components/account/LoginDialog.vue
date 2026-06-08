<script setup lang="ts">
/**
 * LoginDialog — Account login dialog with Microsoft and offline tabs.
 */
import { ref, computed, watch, onUnmounted } from 'vue';
import PixelDialog from '@/components/common/PixelDialog.vue';
import PixelInput from '@/components/common/PixelInput.vue';
import PixelButton from '@/components/common/PixelButton.vue';
import { useAccountStore } from '@/stores/account';
import { validateUsername } from '@/utils/validators';
import { useToast } from '@/components/common/useToast';

export interface LoginDialogProps {
  /** Whether the dialog is visible (v-model). */
  modelValue: boolean;
}

defineProps<LoginDialogProps>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  loggedIn: [];
}>();

const accountStore = useAccountStore();
const toast = useToast();

// Tab state
const activeTab = ref<'microsoft' | 'offline'>('microsoft');

// Offline login state
const offlineUsername = ref('');
const offlineUsernameError = computed(() => {
  if (!offlineUsername.value) return '';
  const result = validateUsername(offlineUsername.value);
  return result.valid ? '' : result.message;
});
const offlineLoggingIn = ref(false);

// Microsoft login state
const msLoggingIn = ref(false);
const countdown = ref(0);
let countdownTimer: ReturnType<typeof setInterval> | null = null;

/** Device code for Microsoft login. */
const deviceCode = computed(() => accountStore.deviceCode);

/** Start Microsoft login flow. */
async function startMicrosoftLogin(): Promise<void> {
  msLoggingIn.value = true;
  try {
    await accountStore.loginMicrosoft();
    // Start countdown if device code is available
    if (deviceCode.value) {
      // DeviceCodeResponse only has user_code and verification_uri
      // countdown is not directly available; set a default 5-minute window
      countdown.value = 300;
      countdownTimer = setInterval(() => {
        countdown.value--;
        if (countdown.value <= 0) {
          if (countdownTimer) clearInterval(countdownTimer);
          countdownTimer = null;
        }
      }, 1000);
    }
  } catch (e) {
    toast.error('微软登录启动失败');
  } finally {
    msLoggingIn.value = false;
  }
}

/** Handle offline login. */
async function handleOfflineLogin(): Promise<void> {
  if (offlineUsernameError.value || !offlineUsername.value.trim()) return;

  offlineLoggingIn.value = true;
  try {
    const account = await accountStore.loginOffline(offlineUsername.value.trim());
    if (account) {
      toast.success(`欢迎, ${account.username}!`);
      emit('loggedIn');
      handleClose();
    }
  } catch (e) {
    toast.error('登录失败');
  } finally {
    offlineLoggingIn.value = false;
  }
}

/** Copy device code to clipboard. */
async function copyDeviceCode(): Promise<void> {
  if (!deviceCode.value) return;
  try {
    await navigator.clipboard.writeText(deviceCode.value.user_code);
    toast.info('设备码已复制');
  } catch {
    toast.error('复制失败');
  }
}

/** Close the dialog. */
function handleClose(): void {
  accountStore.resetLoginState();
  offlineUsername.value = '';
  if (countdownTimer) {
    clearInterval(countdownTimer);
    countdownTimer = null;
  }
  emit('update:modelValue', false);
}

// Watch for login success
watch(
  () => accountStore.loginState,
  (state) => {
    if (state === 'success') {
      toast.success('登录成功！');
      emit('loggedIn');
      handleClose();
    } else if (state === 'error') {
      toast.error(accountStore.error ?? '登录失败');
    }
  }
);

onUnmounted(() => {
  if (countdownTimer) {
    clearInterval(countdownTimer);
    countdownTimer = null;
  }
});
</script>

<template>
  <PixelDialog
    :model-value="modelValue"
    title="账号登录"
    width="440px"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <!-- Tab headers -->
    <div class="login-tabs">
      <button
        class="login-tabs__btn"
        :class="{ 'login-tabs__btn--active': activeTab === 'microsoft' }"
        @click="activeTab = 'microsoft'"
      >
        微软登录
      </button>
      <button
        class="login-tabs__btn"
        :class="{ 'login-tabs__btn--active': activeTab === 'offline' }"
        @click="activeTab = 'offline'"
      >
        离线登录
      </button>
    </div>

    <!-- Microsoft login -->
    <div v-if="activeTab === 'microsoft'" class="login-content">
      <div v-if="!deviceCode" class="login-ms__start">
        <p class="login-ms__desc">通过微软账号登录以获取正版皮肤和联机功能。</p>
        <PixelButton
          variant="primary"
          :loading="msLoggingIn"
          @click="startMicrosoftLogin"
        >
          微软账号登录
        </PixelButton>
      </div>

      <div v-else class="login-ms__code">
        <p class="login-ms__instruction">请访问以下链接并输入设备码：</p>
        <a
          :href="deviceCode.verification_uri"
          target="_blank"
          class="login-ms__link"
        >
          {{ deviceCode.verification_uri }}
        </a>
        <div class="login-ms__device-code">
          <span class="login-ms__code-text">{{ deviceCode.user_code }}</span>
          <PixelButton size="sm" @click="copyDeviceCode">复制</PixelButton>
        </div>
        <p v-if="countdown > 0" class="login-ms__countdown">
          等待验证中... ({{ countdown }}s)
        </p>
        <p v-if="accountStore.loginState === 'pending'" class="login-ms__waiting">
          等待验证...
        </p>
      </div>

      <p v-if="accountStore.error" class="login-error">{{ accountStore.error }}</p>
    </div>

    <!-- Offline login -->
    <div v-if="activeTab === 'offline'" class="login-content">
      <PixelInput
        v-model="offlineUsername"
        label="用户名"
        placeholder="输入离线用户名..."
        :error="offlineUsernameError"
      />
      <PixelButton
        variant="primary"
        :loading="offlineLoggingIn"
        :disabled="!!offlineUsernameError || !offlineUsername.trim()"
        style="margin-top: 16px; width: 100%;"
        @click="handleOfflineLogin"
      >
        离线登录
      </PixelButton>
    </div>

    <template #footer>
      <PixelButton variant="ghost" @click="handleClose">取消</PixelButton>
    </template>
  </PixelDialog>
</template>

<style scoped>
.login-tabs {
  display: flex;
  gap: 2px;
  margin-bottom: 16px;
  border-bottom: 2px solid var(--color-border);
}

.login-tabs__btn {
  flex: 1;
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

.login-tabs__btn:hover {
  color: var(--color-text);
}

.login-tabs__btn--active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.login-content {
  min-height: 120px;
}

.login-ms__start {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 16px 0;
}

.login-ms__desc {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  text-align: center;
  line-height: 1.6;
}

.login-ms__code {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 16px 0;
}

.login-ms__instruction {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  text-align: center;
}

.login-ms__link {
  font-family: var(--font-body);
  font-size: var(--font-size-base);
  color: var(--color-info);
  text-decoration: underline;
  word-break: break-all;
}

.login-ms__device-code {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: rgba(0, 0, 0, 0.3);
  border: 2px solid var(--color-primary);
  border-radius: var(--border-radius);
}

.login-ms__code-text {
  font-family: var(--font-display);
  font-size: var(--font-size-lg);
  color: var(--color-primary);
  letter-spacing: 4px;
}

.login-ms__countdown {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}

.login-ms__waiting {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-info);
  animation: glowPulse 2s ease-in-out infinite alternate;
}

.login-error {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-danger);
  margin-top: 8px;
  text-align: center;
}
</style>
