<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <template v-if="selectedTitle">
      <div class="space-y-1">
        <p class="text-xs uppercase tracking-[0.18em] text-[var(--app-text-faint)]">Workspace</p>
        <h2 class="text-xl font-semibold text-[var(--app-text-strong)]">{{ title }}</h2>
      </div>

      <div class="grid min-h-0 gap-4 xl:grid-rows-[minmax(0,1fr)_minmax(0,1fr)]">
        <section class="flex min-h-0 flex-col rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
          <div class="space-y-3">
            <div class="flex flex-wrap items-center gap-3">
              <div class="flex items-center gap-2">
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">{{ assignedTitle }}</p>
                <span class="rounded-full border border-[var(--app-border)] bg-white/60 px-2 py-1 text-[11px] text-[var(--app-text-faint)]">
                  {{ assignedItems.length }}
                </span>
              </div>
              <input v-model="assignedSearch" class="app-input max-w-[220px]" type="search" placeholder="搜索已关联内容" />
              <p class="text-xs text-[var(--app-text-faint)]">拖动排序，控制执行和命中顺序。</p>
            </div>
          </div>

          <div class="mt-4 min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
            <div v-if="filteredAssigned.length" class="space-y-2">
              <article
                v-for="(item, index) in filteredAssigned"
                :key="item.id"
                class="app-list-item transition-colors"
                :class="{
                  'editor-drop-target': overAssignedId === item.id && draggingAssignedId !== null && draggingAssignedId !== item.id,
                  'editor-dragging-item': draggingAssignedId === item.id,
                }"
                :data-testid="`editor-relation-assigned-${item.id}`"
                :data-relation-id="item.id"
                @mouseenter="handleAssignedHover(item.id)"
                @mousemove="handleAssignedHover(item.id)"
                @dragenter.prevent="handleAssignedNativeEnter(item.id)"
                @dragover.prevent="handleAssignedNativeOver($event, item.id)"
                @drop.prevent="handleAssignedNativeDrop(item.id)"
              >
                <div class="grid grid-cols-[44px_minmax(0,1fr)_minmax(0,1fr)_auto] items-center gap-3">
                  <span class="flex h-9 w-9 items-center justify-center rounded-full border border-[var(--app-border)] bg-white/70 text-sm font-semibold text-[var(--app-text-strong)]">
                    {{ index + 1 }}
                  </span>
                  <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ item.title }}</p>
                  <p class="truncate text-sm text-[var(--app-text-faint)]">{{ item.subtitle }}</p>
                  <div class="flex items-center gap-2">
                    <button
                      class="editor-drag-handle"
                      :class="{ 'editor-drag-handle-active': draggingAssignedId === item.id }"
                      type="button"
                      aria-label="拖动排序"
                      :data-testid="`editor-relation-drag-${index}`"
                      draggable="true"
                      @dragstart="handleAssignedNativeStart($event, item.id)"
                      @dragend="resetAssignedDrag"
                      @pointerdown.prevent="startAssignedDrag(item.id)"
                      @mousedown.prevent="startAssignedDrag(item.id)"
                      @click.stop
                    >
                      <GripVertical class="h-4 w-4" />
                    </button>
                    <button class="app-button app-button-danger app-toolbar-button shrink-0" type="button" @click="$emit('unlink', item.id)">
                      移除
                    </button>
                  </div>
                </div>
              </article>
            </div>

            <EmptyState v-else title="还没有关联内容" description="从下方未关联列表中挑选并加入当前集合。" />
          </div>
        </section>

        <section class="flex min-h-0 flex-col rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
          <div class="space-y-3">
            <div class="flex flex-wrap items-center gap-3">
              <div class="flex items-center gap-2">
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">{{ unassignedTitle }}</p>
                <span class="rounded-full border border-[var(--app-border)] bg-white/60 px-2 py-1 text-[11px] text-[var(--app-text-faint)]">
                  {{ unassignedItems.length }}
                </span>
              </div>
              <input v-model="unassignedSearch" class="app-input max-w-[220px]" type="search" placeholder="搜索未关联内容" />
              <p class="text-xs text-[var(--app-text-faint)]">点击加入当前 {{ selectedTitle }}。</p>
            </div>
          </div>

          <div class="mt-4 min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
            <div v-if="filteredUnassigned.length" class="space-y-2">
              <article v-for="item in filteredUnassigned" :key="item.id" class="app-list-item">
                <div class="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] items-center gap-3">
                  <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ item.title }}</p>
                  <p class="truncate text-sm text-[var(--app-text-faint)]">{{ item.subtitle }}</p>
                  <button class="app-button app-button-primary app-toolbar-button shrink-0" type="button" @click="$emit('link', item.id)">
                    添加
                  </button>
                </div>
              </article>
            </div>

            <EmptyState v-else title="没有更多可选内容" description="当前脚本中的可选项都已经关联了。" />
          </div>
        </section>
      </div>
    </template>

    <EmptyState v-else title="先选择一个对象" description="左侧选中策略组或策略集后，再在这里调整关联关系。" />
  </SurfacePanel>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { GripVertical } from 'lucide-vue-next';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { EditorNamedItem } from '@/views/script-editor/editor-policy/editorPolicy';

