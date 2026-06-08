import { defineStore } from "pinia";
import { ref, type Ref } from "vue";
import { tauriCommand, type CommandError } from "@/composables/useTauriCommand";
import type {
  Account,
  DeviceCodeResponse,
  LoginState,
} from "@/types/account";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const useAccountStore = defineStore("account", () => {
  // ── State ───────────────────────────────────────────────
  const accounts: Ref<Account[]> = ref([]);
  const activeAccount: Ref<Account | null> = ref(null);
  const loginState: Ref<LoginState> = ref('idle');
  const deviceCode: Ref<DeviceCodeResponse | null> = ref(null);
  const error: Ref<string | null> = ref(null);

  /** Unlisteners for Tauri events. */
  const unlisteners: UnlistenFn[] = [];

  // ── Actions ────────────────────────────────────────────

  /** Load all accounts from the backend. */
  async function loadAccounts(): Promise<void> {
    try {
      // 命令名与 Rust #[tauri::command] fn 名一致：get_accounts
      accounts.value = await tauriCommand<Account[]>("get_accounts");
      activeAccount.value = accounts.value.find((a) => a.is_active) ?? null;
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
    }
  }

  /** Start Microsoft login via device code flow. */
  async function loginMicrosoft(): Promise<void> {
    loginState.value = 'pending';
    error.value = null;
    deviceCode.value = null;
    try {
      // login_microsoft 是一个完整的异步流程（设备码轮询）
      // Rust 后端通过事件推送设备码，命令完成后返回账号
      // 先注册事件监听，再调用命令
      const unlisten = await listen<DeviceCodeResponse>("account:device_code", (event) => {
        deviceCode.value = event.payload;
        loginState.value = 'awaiting_user';
      });
      unlisteners.push(unlisten);

      // login_microsoft 会阻塞直到用户完成授权（最长5分钟）
      const account = await tauriCommand<Account>("login_microsoft");
      accounts.value.push(account);
      activeAccount.value = account;
      loginState.value = 'success';
      deviceCode.value = null;
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
      loginState.value = 'error';
      deviceCode.value = null;
    }
  }

  /** Login with an offline (cracked) account. */
  async function loginOffline(username: string): Promise<Account | null> {
    loginState.value = 'pending';
    error.value = null;
    try {
      const account = await tauriCommand<Account>("login_offline", { username });
      accounts.value.push(account);
      activeAccount.value = account;
      loginState.value = 'success';
      return account;
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
      loginState.value = 'error';
      return null;
    }
  }

  /** Logout and remove an account. */
  async function logout(id: string): Promise<boolean> {
    try {
      await tauriCommand<void>("logout", { id });
      accounts.value = accounts.value.filter((a) => a.id !== id);
      if (activeAccount.value?.id === id) {
        activeAccount.value = accounts.value.find((a) => a.is_active) ?? null;
      }
      return true;
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
      return false;
    }
  }

  /** Set the active account. */
  async function setActive(id: string): Promise<boolean> {
    try {
      await tauriCommand<void>("set_active_account", { id });
      const account = accounts.value.find((a) => a.id === id);
      if (account) {
        // Update is_active flags locally
        for (const a of accounts.value) {
          a.is_active = a.id === id;
        }
        activeAccount.value = account;
      }
      return true;
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
      return false;
    }
  }

  /** Refresh an account's access token. */
  async function refreshAccount(id: string): Promise<boolean> {
    try {
      const updated = await tauriCommand<Account>("refresh_account", { id });
      const idx = accounts.value.findIndex((a) => a.id === updated.id);
      if (idx >= 0) {
        accounts.value[idx] = updated;
      }
      if (activeAccount.value?.id === updated.id) {
        activeAccount.value = updated;
      }
      return true;
    } catch (e) {
      const cmdErr = e as CommandError;
      error.value = cmdErr.message;
      return false;
    }
  }

  /** Reset login state. */
  function resetLoginState(): void {
    loginState.value = 'idle';
    deviceCode.value = null;
    error.value = null;
  }

  /** Cleanup event listeners. */
  function cleanup(): void {
    for (const unlisten of unlisteners) {
      unlisten();
    }
    unlisteners.length = 0;
  }

  return {
    accounts,
    activeAccount,
    loginState,
    deviceCode,
    error,
    loadAccounts,
    loginMicrosoft,
    loginOffline,
    logout,
    setActive,
    refreshAccount,
    resetLoginState,
    cleanup,
  };
});
