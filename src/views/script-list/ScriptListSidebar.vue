<template>
  <SurfacePanel class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <div class="sticky top-0 z-10 flex shrink-0 items-center justify-between gap-3 bg-(--app-panel)">
      <input
          :value="searchQuery"
          class="app-input"
          placeholder="按脚本名或描述搜索"
          @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
      />
      <button class="app-button app-button-primary shadow-lg shadow-(--app-accent-soft)" data-testid="script-list-create-button" type="button" @click="$emit('create')">
        <AppIcon name="plus" :size="16" />
      </button>
    </div>

    <div class="min-h-0 flex-1 space-y-2 overflow-y-auto custom-scrollbar pr-1">
      <button
        v-for="script in scripts"
        :key="script.id"
        type="button"
        class="app-list-item"
        :class="{ 'app-list-item-active': script.id === selectedScriptId }"
        @click="$emit('select', script.id)"
      >
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <p class="truncate text-sm font-semibold text-(--app-text-strong)">{{ script.data.name }}</p>
            <p class="mt-1 truncate text-xs text-(--app-text-faint)">版本：{{ script.data.verName }}</p>
          </div>
          <span v-if="script.data.scriptType === 'published'" class="shrink-0 rounded-full bg-green-100 px-2 py-1 text-[11px] text-(--app-text-strong) font-semibold">
            云
          </span>
        </div>
      </button>
    </div>
  </SurfacePanel>
</template>

<script setup lang="ts">
import AppIcon from '@/components/shared/AppIcon.vue';
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
