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

    <div v-if="!collapsed" class="min-h-0 flex-1 overflow-y-auto custom-scrollbar" @scroll="closeContextMenu">
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
          @contextmenu="handleItemContextMenu($event, item.id)"
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
            <div class="min-w-0 text-left" @click="selectItem(item.id)">
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
            <button class="app-icon-button app-icon-button-sec" type="button" aria-label="复制" title="复制" @click.stop="$emit('duplicate', item.id)">
              <Copy class="h-4 w-4" />
            </button>
            <button class="app-icon-button app-crash-icon app-icon-button-sec" type="button" aria-label="删除" title="删除" @click.stop="$emit('remove', item.id)">
              <Trash2 class="h-4 w-4" />
            </button>
          </div>
        </article>
      </div>

      <EmptyState v-else :title="emptyTitle" :description="emptyDescription" />
    </div>
  </SurfacePanel>

  <Teleport to="body">
    <div
      v-if="contextMenu"
      ref="contextMenuRoot"
      class="editor-collection-menu app-select-menu app-select-menu-floating"
      :data-testid="`${testIdPrefix}-context-menu`"
      :style="contextMenuStyle"
    >
      <button
        class="editor-collection-menu-item"
        :data-testid="`${testIdPrefix}-move-current-top`"
        type="button"
        @click="emitMove('current', 'top')"
      >
        移动到顶部
      </button>
      <button
        class="editor-collection-menu-item"
        :data-testid="`${testIdPrefix}-move-current-bottom`"
        type="button"
        @click="emitMove('current', 'bottom')"
      >
        移动到底部
      </button>
      <button
        class="editor-collection-menu-item editor-collection-menu-item-branch"
        :data-testid="`${testIdPrefix}-move-item`"
        type="button"
        @mouseenter="openTargetMenu($event)"
      >
        <span>移动到...</span>
        <ChevronRight class="h-4 w-4" />
      </button>
    </div>

    <div
      v-if="contextMenu && targetMenuOpen"
      ref="targetMenuRoot"
      class="editor-collection-menu app-select-menu app-select-menu-floating"
      :data-testid="`${testIdPrefix}-context-item-menu`"
      :style="targetMenuStyle"
    >
      <button
        v-for="item in targetItems"
        :key="item.id"
        class="editor-collection-menu-item editor-collection-menu-item-branch"
        :data-testid="`${testIdPrefix}-move-item-${item.id}`"
        type="button"
        @mouseenter="setActiveTarget(item.id, $event)"
      >
        <span class="truncate">{{ item.title }}</span>
        <ChevronRight class="h-4 w-4" />
      </button>
      <div v-if="!targetItems.length" class="app-select-empty">没有匹配项</div>
    </div>

    <div
      v-if="contextMenu && targetMenuOpen && activeTarget"
      ref="actionMenuRoot"
      class="editor-collection-menu app-select-menu app-select-menu-floating"
      :data-testid="`${testIdPrefix}-context-item-action-menu`"
      :style="actionMenuStyle"
    >
      <button
        class="editor-collection-menu-item"
        :data-testid="`${testIdPrefix}-move-item-${activeTarget.id}-top`"
        type="button"
        @click="emitMove('item', 'top', activeTarget.id)"
      >
        顶部
      </button>
      <button
        class="editor-collection-menu-item"
        :data-testid="`${testIdPrefix}-move-item-${activeTarget.id}-bottom`"
        type="button"
        @click="emitMove('item', 'bottom', activeTarget.id)"
      >
        底部
      </button>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { EditorCollectionMoveAction, EditorNamedItem } from '@/views/script-editor/editor-policy/editorPolicy';
import { ChevronRight, Copy, GripVertical, Plus, Trash2 } from 'lucide-vue-next';

type MenuRect = {
  top: number;
  left: number;
  right: number;
  bottom: number;
};

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
  'move-item': [itemId: string, action: EditorCollectionMoveAction];
}>();

const ROOT_MENU_WIDTH = 220;
const SUBMENU_WIDTH = 240;
const ACTION_MENU_WIDTH = 144;
const VIEWPORT_PADDING = 12;
const EXPANDED_MENU_MAX_HEIGHT = 320;

