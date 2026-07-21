<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <template v-if="hasSelection">
      <div class="space-y-1">
      </div>

      <div
        ref="relationPane"
        class="grid min-h-0 flex-1 overflow-hidden"
        :style="relationPaneStyle"
      >
        <section class="flex min-h-0 flex-col rounded-[18px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
          <div class="space-y-3">
            <div class="flex flex-wrap items-center gap-3">
              <div class="flex items-center gap-2">
                <p class="text-sm font-semibold text-(--app-text-strong)">{{ assignedTitle }}</p>
                <span class="rounded-full border border-(--app-border) px-2 py-1 text-[12px]">
                  {{ assignedItems.length }}
                </span>
              </div>
              <button
                v-if="showReverseAction && assignedItems.length > 1"
                class="app-button app-button-ghost app-toolbar-button"
                type="button"
                data-testid="editor-relation-reverse"
                @click="$emit('reverse')"
              >
                {{ reverseActionLabel }}
              </button>
              <input v-model="assignedSearch" class="app-input max-w-[220px]" type="search" placeholder="搜索已关联内容" data-testid="editor-relation-assigned-search" />
              <p class="text-xs text-(--app-text-faint)">拖动排序，控制执行和命中顺序。</p>
            </div>
          </div>

          <div class="mt-4 min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
            <TransitionGroup
              v-if="previewAssigned.length"
              tag="div"
              class="space-y-2"
              move-class="editor-relation-reorder-move"
            >
              <article
                v-for="(item, index) in previewAssigned"
                :key="item.id"
                class="app-list-item transition-all duration-200"
                :class="{
                  'editor-drop-target': overAssignedId === item.id && draggingAssignedId !== null && draggingAssignedId !== item.id,
                  'editor-dragging-item': draggingAssignedId === item.id,
                }"
                :data-testid="`editor-relation-assigned-${item.id}`"
                :data-relation-id="item.id"
              >
                <div class="grid grid-cols-[44px_minmax(0,1fr)_minmax(0,1fr)_auto] items-center gap-3">
                  <span class="flex h-9 w-9 items-center justify-center rounded-full border border-(--app-border) bg-(--app-panel-muted) text-sm font-semibold text-(--app-text-strong)">
                    {{ index + 1 }}
                  </span>
                  <p class="truncate text-sm font-semibold text-(--app-text-strong)">{{ item.title }}</p>
                  <p class="truncate text-sm text-(--app-text-faint)">{{ item.subtitle }}</p>
                  <div class="flex items-center gap-2">
                    <button
                      class="app-icon-button app-icon-button-sec shrink-0"
                      type="button"
                      aria-label="定位"
                      title="定位"
                      :data-testid="`editor-relation-locate-${item.id}`"
                      @click="$emit('locate', item.id)"
                    >
                      <Crosshair class="h-4 w-4" />
                    </button>
                    <button
                      class="app-drag-handle"
                      :class="{ 'app-drag-handle-active': draggingAssignedId === item.id }"
                      type="button"
                      aria-label="拖动排序"
                      :data-testid="`editor-relation-drag-${index}`"
                      @mousedown.prevent="startAssignedDrag(item.id, $event)"
                      @click.stop
                    >
                      <GripVertical class="h-4 w-4" />
                    </button>
                    <button class="app-icon-button app-crash-icon app-icon-button-sec shrink-0" type="button" aria-label="移除" title="移除" @click="$emit('unlink', item.id)">
                      <Trash2 class="h-4 w-4" />
                    </button>
                  </div>
                </div>
              </article>
            </TransitionGroup>

            <EmptyState v-else title="还没有关联内容" description="从下方未关联列表中挑选并加入当前集合。" />
          </div>
        </section>

        <div
          class="editor-relation-resize"
          title="拖动调整上下区域高度"
          data-testid="editor-relation-resize"
          @mousedown.prevent="startPaneResize"
        />

        <section class="flex min-h-0 flex-col rounded-[18px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
          <div class="space-y-3">
            <div class="flex flex-wrap items-center gap-3">
              <div class="flex items-center gap-2">
                <p class="text-sm font-semibold text-(--app-text-strong)">{{ unassignedTitle }}</p>
                <span class="rounded-full border border-(--app-border) px-2 py-1 text-[12px]">
                  {{ unassignedItems.length }}
                </span>
              </div>
              <input v-model="unassignedSearch" class="app-input max-w-[220px]" type="search" placeholder="搜索未关联内容" data-testid="editor-relation-unassigned-search" />
              <p class="text-xs text-(--app-text-faint)">点击加入当前 {{ selectedTitleText }}。</p>
            </div>
          </div>

          <div class="mt-4 min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
            <div v-if="filteredUnassigned.length" class="space-y-2">
              <article v-for="item in filteredUnassigned" :key="item.id" class="app-list-item">
                <div class="grid grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] items-center gap-3">
                  <p class="truncate text-sm font-semibold text-(--app-text-strong)">{{ item.title }}</p>
                  <p class="truncate text-sm text-(--app-text-faint)">{{ item.subtitle }}</p>
                  <button class="app-button app-button-primary app-toolbar-button justify-center shrink-0" type="button" @click="$emit('link', item.id)">
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

  <Teleport to="body">
    <article
      v-if="draggingAssignedItem"
      class="editor-relation-drag-overlay app-list-item"
      :style="dragOverlayStyle"
    >
      <div class="grid grid-cols-[44px_minmax(0,1fr)_minmax(0,1fr)_auto] items-center gap-3">
        <span class="flex h-9 w-9 items-center justify-center rounded-full border border-(--app-border) bg-(--app-panel-muted) text-sm font-semibold text-(--app-text-strong)">
          {{ draggingAssignedIndex + 1 }}
        </span>
        <p class="truncate text-sm font-semibold text-(--app-text-strong)">{{ draggingAssignedItem.title }}</p>
        <p class="truncate text-sm text-(--app-text-faint)">{{ draggingAssignedItem.subtitle }}</p>
        <div class="app-drag-handle app-drag-handle-active pointer-events-none">
          <GripVertical class="h-4 w-4" />
        </div>
      </div>
    </article>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { Crosshair, GripVertical, Trash2 } from '@lucide/vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { EditorNamedItem } from '@/views/script-editor/editor-policy/editorPolicy';

