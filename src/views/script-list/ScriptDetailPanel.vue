<template>
  <SurfacePanel v-if="script" class="space-y-6">
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div class="space-y-2">
        <div class="flex flex-wrap items-center gap-2">
          <h2 class="text-2xl font-semibold text-[var(--app-text-strong)]">{{ script.data.name }}</h2>
          <StatusBadge :label="formatScriptType(script)" :tone="script.data.scriptType === 'published' ? 'info' : 'success'" />
        </div>
        <p class="max-w-2xl text-sm leading-6 text-[var(--app-text-soft)]">
          {{ script.data.description || '这个脚本还没有补充说明，建议在进入编辑器后先整理运行目标和输入输出约束。' }}
        </p>
      </div>

      <div class="flex flex-wrap gap-2">
        <button class="app-button app-button-primary" type="button" @click="$emit('open-editor', script.id)">
          打开编辑器
        </button>
        <button class="app-button app-button-ghost" type="button" @click="$emit('upload', script.id)">
          上传云端
        </button>
        <button class="app-button app-button-ghost" type="button" @click="$emit('clone', script.id)">
          克隆副本
        </button>
        <button class="app-button app-button-warning" type="button" @click="$emit('clear-logs', script.id)">
          清理记录
        </button>
        <button class="app-button app-button-danger" type="button" @click="$emit('delete', script.id)">
          删除
        </button>
      </div>
    </div>

    <div class="grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
      <SurfacePanel tone="muted" padding="sm" class="space-y-4">
        <p class="text-sm font-semibold text-[var(--app-text-strong)]">脚本概览</p>
        <div class="grid gap-3 md:grid-cols-2">
          <div class="app-stat">
            <p class="app-stat-label">运行时</p>
            <p class="app-stat-value text-base">{{ formatRuntimeLabel(script.data.runtimeType) }}</p>
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
        <p class="text-sm font-semibold text-[var(--app-text-strong)]">元信息</p>
        <div class="space-y-3 text-sm text-[var(--app-text-soft)]">
          <div class="flex justify-between gap-4">
            <span>作者</span>
            <span class="text-[var(--app-text-strong)]">{{ script.data.userName || '本地用户' }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span>创建时间</span>
            <span class="text-[var(--app-text-strong)]">{{ formatDate(script.data.createTime) }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span>更新时间</span>
            <span class="text-[var(--app-text-strong)]">{{ formatDate(script.data.updateTime) }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span>云端关联</span>
            <span class="truncate text-[var(--app-text-strong)]">{{ script.data.cloudId || '未上传' }}</span>
          </div>
          <div class="flex justify-between gap-4">
            <span>包名</span>
            <span class="truncate text-[var(--app-text-strong)]">{{ script.data.pkgName || '未指定' }}</span>
          </div>
        </div>
      </SurfacePanel>
    </div>
  </SurfacePanel>

  <EmptyState
    v-else
    title="选择一个脚本查看详情"
    description="左侧脚本列表承载浏览，右侧负责解释这个脚本的结构和动作。现在先选一个脚本，让我们继续往编辑器工作区走。"
  />
</template>

<script setup lang="ts">
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import StatusBadge from '@/components/shared/StatusBadge.vue';
import type { ScriptTableRecord } from '@/types/app/domain';
import { formatDate, formatNumberLike, formatRuntimeLabel, formatScriptType } from '@/utils/presenters';

defineProps<{
  script: ScriptTableRecord | null;
}>();

defineEmits<{
  'open-editor': [scriptId: string];
  upload: [scriptId: string];
  clone: [scriptId: string];
  'clear-logs': [scriptId: string];
  delete: [scriptId: string];
}>();
</script>
