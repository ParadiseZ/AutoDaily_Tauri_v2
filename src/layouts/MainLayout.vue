<template>
  <div class="app-shell flex h-screen w-full overflow-hidden">
    <aside class="app-sidebar hidden w-[280px] shrink-0 flex-col justify-between px-4 py-5 lg:flex">
      <div class="space-y-6">
        <div class="space-y-3 px-2" data-tauri-drag-region>
          <div class="flex items-center gap-3">
            <AppIcon name="logo" type="custom" :size="40" class="text-[var(--app-accent)] drop-shadow-md" />
            <div>
              <p class="text-lg font-semibold text-[var(--app-text-strong)]">AutoDaily</p>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-3 gap-3">
          <div class="app-stat">
            <p class="app-stat-label">在线</p>
            <p class="app-stat-value">{{ deviceStore.deviceSummary.online }}</p>
          </div>
          <div class="app-stat">
            <p class="app-stat-label">运行</p>
            <p class="app-stat-value">{{ deviceStore.deviceSummary.running }}</p>
          </div>
          <div class="app-stat">
            <p class="app-stat-label">启用</p>
            <p class="app-stat-value">{{ deviceStore.deviceSummary.enabled }}/{{ deviceStore.deviceSummary.total }}</p>
          </div>
        </div>

        <nav class="space-y-2">
          <template v-for="route in primaryRoutes" :key="route.path">
            <button
              v-if="route.path === '/vision-lab'"
              type="button"
              class="app-sidebar-link group my-1 w-full py-3 text-left"
              @click="handleOpenVisionLab"
            >
              <AppIcon v-if="route.icon" :name="route.icon" :size="18" class="text-[var(--app-text-faint)] group-hover:text-[var(--app-accent)] transition-colors" />
              <span class="font-medium tracking-wide">{{ route.label }}</span>
            </button>
            <RouterLink
              v-else
              :to="route.path"
              class="app-sidebar-link group my-1 py-3"
              :class="{ 'app-sidebar-link-active': isActive(route.path) }"
            >
              <AppIcon v-if="route.icon" :name="route.icon" :size="18" class="text-[var(--app-text-faint)] group-hover:text-[var(--app-accent)] group-[.app-sidebar-link-active]:text-[var(--app-accent)] transition-colors" />
              <span class="font-medium tracking-wide">{{ route.label }}</span>
            </RouterLink>
          </template>
        </nav>
      </div>

      <div class="space-y-3 border-t border-[var(--app-border)] pt-4">
        <nav class="space-y-2">
          <template v-for="route in secondaryRoutes" :key="route.path">
            <button
              v-if="route.path === '/vision-lab'"
              type="button"
              class="app-sidebar-link group my-1 w-full py-3 text-left"
              @click="handleOpenVisionLab"
            >
              <AppIcon v-if="route.icon" :name="route.icon" :size="18" class="text-[var(--app-text-faint)] group-hover:text-[var(--app-accent)] transition-colors" />
              <span class="font-medium tracking-wide">{{ route.label }}</span>
            </button>
            <RouterLink
              v-else
              :to="route.path"
              class="app-sidebar-link group my-1 py-3"
              :class="{ 'app-sidebar-link-active': isActive(route.path) }"
            >
              <AppIcon v-if="route.icon" :name="route.icon" :size="18" class="text-[var(--app-text-faint)] group-hover:text-[var(--app-accent)] group-[.app-sidebar-link-active]:text-[var(--app-accent)] transition-colors" />
              <span class="font-medium tracking-wide">{{ route.label }}</span>
            </RouterLink>
          </template>
        </nav>

        <button class="app-panel flex w-full items-center gap-3 p-3 text-left" type="button" @click="handleUserClick">
          <div class="flex h-11 w-11 items-center justify-center rounded-2xl bg-[var(--app-accent-soft)] text-sm font-semibold text-[var(--app-accent)]">
            {{ userInitial }}
          </div>
          <div class="min-w-0">
            <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ userName }}</p>
            <p class="truncate text-xs text-[var(--app-text-soft)]">{{ userState }}</p>
          </div>
        </button>
      </div>
    </aside>

    <main class="min-w-0 flex-1">
      <div class="h-full overflow-y-auto custom-scrollbar px-4 pb-4 pt-6 lg:px-8">
        <router-view v-slot="{ Component }">
          <transition name="shell-fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </div>
    </main>

    <div class="fixed bottom-4 right-4 z-50 flex items-center gap-2">
      <button
        class="inline-flex items-center gap-2 rounded-full border border-[var(--app-border)] bg-[var(--app-panel)] px-4 py-2 text-sm font-semibold text-[var(--app-text-strong)] shadow-lg shadow-slate-900/10 transition hover:border-[var(--app-accent)] hover:text-[var(--app-accent)]"
        type="button"
        title="刷新当前页面"
        @click="reloadCurrentPage"
      >
        <AppIcon name="refresh-cw" :size="16" />
        刷新页面
      </button>
      <button
        class="inline-flex items-center gap-2 rounded-full border border-[var(--app-border)] bg-[var(--app-panel)] px-4 py-2 text-sm font-semibold text-[var(--app-text-strong)] shadow-lg shadow-slate-900/10 transition hover:border-[var(--app-accent)] hover:text-[var(--app-accent)]"
        type="button"
        title="打开开发者工具"
        @click="openCurrentDevtools"
      >
        <AppIcon name="bug" :size="16" />
        开发者工具
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import AppIcon from '@/components/shared/AppIcon.vue';
import { routesMenu } from '@/router';
import { useUserStore } from '@/store/user';
import { useDeviceStore } from '@/store/device';
import { openVisionLabWindow } from '@/utils/visionLabWindow';
import { openCurrentDevtools, reloadCurrentPage } from '@/services/devtoolsService';

const route = useRoute();
const userStore = useUserStore();
const deviceStore = useDeviceStore();

const secondaryRoutePaths = ['/vision-lab', '/settings', '/about'];
const primaryRoutes = computed(() => routesMenu.filter((item) => !secondaryRoutePaths.includes(item.path)).filter((item) => item.path !== '/editor'));
const secondaryRoutes = computed(() => routesMenu.filter((item) => secondaryRoutePaths.includes(item.path)));


const userInitial = computed(() => userStore.userProfile?.username?.slice(0, 1).toUpperCase() || 'A');
const userName = computed(() => userStore.userProfile?.username || '未登录');
const userState = computed(() => {
  if (!userStore.isLoggedIn) {
    return '登录后同步云端能力';
  }
  return userStore.isDeveloper ? '开发者权限已激活' : '本地模式';
});

const isActive = (path: string) => {
  if (path === '/') {
    return route.path === '/';
  }
  return route.path === path || route.path.startsWith(`${path}/`);
};

const handleUserClick = () => {
  if (!userStore.isLoggedIn) {
    userStore.openAuthModal();
  }
};

const handleOpenVisionLab = () => {
  void openVisionLabWindow();
};
</script>

<style scoped>
.shell-fade-enter-active,
.shell-fade-leave-active {
  transition: opacity 0.14s ease;
}

.shell-fade-enter-from,
.shell-fade-leave-to {
  opacity: 0;
}
</style>