const props = withDefaults(
  defineProps<{
    hasSelection: boolean;
    selectedTitle: string | null;
    assignedTitle: string;
    unassignedTitle: string;
    assignedItems: EditorNamedItem[];
    unassignedItems: EditorNamedItem[];
    showReverseAction?: boolean;
    reverseActionLabel?: string;
  }>(),
  {
    showReverseAction: false,
    reverseActionLabel: '逆序',
  },
);

const emit = defineEmits<{
  link: [id: string];
  locate: [id: string];
  unlink: [id: string];
  reorder: [draggedId: string, targetId: string];
  reverse: [];
}>();

const assignedSearch = ref('');
const unassignedSearch = ref('');
const draggingAssignedId = ref<string | null>(null);
const overAssignedId = ref<string | null>(null);
const dragPointer = ref({ x: 0, y: 0, width: 0, height: 0 });
const dragStartY = ref(0);
const dragTargetCenters = ref<Array<{ id: string; y: number }>>([]);
const relationPane = ref<HTMLElement | null>(null);
const relationPaneRatio = ref(0.5);
const resizingPane = ref(false);
const paneResizeStartY = ref(0);
const paneResizeStartRatio = ref(relationPaneRatio.value);
const selectedTitleText = computed(() => props.selectedTitle?.trim() || '当前集合');
const RELATION_PANE_HANDLE_HEIGHT = 10;
const RELATION_PANE_MIN_HEIGHT = 140;

const matchesSearch = (item: EditorNamedItem, keyword: string) =>
  `${item.title} ${item.subtitle}`.toLowerCase().includes(keyword);

