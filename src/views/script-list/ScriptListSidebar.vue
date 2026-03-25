<template>
  <SurfacePanel class="flex h-full flex-col gap-4">
    <div class="flex items-center justify-between gap-3">
      <div>
        <p class="text-sm font-semibold text-[var(--app-text-strong)]">本地脚本</p>
        <p class="text-xs text-[var(--app-text-faint)]">最近修改的脚本会优先排在上面。</p>
      </div>
      <button class="app-button app-button-primary app-toolbar-button" data-testid="script-list-create-button" type="button" @click="$emit('create')">
        <Plus class="h-4 w-4" />
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
        class="app-list-item"
        :class="{ 'app-list-item-active': script.id === selectedScriptId }"
        @click="$emit('select', script.id)"
      >
        <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ script.data.name }}</p>
        <p class="mt-1 truncate text-xs text-[var(--app-text-faint)]">{{ script.data.description || '暂无描述' }}</p>
      </button>
    </div>
  </SurfacePanel>
</template>

<script setup lang="ts">
import { Plus } from 'lucide-vue-next';
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
