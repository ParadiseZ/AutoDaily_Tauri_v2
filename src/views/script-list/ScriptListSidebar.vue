<template>
  <SurfacePanel class="flex h-full flex-col gap-4">
    <div class="flex items-center justify-between gap-2">
      <input
          :value="searchQuery"
          class="app-input"
          placeholder="按脚本名或描述搜索"
          @input="$emit('update:searchQuery', ($event.target as HTMLInputElement).value)"
      />
      <button class="app-button app-button-primary shadow-lg shadow-[var(--app-accent-soft)]" data-testid="script-list-create-button" type="button" @click="$emit('create')">
        <AppIcon name="plus" :size="16" />
      </button>
    </div>



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
