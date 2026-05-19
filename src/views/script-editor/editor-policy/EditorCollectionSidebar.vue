<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <div class="space-y-4">
      <slot name="mode-switch" />
      <div v-if="!collapsed" class="grid grid-cols-[minmax(0,1fr)_44px] items-center gap-2">
        <input v-model="search" class="app-input" type="search" :placeholder="searchPlaceholder" />
        <button class="app-button app-button-primary app-toolbar-button" type="button" :data-testid="createTestId" @click="$emit('create')">
          <Plus class="h-4 w-4" />
        </button>
      </div>
    </div>

    <div v-if="!collapsed" class="min-h-0 flex-1 overflow-y-auto custom-scrollbar">
      <div v-if="filteredItems.length" class="space-y-2 pr-1">
        <article
          v-for="item in filteredItems"
          :key="item.id"
          class="app-list-item space-y-3 transition-colors"
          :class="{
            'app-list-item-active': selectedId === item.id,
            'editor-drop-target': overId === item.id && draggingId !== null && draggingId !== item.id,
            'editor-dragging-item': draggingId === item.id,
          }"
          :data-testid="`${itemTestIdPrefix}-${item.id}`"
          @mouseenter="handleMouseEnter(item.id)"
          @mouseup="handleMouseUp(item.id)"
        >
          <div class="grid grid-cols-[34px_minmax(0,1fr)] items-start gap-2">
            <button
              class="app-drag-handle"
              :class="{ 'app-drag-handle-active': draggingId === item.id }"
              type="button"
              aria-label="拖动排序"
              @mousedown.prevent="startDrag(item.id)"
              @click.stop
            >
              <GripVertical class="h-4 w-4" />
            </button>
            <div class="min-w-0 text-left" type="button" @click="$emit('select', item.id)">
              <div class="flex items-center gap-2">
                <p class="truncate text-sm font-semibold text-(--app-text-strong)">{{ item.title }}</p>
                <span v-if="item.badge" class="rounded-full border border-(--app-border) px-2 py-1 text-[11px] text-(--app-text-faint)">
                  {{ item.badge }}
                </span>
              </div>
              <p class="mt-2 text-xs text-(--app-text-faint)">{{ item.subtitle }}</p>
            </div>
          </div>

          <div class="flex flex-wrap gap-2">
            <button class="app-icon-button" type="button" aria-label="复制" title="复制" @click.stop="$emit('duplicate', item.id)">
              <Copy class="h-4 w-4" />
            </button>
            <button class="app-icon-button text-red-600" type="button" aria-label="删除" title="删除" @click.stop="$emit('remove', item.id)">
              <Trash2 class="h-4 w-4" />
            </button>
          </div>
        </article>
      </div>

      <EmptyState v-else :title="emptyTitle" :description="emptyDescription" />
    </div>
  </SurfacePanel>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { EditorNamedItem } from '@/views/script-editor/editor-policy/editorPolicy';
import { Copy, GripVertical, Plus, Trash2 } from 'lucide-vue-next';

const props = defineProps<{
  searchPlaceholder: string;
  items: EditorNamedItem[];
  selectedId: string | null;
  emptyTitle: string;
  emptyDescription: string;
  createTestId: string;
  itemTestIdPrefix: string;
  collapsed?: boolean;
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

const startDrag = (id: string) => {
  draggingId.value = id;
  overId.value = id;
};

const handleMouseEnter = (id: string) => {
  if (!draggingId.value) {
    return;
  }
  overId.value = id;
};

const handleMouseUp = (targetId: string) => {
  if (!draggingId.value || draggingId.value === targetId) {
    resetDrag();
    return;
  }
  emit('reorder', draggingId.value, targetId);
  resetDrag();
};

onMounted(() => {
  window.addEventListener('mouseup', resetDrag);
});

onBeforeUnmount(() => {
  window.removeEventListener('mouseup', resetDrag);
});
</script>

<style scoped>
.editor-drop-target {
  box-shadow: inset 0 0 0 1px rgba(70, 110, 255, 0.22);
  background: color-mix(in srgb, var(--app-state-active-bg) 84%, white);
}

.editor-dragging-item {
  border-color: rgba(70, 110, 255, 0.24);
  background: rgba(70, 110, 255, 0.08);
}

</style>