const reorderPreviewItems = <T extends { id: string }>(items: T[], draggedId: string | null, targetId: string | null) => {
  if (!draggedId || !targetId || draggedId === targetId) {
    return items;
  }
  const nextItems = [...items];
  const fromIndex = nextItems.findIndex((item) => item.id === draggedId);
  const toIndex = nextItems.findIndex((item) => item.id === targetId);
  if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) {
    return items;
  }
  const [movedItem] = nextItems.splice(fromIndex, 1);
  if (!movedItem) {
    return items;
  }
  nextItems.splice(toIndex, 0, movedItem);
  return nextItems;
};

const filteredAssigned = computed(() => {
  const keyword = assignedSearch.value.trim().toLowerCase();
  if (!keyword) return props.assignedItems;
  return props.assignedItems.filter((item) => matchesSearch(item, keyword));
});
const previewAssigned = computed(() => reorderPreviewItems(filteredAssigned.value, draggingAssignedId.value, overAssignedId.value));

const filteredUnassigned = computed(() => {
  const keyword = unassignedSearch.value.trim().toLowerCase();
  if (!keyword) return props.unassignedItems;
  return props.unassignedItems.filter((item) => matchesSearch(item, keyword));
});
const draggingAssignedItem = computed(() => props.assignedItems.find((item) => item.id === draggingAssignedId.value) ?? null);
const draggingAssignedIndex = computed(() => props.assignedItems.findIndex((item) => item.id === draggingAssignedId.value));
const dragOverlayStyle = computed(() => ({
  left: `${dragPointer.value.x}px`,
  top: `${dragPointer.value.y - dragPointer.value.height / 2}px`,
  ...(dragPointer.value.width ? { width: `${dragPointer.value.width}px` } : {}),
  ...(dragPointer.value.height ? { height: `${dragPointer.value.height}px` } : {}),
}));

const relationPaneStyle = computed(() => ({
  gridTemplateRows: `minmax(0, ${relationPaneRatio.value}fr) ${RELATION_PANE_HANDLE_HEIGHT}px minmax(0, ${1 - relationPaneRatio.value}fr)`,
}));

const clampRelationPaneRatio = (value: number) => {
  const height = relationPane.value?.getBoundingClientRect().height ?? 0;
  const availableHeight = height - RELATION_PANE_HANDLE_HEIGHT;
  if (availableHeight <= RELATION_PANE_MIN_HEIGHT * 2) {
    return 0.5;
  }
  const minRatio = RELATION_PANE_MIN_HEIGHT / availableHeight;
  return Math.min(1 - minRatio, Math.max(minRatio, value));
};

const applyDraggingUi = (active: boolean) => {
  document.body.style.userSelect = active ? 'none' : '';
  document.body.style.cursor = active ? 'grabbing' : '';
};

const updateDragPointer = (
  event: MouseEvent,
  x = dragPointer.value.x,
  width = dragPointer.value.width,
  height = dragPointer.value.height,
) => {
  dragPointer.value = {
    x,
    y: event.clientY,
    width,
    height,
  };
};

const resolveRelationIdAtPoint = (event: MouseEvent) => {
  const targetCenters = dragTargetCenters.value;
  if (event.clientY > dragStartY.value) {
    return targetCenters.filter((target) => target.y > dragStartY.value && target.y < event.clientY).at(-1)?.id ?? null;
  }
  if (event.clientY < dragStartY.value) {
    return targetCenters.find((target) => target.y < dragStartY.value && target.y > event.clientY)?.id ?? null;
  }
  return null;
};

const resetAssignedDrag = () => {
  draggingAssignedId.value = null;
  overAssignedId.value = null;
  dragTargetCenters.value = [];
  applyDraggingUi(false);
};

