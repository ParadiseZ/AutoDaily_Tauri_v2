<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <div class="space-y-4">
      <slot name="mode-switch" />
      <div v-if="!collapsed" class="grid grid-cols-[minmax(0,1fr)_44px] items-center gap-2">
        <input
          v-model="search"
          class="app-input"
          type="search"
          placeholder="名称或说明检索"
          data-testid="editor-task-search"
        />
        <button
          class="app-button app-button-primary app-toolbar-button justify-center"
          type="button"
          data-testid="editor-task-create"
          aria-label="新建任务"
          @click="$emit('create')"
        >
          <Plus class="h-4 w-4" />
        </button>
      </div>

      <div
        v-if="!collapsed"
        class="grid grid-cols-[auto_1fr_auto_1fr] items-center gap-x-3 gap-y-2 rounded-[18px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3"
      >
        <span class="text-xs uppercase tracking-[0.12em] text-(--app-text-faint)">任务</span>
        <span class="text-xl font-semibold text-(--app-text-strong)">{{ tasks.length }}</span>
        <span class="text-xs uppercase tracking-[0.12em] text-(--app-text-faint)">隐藏</span>
        <span class="text-xl font-semibold text-(--app-text-strong)">{{ hiddenCount }}</span>
      </div>
    </div>

    <div
      v-if="!collapsed"
      ref="scrollRoot"
      class="min-h-0 flex-1 overflow-y-auto custom-scrollbar"
      data-testid="editor-task-sidebar-scroll"
      @scroll="closeContextMenu"
    >
      <div v-if="hasVisibleRows" class="space-y-3 pr-1">
        <div v-if="ungroupedTasks.length" class="space-y-2">
          <EditorTaskSidebarTaskCard
            v-for="task in ungroupedTasks"
            :key="task.id"
            :task="task"
            :selected="selectedTaskId === task.id"
            :drop-target="overTaskId === task.id && draggingTaskId !== null && draggingTaskId !== task.id"
            :dragging="draggingTaskId === task.id"
            @mouseenter="handleMouseEnter"
            @mouseup="handleMouseUp"
            @contextmenu="handleTaskContextMenu"
            @drag-start="startDrag"
            @select="selectTask"
          />
        </div>

        <section v-for="title in visibleTitleRows" :key="title.id" class="space-y-2">
          <EditorTaskSidebarTitleCard
            :title="title"
            :selected="selectedTaskId === title.id"
            :dragging="draggingTaskId === title.id"
            :drop-target="overTaskId === title.id && draggingTaskId !== null && draggingTaskId !== title.id"
            :expanded="isTitleExpanded(title.id)"
            :task-count="groupedTasksByTitle[title.id]?.length ?? 0"
            @mouseenter="handleMouseEnter"
            @mouseup="handleMouseUp"
            @contextmenu="handleTaskContextMenu"
            @drag-start="startDrag"
            @select="selectTask"
            @toggle-collapsed="toggleTitleCollapsed"
          />

          <div v-if="isTitleExpanded(title.id)" class="space-y-2">
            <EditorTaskSidebarTaskCard
              v-for="task in groupedTasksByTitle[title.id] ?? []"
              :key="task.id"
              :task="task"
              :selected="selectedTaskId === task.id"
              :drop-target="overTaskId === task.id && draggingTaskId !== null && draggingTaskId !== task.id"
              :dragging="draggingTaskId === task.id"
              @mouseenter="handleMouseEnter"
              @mouseup="handleMouseUp"
              @contextmenu="handleTaskContextMenu"
              @drag-start="startDrag"
              @select="selectTask"
            />
          </div>
        </section>
      </div>

      <EmptyState
        v-else
        title="没有可显示的任务"
        description="可以直接新建空白任务，或者调整搜索词查看已有内容。"
      />
    </div>
  </SurfacePanel>

  <Teleport to="body">
    <div
      v-if="contextMenu"
      ref="contextMenuRoot"
      class="editor-task-menu app-select-menu app-select-menu-floating"
      data-testid="editor-task-context-menu"
      :style="contextMenuStyle"
    >
      <button
        class="editor-task-menu-item"
        data-testid="editor-task-duplicate"
        type="button"
        @click="emitDuplicate"
      >
        复制
      </button>
      <button
        class="editor-task-menu-item"
        data-testid="editor-task-remove"
        type="button"
        :disabled="!canRemoveTask"
        @click="emitRemove"
      >
        删除
      </button>
      <button
        class="editor-task-menu-item"
        data-testid="editor-task-toggle-hidden"
        type="button"
        @click="emitToggleHidden"
      >
        {{ currentContextRow?.isHidden ? '在 UI 上显示' : '在 UI 上隐藏' }}
      </button>
      <button
        class="editor-task-menu-item"
        data-testid="editor-task-move-current-top"
        type="button"
        @click="emitMove('current', 'top')"
      >
        移动到顶部
      </button>
      <button
        class="editor-task-menu-item"
        data-testid="editor-task-move-current-bottom"
        type="button"
        @click="emitMove('current', 'bottom')"
      >
        移动到底部
      </button>
      <button
        v-if="currentContextRow?.rowType === 'task'"
        class="editor-task-menu-item editor-task-menu-item-branch"
        data-testid="editor-task-move-section"
        type="button"
        @mouseenter="openBranchMenu('section', $event)"
      >
        <span>移动到分组</span>
        <ChevronRight class="h-4 w-4" />
      </button>
      <button
        v-if="currentContextRow?.rowType === 'task'"
        class="editor-task-menu-item editor-task-menu-item-branch"
        data-testid="editor-task-move-task"
        type="button"
        @mouseenter="openBranchMenu('task', $event)"
      >
        <span>移动到任务</span>
        <ChevronRight class="h-4 w-4" />
      </button>
    </div>

    <div
      v-if="contextMenu && activeBranchMenu === 'section'"
      ref="sectionMenuRoot"
      class="editor-task-menu app-select-menu app-select-menu-floating"
      data-testid="editor-task-context-section-menu"
      :style="sectionMenuStyle"
    >
      <button
        v-for="title in sectionMenuTargets"
        :key="title.id"
        class="editor-task-menu-item editor-task-menu-item-branch"
        :data-testid="`editor-task-move-section-item-${title.id}`"
        type="button"
        @mouseenter="setActiveSectionTarget(title.id, $event)"
      >
        <span class="truncate">{{ title.name }}</span>
        <ChevronRight class="h-4 w-4" />
      </button>
      <div v-if="!sectionMenuTargets.length" class="app-select-empty">没有匹配分组</div>
    </div>

    <div
      v-if="contextMenu && activeBranchMenu === 'task'"
      ref="taskMenuRoot"
      class="editor-task-menu app-select-menu app-select-menu-floating"
      data-testid="editor-task-context-task-menu"
      :style="taskMenuStyle"
    >
      <button
        v-for="task in taskMenuTargets"
        :key="task.id"
        class="editor-task-menu-item editor-task-menu-item-branch"
        :data-testid="`editor-task-move-task-item-${task.id}`"
        type="button"
        @mouseenter="setActiveTaskTarget(task.id, $event)"
      >
        <span class="truncate">{{ task.name }}</span>
        <ChevronRight class="h-4 w-4" />
      </button>
      <div v-if="!taskMenuTargets.length" class="app-select-empty">没有匹配任务</div>
    </div>

    <div
      v-if="contextMenu && activeBranchMenu === 'section' && activeSectionTarget"
      ref="actionMenuRoot"
      class="editor-task-menu app-select-menu app-select-menu-floating"
      data-testid="editor-task-context-section-action-menu"
      :style="sectionActionMenuStyle"
    >
      <button
        class="editor-task-menu-item"
        :data-testid="`editor-task-move-section-${activeSectionTarget.id}-top`"
        type="button"
        @click="emitMove('section', 'top', activeSectionTarget.id)"
      >
        顶部
      </button>
      <button
        class="editor-task-menu-item"
        :data-testid="`editor-task-move-section-${activeSectionTarget.id}-bottom`"
        type="button"
        @click="emitMove('section', 'bottom', activeSectionTarget.id)"
      >
        底部
      </button>
    </div>

    <div
      v-if="contextMenu && activeBranchMenu === 'task' && activeTaskTarget"
      ref="actionMenuRoot"
      class="editor-task-menu app-select-menu app-select-menu-floating"
      data-testid="editor-task-context-task-action-menu"
      :style="taskActionMenuStyle"
    >
      <button
        class="editor-task-menu-item"
        :data-testid="`editor-task-move-task-${activeTaskTarget.id}-top`"
        type="button"
        @click="emitMove('task', 'top', activeTaskTarget.id)"
      >
        顶部
      </button>
      <button
        class="editor-task-menu-item"
        :data-testid="`editor-task-move-task-${activeTaskTarget.id}-bottom`"
        type="button"
        @click="emitMove('task', 'bottom', activeTaskTarget.id)"
      >
        底部
      </button>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { ChevronRight, Plus } from 'lucide-vue-next';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import EditorTaskSidebarTaskCard from '@/views/script-editor/EditorTaskSidebarTaskCard.vue';
