<template>
  <div class="rounded-[22px] border border-[var(--app-border)] bg-[linear-gradient(180deg,rgba(255,255,255,0.92),rgba(245,247,252,0.88))] px-5 py-5 shadow-[var(--app-shadow-soft)]">
    <div class="flex items-center justify-between gap-3">
      <div>
        <p class="text-xs uppercase tracking-[0.18em] text-[var(--app-text-faint)]">Script Preview</p>
        <h3 class="text-lg font-semibold text-[var(--app-text-strong)]">整表任务预览</h3>
      </div>
      <span class="rounded-full border border-[var(--app-border)] bg-white/60 px-3 py-1 text-xs text-[var(--app-text-faint)]">
        {{ taskCount }} 个任务行
      </span>
    </div>

    <div class="mt-4 min-h-0 space-y-4 overflow-y-auto pr-1 custom-scrollbar">
      <div v-if="ungroupedTasks.length && titleRows.length" class="space-y-2">
        <p class="text-[11px] uppercase tracking-[0.14em] text-[var(--app-text-faint)]">未分组任务</p>
        <button
          v-for="task in ungroupedTasks"
          :key="task.id"
          type="button"
          class="preview-task-row"
          :class="{ 'preview-task-row-active': selectedTaskId === task.id }"
          :style="{ paddingLeft: `${1.2 + task.indentLevel * 1.1}rem` }"
          @click="$emit('select-task', task.id)"
        >
          <span class="preview-task-tone" :class="toneClass(task.taskTone)" />
          <label v-if="task.showEnabledToggle" class="preview-task-toggle" @click.stop>
            <input type="checkbox" :checked="resolveEnabled(task.id, task.defaultEnabled)" @change="setEnabled(task.id, ($event.target as HTMLInputElement).checked)" />
          </label>
          <span class="preview-task-name" :class="nameClass(task.taskTone)">{{ task.name }}</span>
          <span class="preview-task-cycle">{{ formatTaskCycleLabel(task.defaultTaskCycle) }}</span>
          <span class="preview-task-trigger">{{ formatTaskTriggerModeLabel(task.triggerMode) }}</span>
          <span v-if="previewFieldSummary(task).length" class="preview-task-fields">{{ previewFieldSummary(task) }}</span>
        </button>
      </div>

      <div v-for="title in titleRows" :key="title.id" class="space-y-2">
        <button
          type="button"
          class="preview-title-row"
          :class="{ 'preview-title-row-active': selectedTaskId === title.id }"
          @click="$emit('select-task', title.id)"
        >
          <span class="preview-title-dot" />
          <span>{{ title.name }}</span>
        </button>

        <div v-if="groupedTasksByTitle[title.id]?.length" class="space-y-2">
          <button
            v-for="task in groupedTasksByTitle[title.id]"
            :key="task.id"
            type="button"
            class="preview-task-row"
            :class="{ 'preview-task-row-active': selectedTaskId === task.id }"
            :style="{ paddingLeft: `${1.2 + task.indentLevel * 1.1}rem` }"
            @click="$emit('select-task', task.id)"
          >
            <span class="preview-task-tone" :class="toneClass(task.taskTone)" />
            <label v-if="task.showEnabledToggle" class="preview-task-toggle" @click.stop>
              <input type="checkbox" :checked="resolveEnabled(task.id, task.defaultEnabled)" @change="setEnabled(task.id, ($event.target as HTMLInputElement).checked)" />
            </label>
            <span class="preview-task-name" :class="nameClass(task.taskTone)">{{ task.name }}</span>
            <span class="preview-task-cycle">{{ formatTaskCycleLabel(task.defaultTaskCycle) }}</span>
            <span class="preview-task-trigger">{{ formatTaskTriggerModeLabel(task.triggerMode) }}</span>
            <span v-if="previewFieldSummary(task).length" class="preview-task-fields">{{ previewFieldSummary(task) }}</span>
          </button>
        </div>

        <div v-else class="rounded-[16px] border border-dashed border-[var(--app-border)] px-4 py-3 text-sm text-[var(--app-text-soft)]">
          当前标题下还没有任务。
        </div>
      </div>

      <div v-if="!titleRows.length && !ungroupedTasks.length" class="rounded-[16px] border border-dashed border-[var(--app-border)] px-4 py-4 text-sm text-[var(--app-text-soft)]">
        还没有任务可预览。
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import { formatTaskCycleLabel, formatTaskTriggerModeLabel } from '@/utils/presenters';
import { parseUiSchema } from '@/views/script-editor/editorSchema';

