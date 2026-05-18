<template>
  <div class="pointer-events-none relative z-20 flex w-full justify-center">
    <button
      class="pointer-events-auto flex h-10 w-10 items-center justify-center rounded-full border border-(--app-border) bg-(--app-panel)/92 text-(--app-text-soft) shadow-[0_10px_30px_rgba(15,23,42,0.12)] backdrop-blur transition-all duration-200 hover:border-(--app-accent)/40 hover:text-(--app-text-strong)"
      :class="open ? 'border-(--app-accent)/35 text-(--app-text-strong)' : ''"
      :title="title"
      type="button"
      @click="$emit('toggle')"
    >
      <AppIcon
        name="chevron-down"
        :size="16"
        class="transition-transform duration-200"
        :class="open ? 'rotate-180' : ''"
      />
    </button>

    <transition name="transfer-history-sheet">
      <div
        v-if="open"
        class="pointer-events-auto absolute inset-x-0 top-full z-30 mt-3 overflow-hidden rounded-[26px] shadow-[0_24px_64px_rgba(15,23,42,0.22)]"
      >
        <SurfacePanel tone="muted" padding="sm" class="max-h-[min(72vh,760px)] overflow-y-auto border border-(--app-border) bg-(--app-panel)/96 backdrop-blur">
          <div
            v-for="record in records"
            :key="record.id"
            class="flex items-start gap-3 border-b border-(--app-border)/80 px-3 py-3 last:border-b-0"
          >
            <div class="mt-0.5 flex h-11 w-11 shrink-0 items-center justify-center rounded-2xl bg-(--app-panel-muted) text-(--app-accent)">
              <AppIcon :name="directionIconName(record)" :size="18" />
            </div>

            <div class="min-w-0 flex-1 space-y-2">
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0 flex-1">
                  <div class="flex items-center gap-2">
                    <span class="truncate text-sm font-medium text-(--app-text-strong)">
                      {{ primaryTitle(record) }}
                    </span>
                    <span class="shrink-0 text-[11px]" :class="statusTextClass(record.status)">
                      {{ formatScriptTransferStatusLabel(record.status) }}
                    </span>
                  </div>
                  <p v-if="secondaryTitle(record)" class="truncate pt-0.5 text-xs text-(--app-text-soft)">
                    {{ secondaryTitle(record) }}
                  </p>
                </div>

                <div class="flex shrink-0 items-center gap-1">
                  <button
                    v-if="record.status === 'running'"
                    class="transfer-action-button"
                    type="button"
                    title="暂停"
                    @click="$emit('pause-record', record.id)"
                  >
                    <AppIcon name="pause" :size="14" />
                  </button>
                  <button
                    v-else-if="record.status === 'paused'"
                    class="transfer-action-button"
                    type="button"
                    title="继续"
                    @click="$emit('resume-record', record.id)"
                  >
                    <AppIcon name="play" :size="14" />
                  </button>
                  <button
                    class="transfer-action-button"
                    type="button"
                    :title="record.status === 'running' || record.status === 'paused' ? '删除并停止' : '删除记录'"
                    @click="$emit('delete-record', record.id)"
                  >
                    <AppIcon name="trash-2" :size="14" />
                  </button>
                </div>
              </div>

              <div class="flex flex-wrap items-center gap-x-4 gap-y-1 text-[11px] text-(--app-text-soft)">
                <span>模型 {{ record.completedModelFileCount }} / {{ record.modelFileCount }}</span>
                <span>{{ progressLabel(record) }}</span>
                <span>{{ speedLabel(record) }}</span>
                <span>{{ etaLabel(record) }}</span>
                <span>{{ formatDateTime(record.updatedAt || record.startedAt) }}</span>
              </div>

              <template v-if="percent(record) !== null">
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-[11px] text-(--app-text-faint)">
                    <span>{{ activeFileName(record) || '未记录文件名' }}</span>
                    <span>{{ percent(record) }}%</span>
                  </div>
                  <div class="h-1.5 overflow-hidden rounded-full bg-(--app-border)">
                    <div class="h-full rounded-full bg-(--app-accent)" :style="{ width: `${percent(record)}%` }" />
                  </div>
                </div>
              </template>

              <p v-if="record.errorMessage" class="text-xs text-red-600">{{ record.errorMessage }}</p>
            </div>
          </div>

          <EmptyState
            v-if="!records.length"
            :title="emptyTitle"
            :description="emptyDescription"
          />
        </SurfacePanel>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import type { ScriptTransferProgressEvent, ScriptTransferRecord } from '@/types/app/domain';
