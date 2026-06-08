<script setup lang="ts">
/**
 * AccountBadge — Displays current account info with dropdown menu.
 */
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useAccountStore } from '@/stores/account';
import LoginDialog from '@/components/account/LoginDialog.vue';
import { useToast } from '@/components/common/useToast';

const accountStore = useAccountStore();
const toast = useToast();

const showMenu = ref(false);
const showLoginDialog = ref(false);
const menuRef = ref<HTMLElement | null>(null);

/** Current active account. */
const activeAccount = computed(() => accountStore.activeAccount);

/** All accounts. */
const accounts = computed(() => accountStore.accounts);

/** Minecraft head avatar URL. */
const avatarUrl = computed(() => {
  if (!activeAccount.value) return '';
  return `https://mc-heads.net/avatar/${activeAccount.value.uuid}/32`;
});

/** Toggle the account menu dropdown. */
function toggleMenu(): void {
  showMenu.value = !showMenu.value;
}

/** Set the active account. */
async function switchAccount(id: string): Promise<void> {
  const success = await accountStore.setActive(id);
  if (success) {
    toast.info('已切换账号');
  }
  showMenu.value = false;
}

/** Logout from an account. */
async function logoutAccount(id: string): Promise<void> {
  const success = await accountStore.logout(id);
  if (success) {
    toast.info('已登出');
  }
  showMenu.value = false;
}

/** Open the login dialog. */
function openLoginDialog(): void {
  showMenu.value = false;
  showLoginDialog.value = true;
}

/** Handle login success. */
function onLoggedIn(): void {
  showLoginDialog.value = false;
}

/** Close menu on outside click. */
function handleClickOutside(event: MouseEvent): void {
  if (menuRef.value && !menuRef.value.contains(event.target as Node)) {
    showMenu.value = false;
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
  if (accounts.value.length === 0) {
    accountStore.loadAccounts();
  }
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<template>
  <div ref="menuRef" class="account-badge" @click.stop>
    <!-- Active account display -->
    <div class="account-badge__trigger" @click="toggleMenu">
      <img
        v-if="activeAccount"
        :src="avatarUrl"
        alt="Avatar"
        class="account-badge__avatar"
      />
      <div v-else class="account-badge__avatar-placeholder">?</div>
      <span class="account-badge__name">
        {{ activeAccount?.username ?? '未登录' }}
      </span>
      <span class="account-badge__arrow" :class="{ 'account-badge__arrow--open': showMenu }">▼</span>
    </div>

    <!-- Dropdown menu -->
    <Transition name="dropdown">
      <div v-if="showMenu" class="account-badge__menu">
        <!-- Account list -->
        <div
          v-for="account in accounts"
          :key="account.id"
          class="account-badge__menu-item"
          :class="{ 'account-badge__menu-item--active': account.is_active }"
          @click="switchAccount(account.id)"
        >
          <img
            :src="`https://mc-heads.net/avatar/${account.uuid}/24`"
            alt=""
            class="account-badge__menu-avatar"
          />
          <span class="account-badge__menu-name">{{ account.username }}</span>
          <span class="account-badge__menu-type tag-pixel">
            {{ account.account_type === 'Microsoft' ? '正版' : '离线' }}
          </span>
        </div>

        <hr class="divider-pixel" />

        <!-- Actions -->
        <div class="account-badge__menu-item" @click="openLoginDialog">
          <span>➕ 添加账号</span>
        </div>
        <div
          v-if="activeAccount"
          class="account-badge__menu-item account-badge__menu-item--danger"
          @click="logoutAccount(activeAccount.id)"
        >
          <span>🚪 登出</span>
        </div>
      </div>
    </Transition>

    <!-- Login dialog -->
    <LoginDialog v-model="showLoginDialog" @logged-in="onLoggedIn" />
  </div>
</template>

<style scoped>
.account-badge {
  position: relative;
}

.account-badge__trigger {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 10px;
  border-radius: var(--border-radius);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.account-badge__trigger:hover {
  background: var(--color-surface-hover);
}

.account-badge__avatar {
  width: 24px;
  height: 24px;
  border-radius: 2px;
  image-rendering: pixelated;
  border: 1px solid var(--color-border);
}

.account-badge__avatar-placeholder {
  width: 24px;
  height: 24px;
  border-radius: 2px;
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  color: var(--color-text-muted);
}

.account-badge__name {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-secondary);
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 120px;
}

.account-badge__arrow {
  font-size: 8px;
  color: var(--color-text-muted);
  transition: transform var(--transition-fast);
}

.account-badge__arrow--open {
  transform: rotate(180deg);
}

.account-badge__menu {
  position: absolute;
  bottom: 100%;
  left: 0;
  right: 0;
  margin-bottom: 4px;
  background: #1A1A2E;
  border: var(--border-width) solid var(--color-border);
  border-radius: var(--border-radius);
  box-shadow: var(--shadow-pixel);
  overflow: hidden;
  z-index: 100;
}

.account-badge__menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text);
  cursor: pointer;
  transition: all var(--transition-fast);
}

.account-badge__menu-item:hover {
  background: var(--color-surface-hover);
}

.account-badge__menu-item--active {
  background: rgba(126, 200, 80, 0.08);
  color: var(--color-primary);
}

.account-badge__menu-item--danger:hover {
  color: var(--color-danger);
}

.account-badge__menu-avatar {
  width: 20px;
  height: 20px;
  border-radius: 2px;
  image-rendering: pixelated;
}

.account-badge__menu-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* Dropdown transition */
.dropdown-enter-active {
  transition: all 0.15s ease-out;
}

.dropdown-leave-active {
  transition: all 0.1s ease-in;
}

.dropdown-enter-from {
  opacity: 0;
  transform: translateY(4px);
}

.dropdown-leave-to {
  opacity: 0;
  transform: translateY(4px);
}
</style>
