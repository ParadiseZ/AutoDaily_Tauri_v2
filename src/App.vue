<template>
  <div v-if="useMainWindowChrome" class="app-shell flex h-screen w-full flex-col overflow-hidden">
    <EditorWindowTitlebar class="main-app-titlebar" :icon="true" title="Auto Daily" />
    <MainLayout class="min-h-0 flex-1" />
  </div>
  <router-view v-else />
  <AuthModal />
  <AppConfirmHost />
  <AppUpdateDialogHost />
</template>

<script setup lang="ts">
import { onMounted, computed } from 'vue';
import { useRoute } from 'vue-router';
import MainLayout from './layouts/MainLayout.vue';
import EditorWindowTitlebar from '@/views/script-editor/EditorWindowTitlebar.vue';
import { useThemeManager } from './composables/useThemeManager';
import { appThemeKey } from './store/store';
import { useUserStore } from './store/user';
import { useSettingsStore } from './store/settings';
import { useDeviceStore } from './store/device';
import { useRuntimeStore } from './store/runtime';
import { useScriptTransferStore } from './store/scriptTransfer';
import AuthModal from "@/components/AuthModal.vue";
import AppConfirmHost from "@/components/shared/AppConfirmHost.vue";
import AppUpdateDialogHost from "@/components/shared/AppUpdateDialogHost.vue";
import { checkForAppUpdateSilently } from '@/services/appUpdateService';

const { initTheme } = useThemeManager();
const route = useRoute();
const userStore = useUserStore();
const settingsStore = useSettingsStore();
const deviceStore = useDeviceStore();
const runtimeStore = useRuntimeStore();
const scriptTransferStore = useScriptTransferStore();

const useMainWindowChrome = computed(() => route.path !== '/editor' && route.path !== '/vision-lab');

onMounted(async () => {
  await settingsStore.loadPreferences();
  await initTheme(appThemeKey);
  await Promise.all([userStore.hydrateAuthSession(), deviceStore.refreshAll()]);
  void userStore.checkProfile();
  await Promise.all([
    deviceStore.initIpcListeners(),
    runtimeStore.initIpcListeners(),
    scriptTransferStore.initListener(),
  ]);
  void checkForAppUpdateSilently();
});
</script>

<style scoped>
:deep(.main-app-titlebar.editor-window-titlebar) {
  min-height: 48px;
  max-height: 48px;
  gap: 0.5rem;
  border-radius: 0;
  padding: 0 0 0 1rem !important;
}

:deep(.main-app-titlebar .editor-window-titlebar__window-button) {
  width: 2.25rem;
  min-height: 2rem;
}

:deep(.main-app-titlebar .editor-window-titlebar__window-controls) {
  margin-left: 1rem;
  margin-right: 0;
}
</style>