import EditorTaskSidebarTitleCard from '@/views/script-editor/EditorTaskSidebarTitleCard.vue';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { EditorTaskMoveAction } from '@/views/script-editor/editorTaskMove';

type BranchMenuKind = 'section' | 'task' | null;
type MenuRect = {
  top: number;
  left: number;
  right: number;
  bottom: number;
};

const props = defineProps<{
  tasks: ScriptTaskTable[];
  selectedTaskId: string | null;
  collapsed?: boolean;
}>();

const emit = defineEmits<{
  create: [];
  select: [taskId: string];
  duplicate: [taskId: string];
  'toggle-hidden': [taskId: string];
  remove: [taskId: string];
  reorder: [draggedTaskId: string, targetTaskId: string];
  'move-task': [taskId: string, action: EditorTaskMoveAction];
}>();

const ROOT_MENU_WIDTH = 220;
const SUBMENU_WIDTH = 240;
const ACTION_MENU_WIDTH = 144;
const VIEWPORT_PADDING = 12;
const EXPANDED_MENU_MAX_HEIGHT = 320;

const search = ref('');
const draggingTaskId = ref<string | null>(null);
const overTaskId = ref<string | null>(null);
const collapsedTitleIds = ref<string[]>([]);
const contextMenu = ref<{ taskId: string; x: number; y: number } | null>(null);
const activeBranchMenu = ref<BranchMenuKind>(null);
const branchAnchor = ref<MenuRect | null>(null);
const activeSectionTargetId = ref<string | null>(null);
const activeTaskTargetId = ref<string | null>(null);
const activeLeafAnchor = ref<MenuRect | null>(null);
const contextMenuRoot = ref<HTMLElement | null>(null);
const sectionMenuRoot = ref<HTMLElement | null>(null);
const taskMenuRoot = ref<HTMLElement | null>(null);
const actionMenuRoot = ref<HTMLElement | null>(null);
const scrollRoot = ref<HTMLElement | null>(null);

