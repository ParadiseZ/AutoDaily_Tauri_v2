<template>
  <SurfacePanel v-if="script" class="space-y-5">
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div class="min-w-0 flex-1 space-y-2">
        <div class="flex flex-wrap items-center gap-2">
          <h2 class="truncate text-xl font-semibold text-(--app-text-strong)">{{ script.data.name }}</h2>
          <StatusBadge :label="formatScriptType(script)" :tone="script.data.scriptType === 'published' ? 'info' : 'success'" />
        </div>
        <p class="max-w-2xl text-sm leading-6 text-(--app-text-soft)">
          {{ script.data.description }}
        </p>
      </div>

      <div class="flex shrink-0 flex-col items-stretch gap-2">
        <button class="app-button app-button-primary justify-center shadow-none" type="button" @click="$emit('open-editor', script.id)">
          <AppIcon name="square-pen" :size="16" />
          打开编辑器
        </button>
        <button class="app-button app-button-ghost justify-center" type="button" @click="$emit('edit-info', script.id)">
          <AppIcon name="pencil-line" :size="16" />
          编辑信息
        </button>
      </div>
    </div>

    <div class="flex flex-wrap items-center justify-between gap-3 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted)/65 px-3 py-3">
      <div class="flex flex-wrap gap-2">
        <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('upload', script.id)">
          <AppIcon name="cloud-upload" :size="14" />
          上传
        </button>
        <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('clone', script.id)">
          <AppIcon name="copy" :size="14" />
          克隆
        </button>
      </div>
      <div class="flex flex-wrap gap-2">
        <button class="app-button app-button-warning app-toolbar-button shadow-none" type="button" @click="$emit('clear-logs', script.id)">
          <AppIcon name="eraser" :size="14" />
          清理记录
        </button>
        <button class="app-button app-button-danger app-toolbar-button" type="button" @click="$emit('delete', script.id)">
          <AppIcon name="trash-2" :size="14" />
          删除
        </button>
      </div>
    </div>

    <div class="grid gap-4 2xl:grid-cols-[1.1fr_0.9fr]">
      <SurfacePanel tone="muted" padding="sm" class="space-y-4">
        <p class="text-sm font-semibold text-(--app-text-strong)">脚本概览</p>
        <div class="grid gap-3 md:grid-cols-2">
          <div class="app-stat">
            <p class="app-stat-label">运行时</p>
            <p class="app-stat-value text-base">{{ formatRuntimeLabel(script.data.runtimeType) }}</p>
          </div>
          <div class="app-stat">
            <p class="app-stat-label">脚本平台</p>
            <p class="app-stat-value text-base">{{ formatPlatformLabel(script.data.platform) }}</p>
          </div>
          <div class="app-stat">
            <p class="app-stat-label">版本</p>
            <p class="app-stat-value text-base">{{ script.data.verName }}</p>
          </div>
          <div class="app-stat">
            <p class="app-stat-label">下载次数</p>
            <p class="app-stat-value text-base">{{ formatNumberLike(script.data.downloadCount) }}</p>
          </div>
          <div class="app-stat">
            <p class="app-stat-label">克隆权限</p>
            <p class="app-stat-value text-base">{{ script.data.allowClone ? '已开放' : '关闭' }}</p>
          </div>
        </div>
      </SurfacePanel>

      <SurfacePanel tone="muted" padding="sm" class="space-y-4">
        <p class="text-sm font-semibold text-(--app-text-strong)">元信息</p>
        <div class="space-y-3 text-sm text-(--app-text-soft)">
          <div class="flex justify-between gap-4">
            <span>作者</span>
            <span class="text-(--app-text-strong)">{{ script.data.userName || '本地用户' }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span>创建时间</span>
            <span class="text-(--app-text-strong)">{{ formatDate(script.data.createTime) }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span>更新时间</span>
            <span class="text-(--app-text-strong)">{{ formatDate(script.data.updateTime) }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span>云端关联</span>
            <span class="truncate text-(--app-text-strong)">{{ script.data.cloudId || '未上传' }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span>包名</span>
            <span class="truncate text-(--app-text-strong)">{{ script.data.pkgName || '未指定' }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span>Activity</span>
            <span class="truncate text-(--app-text-strong)">{{ script.data.activityName || '未指定' }}</span>
          </div>
        </div>
      </SurfacePanel>
    </div>
  </SurfacePanel>

  <EmptyState
    v-else
    title="选择一个脚本查看详情"
  />
</template>

<script setup lang="ts">
import AppIcon from '@/components/shared/AppIcon.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import StatusBadge from '@/components/shared/StatusBadge.vue';
import type { ScriptTableRecord } from '@/types/app/domain';
import { formatDate, formatNumberLike, formatPlatformLabel, formatRuntimeLabel, formatScriptType } from '@/utils/presenters';

defineProps<{
  script: ScriptTableRecord | null;
}>();

defineEmits<{
  'open-editor': [scriptId: string];
  'edit-info': [scriptId: string];
  upload: [scriptId: string];
  clone: [scriptId: string];
  'clear-logs': [scriptId: string];
  delete: [scriptId: string];
}>();
</script>
