<template>
  <SurfacePanel class="flex h-full flex-col gap-4">
    <div class="flex items-center justify-between gap-3">
      <div>
        <p class="text-sm font-semibold text-[var(--app-text-strong)]">本地脚本</p>
        <p class="text-xs text-[var(--app-text-faint)]">最近修改的脚本会优先排在上面。</p>
      </div>
      <button class="app-button app-button-primary h-10 px-4" type="button" @click="$emit('create')">
        新建
      </button>
    </div>

    <input
      :value="searchQuery"
      class="app-input"
      placeholder="按脚本名或描述搜索"
      @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
    />

    <div class="flex-1 space-y-2 overflow-y-auto custom-scrollbar pr-1">
      <button
        v-for="script in scripts"
        :key="script.id"
        type="button"
        class="w-full rounded-[18px] border border-[var(--app-border)] px-4 py-3 text-left transition hover:bg-white/20 dark:hover:bg-white/5"
        :class="{ 'bg-[var(--app-accent-soft)]': script.id === selectedScriptId }"
        @click="$emit('select', script.id)"
      >
        <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ script.data.name }}</p>
        <p class="mt-1 truncate text-xs text-[var(--app-text-faint)]">{{ script.data.description || '暂无描述' }}</p>
      </button>
    </div>
  </SurfacePanel>
</template>

<script setup lang="ts">
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { ScriptTableRecord } from '@/types/app/domain';

defineProps<{
  scripts: ScriptTableRecord[];
  selectedScriptId: string | null;
  searchQuery: string;
}>();

defineEmits<{
  create: [];
  select: [scriptId: string];
  'update:searchQuery': [value: string];
}>();
</script>
