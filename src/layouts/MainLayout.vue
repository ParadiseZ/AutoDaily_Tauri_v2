<template>
  <div class="app-shell flex h-screen w-full flex-col overflow-hidden">
    <EditorWindowTitlebar class="main-layout-titlebar" title="AutoDaily">
      <template #prefix>
        <div class="flex items-center gap-3">
          <AppIcon name="logo" type="custom" :size="24" class="text-(--app-accent) drop-shadow-md" />
          <div v-if="appUpdateState.phase === 'available'" class="flex items-baseline gap-2">
            <button
              class="text-sm font-bold text-red-600 transition hover:text-red-700"
              type="button"
              @click="openAppUpdateDialog"
            >
              New
            </button>
          </div>
        </div>
      </template>
    </EditorWindowTitlebar>

    <div class="flex min-h-0 flex-1 overflow-hidden">
      <aside class="app-sidebar hidden w-[280px] shrink-0 flex-col justify-between px-4 py-5 lg:flex">
        <div class="space-y-6">
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
                <AppIcon v-if="route.icon" :name="route.icon" :size="18" class="text-(--app-text-faint) group-hover:text-(--app-accent) transition-colors" />
                <span class="font-medium tracking-wide">{{ route.label }}</span>
              </button>
              <RouterLink
                v-else
                :to="route.path"
                class="app-sidebar-link group my-1 py-3"
                :class="{ 'app-sidebar-link-active': isActive(route.path) }"
              >
                <AppIcon v-if="route.icon" :name="route.icon" :size="18" class="text-(--app-text-faint) group-hover:text-(--app-accent) group-[.app-sidebar-link-active]:text-(--app-accent) transition-colors" />
                <span class="font-medium tracking-wide">{{ route.label }}</span>
              </RouterLink>
            </template>
          </nav>
        </div>

        <div class="space-y-3 border-t border-(--app-border) pt-4">
          <nav class="space-y-2">
            <template v-for="route in secondaryRoutes" :key="route.path">
              <button
                v-if="route.path === '/vision-lab'"
                type="button"
                class="app-sidebar-link group my-1 w-full py-3 text-left"
                @click="handleOpenVisionLab"
              >
                <AppIcon v-if="route.icon" :name="route.icon" :size="18" class="text-(--app-text-faint) group-hover:text-(--app-accent) transition-colors" />
                <span class="font-medium tracking-wide">{{ route.label }}</span>
              </button>
              <RouterLink
                v-else
                :to="route.path"
                class="app-sidebar-link group my-1 py-3"
                :class="{ 'app-sidebar-link-active': isActive(route.path) }"
              >
                <AppIcon v-if="route.icon" :name="route.icon" :size="18" class="text-(--app-text-faint) group-hover:text-(--app-accent) group-[.app-sidebar-link-active]:text-(--app-accent) transition-colors" />
                <span class="font-medium tracking-wide">{{ route.label }}</span>
              </RouterLink>
            </template>
          </nav>
          <div class="app-panel flex w-full items-center gap-3 p-3 text-left hover:cursor-pointer" type="button" @click="handleUserClick">
            <div class="flex h-11 w-11 items-center justify-center rounded-2xl bg-(--app-accent-soft) text-sm font-semibold text-(--app-accent)">
              {{ userInitial }}
            </div>
            <div class="min-w-0">
              <p class="truncate text-sm font-semibold text-(--app-text-strong)">{{ userName }}</p>
              <p class="truncate text-xs text-(--app-text-soft)">{{ userState }}</p>
            </div>
          </div>
        </div>
      </aside>

      <main class="min-w-0 flex-1 overflow-hidden">
        <div class="h-full min-h-0 overflow-hidden px-4 pb-4 pt-6 lg:px-8">
          <router-view v-slot="{ Component }">
            <transition name="shell-fade" mode="out-in">
              <component :is="Component" />
            </transition>
          </router-view>
        </div>
      </main>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import AppIcon from '@/components/shared/AppIcon.vue';
import EditorWindowTitlebar from '@/views/script-editor/EditorWindowTitlebar.vue';
import router, { routesMenu } from '@/router';
import { useUserStore } from '@/store/user';
import { useDeviceStore } from '@/store/device';
import { openVisionLabWindow } from '@/utils/visionLabWindow';
import { openCurrentDevtools, reloadCurrentPage } from '@/services/devtoolsService';
import { appUpdateState, openAppUpdateDialog } from '@/services/appUpdateService';

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
    return '游客';
  }
  if (!userStore.userProfile) {
    return userStore.profileLoading ? '正在同步账户信息' : '登录态已恢复';
  }
  return userStore.isDeveloper ? '开发者' : '';
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
  } else {
    router.push("/settings")
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

:deep(.main-layout-titlebar.editor-window-titlebar) {
  min-height: 48px;
  gap: 0.5rem;
  border-radius: 0;
  border-bottom: 1px solid color-mix(in srgb, var(--app-border) 92%, transparent);
  padding: 0 0 0 1rem !important;
}

:deep(.main-layout-titlebar .editor-window-titlebar__prefix),
:deep(.main-layout-titlebar .editor-window-titlebar__actions) {
  gap: 0.375rem;
}

:deep(.main-layout-titlebar .editor-window-titlebar__title) {
  font-size: 0.95rem;
}

:deep(.main-layout-titlebar .editor-window-titlebar__window-button) {
  width: 2.25rem;
  min-height: 2rem;
}

:deep(.main-layout-titlebar .editor-window-titlebar__window-controls) {
  margin-left: 1rem;
  margin-right: -0.75rem;
}
</style>
