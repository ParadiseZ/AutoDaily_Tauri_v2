<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <div class="space-y-4">
      <slot name="mode-switch" />
      <div class="grid grid-cols-[minmax(0,1fr)_44px] items-center gap-2">
        <input v-model="search" class="app-input" type="search" :placeholder="searchPlaceholder" />
        <button class="app-button app-button-primary app-toolbar-button" type="button" :data-testid="createTestId" @click="$emit('create')">
          <Plus class="h-4 w-4" />
        </button>
      </div>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto custom-scrollbar">
      <div v-if="filteredItems.length" class="space-y-2 pr-1">
        <article
          v-for="item in filteredItems"
          :key="item.id"
          class="app-list-item space-y-3 transition-colors"
          :class="{
            'app-list-item-active': selectedId === item.id,
            'editor-drop-target': overId === item.id && draggingId !== item.id,
          }"
          draggable="true"
          :data-testid="`${itemTestIdPrefix}-${item.id}`"
          @dragenter.prevent="overId = item.id"
          @dragover.prevent="handleDragOver($event, item.id)"
          @dragleave="handleDragLeave(item.id)"
          @dragstart="handleDragStart($event, item.id)"
          @dragend="resetDrag"
          @drop.prevent="handleDrop(item.id)"
        >
          <div class="flex items-start justify-between gap-3">
            <button class="min-w-0 flex-1 text-left" type="button" @click="$emit('select', item.id)">
              <div class="flex items-center gap-2">
                <p class="truncate text-sm font-semibold text-(--app-text-strong)">{{ item.title }}</p>
                <span v-if="item.badge" class="rounded-full border border-(--app-border) px-2 py-1 text-[11px] text-(--app-text-faint)">
                  {{ item.badge }}
                </span>
              </div>
              <p class="mt-2 text-xs text-(--app-text-faint)">{{ item.subtitle }}</p>
            </button>
            <span class="text-[11px] uppercase tracking-[0.16em] text-(--app-text-faint)">拖动排序</span>
          </div>

          <div class="flex flex-wrap gap-2">
            <button class="app-button app-button-ghost app-toolbar-button" type="button" @click.stop="$emit('duplicate', item.id)">
              复制
            </button>
            <button class="app-button app-button-danger app-toolbar-button" type="button" @click.stop="$emit('remove', item.id)">
              删除
            </button>
          </div>
        </article>
      </div>

      <EmptyState v-else :title="emptyTitle" :description="emptyDescription" />
    </div>
  </SurfacePanel>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { EditorNamedItem } from '@/views/script-editor/editor-policy/editorPolicy';
import {Plus} from "lucide-vue-next";

const props = defineProps<{
  searchPlaceholder: string;
  items: EditorNamedItem[];
  selectedId: string | null;
  emptyTitle: string;
  emptyDescription: string;
  createTestId: string;
  itemTestIdPrefix: string;
}>();

const emit = defineEmits<{
  create: [];
  select: [id: string];
  duplicate: [id: string];
  remove: [id: string];
  reorder: [draggedId: string, targetId: string];
}>();

const search = ref('');
const draggingId = ref<string | null>(null);
const overId = ref<string | null>(null);

const filteredItems = computed(() => {
  const keyword = search.value.trim().toLowerCase();
  if (!keyword) {
    return props.items;
  }
  return props.items.filter((item) => `${item.title} ${item.subtitle}`.toLowerCase().includes(keyword));
});

const resetDrag = () => {
  draggingId.value = null;
  overId.value = null;
};

const handleDragLeave = (id: string) => {
  if (overId.value === id) {
    overId.value = null;
  }
};

const handleDragStart = (event: DragEvent, id: string) => {
  draggingId.value = id;
  event.dataTransfer?.setData('text/plain', id);
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
  }
};

const handleDragOver = (event: DragEvent, id: string) => {
  overId.value = id;
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move';
  }
};

const handleDrop = (targetId: string) => {
  if (!draggingId.value || draggingId.value === targetId) {
    resetDrag();
    return;
  }
  emit('reorder', draggingId.value, targetId);
  resetDrag();
};
</script>

<style scoped>
.editor-drop-target {
  box-shadow: inset 0 0 0 1px rgba(70, 110, 255, 0.22);
  background: color-mix(in srgb, var(--app-state-active-bg) 84%, white);
}
</style>