const search = ref('');
const draggingId = ref<string | null>(null);
const overId = ref<string | null>(null);
const contextMenu = ref<{ itemId: string; x: number; y: number } | null>(null);
const targetMenuOpen = ref(false);
const targetAnchor = ref<MenuRect | null>(null);
const activeTargetId = ref<string | null>(null);
const activeLeafAnchor = ref<MenuRect | null>(null);
const contextMenuRoot = ref<HTMLElement | null>(null);
const targetMenuRoot = ref<HTMLElement | null>(null);
const actionMenuRoot = ref<HTMLElement | null>(null);

const filteredItems = computed(() => {
  const keyword = search.value.trim().toLowerCase();
  if (!keyword) {
    return props.items;
  }
  return props.items.filter((item) => `${item.title} ${item.subtitle}`.toLowerCase().includes(keyword));
});
const testIdPrefix = computed(() => props.itemTestIdPrefix.replace(/-item$/, ''));
const currentContextItem = computed(() => props.items.find((item) => item.id === contextMenu.value?.itemId) ?? null);
const targetItems = computed(() => filteredItems.value.filter((item) => item.id !== contextMenu.value?.itemId));
const activeTarget = computed(() => props.items.find((item) => item.id === activeTargetId.value) ?? null);

const resetDrag = () => {
  draggingId.value = null;
  overId.value = null;
};

const closeContextMenu = () => {
  contextMenu.value = null;
  targetMenuOpen.value = false;
  targetAnchor.value = null;
  activeTargetId.value = null;
  activeLeafAnchor.value = null;
};

