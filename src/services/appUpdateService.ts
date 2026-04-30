import { reactive } from 'vue';
import type { DownloadEvent, Update } from '@tauri-apps/plugin-updater';

export type AppUpdatePhase = 'idle' | 'checking' | 'available' | 'downloading' | 'installing' | 'installed' | 'error';

export const appUpdateState = reactive({
  phase: 'idle' as AppUpdatePhase,
  dialogOpen: false,
  version: '',
  date: '',
  body: '',
  error: '',
  downloaded: 0,
  contentLength: 0,
});

let pendingUpdate: Update | null = null;

const isTauriRuntime = () =>
  typeof navigator !== 'undefined' && navigator.userAgent.includes('Tauri');

const resetProgress = () => {
  appUpdateState.downloaded = 0;
  appUpdateState.contentLength = 0;
};

const applyUpdateMetadata = (update: Update) => {
  pendingUpdate = update;
  appUpdateState.phase = 'available';
  appUpdateState.version = update.version;
  appUpdateState.date = update.date ?? '';
  appUpdateState.body = update.body ?? '';
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

export const checkForAppUpdateSilently = async () => {
  if (!isTauriRuntime() || appUpdateState.phase === 'checking') {
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
      return null;
    }

    applyUpdateMetadata(update);
    return update;
  } catch (error) {
    pendingUpdate = null;
    appUpdateState.phase = 'error';
    appUpdateState.error = error instanceof Error ? error.message : String(error);
    return null;
  }
};

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
    appUpdateState.error = error instanceof Error ? error.message : String(error);
  }
};
