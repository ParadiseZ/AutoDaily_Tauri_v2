<template>
  <div class="app-shell flex h-screen w-full overflow-hidden">
    <aside class="app-sidebar hidden w-[280px] shrink-0 flex-col justify-between px-4 py-5 lg:flex">
      <div class="space-y-6">
        <div class="space-y-3 px-2" data-tauri-drag-region>
          <div class="flex items-center gap-3">
            <AppIcon name="logo" type="custom" :size="40" class="text-[var(--app-accent)] drop-shadow-md" />
            <div>
              <p class="text-sm font-semibold tracking-[0.18em] text-[var(--app-text-faint)]">AUTODAILY</p>
              <p class="text-lg font-semibold text-[var(--app-text-strong)]">控制台</p>
            </div>
          </div>
          <p class="text-sm leading-6 text-[var(--app-text-soft)]">
            多设备自动化调度、脚本资产管理与系统配置统一收敛在同一工作台里。
          </p>
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
          <RouterLink
            v-for="route in primaryRoutes"
            :key="route.path"
            :to="route.path"
            class="app-sidebar-link group my-1 py-3"
            :class="{ 'app-sidebar-link-active': isActive(route.path) }"
          >
            <AppIcon v-if="route.icon" :name="route.icon" :size="18" class="text-[var(--app-text-faint)] group-hover:text-[var(--app-accent)] group-[.app-sidebar-link-active]:text-[var(--app-accent)] transition-colors" />
            <span class="font-medium tracking-wide">{{ route.label }}</span>
          </RouterLink>
        </nav>
      </div>

      <div class="space-y-3 border-t border-[var(--app-border)] pt-4">
        <nav class="space-y-2">
          <RouterLink
            v-for="route in secondaryRoutes"
            :key="route.path"
            :to="route.path"
            class="app-sidebar-link group my-1 py-3"
            :class="{ 'app-sidebar-link-active': isActive(route.path) }"
          >
            <AppIcon v-if="route.icon" :name="route.icon" :size="18" class="text-[var(--app-text-faint)] group-hover:text-[var(--app-accent)] group-[.app-sidebar-link-active]:text-[var(--app-accent)] transition-colors" />
            <span class="font-medium tracking-wide">{{ route.label }}</span>
          </RouterLink>
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
      <div class="h-[calc(100vh-68px)] overflow-y-auto custom-scrollbar px-4 pb-8 pt-6 lg:px-8">
        <router-view v-slot="{ Component }">
          <transition name="shell-fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import AppIcon from '@/components/shared/AppIcon.vue';
import { routesMenu } from '@/router';
import { useUserStore } from '@/store/user';
import { useDeviceStore } from '@/store/device';

const route = useRoute();
const userStore = useUserStore();
const deviceStore = useDeviceStore();

const primaryRoutes = computed(() => routesMenu.filter((item) => !['/settings', '/about'].includes(item.path)));
const secondaryRoutes = computed(() => routesMenu.filter((item) => ['/settings', '/about'].includes(item.path)));

const currentRouteMeta = computed(() => {
  return routesMenu.find((item) => route.path === item.path || route.path.startsWith(`${item.path}/`));
});

const routeSummary = computed(() => {
  if (route.path === '/tasks') {
    return '多设备任务中心';
  }
  if (route.path === '/devices') {
    return '设备连接与执行配置';
  }
  if (route.path === '/scripts') {
    return '本地脚本与任务关系';
  }
  if (route.path === '/market') {
    return '云端脚本浏览与下载';
  }
  if (route.path === '/logs') {
    return '长时间运行排查面板';
  }
  if (route.path === '/settings') {
    return '系统行为与账户管理';
  }
  return 'AutoDaily';
});

const userInitial = computed(() => userStore.userProfile?.username?.slice(0, 1).toUpperCase() || 'A');
const userName = computed(() => userStore.userProfile?.username || '未登录');
const userState = computed(() => {
  if (!userStore.isLoggedIn) {
    return '点击登录后同步云端能力';
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