const keyword = computed(() => search.value.trim().toLowerCase());
const sortedTasks = computed(() => [...props.tasks].sort((left, right) => left.index - right.index));
const titleRows = computed(() => sortedTasks.value.filter((task) => task.rowType === 'title'));
const titleIdSet = computed(() => new Set(titleRows.value.map((task) => task.id)));
const taskRows = computed(() => sortedTasks.value.filter((task) => task.rowType === 'task'));

const matchesKeyword = (task: ScriptTaskTable) =>
  !keyword.value || `${task.name || ''} ${task.description || ''}`.toLowerCase().includes(keyword.value);

const groupedTasksByTitle = computed<Record<string, ScriptTaskTable[]>>(() =>
  Object.fromEntries(
    titleRows.value.map((title) => [
      title.id,
      taskRows.value.filter((task) => task.sectionId === title.id && matchesKeyword(task)),
    ]),
  ),
);

const visibleTitleRows = computed(() =>
  titleRows.value.filter((title) => matchesKeyword(title) || (groupedTasksByTitle.value[title.id]?.length ?? 0) > 0),
);

const ungroupedTasks = computed(() =>
  taskRows.value.filter((task) => (!task.sectionId || !titleIdSet.value.has(task.sectionId)) && matchesKeyword(task)),
);

const hasVisibleRows = computed(() => ungroupedTasks.value.length > 0 || visibleTitleRows.value.length > 0);
const hiddenCount = computed(() => props.tasks.filter((task) => task.isHidden).length);
const currentContextRow = computed(() => sortedTasks.value.find((task) => task.id === contextMenu.value?.taskId) ?? null);
const currentContextTask = computed(() => taskRows.value.find((task) => task.id === contextMenu.value?.taskId) ?? null);
const canRemoveTask = computed(() => props.tasks.length > 1);
const sectionMenuTargets = computed(() =>
  titleRows.value.filter((title) => !keyword.value || title.name.toLowerCase().includes(keyword.value)),
);
const taskMenuTargets = computed(() =>
  taskRows.value.filter((task) => task.id !== contextMenu.value?.taskId && matchesKeyword(task)),
);
const activeSectionTarget = computed(() => titleRows.value.find((title) => title.id === activeSectionTargetId.value) ?? null);
const activeTaskTarget = computed(() => taskRows.value.find((task) => task.id === activeTaskTargetId.value) ?? null);
const visibleRowSignature = computed(() =>
  [
    ...ungroupedTasks.value.map((task) => task.id),
    ...visibleTitleRows.value.flatMap((title) => [
      title.id,
      ...(isTitleExpanded(title.id) ? (groupedTasksByTitle.value[title.id] ?? []).map((task) => task.id) : []),
    ]),
  ].join('|'),
);
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
    if (options?.alignBottomToRect) {
      top = options.alignBottomToRect.bottom - menuHeight;
    } else {
      top = window.innerHeight - VIEWPORT_PADDING - menuHeight;
    }
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