const props = defineProps<{
  title: string;
  selectedTitle: string | null;
  assignedTitle: string;
  unassignedTitle: string;
  assignedItems: EditorNamedItem[];
  unassignedItems: EditorNamedItem[];
}>();

const emit = defineEmits<{
  link: [id: string];
  unlink: [id: string];
  reorder: [draggedId: string, targetId: string];
}>();

const assignedSearch = ref('');
const unassignedSearch = ref('');
const draggingAssignedId = ref<string | null>(null);
const overAssignedId = ref<string | null>(null);

const matchesSearch = (item: EditorNamedItem, keyword: string) =>
  `${item.title} ${item.subtitle}`.toLowerCase().includes(keyword);

const filteredAssigned = computed(() => {
  const keyword = assignedSearch.value.trim().toLowerCase();
  if (!keyword) return props.assignedItems;
  return props.assignedItems.filter((item) => matchesSearch(item, keyword));
});

const filteredUnassigned = computed(() => {
  const keyword = unassignedSearch.value.trim().toLowerCase();
  if (!keyword) return props.unassignedItems;
  return props.unassignedItems.filter((item) => matchesSearch(item, keyword));
});

const resetAssignedDrag = () => {
  draggingAssignedId.value = null;
  overAssignedId.value = null;
};

const findRelationItemAtPoint = (clientX: number, clientY: number) => {
  const items = Array.from(document.querySelectorAll<HTMLElement>('[data-relation-id]'));
  for (const item of items) {
    const rect = item.getBoundingClientRect();
    if (clientX >= rect.left && clientX <= rect.right && clientY >= rect.top && clientY <= rect.bottom) {
      return item.dataset.relationId ?? null;
    }
  }
  return null;
};

const startAssignedDrag = (id: string) => {
  draggingAssignedId.value = id;
  overAssignedId.value = id;
};

const handleAssignedHover = (targetId: string) => {
  if (!draggingAssignedId.value) {
    return;
  }
  if (targetId !== draggingAssignedId.value && targetId !== overAssignedId.value) {
    emit('reorder', draggingAssignedId.value, targetId);
  }
  overAssignedId.value = targetId;
};

const handleAssignedNativeStart = (event: DragEvent, id: string) => {
  draggingAssignedId.value = id;
  overAssignedId.value = id;
  event.dataTransfer?.setData('text/plain', id);
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
  }
};

const handleAssignedNativeEnter = (id: string) => {
  handleAssignedHover(id);
};

const handleAssignedNativeOver = (event: DragEvent, id: string) => {
  handleAssignedHover(id);
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move';
  }
};

const handleAssignedNativeDrop = (targetId: string) => {
  if (!draggingAssignedId.value || draggingAssignedId.value === targetId) {
    resetAssignedDrag();
    return;
  }
  emit('reorder', draggingAssignedId.value, targetId);
  resetAssignedDrag();
};

const handleWindowMouseUp = () => {
  resetAssignedDrag();
};

const handleWindowMouseMove = (event: MouseEvent) => {
  if (!draggingAssignedId.value) {
    return;
  }
  const targetId = findRelationItemAtPoint(event.clientX, event.clientY);
  if (!targetId) {
    return;
  }
  handleAssignedHover(targetId);
};

onMounted(() => {
  window.addEventListener('mousemove', handleWindowMouseMove);
  window.addEventListener('mouseup', handleWindowMouseUp);
});

onBeforeUnmount(() => {
  window.removeEventListener('mousemove', handleWindowMouseMove);
  window.removeEventListener('mouseup', handleWindowMouseUp);
});
</script>

<style scoped>
.editor-drag-handle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 12px;
  border: 1px dashed var(--app-border);
  background: rgba(255, 255, 255, 0.55);
  color: var(--app-text-faint);
  cursor: grab;
}

.editor-drag-handle:active,
.editor-drag-handle-active {
  cursor: grabbing;
}

.editor-drag-handle-active {
  border-color: var(--app-state-active-border);
  background: var(--app-state-active-bg);
  color: var(--app-text-strong);
}

.editor-dragging-item {
  border-color: rgba(70, 110, 255, 0.24);
  background: rgba(70, 110, 255, 0.08);
}

.editor-drop-target {
  box-shadow: inset 0 0 0 1px rgba(70, 110, 255, 0.22);
  background: color-mix(in srgb, var(--app-state-active-bg) 84%, white);
}
</style>
