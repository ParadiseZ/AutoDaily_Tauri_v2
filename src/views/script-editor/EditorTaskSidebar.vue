<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <div class="space-y-4">
      <slot name="mode-switch" />

      <div class="flex items-start justify-between gap-3">
        <div class="space-y-1">
          <p class="text-xs uppercase tracking-[0.18em] text-[var(--app-text-faint)]">Task Mode</p>
          <h2 class="text-xl font-semibold text-[var(--app-text-strong)]">任务列表</h2>
        </div>
        <button
          class="app-button app-button-primary app-toolbar-button"
          type="button"
          data-testid="editor-task-create"
          @click="$emit('create')"
        >
          新建任务
        </button>
      </div>

      <div class="grid grid-cols-2 gap-3">
        <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-3">
          <p class="text-xs uppercase tracking-[0.12em] text-[var(--app-text-faint)]">任务数</p>
          <p class="mt-1 text-2xl font-semibold text-[var(--app-text-strong)]">{{ tasks.length }}</p>
        </div>
        <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-3">
          <p class="text-xs uppercase tracking-[0.12em] text-[var(--app-text-faint)]">隐藏</p>
          <p class="mt-1 text-2xl font-semibold text-[var(--app-text-strong)]">{{ hiddenCount }}</p>
        </div>
      </div>

      <label class="space-y-2">
        <span class="text-xs font-medium uppercase tracking-[0.14em] text-[var(--app-text-faint)]">搜索</span>
        <input
          v-model="search"
          class="app-input"
          type="search"
          placeholder="按名称检索任务"
          data-testid="editor-task-search"
        />
      </label>
    </div>

    <div class="min-h-0 flex-1 overflow-y-auto custom-scrollbar">
      <div v-if="filteredTasks.length" class="space-y-2 pr-1">
        <article
          v-for="task in filteredTasks"
          :key="task.id"
          class="app-list-item space-y-3 transition-colors"
          :class="{
            'app-list-item-active': selectedTaskId === task.id,
            'editor-task-drop-target': overTaskId === task.id && draggingTaskId !== task.id,
          }"
          :data-testid="`editor-task-item-${task.id}`"
          draggable="true"
          @dragenter.prevent="overTaskId = task.id"
          @dragover.prevent="handleDragOver($event, task.id)"
          @dragleave="handleDragLeave(task.id)"
          @dragstart="handleDragStart($event, task.id)"
          @dragend="resetDrag"
          @drop.prevent="handleDrop(task.id)"
        >
          <div class="flex items-start justify-between gap-3">
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
              <span class="text-[11px] uppercase tracking-[0.18em] text-[var(--app-text-faint)]">拖动排序</span>
            </div>
          </div>

          <div class="flex flex-wrap gap-2">
            <button class="app-button app-button-ghost app-toolbar-button" type="button" @click.stop="$emit('duplicate', task.id)">
              复制
            </button>
            <button class="app-button app-button-ghost app-toolbar-button" type="button" @click.stop="$emit('toggle-hidden', task.id)">
              {{ task.isHidden ? '显示' : '隐藏' }}
            </button>
            <button
              class="app-button app-button-danger app-toolbar-button"
              type="button"
              :disabled="tasks.length <= 1"
              @click.stop="$emit('remove', task.id)"
            >
              删除
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
import { computed, ref } from 'vue';
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

const handleDragLeave = (taskId: string) => {
  if (overTaskId.value === taskId) {
    overTaskId.value = null;
  }
};

const handleDragStart = (event: DragEvent, taskId: string) => {
  draggingTaskId.value = taskId;
  event.dataTransfer?.setData('text/plain', taskId);
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
  }
};

const handleDragOver = (event: DragEvent, taskId: string) => {
  overTaskId.value = taskId;
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move';
  }
};

const handleDrop = (targetTaskId: string) => {
  if (!draggingTaskId.value || draggingTaskId.value === targetTaskId) {
    resetDrag();
    return;
  }

  emit('reorder', draggingTaskId.value, targetTaskId);
  resetDrag();
};
</script>

<style scoped>
.editor-task-drop-target {
  box-shadow: inset 0 0 0 1px rgba(70, 110, 255, 0.22);
  background: color-mix(in srgb, var(--app-state-active-bg) 84%, white);
}
</style>