const contextMenuStyle = computed(() =>
  contextMenu.value ? buildFloatingMenuStyle(contextMenu.value.x, contextMenu.value.y, ROOT_MENU_WIDTH, contextMenuRoot.value) : {},
);
const rootContextMenuRect = computed(() => readElementRect(contextMenuRoot.value));
const activeBranchPanelRect = computed(() =>
  activeBranchMenu.value === 'section' ? readElementRect(sectionMenuRoot.value) : readElementRect(taskMenuRoot.value),
);
const sectionMenuStyle = computed(() => buildSubmenuPosition(branchAnchor.value, SUBMENU_WIDTH, sectionMenuRoot.value, rootContextMenuRect.value));
const taskMenuStyle = computed(() => buildSubmenuPosition(branchAnchor.value, SUBMENU_WIDTH, taskMenuRoot.value, rootContextMenuRect.value));
const sectionActionMenuStyle = computed(() =>
  buildSubmenuPosition(activeLeafAnchor.value, ACTION_MENU_WIDTH, actionMenuRoot.value, activeBranchPanelRect.value),
);
const taskActionMenuStyle = computed(() =>
  buildSubmenuPosition(activeLeafAnchor.value, ACTION_MENU_WIDTH, actionMenuRoot.value, activeBranchPanelRect.value),
);

const isTitleExpanded = (titleId: string) => keyword.value.length > 0 || !collapsedTitleIds.value.includes(titleId);

const selectTask = (taskId: string) => {
  closeContextMenu();
  emit('select', taskId);
};

const emitDuplicate = () => {
  if (!currentContextRow.value) {
    closeContextMenu();
    return;
  }

  emit('duplicate', currentContextRow.value.id);
  closeContextMenu();
};

const emitRemove = () => {
  if (!currentContextRow.value || !canRemoveTask.value) {
    closeContextMenu();
    return;
  }

  emit('remove', currentContextRow.value.id);
  closeContextMenu();
};

const emitToggleHidden = () => {
  if (!currentContextRow.value) {
    closeContextMenu();
    return;
  }

  emit('toggle-hidden', currentContextRow.value.id);
  closeContextMenu();
};

const scrollSelectedTaskIntoView = async () => {
  if (props.collapsed || !props.selectedTaskId) {
    return;
  }

  await nextTick();
  scrollRoot.value
    ?.querySelector<HTMLElement>(`[data-testid="editor-task-item-${props.selectedTaskId}"]`)
    ?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
};

const toggleTitleCollapsed = (titleId: string) => {
  if (keyword.value) {
    return;
  }

  collapsedTitleIds.value = collapsedTitleIds.value.includes(titleId)
    ? collapsedTitleIds.value.filter((id) => id !== titleId)
    : [...collapsedTitleIds.value, titleId];
};

const resetDrag = () => {
  draggingTaskId.value = null;
  overTaskId.value = null;
};

const closeContextMenu = () => {
  contextMenu.value = null;
  activeBranchMenu.value = null;
  branchAnchor.value = null;
  activeSectionTargetId.value = null;
  activeTaskTargetId.value = null;
  activeLeafAnchor.value = null;
};

const startDrag = (taskId: string) => {
  closeContextMenu();
  draggingTaskId.value = taskId;
  overTaskId.value = taskId;
};

const handleMouseEnter = (taskId: string) => {
  if (!draggingTaskId.value) {
    return;
  }
  overTaskId.value = taskId;
};

