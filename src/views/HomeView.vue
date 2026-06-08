<script setup lang="ts">
/**
 * HomeView — Main landing page with welcome banner, recent instances, and news.
 */
import { onMounted, ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useInstanceStore } from '@/stores/instance';
import { useAccountStore } from '@/stores/account';
import InstanceCard from '@/components/instance/InstanceCard.vue';
import PixelButton from '@/components/common/PixelButton.vue';
import GlassPanel from '@/components/common/GlassPanel.vue';
import type { Instance } from '@/types/instance';

const router = useRouter();
const instanceStore = useInstanceStore();
const accountStore = useAccountStore();

const greeting = ref<string>('欢迎来到 Aurora Launcher');
const loading = ref(true);

onMounted(async () => {
  try {
    const { tauriCommand } = await import('@/composables/useTauriCommand');
    greeting.value = await tauriCommand<string>('greet', { name: '冒险家' });
  } catch {
    greeting.value = '欢迎来到 Aurora Launcher';
  } finally {
    loading.value = false;
  }
  instanceStore.loadInstances();
  accountStore.loadAccounts();
});

/** Recent instances (most recently updated, up to 4). */
const recentInstances = computed(() => {
  const instances = [...instanceStore.instances];
  return instances
    .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
    .slice(0, 4);
});

/** Handle launch from instance card. */
function handleLaunch(instance: Instance): void {
  router.push(`/instances/${instance.id}`);
}

/** Handle edit from instance card. */
function handleEdit(instance: Instance): void {
  router.push(`/instances/${instance.id}`);
}

/** Handle delete from instance card. */
async function handleDelete(instance: Instance): Promise<void> {
  const confirmed = window.confirm(`确定要删除实例 "${instance.name}" 吗？此操作不可撤销。`);
  if (confirmed) {
    await instanceStore.deleteInstance(instance.id);
  }
}
</script>

<template>
  <div class="home-view animate-fade-in">
    <!-- Hero Section -->
    <section class="home-hero glass-panel-grid">
      <div class="home-hero__content">
        <h1 class="home-hero__title">AURORA LAUNCHER</h1>
        <p class="home-hero__subtitle">{{ loading ? '加载中...' : greeting }}</p>

        <div v-if="accountStore.activeAccount" class="home-hero__account">
          <img
            :src="`https://mc-heads.net/avatar/${accountStore.activeAccount.uuid}/32`"
            alt="Avatar"
            class="home-hero__avatar"
          />
          <span class="home-hero__username">{{ accountStore.activeAccount.username }}</span>
        </div>

        <router-link to="/instances" class="btn-pixel btn-pixel-primary home-hero__cta">
          🎮 开始游戏
        </router-link>
      </div>
    </section>

    <!-- Recent Instances -->
    <section class="home-section">
      <div class="home-section__header">
        <h2 class="section-title">🕐 最近游玩</h2>
        <PixelButton size="sm" @click="router.push('/instances')">查看全部</PixelButton>
      </div>

      <div v-if="instanceStore.loading" class="home-section__loading">
        加载中...
      </div>

      <div v-else-if="recentInstances.length > 0" class="home-section__grid">
        <InstanceCard
          v-for="instance in recentInstances"
          :key="instance.id"
          :instance="instance"
          @launch="handleLaunch"
          @edit="handleEdit"
          @delete="handleDelete"
        />
      </div>

      <GlassPanel v-else padding="32px" class="home-section__empty">
        <p class="home-section__empty-icon">🏗️</p>
        <p class="home-section__empty-text">还没有游玩记录</p>
        <PixelButton variant="primary" size="sm" @click="router.push('/instances')">
          创建实例
        </PixelButton>
      </GlassPanel>
    </section>

    <!-- News / Announcements placeholder -->
    <section class="home-section">
      <h2 class="section-title">📢 公告</h2>
      <GlassPanel padding="24px">
        <div class="home-news__item">
          <span class="home-news__icon">🆕</span>
          <div>
            <p class="home-news__title">Aurora Launcher v0.1.0</p>
            <p class="home-news__desc">首个公开版本发布，支持实例管理、版本下载和微软登录。</p>
          </div>
        </div>
      </GlassPanel>
    </section>
  </div>
</template>

<style scoped>
.home-view {
  display: flex;
  flex-direction: column;
  gap: var(--space-lg);
}

.home-hero {
  padding: 40px 32px;
  margin-bottom: 0;
}

.home-hero__content {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  gap: 16px;
}

.home-hero__title {
  font-family: var(--font-display);
  font-size: var(--font-size-3xl);
  color: var(--color-primary);
  text-shadow: 0 0 20px rgba(126, 200, 80, 0.4);
  letter-spacing: 4px;
}

.home-hero__subtitle {
  font-family: var(--font-body);
  font-size: var(--font-size-base);
  color: var(--color-text-secondary);
}

.home-hero__account {
  display: flex;
  align-items: center;
  gap: 8px;
}

.home-hero__avatar {
  width: 28px;
  height: 28px;
  border-radius: 2px;
  image-rendering: pixelated;
  border: 1px solid var(--color-border);
}

.home-hero__username {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
}

.home-hero__cta {
  font-size: var(--font-size-base);
  padding: 12px 32px;
  text-decoration: none;
  margin-top: 8px;
}

.home-section {
  display: flex;
  flex-direction: column;
  gap: var(--space-md);
}

.home-section__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.home-section__loading {
  padding: 32px;
  text-align: center;
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
}

.home-section__grid {
  display: grid;
  gap: 10px;
}

.home-section__empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  text-align: center;
}

.home-section__empty-icon {
  font-size: 36px;
}

.home-section__empty-text {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
}

.home-news__item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.home-news__icon {
  font-size: 20px;
  flex-shrink: 0;
}

.home-news__title {
  font-family: var(--font-body);
  font-size: var(--font-size-sm);
  color: var(--color-text);
  font-weight: bold;
  margin-bottom: 4px;
}

.home-news__desc {
  font-family: var(--font-body);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}
</style>