const startDrag = (id: string) => {
  closeContextMenu();
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

const clampHorizontalPosition = (x: number, width: number) =>
  Math.max(VIEWPORT_PADDING, Math.min(x, window.innerWidth - width - VIEWPORT_PADDING));

const resolveFloatingMenuHeight = (element: HTMLElement | null, maxHeight?: number) => {
  const viewportLimit = window.innerHeight - VIEWPORT_PADDING * 2;
  const naturalHeight = element ? Math.ceil(element.scrollHeight) : 0;
  const cappedByMenu = typeof maxHeight === 'number' ? Math.min(naturalHeight || maxHeight, maxHeight) : naturalHeight;
  return Math.min(cappedByMenu || 0, viewportLimit);
};

const buildFloatingMenuStyle = (
  x: number,
  y: number,
  width: number,
  element: HTMLElement | null,
  options?: {
    maxHeight?: number;
    alignBottomToRect?: MenuRect | null;
  },
) => {
  const left = clampHorizontalPosition(x, width);
  const maxHeight = Math.min(options?.maxHeight ?? Number.POSITIVE_INFINITY, window.innerHeight - VIEWPORT_PADDING * 2);
  const menuHeight = resolveFloatingMenuHeight(element, Number.isFinite(maxHeight) ? maxHeight : undefined);
  let top = y;

  if (menuHeight > 0 && top + menuHeight > window.innerHeight - VIEWPORT_PADDING) {
    top = options?.alignBottomToRect ? options.alignBottomToRect.bottom - menuHeight : window.innerHeight - VIEWPORT_PADDING - menuHeight;
  }

  top = Math.max(VIEWPORT_PADDING, top);

  return {
    left: `${left}px`,
    top: `${top}px`,
    ...(Number.isFinite(maxHeight) ? { maxHeight: `${maxHeight}px` } : {}),
  };
};

const buildSubmenuPosition = (
  triggerRect: MenuRect | null,
  width: number,
  element: HTMLElement | null,
  alignBottomToRect?: MenuRect | null,
) => {
  if (!triggerRect) {
    return buildFloatingMenuStyle(VIEWPORT_PADDING, VIEWPORT_PADDING, width, element, {
      maxHeight: EXPANDED_MENU_MAX_HEIGHT,
    });
  }

  const prefersRight = triggerRect.right + width + 8 <= window.innerWidth - VIEWPORT_PADDING;
  const x = prefersRight ? triggerRect.right + 8 : triggerRect.left - width - 8;
  return buildFloatingMenuStyle(x, triggerRect.top, width, element, {
    maxHeight: EXPANDED_MENU_MAX_HEIGHT,
    alignBottomToRect,
  });
};

const readElementRect = (element: HTMLElement | null): MenuRect | null => {
  if (!element) {
    return null;
  }

  const rect = element.getBoundingClientRect();
  return {
    top: rect.top,
    left: rect.left,
    right: rect.right,
    bottom: rect.bottom,
  };
};

const readRect = (event: MouseEvent): MenuRect => {
  const rect = (event.currentTarget as HTMLElement).getBoundingClientRect();
  return {
    top: rect.top,
    left: rect.left,
    right: rect.right,
    bottom: rect.bottom,
  };
};

const contextMenuStyle = computed(() =>
  contextMenu.value ? buildFloatingMenuStyle(contextMenu.value.x, contextMenu.value.y, ROOT_MENU_WIDTH, contextMenuRoot.value) : {},
);
const rootContextMenuRect = computed(() => readElementRect(contextMenuRoot.value));
const targetMenuRect = computed(() => readElementRect(targetMenuRoot.value));
const targetMenuStyle = computed(() => buildSubmenuPosition(targetAnchor.value, SUBMENU_WIDTH, targetMenuRoot.value, rootContextMenuRect.value));
const actionMenuStyle = computed(() =>
  buildSubmenuPosition(activeLeafAnchor.value, ACTION_MENU_WIDTH, actionMenuRoot.value, targetMenuRect.value),
);

const selectItem = (id: string) => {
  closeContextMenu();
  emit('select', id);
};

const handleItemContextMenu = (event: MouseEvent, itemId: string) => {
  const allowNativeContextMenu = import.meta.env.DEV && event.shiftKey;
  if (allowNativeContextMenu) {
    closeContextMenu();
    return;
  }

  event.preventDefault();
  closeContextMenu();
  contextMenu.value = {
    itemId,
    x: event.clientX,
    y: event.clientY,
  };
};

const openTargetMenu = (event: MouseEvent) => {
  targetMenuOpen.value = true;
  targetAnchor.value = readRect(event);
  activeTargetId.value = null;
  activeLeafAnchor.value = null;
};

const setActiveTarget = (itemId: string, event: MouseEvent) => {
  activeTargetId.value = itemId;
  activeLeafAnchor.value = readRect(event);
};

const emitMove = (kind: EditorCollectionMoveAction['kind'], position: 'top' | 'bottom', itemId?: string) => {
  if (!currentContextItem.value) {
    closeContextMenu();
    return;
  }

  if (kind === 'current') {
    emit('move-item', currentContextItem.value.id, { kind, position });
  } else if (itemId) {
    emit('move-item', currentContextItem.value.id, { kind, itemId, position });
  }

  closeContextMenu();
};

const handleDocumentClick = (event: MouseEvent) => {
  const target = event.target as Node;
  if (
    contextMenuRoot.value?.contains(target) ||
    targetMenuRoot.value?.contains(target) ||
    actionMenuRoot.value?.contains(target)
  ) {
    return;
  }

  closeContextMenu();
};

const handleWindowScroll = (event: Event) => {
  const target = event.target as Node | null;
  if (
    (target && contextMenuRoot.value?.contains(target)) ||
    (target && targetMenuRoot.value?.contains(target)) ||
    (target && actionMenuRoot.value?.contains(target))
  ) {
    return;
  }

  closeContextMenu();
};

const handleWindowKeydown = (event: KeyboardEvent) => {
  if (event.key === 'Escape') {
    closeContextMenu();
    resetDrag();
  }
};

watch(
  () => props.collapsed,
  (nextCollapsed) => {
    if (nextCollapsed) {
      closeContextMenu();
    }
  },
);

onMounted(() => {
  window.addEventListener('mouseup', resetDrag);
  window.addEventListener('resize', closeContextMenu);
  window.addEventListener('scroll', handleWindowScroll, true);
  window.addEventListener('keydown', handleWindowKeydown);
  document.addEventListener('click', handleDocumentClick);
});

onBeforeUnmount(() => {
  window.removeEventListener('mouseup', resetDrag);
  window.removeEventListener('resize', closeContextMenu);
  window.removeEventListener('scroll', handleWindowScroll, true);
  window.removeEventListener('keydown', handleWindowKeydown);
  document.removeEventListener('click', handleDocumentClick);
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

.editor-collection-menu {
  width: 220px;
  min-width: 220px;
  max-width: 240px;
  padding: 0.35rem;
  overscroll-behavior: contain;
}

.editor-collection-menu-item {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  border: 1px solid transparent;
  border-radius: 12px;
  padding: 0.72rem 0.8rem;
  text-align: left;
  color: var(--app-text);
  transition: background 0.14s ease, border-color 0.14s ease, color 0.14s ease;
}

.editor-collection-menu-item:hover,
.editor-collection-menu-item-branch {
  cursor: pointer;
}

.editor-collection-menu-item:hover {
  background: var(--app-state-hover-bg);
  border-color: var(--app-state-hover-border);
}

</style>
