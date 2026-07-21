<template>
  <template v-if="startupState === 'accepted'">
    <div v-if="useMainWindowChrome" class="app-shell flex h-screen w-full flex-col overflow-hidden">
      <EditorWindowTitlebar class="main-app-titlebar" :icon="true" title="Auto Daily" />
      <MainLayout class="min-h-0 flex-1" />
    </div>
    <router-view v-else />
    <AuthModal />
    <AppConfirmHost />
    <AppUpdateDialogHost />
  </template>
  <div v-else class="app-shell h-screen w-full" />
  <DisclaimerDialogHost
    :open="startupState === 'required'"
    :busy="acceptingDisclaimer"
    :error="disclaimerError"
    @accept="handleAcceptDisclaimer"
    @decline="handleDeclineDisclaimer"
  />
</template>

<script setup lang="ts">
import { onMounted, computed, ref } from 'vue';
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
import DisclaimerDialogHost from '@/components/shared/DisclaimerDialogHost.vue';
import { checkForAppUpdateSilently } from '@/services/appUpdateService';
import { acceptDisclaimer, declineDisclaimer, hasAcceptedDisclaimer } from '@/services/disclaimerService';

const { initTheme } = useThemeManager();
const route = useRoute();
const userStore = useUserStore();
const settingsStore = useSettingsStore();
const deviceStore = useDeviceStore();
const runtimeStore = useRuntimeStore();
const scriptTransferStore = useScriptTransferStore();
const startupState = ref<'checking' | 'required' | 'accepted'>('checking');
const acceptingDisclaimer = ref(false);
const disclaimerError = ref('');
let initializationPromise: Promise<void> | null = null;

const useMainWindowChrome = computed(() => route.path !== '/editor' && route.path !== '/vision-lab');

const initializeApplication = () => {
  if (initializationPromise) return initializationPromise;
  initializationPromise = (async () => {
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
  })();
  return initializationPromise;
};

async function handleAcceptDisclaimer() {
  acceptingDisclaimer.value = true;
  disclaimerError.value = '';
  try {
    await acceptDisclaimer();
  } catch {
    disclaimerError.value = '保存同意状态失败，请重试。';
    return;
  } finally {
    acceptingDisclaimer.value = false;
  }

  startupState.value = 'accepted';
  await initializeApplication();
}

async function handleDeclineDisclaimer() {
  try {
    await declineDisclaimer();
  } catch {
    disclaimerError.value = '退出程序失败，请手动关闭窗口。';
  }
}

onMounted(async () => {
  try {
    if (await hasAcceptedDisclaimer()) {
      startupState.value = 'accepted';
      await initializeApplication();
      return;
    }
  } catch {
    disclaimerError.value = '无法读取免责声明同意状态，请确认后继续。';
  }
  startupState.value = 'required';
});
</script>

<style scoped>
:deep(.main-app-titlebar.editor-window-titlebar) {
  grid-template-columns: auto minmax(0, 1fr) auto;
  gap: 0.5rem;
  min-height: 40px;
  max-height: 40px;
  border-radius: 0;
  padding: 0 0 0 0.75rem !important;
}

</style>