defineOptions({ name: 'EditorTaskTablePreview' });

const props = defineProps<{
  tasks: ScriptTaskTable[];
  selectedTaskId: string | null;
}>();

defineEmits<{
  'select-task': [taskId: string];
}>();

const enabledState = ref<Record<string, boolean>>({});

const sortedTasks = computed(() => [...props.tasks].sort((left, right) => left.index - right.index));
const titleRows = computed(() => sortedTasks.value.filter((task) => task.rowType === 'title'));
const groupedTasksByTitle = computed<Record<string, ScriptTaskTable[]>>(() =>
  Object.fromEntries(
    titleRows.value.map((title) => [
      title.id,
      sortedTasks.value.filter((task) => task.rowType === 'task' && task.sectionId === title.id),
    ]),
  ),
);
const groupedTaskIds = computed(() => new Set(Object.values(groupedTasksByTitle.value).flat().map((task) => task.id)));
const ungroupedTasks = computed(() =>
  sortedTasks.value.filter((task) => task.rowType === 'task' && !groupedTaskIds.value.has(task.id)),
);
const taskCount = computed(() => props.tasks.length);

const resolveEnabled = (taskId: string, fallback: boolean) => enabledState.value[taskId] ?? fallback;

const setEnabled = (taskId: string, value: boolean) => {
  enabledState.value = {
    ...enabledState.value,
    [taskId]: value,
  };
};

const previewFieldSummary = (task: ScriptTaskTable) =>
  parseUiSchema(task.data.uiData ?? {})
    .fields
    .map((field) => field.label || field.inputKey || field.key)
    .filter(Boolean)
    .slice(0, 3)
    .join(' / ');

const toneClass = (tone: ScriptTaskTable['taskTone']) => {
  if (tone === 'warning') return 'preview-task-tone-warning';
  if (tone === 'danger') return 'preview-task-tone-danger';
  return 'preview-task-tone-normal';
};

const nameClass = (tone: ScriptTaskTable['taskTone']) => {
  if (tone === 'warning') return 'preview-task-name-warning';
  if (tone === 'danger') return 'preview-task-name-danger';
  return '';
};

watch(
  () => props.tasks.map((task) => `${task.id}:${task.defaultEnabled}`).join('|'),
  () => {
    enabledState.value = Object.fromEntries(props.tasks.map((task) => [task.id, task.defaultEnabled]));
  },
  { immediate: true },
);
</script>

<style scoped>
.preview-title-row,
.preview-task-row {
  display: flex;
  width: 100%;
  align-items: center;
  gap: 0.65rem;
  border-radius: 16px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.72);
  padding: 0.85rem 1rem;
  text-align: left;
  transition: border-color 0.16s ease, background 0.16s ease, box-shadow 0.16s ease;
}

.preview-title-row-active,
.preview-task-row-active {
  border-color: color-mix(in srgb, var(--app-accent) 34%, var(--app-border));
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--app-accent) 18%, transparent);
}

.preview-title-row {
  background: linear-gradient(135deg, rgba(244, 247, 255, 0.96), rgba(250, 250, 252, 0.9));
  font-weight: 700;
  color: var(--app-text-strong);
}

.preview-title-dot {
  width: 0.7rem;
  height: 0.7rem;
  border-radius: 999px;
  background: color-mix(in srgb, var(--app-accent) 78%, white);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--app-accent) 14%, transparent);
}

.preview-task-tone {
  width: 0.28rem;
  align-self: stretch;
  border-radius: 999px;
}

.preview-task-tone-normal {
  background: rgba(148, 163, 184, 0.42);
}

.preview-task-tone-warning {
  background: rgba(245, 158, 11, 0.9);
}

.preview-task-tone-danger {
  background: rgba(239, 68, 68, 0.92);
}

.preview-task-toggle input {
  width: 1rem;
  height: 1rem;
  accent-color: var(--app-accent);
}

.preview-task-name {
  min-width: 0;
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--app-text-strong);
}

.preview-task-name-warning {
  color: #a16207;
}

.preview-task-name-danger {
  color: #b91c1c;
}

.preview-task-cycle,
.preview-task-trigger,
.preview-task-fields {
  border-radius: 999px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.84);
  padding: 0.22rem 0.55rem;
  font-size: 0.72rem;
  color: var(--app-text-faint);
}

.preview-task-fields {
  min-width: 0;
  max-width: min(40%, 420px);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
