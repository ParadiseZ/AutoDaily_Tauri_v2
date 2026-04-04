<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <div class="space-y-4">
      <slot name="mode-switch" />

      <div class="space-y-1">
        <p class="text-xs uppercase tracking-[0.18em] text-[var(--app-text-faint)]">Task Mode</p>
        <h2 class="text-xl font-semibold text-[var(--app-text-strong)]">任务列表</h2>
      </div>

      <div class="grid grid-cols-[minmax(0,1fr)_44px] items-center gap-2">
        <input
          v-model="search"
          class="app-input"
          type="search"
          placeholder="按名称检索任务"
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

      <div class="grid grid-cols-[auto_1fr_auto_1fr] items-center gap-x-3 gap-y-2 rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-3">
        <span class="text-xs uppercase tracking-[0.12em] text-[var(--app-text-faint)]">任务</span>
        <span class="text-xl font-semibold text-[var(--app-text-strong)]">{{ tasks.length }}</span>
        <span class="text-xs uppercase tracking-[0.12em] text-[var(--app-text-faint)]">隐藏</span>
        <span class="text-xl font-semibold text-[var(--app-text-strong)]">{{ hiddenCount }}</span>
      </div>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto custom-scrollbar">
      <div v-if="filteredTasks.length" class="space-y-2 pr-1">
        <article
          v-for="task in filteredTasks"
          :key="task.id"
          class="app-list-item space-y-3 transition-colors"
          :class="{
            'app-list-item-active': selectedTaskId === task.id,
            'editor-task-drop-target': overTaskId === task.id && draggingTaskId !== null && draggingTaskId !== task.id,
            'editor-task-card-dragging': draggingTaskId === task.id,
          }"
          :data-testid="`editor-task-item-${task.id}`"
          @mouseenter="handleMouseEnter(task.id)"
          @mouseup="handleMouseUp(task.id)"
        >
          <div class="grid grid-cols-[34px_minmax(0,1fr)_auto] items-start gap-2">
            <button
              class="editor-task-card-handle"
              :class="{ 'editor-task-card-handle-active': draggingTaskId === task.id }"
              :data-testid="`editor-task-drag-${task.id}`"
              type="button"
              aria-label="拖动排序"
              @mousedown.prevent="startDrag(task.id)"
              @click.stop
            >
              <GripVertical class="h-4 w-4" />
            </button>

            <button
              class="min-w-0 flex-1 text-left"
              type="button"
              :style="task.rowType === 'title' ? undefined : { paddingLeft: `${task.indentLevel * 0.85}rem` }"
              @click="$emit('select', task.id)"
            >
              <div class="flex items-center gap-2">
                <span class="rounded-full border border-[var(--app-border)] px-2 py-1 text-[11px] text-[var(--app-text-faint)]">
                  {{ task.index + 1 }}
                </span>
                <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">
                  {{ task.name }}
                </p>
              </div>
              <p class="mt-2 text-xs text-[var(--app-text-faint)]">
                <template v-if="task.rowType === 'title'">标题行 · 分组标题</template>
                <template v-else>{{ formatTriggerModeLabel(task.triggerMode) }} · {{ task.data.steps.length }} 个步骤</template>
              </p>
            </button>

            <div class="flex flex-col items-end gap-2">
              <span
                class="rounded-full px-2 py-1 text-[11px] font-medium"
                :class="
                  task.isHidden
                    ? 'bg-amber-500/12 text-amber-700'
                    : 'bg-emerald-500/10 text-emerald-700'
                "
              >
                {{ task.isHidden ? '已隐藏' : '可见' }}
              </span>
            </div>
          </div>

          <div class="flex flex-wrap gap-2">
            <button class="app-button app-button-ghost app-toolbar-button" type="button" aria-label="复制任务" title="复制任务" @click.stop="$emit('duplicate', task.id)">
              <Copy class="h-4 w-4" />
            </button>
            <button
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              :aria-label="task.isHidden ? '显示任务' : '隐藏任务'"
              :title="task.isHidden ? '显示任务' : '隐藏任务'"
              @click.stop="$emit('toggle-hidden', task.id)"
            >
              <Eye v-if="task.isHidden" class="h-4 w-4" />
              <EyeOff v-else class="h-4 w-4" />
            </button>
            <button
              class="app-button app-button-danger app-toolbar-button"
              type="button"
              :disabled="tasks.length <= 1"
              aria-label="删除任务"
              title="删除任务"
              @click.stop="$emit('remove', task.id)"
            >
              <Trash2 class="h-4 w-4" />
            </button>
          </div>
        </article>
      </div>

      <EmptyState
        v-else
        title="没有可显示的任务"
        description="可以直接新建空白任务，或者调整搜索词查看已有内容。"
      />
    </div>
  </SurfacePanel>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { Copy, Eye, EyeOff, GripVertical, Plus, Trash2 } from 'lucide-vue-next';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import { formatTaskTriggerModeLabel } from '@/utils/presenters';

const props = defineProps<{
  tasks: ScriptTaskTable[];
  selectedTaskId: string | null;
}>();

const emit = defineEmits<{
  create: [];
  select: [taskId: string];
  duplicate: [taskId: string];
  'toggle-hidden': [taskId: string];
  remove: [taskId: string];
  reorder: [draggedTaskId: string, targetTaskId: string];
}>();

const search = ref('');
const draggingTaskId = ref<string | null>(null);
const overTaskId = ref<string | null>(null);

const filteredTasks = computed(() => {
  const keyword = search.value.trim().toLowerCase();
  if (!keyword) {
    return props.tasks;
  }

  return props.tasks.filter((task) => task.name.toLowerCase().includes(keyword));
});

const hiddenCount = computed(() => props.tasks.filter((task) => task.isHidden).length);
const formatTriggerModeLabel = (value: ScriptTaskTable['triggerMode']) => formatTaskTriggerModeLabel(value);

const resetDrag = () => {
  draggingTaskId.value = null;
  overTaskId.value = null;
};

const startDrag = (taskId: string) => {
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

const handleWindowMouseUp = () => {
  resetDrag();
};

onMounted(() => {
  window.addEventListener('mouseup', handleWindowMouseUp);
});

onBeforeUnmount(() => {
  window.removeEventListener('mouseup', handleWindowMouseUp);
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

.editor-task-card-handle {
  display: inline-flex;
  min-height: 32px;
  align-items: center;
  justify-content: center;
  align-self: center;
  border-radius: 12px;
  border: 1px dashed var(--app-border);
  background: rgba(255, 255, 255, 0.55);
  color: var(--app-text-faint);
  cursor: grab;
}

.editor-task-card-handle-active {
  border-color: var(--app-state-active-border);
  background: var(--app-state-active-bg);
  color: var(--app-text-strong);
  cursor: grabbing;
}
</style>
