<template>
  <component :is="layout">
    <router-view />
  </component>
  <AuthModal />
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue';
import { useRoute } from 'vue-router';
import MainLayout from './layouts/MainLayout.vue';
import { useThemeManager } from './composables/useThemeManager';
import { appThemeKey } from './store/store';
import { useUserStore } from './store/user';
import { useSettingsStore } from './store/settings';
import { useDeviceStore } from './store/device';
import { useLogsStore } from './store/logs';
import { useRuntimeStore } from './store/runtime';
import AuthModal from "@/components/AuthModal.vue";

const { initTheme } = useThemeManager();
const route = useRoute();
const userStore = useUserStore();
const settingsStore = useSettingsStore();
const deviceStore = useDeviceStore();
const logsStore = useLogsStore();
const runtimeStore = useRuntimeStore();

const layout = computed(() => {
  return route.path === '/editor' ? 'div' : MainLayout;
});

onMounted(async () => {
  await settingsStore.loadPreferences();
  await initTheme(appThemeKey);
  await Promise.all([userStore.hydrateAuthSession(), deviceStore.refreshAll()]);
  void userStore.checkProfile();
  await Promise.all([
    deviceStore.initIpcListeners(),
    logsStore.initListener(),
    runtimeStore.initIpcListeners(),
  ]);
});
</script>
