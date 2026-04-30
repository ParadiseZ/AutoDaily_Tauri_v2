<template>
  <AppDialog
    :open="appUpdateState.dialogOpen"
    title="发现新版本"
    :description="description"
    width-class="max-w-2xl"
    @close="closeAppUpdateDialog"
  >
    <div class="space-y-5">
      <div class="rounded-md border border-(--app-border) bg-(--app-bg-muted) p-4">
        <div class="mb-3 flex flex-wrap items-center justify-between gap-3">
          <div>
            <p class="text-sm font-semibold text-(--app-text-strong)">AutoDaily {{ appUpdateState.version }}</p>
            <p v-if="appUpdateState.date" class="mt-1 text-xs text-(--app-text-soft)">
              {{ appUpdateState.date }}
            </p>
          </div>
          <span class="rounded-full bg-red-500/12 px-3 py-1 text-xs font-semibold text-red-600">New</span>
        </div>
        <pre class="max-h-72 overflow-auto whitespace-pre-wrap break-words text-sm leading-6 text-(--app-text-soft)">{{ releaseNotes }}</pre>
      </div>

      <div v-if="isDownloading" class="space-y-2">
        <div class="flex items-center justify-between text-xs text-(--app-text-soft)">
          <span>{{ progressLabel }}</span>
          <span>{{ progressPercent }}%</span>
        </div>
        <div class="h-2 overflow-hidden rounded-full bg-(--app-border)">
          <div class="h-full rounded-full bg-(--app-accent)" :style="{ width: `${progressPercent}%` }" />
        </div>
      </div>

      <p v-if="appUpdateState.phase === 'installing'" class="text-sm text-(--app-text-soft)">
        更新包已下载，正在安装。Windows 安装阶段可能会自动退出当前窗口。
      </p>
      <p v-if="appUpdateState.error" class="rounded-md bg-red-500/10 px-3 py-2 text-sm text-red-700">
        {{ appUpdateState.error }}
      </p>

      <div class="flex flex-wrap justify-end gap-2">
        <button
          class="app-button app-button-ghost"
          type="button"
          :disabled="isBusy"
          @click="closeAppUpdateDialog"
        >
          暂不更新
        </button>
        <button
          class="app-button app-button-primary"
          type="button"
          :disabled="appUpdateState.phase !== 'available'"
          @click="installPendingAppUpdate"
        >
          立即更新
        </button>
      </div>
    </div>
  </AppDialog>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import AppDialog from '@/components/shared/AppDialog.vue';
import {
  appUpdateState,
  closeAppUpdateDialog,
  installPendingAppUpdate,
} from '@/services/appUpdateService';

const description = computed(() => `可更新到 ${appUpdateState.version || '新版本'}`);
const releaseNotes = computed(() => appUpdateState.body.trim() || '本次更新未提供更新日志。');
const isDownloading = computed(() => appUpdateState.phase === 'downloading' || appUpdateState.phase === 'installing');
const isBusy = computed(() => appUpdateState.phase === 'downloading' || appUpdateState.phase === 'installing');
const progressPercent = computed(() => {
  if (!appUpdateState.contentLength) {
    return appUpdateState.downloaded > 0 ? 100 : 0;
  }
  return Math.min(100, Math.round((appUpdateState.downloaded / appUpdateState.contentLength) * 100));
});
const progressLabel = computed(() => {
  if (!appUpdateState.contentLength) {
    return '正在下载更新包';
  }
  return `${formatBytes(appUpdateState.downloaded)} / ${formatBytes(appUpdateState.contentLength)}`;
});

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / 1024 / 1024).toFixed(1)} MB`;
}
</script>