const handleMouseUp = (targetTaskId: string) => {
  if (!draggingTaskId.value || draggingTaskId.value === targetTaskId) {
    resetDrag();
    return;
  }

  emit('reorder', draggingTaskId.value, targetTaskId);
  resetDrag();
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

const openContextMenu = (event: MouseEvent, taskId: string) => {
  closeContextMenu();
  contextMenu.value = {
    taskId,
    x: event.clientX,
    y: event.clientY,
  };
};

const handleTaskContextMenu = (event: MouseEvent, taskId: string) => {
  const allowNativeContextMenu = import.meta.env.DEV && event.shiftKey;
  if (allowNativeContextMenu) {
    closeContextMenu();
    return;
  }

  event.preventDefault();
  openContextMenu(event, taskId);
};

const openBranchMenu = (kind: Exclude<BranchMenuKind, null>, event: MouseEvent) => {
  activeBranchMenu.value = kind;
  branchAnchor.value = readRect(event);
  activeLeafAnchor.value = null;
  activeSectionTargetId.value = null;
  activeTaskTargetId.value = null;
};

const setActiveSectionTarget = (sectionId: string, event: MouseEvent) => {
  activeSectionTargetId.value = sectionId;
  activeLeafAnchor.value = readRect(event);
};

const setActiveTaskTarget = (taskId: string, event: MouseEvent) => {
  activeTaskTargetId.value = taskId;
  activeLeafAnchor.value = readRect(event);
};

const emitMove = (kind: EditorTaskMoveAction['kind'], position: 'top' | 'bottom', targetId?: string) => {
  if (!currentContextTask.value) {
    closeContextMenu();
    return;
  }

  if (kind === 'current') {
    emit('move-task', currentContextTask.value.id, { kind, position });
  } else if (kind === 'section' && targetId) {
    emit('move-task', currentContextTask.value.id, { kind, sectionId: targetId, position });
  } else if (kind === 'task' && targetId) {
    emit('move-task', currentContextTask.value.id, { kind, taskId: targetId, position });
  }

  closeContextMenu();
};

const handleWindowMouseUp = () => {
  resetDrag();
};

const handleDocumentClick = (event: MouseEvent) => {
  const target = event.target as Node;
  if (
    contextMenuRoot.value?.contains(target) ||
    sectionMenuRoot.value?.contains(target) ||
    taskMenuRoot.value?.contains(target) ||
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
    (target && sectionMenuRoot.value?.contains(target)) ||
    (target && taskMenuRoot.value?.contains(target)) ||
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

watch(titleRows, (titles) => {
  const nextIds = new Set(titles.map((task) => task.id));
  collapsedTitleIds.value = collapsedTitleIds.value.filter((id) => nextIds.has(id));
}, { immediate: true });

watch(
  [() => props.selectedTaskId, () => props.collapsed, visibleRowSignature],
  () => {
    void scrollSelectedTaskIntoView();
  },
  { immediate: true },
);

watch(
  () => props.collapsed,
  (nextCollapsed) => {
    if (nextCollapsed) {
      closeContextMenu();
    }
  },
);

onMounted(() => {
  window.addEventListener('mouseup', handleWindowMouseUp);
  window.addEventListener('resize', closeContextMenu);
  window.addEventListener('scroll', handleWindowScroll, true);
  window.addEventListener('keydown', handleWindowKeydown);
  document.addEventListener('click', handleDocumentClick);
});

onBeforeUnmount(() => {
  window.removeEventListener('mouseup', handleWindowMouseUp);
  window.removeEventListener('resize', closeContextMenu);
  window.removeEventListener('scroll', handleWindowScroll, true);
  window.removeEventListener('keydown', handleWindowKeydown);
  document.removeEventListener('click', handleDocumentClick);
});
</script>

<style scoped>
.editor-task-drop-target {
  box-shadow: inset 0 0 0 1px rgba(70, 110, 255, 0.22);
  background: color-mix(in srgb, var(--app-state-active-bg) 84%, white);
}

.editor-task-card-dragging {
  border-color: rgba(70, 110, 255, 0.24);
  background: rgba(70, 110, 255, 0.08);
}

.editor-task-card-hidden {
  color: var(--app-text-faint);
  /* background: color-mix(in srgb, var(--app-panel-muted) 40%, #A9A9A9 18%); */
}
.editor-task-card-expanded{
  /* background: color-mix(in srgb, var(--app-panel-muted) 40%, #A9A9A9 18%); */
  background: var(--app-accent);
  color: white;
}


.editor-task-collapse-trigger {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 10px;
  color: var(--app-text-faint);
  transition: background 0.14s ease, color 0.14s ease;
}

.editor-task-collapse-trigger:hover {
  background: var(--app-state-hover-bg);
  color: var(--app-text-strong);
}

.editor-task-menu {
  width: 220px;
  min-width: 220px;
  max-width: 240px;
  padding: 0.35rem;
  overscroll-behavior: contain;
}

.editor-task-menu-item {
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

.editor-task-menu-item:hover,
.editor-task-menu-item-branch {
  cursor: pointer;
}

.editor-task-menu-item:hover {
  background: var(--app-state-hover-bg);
  border-color: var(--app-state-hover-border);
}

.editor-task-menu-item:disabled {
  cursor: not-allowed;
  opacity: 0.48;
}
</style>
