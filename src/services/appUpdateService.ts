import { reactive } from 'vue';
import type { DownloadEvent, Update } from '@tauri-apps/plugin-updater';
import { createServerRequestError } from '@/utils/api';

export type AppUpdatePhase = 'idle' | 'checking' | 'available' | 'downloading' | 'installing' | 'installed' | 'error';

export const appUpdateState = reactive({
  phase: 'idle' as AppUpdatePhase,
  dialogOpen: false,
  version: '',
  date: '',
  hasChecked: false,
  error: '',
  downloaded: 0,
  contentLength: 0,
});

let pendingUpdate: Update | null = null;

const isTauriRuntime = () => {
  if (typeof window === 'undefined') {
    return false;
  }
  const tauriWindow = window as typeof window & { __TAURI_INTERNALS__?: unknown };
  return Boolean(tauriWindow.__TAURI_INTERNALS__);
};

const resetProgress = () => {
  appUpdateState.downloaded = 0;
  appUpdateState.contentLength = 0;
};

const markUnsupportedRuntime = () => {
  pendingUpdate = null;
  appUpdateState.phase = 'error';
  appUpdateState.version = '';
  appUpdateState.date = '';
  appUpdateState.hasChecked = true;
  appUpdateState.error = '当前环境不支持检查更新，请在 Tauri 桌面端使用。';
  resetProgress();
};

const applyUpdateMetadata = (update: Update) => {
  pendingUpdate = update;
  appUpdateState.phase = 'available';
  appUpdateState.version = update.version;
  appUpdateState.date = update.date ?? '';
  appUpdateState.hasChecked = true;
  appUpdateState.error = '';
  resetProgress();
};

const handleDownloadEvent = (event: DownloadEvent) => {
  switch (event.event) {
    case 'Started':
      appUpdateState.contentLength = event.data.contentLength ?? 0;
      appUpdateState.downloaded = 0;
      break;
    case 'Progress':
      appUpdateState.downloaded += event.data.chunkLength;
      break;
    case 'Finished':
      appUpdateState.downloaded = appUpdateState.contentLength || appUpdateState.downloaded;
      appUpdateState.phase = 'installing';
      break;
  }
};

const performCheckForAppUpdate = async (reportUnsupportedRuntime: boolean) => {
  if (appUpdateState.phase === 'checking') {
    return null;
  }

  if (!isTauriRuntime()) {
    if (reportUnsupportedRuntime) {
      markUnsupportedRuntime();
    }
    return null;
  }

  appUpdateState.phase = 'checking';
  appUpdateState.error = '';

  try {
    const { check } = await import('@tauri-apps/plugin-updater');
    const update = await check({ timeout: 8000 });
    if (!update) {
      pendingUpdate = null;
      appUpdateState.phase = 'idle';
      appUpdateState.version = '';
      appUpdateState.date = '';
      appUpdateState.hasChecked = true;
      return null;
    }

    applyUpdateMetadata(update);
    return update;
  } catch (error) {
    pendingUpdate = null;
    appUpdateState.phase = 'error';
    appUpdateState.version = '';
    appUpdateState.date = '';
    appUpdateState.hasChecked = true;
    appUpdateState.error = createServerRequestError('app-update-check', error, '检查更新失败，请检查网络后重试。').message;
    return null;
  }
};

export const checkForAppUpdate = () => performCheckForAppUpdate(true);

export const checkForAppUpdateSilently = () => performCheckForAppUpdate(false);

export const openAppUpdateDialog = () => {
  if (appUpdateState.phase === 'available' || appUpdateState.phase === 'error') {
    appUpdateState.dialogOpen = true;
  }
};

export const closeAppUpdateDialog = () => {
  if (appUpdateState.phase === 'downloading' || appUpdateState.phase === 'installing') {
    return;
  }
  appUpdateState.dialogOpen = false;
};

export const installPendingAppUpdate = async () => {
  if (!pendingUpdate || appUpdateState.phase !== 'available') {
    return;
  }

  appUpdateState.phase = 'downloading';
  appUpdateState.error = '';
  resetProgress();

  try {
    await pendingUpdate.downloadAndInstall(handleDownloadEvent);
    appUpdateState.phase = 'installed';
    const { relaunch } = await import('@tauri-apps/plugin-process');
    await relaunch();
  } catch (error) {
    appUpdateState.phase = 'error';
    appUpdateState.error = createServerRequestError('app-update-install', error, '下载更新失败，请检查网络后重试。').message;
  }
};