import AppIcon from '@/components/shared/AppIcon.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import { formatDateTime, formatScriptTransferStatusLabel } from '@/utils/presenters';

const props = defineProps<{
  title: string;
  emptyTitle: string;
  emptyDescription: string;
  open: boolean;
  records: ScriptTransferRecord[];
  getProgressEvent: (recordId: string) => ScriptTransferProgressEvent | null;
}>();

defineEmits<{
  toggle: [];
  'delete-record': [recordId: string];
  'pause-record': [recordId: string];
  'resume-record': [recordId: string];
}>();

const fallbackTitle = (record: ScriptTransferRecord) => (record.direction === 'upload' ? '上传记录' : '下载记录');
const directionIconName = (record: ScriptTransferRecord) =>
  record.direction === 'upload' ? 'upload' : 'download';
const statusTextClass = (status: ScriptTransferRecord['status']) => {
  if (status === 'running') return 'text-sky-600';
  if (status === 'paused') return 'text-amber-600';
  if (status === 'success') return 'text-emerald-600';
  return 'text-red-600';
};

const activeEvent = (record: ScriptTransferRecord) => props.getProgressEvent(record.id);
const activeFileName = (record: ScriptTransferRecord) => activeEvent(record)?.currentFileName || record.latestFileName;
const primaryTitle = (record: ScriptTransferRecord) => record.scriptName || activeFileName(record) || fallbackTitle(record);
const secondaryTitle = (record: ScriptTransferRecord) => {
  const fileName = activeFileName(record);
  if (record.scriptName && fileName) {
    return fileName;
  }
  return record.latestMessage || null;
};

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
  if (!event || (record.status !== 'running' && record.status !== 'paused')) {
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

const speedLabel = (record: ScriptTransferRecord) => {
  const event = activeEvent(record);
  if (!event || (record.status !== 'running' && record.status !== 'paused')) {
    return '未统计';
  }

  const startedAt = Date.parse(event.startedAt);
  const updatedAt = Date.parse(event.updatedAt);
  if (!Number.isFinite(startedAt) || !Number.isFinite(updatedAt) || updatedAt <= startedAt || event.bytesTransferred <= 0) {
    return '计算中';
  }

  const elapsedSeconds = Math.max(1, (updatedAt - startedAt) / 1000);
  return `${formatBytes(event.bytesTransferred / elapsedSeconds)}/s`;
};

function formatBytes(value: number) {
  if (value < 1024) return `${value} B`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB`;
  if (value < 1024 * 1024 * 1024) return `${(value / 1024 / 1024).toFixed(1)} MB`;
  return `${(value / 1024 / 1024 / 1024).toFixed(2)} GB`;
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

<style scoped>
.transfer-action-button {
  display: inline-flex;
  height: 2rem;
  width: 2rem;
  align-items: center;
  justify-content: center;
  border-radius: 9999px;
  border: 1px solid color-mix(in srgb, var(--app-border) 88%, transparent);
  background: color-mix(in srgb, var(--app-panel) 86%, transparent);
  color: var(--app-text-soft);
  transition:
    color 0.2s ease,
    border-color 0.2s ease,
    background-color 0.2s ease,
    transform 0.2s ease;
}

.transfer-action-button:hover {
  color: var(--app-text-strong);
  border-color: color-mix(in srgb, var(--app-accent) 36%, transparent);
  background: color-mix(in srgb, var(--app-panel-muted) 92%, transparent);
  transform: translateY(-1px);
}

.transfer-history-sheet-enter-active,
.transfer-history-sheet-leave-active {
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
}

.transfer-history-sheet-enter-from,
.transfer-history-sheet-leave-to {
  opacity: 0;
  transform: translateY(-8px) scaleY(0.84);
}

.transfer-history-sheet-enter-to,
.transfer-history-sheet-leave-from {
  opacity: 1;
  transform: translateY(0) scaleY(1);
}

.transfer-history-sheet-enter-from,
.transfer-history-sheet-enter-to,
.transfer-history-sheet-leave-from,
.transfer-history-sheet-leave-to {
  transform-origin: top center;
}
</style>
