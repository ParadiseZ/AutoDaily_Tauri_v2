<template>
  <SurfacePanel class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <div class="sticky top-0 z-10 flex shrink-0 items-center justify-between gap-3 bg-(--app-panel)">
      <p class="text-sm font-semibold text-(--app-text-strong)">更新日志</p>
      <span v-if="loadFailed" class="text-xs text-amber-700">网络异常，仅显示当前版本</span>
    </div>

    <template v-if="script">
      <div class="min-h-0 flex-1 overflow-y-auto custom-scrollbar pr-1">
        <div v-if="loading">
          <div class="space-y-3">
            <div v-for="index in 3" :key="index" class="log-skeleton">
              <div class="log-skeleton-line w-36" />
              <div class="log-skeleton-line mt-3 w-full" />
              <div class="log-skeleton-line mt-2 w-[88%]" />
              <div class="log-skeleton-line mt-2 w-[72%]" />
            </div>
          </div>
        </div>

        <div v-else>
          <div v-if="logs.length" class="space-y-4">
            <section
              v-for="log in logs"
              :key="`${log.versionNum ?? 'null'}-${log.versionName ?? 'unnamed'}-${log.updatedAt ?? ''}`"
              class="rounded-[16px] border border-(--app-border) bg-white/55 px-4 py-4"
            >
              <div class="flex items-center justify-between gap-3">
                <p class="text-sm font-semibold text-(--app-text-strong)">
                  {{ log.versionName || `版本 ${log.versionNum ?? '-'}` }}
                </p>
                <span class="text-xs text-(--app-text-faint)">
                  {{ formatDateTime(log.updatedAt || log.createdAt) }}
                </span>
              </div>
              <div class="mt-3">
                <MarkdownView :content="log.contentMd" empty-text="该版本没有填写更新日志。" />
              </div>
            </section>
          </div>

          <EmptyState
            v-else
            title="暂无更新日志"
          />
        </div>
      </div>
    </template>

    <EmptyState
      v-else
      title="选择一个脚本查看更新日志"
    />
  </SurfacePanel>
</template>

<script setup lang="ts">
import EmptyState from '@/components/shared/EmptyState.vue';
import MarkdownView from '@/components/shared/MarkdownView.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { ScriptChangeLogRecord, ScriptTableRecord } from '@/types/app/domain';
import { formatDateTime } from '@/utils/presenters';

defineProps<{
  script: ScriptTableRecord | null;
  logs: ScriptChangeLogRecord[];
  loading: boolean;
  loadFailed: boolean;
}>();
</script>

<style scoped>
@reference "../../style.css";

.log-skeleton {
  @apply rounded-[16px] border border-(--app-border) bg-white/55 px-4 py-4;
}

.log-skeleton-line {
  @apply h-3 rounded-full bg-(--app-panel-muted) animate-pulse;
}
</style>