const startAssignedDrag = (id: string, event: MouseEvent) => {
  draggingAssignedId.value = id;
  overAssignedId.value = id;
  const sourceRect = (event.target as HTMLElement).closest<HTMLElement>('[data-relation-id]')?.getBoundingClientRect();
  updateDragPointer(event, sourceRect?.left ?? event.clientX, sourceRect?.width ?? 0, sourceRect?.height ?? 0);
  dragStartY.value = event.clientY;
  dragTargetCenters.value = Array.from(document.querySelectorAll<HTMLElement>('[data-relation-id]'))
    .filter((target) => target.dataset.relationId !== id)
    .map((target) => {
      const rect = target.getBoundingClientRect();
      return { id: target.dataset.relationId!, y: rect.top + rect.height / 2 };
    })
    .filter((target) => target.y > 0);
  applyDraggingUi(true);
};

const handleWindowMouseMove = (event: MouseEvent) => {
  if (!draggingAssignedId.value) {
    return;
  }
  updateDragPointer(event);
  overAssignedId.value = resolveRelationIdAtPoint(event) ?? draggingAssignedId.value;
};

const handleWindowMouseUp = () => {
  if (draggingAssignedId.value && overAssignedId.value && draggingAssignedId.value !== overAssignedId.value) {
    emit('reorder', draggingAssignedId.value, overAssignedId.value);
  }
  resetAssignedDrag();
};

const stopPaneResize = () => {
  if (!resizingPane.value) {
    return;
  }
  resizingPane.value = false;
  document.body.style.cursor = '';
  document.body.style.userSelect = '';
};

const handlePaneResize = (event: MouseEvent) => {
  if (!resizingPane.value || !relationPane.value) {
    return;
  }
  const availableHeight = relationPane.value.getBoundingClientRect().height - RELATION_PANE_HANDLE_HEIGHT;
  if (availableHeight <= 0) {
    return;
  }
  const deltaRatio = (event.clientY - paneResizeStartY.value) / availableHeight;
  relationPaneRatio.value = clampRelationPaneRatio(paneResizeStartRatio.value + deltaRatio);
};

const startPaneResize = (event: MouseEvent) => {
  if (!relationPane.value) {
    return;
  }
  resizingPane.value = true;
  paneResizeStartY.value = event.clientY;
  paneResizeStartRatio.value = relationPaneRatio.value;
  document.body.style.cursor = 'ns-resize';
  document.body.style.userSelect = 'none';
};

onMounted(() => {
  window.addEventListener('mousemove', handleWindowMouseMove);
  window.addEventListener('mouseup', handleWindowMouseUp);
  window.addEventListener('mousemove', handlePaneResize);
  window.addEventListener('mouseup', stopPaneResize);
});

onBeforeUnmount(() => {
  window.removeEventListener('mousemove', handleWindowMouseMove);
  window.removeEventListener('mouseup', handleWindowMouseUp);
  window.removeEventListener('mousemove', handlePaneResize);
  window.removeEventListener('mouseup', stopPaneResize);
  stopPaneResize();
  applyDraggingUi(false);
});
</script>

<style scoped>
.editor-relation-resize {
  cursor: ns-resize;
  background:
    linear-gradient(
      180deg,
      transparent 0,
      transparent 4px,
      color-mix(in srgb, var(--app-border) 70%, transparent) 4px,
      color-mix(in srgb, var(--app-border) 70%, transparent) 6px,
      transparent 6px
    );
}

.editor-relation-resize:hover {
  background-color: rgba(70, 110, 255, 0.96);
}

.editor-dragging-item {
  opacity: 0;
}

.editor-drop-target {
  box-shadow: inset 0 0 0 1px rgba(70, 110, 255, 0.22);
  background: color-mix(in srgb, var(--app-state-active-bg) 84%, white);
  transform: translateX(6px);
}

.editor-relation-reorder-move {
  transition: transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.editor-relation-drag-overlay {
  position: fixed;
  z-index: 70;
  width: min(480px, calc(100vw - 32px));
  pointer-events: none;
  border-color: rgba(70, 110, 255, 0.24);
  background: color-mix(in srgb, var(--app-panel) 92%, white);
  box-shadow: 0 18px 36px rgba(15, 23, 42, 0.2);
  transform: scale(1.03);
}
</style>
