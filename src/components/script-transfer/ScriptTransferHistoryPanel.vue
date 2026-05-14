<template>
  <div class="relative z-20">
    <div
      class="flex flex-wrap items-start justify-between gap-3 rounded-[22px] border border-(--app-border) bg-(--app-panel)/88 px-4 py-3 shadow-[0_10px_30px_rgba(15,23,42,0.08)] backdrop-blur"
      :class="open ? 'border-(--app-accent)/35' : ''"
    >
      <div class="space-y-1">
        <div class="flex items-center gap-2">
          <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('toggle')">
            <AppIcon :name="open ? 'chevron-up' : 'chevron-down'" :size="14" />
            {{ title }}
          </button>
          <span class="text-xs text-(--app-text-faint)">{{ records.length }} 条</span>
        </div>
        <p v-if="description" class="text-xs text-(--app-text-faint)">{{ description }}</p>
      </div>

      <button
        v-if="open && records.length"
        class="app-button app-button-ghost app-toolbar-button"
        type="button"
        @click="$emit('clear')"
      >
        <AppIcon name="eraser" :size="14" />
        清空记录
      </button>
    </div>

    <div
      v-if="open"
      class="absolute inset-x-0 top-full mt-3 z-30 max-h-[min(70vh,720px)] overflow-y-auto rounded-[26px] shadow-[0_24px_64px_rgba(15,23,42,0.22)]"
    >
      <SurfacePanel tone="muted" padding="sm" class="space-y-3 border border-(--app-border) bg-(--app-panel)/96 backdrop-blur">
        <div
          v-for="record in records"
          :key="record.id"
          class="rounded-[18px] border border-(--app-border) bg-(--app-panel)/55 px-4 py-3"
        >
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div class="min-w-0 flex-1 space-y-2">
              <div class="flex flex-wrap items-center gap-2">
                <StatusBadge
                  :label="formatScriptTransferStatusLabel(record.status)"
                  :tone="formatScriptTransferStatusTone(record.status)"
                />
                <span class="truncate text-sm font-medium text-(--app-text-strong)">
                  {{ record.latestMessage || fallbackTitle(record) }}
                </span>
              </div>

              <div class="grid gap-2 text-xs text-(--app-text-soft) md:grid-cols-2">
                <div class="flex items-center justify-between gap-3">
                  <span>模型进度</span>
                  <span class="text-(--app-text-strong)">{{ record.completedModelFileCount }} / {{ record.modelFileCount }}</span>
                </div>
                <div class="flex items-center justify-between gap-3">
                  <span>最近文件</span>
                  <span class="truncate text-(--app-text-strong)">{{ activeFileName(record) || '未记录' }}</span>
                </div>
                <div class="flex items-center justify-between gap-3">
                  <span>传输大小</span>
                  <span class="text-(--app-text-strong)">{{ progressLabel(record) }}</span>
                </div>
                <div class="flex items-center justify-between gap-3">
                  <span>时间</span>
                  <span class="text-(--app-text-strong)">{{ formatDateTime(record.updatedAt || record.startedAt) }}</span>
                </div>
              </div>

              <template v-if="percent(record) !== null">
                <div class="space-y-1 pt-1">
                  <div class="flex items-center justify-between text-[11px] text-(--app-text-faint)">
                    <span>{{ etaLabel(record) }}</span>
                    <span>{{ percent(record) }}%</span>
                  </div>
                  <div class="h-2 overflow-hidden rounded-full bg-(--app-border)">
                    <div class="h-full rounded-full bg-(--app-accent)" :style="{ width: `${percent(record)}%` }" />
                  </div>
                </div>
              </template>

              <p v-if="record.errorMessage" class="text-xs text-red-600">{{ record.errorMessage }}</p>
            </div>

            <button
              class="app-button app-button-ghost app-toolbar-button shrink-0"
              type="button"
              :disabled="record.status === 'running'"
              @click="$emit('delete-record', record.id)"
            >
              <AppIcon name="trash-2" :size="14" />
              删除
            </button>
          </div>
        </div>

        <EmptyState
          v-if="!records.length"
          :title="emptyTitle"
          :description="emptyDescription"
        />
      </SurfacePanel>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ScriptTransferProgressEvent, ScriptTransferRecord } from '@/types/app/domain';
import AppIcon from '@/components/shared/AppIcon.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import StatusBadge from '@/components/shared/StatusBadge.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import { formatDateTime, formatScriptTransferStatusLabel, formatScriptTransferStatusTone } from '@/utils/presenters';

const props = defineProps<{
  title: string;
  description?: string;
  emptyTitle: string;
  emptyDescription: string;
  open: boolean;
  records: ScriptTransferRecord[];
  getProgressEvent: (recordId: string) => ScriptTransferProgressEvent | null;
}>();

defineEmits<{
  toggle: [];
  clear: [];
  'delete-record': [recordId: string];
}>();

const fallbackTitle = (record: ScriptTransferRecord) => (record.direction === 'upload' ? '上传记录' : '下载记录');

const activeEvent = (record: ScriptTransferRecord) => props.getProgressEvent(record.id);
const activeFileName = (record: ScriptTransferRecord) => activeEvent(record)?.currentFileName || record.latestFileName;

const percent = (record: ScriptTransferRecord) => {
  const total = activeEvent(record)?.totalBytes ?? record.totalBytes;
  const transferred = activeEvent(record)?.bytesTransferred ?? record.bytesTransferred;
  if (!total || total <= 0) {
    return null;
  }
  return Math.max(0, Math.min(100, Math.round((transferred / total) * 100)));
};

const progressLabel = (record: ScriptTransferRecord) => {
  const total = activeEvent(record)?.totalBytes ?? record.totalBytes;
  const transferred = activeEvent(record)?.bytesTransferred ?? record.bytesTransferred;
  if (!total || total <= 0) {
    return '未统计模型大小';
  }
  return `${formatBytes(transferred)} / ${formatBytes(total)}`;
};

const etaLabel = (record: ScriptTransferRecord) => {
  const event = activeEvent(record);
  if (!event || record.status !== 'running') {
    return '传输进度';
  }

  const total = event.totalBytes;
  const transferred = event.bytesTransferred;
  const startedAt = Date.parse(event.startedAt);
  const updatedAt = Date.parse(event.updatedAt);
  if (!total || transferred <= 0 || !Number.isFinite(startedAt) || !Number.isFinite(updatedAt) || updatedAt <= startedAt) {
    return '正在估算剩余时间';
  }

  const elapsedSeconds = Math.max(1, Math.round((updatedAt - startedAt) / 1000));
  const bytesPerSecond = transferred / elapsedSeconds;
  if (!Number.isFinite(bytesPerSecond) || bytesPerSecond <= 0) {
    return '正在估算剩余时间';
  }

  const remainingSeconds = Math.max(0, Math.round((total - transferred) / bytesPerSecond));
  return remainingSeconds > 0 ? `预计剩余 ${formatDuration(remainingSeconds)}` : '即将完成';
};

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  return `${(value / 1024 / 1024).toFixed(1)} MB`;
}

function formatDuration(seconds: number) {
  if (seconds < 60) return `${seconds} 秒`;
  const minutes = Math.floor(seconds / 60);
  const remain = seconds % 60;
  if (minutes < 60) return remain > 0 ? `${minutes} 分 ${remain} 秒` : `${minutes} 分`;
  const hours = Math.floor(minutes / 60);
  const remainMinutes = minutes % 60;
  return remainMinutes > 0 ? `${hours} 小时 ${remainMinutes} 分` : `${hours} 小时`;
}
</script>
