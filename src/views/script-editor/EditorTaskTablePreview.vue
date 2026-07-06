<template>
  <div class="flex h-full min-h-0 flex-col rounded-[22px] py-5 ">
    <div v-if="showHeader" class="flex items-center justify-between gap-3">
      <span class="rounded-full border border-(--app-border) bg-(--app-panel-muted) px-3 py-1 text-xs text-(--app-text-faint)">
        {{ taskCount }} 个任务行
      </span>
    </div>

    <div class="preview-scroll-shell min-h-0 flex-1" :class="showHeader ? 'mt-4' : ''">
      <div class="preview-scroll-body space-y-4 overflow-y-auto custom-scrollbar">
      <div v-if="ungroupedTasks.length" class="space-y-2">
        <button
          type="button"
          class="preview-title-row"
          :class="{
            'preview-title-row-active': selectedTaskId === ungroupedTitileId,
            'preview-title-row-expanded': !isTitleCollapsed(ungroupedTitileId),
          }"
          :data-testid="`editor-task-preview-title-${ungroupedTitileId}`"
          :aria-expanded="!isTitleCollapsed(ungroupedTitileId)"
          @click="handleTitleClick(ungroupedTitileId)"
        >
          <span class="preview-title-row-main">
            <span class="preview-title-dot" />
            <span class="preview-title-label">未分组任务</span>
          </span>
          <ChevronDown
            class="preview-title-chevron"
            :class="{ 'preview-title-chevron-collapsed': isTitleCollapsed(ungroupedTitileId) }"
          />
        </button>

        <div
          class="preview-title-panel"
          :class="{ 'preview-title-panel-collapsed': isTitleCollapsed(ungroupedTitileId) }"
          :data-testid="`editor-task-preview-panel-${ungroupedTitileId}`"
        >
          <div class="preview-title-panel-inner">
            <div class="preview-title-panel-content">
              <template v-for="task in ungroupedTasks" :key="task.id">
                <div
                  class="preview-task-wrap"
                  :class="{
                    'preview-task-wrap-clickable': selectedTaskId !== task.id,
                    'preview-task-wrap-active': selectedTaskId === task.id,
                  }"
                  @click="selectedTaskId !== task.id && $emit('select-task', task.id)"
                >
                  <EditorUiPreviewPanel
                    :task-name="task.name"
                    :task-description="task.description || ''"
                    :default-task-cycle="resolveTaskCycle(task)"
                    :default-task-cycle-value="resolveTaskCycleValue(task)"
                    :default-task-cycle-mode="resolveTaskCycleMode(task)"
                    :default-task-cycle-day="resolveTaskCycleDay(task)"
                    :editable-cycle="editAllTasks || selectedTaskId === task.id"
                    :show-enabled-toggle="task.showEnabledToggle"
                    :default-enabled="resolveTaskEnabled(task)"
                    :task-tone="task.taskTone"
                    :require-bound-input="requireBoundInput"
                    :show-task-cycle="showTaskCycle"
                    :embedded="true"
                    :readonly="!editAllTasks && selectedTaskId !== task.id"
                    :active="selectedTaskId === task.id"
                    :indent-level="task.indentLevel"
                    :ui-schema="selectedTaskId === task.id && !editAllTasks ? selectedTaskUiSchema : parseUiSchema(task.data.uiData ?? {})"
                    :selected-ui-field-id="selectedTaskId === task.id ? selectedUiFieldId : null"
                    :input-entries="selectedTaskId === task.id && !editAllTasks ? selectedTaskInputEntries : sharedInputEntries"
                    @select-ui-field="$emit('select-ui-field', $event)"
                    @update-input="forwardUpdateInput"
                    @update:default-enabled="forwardTaskEnabled(task.id, $event)"
                    @update:default-task-cycle-value="forwardTaskCycleValue(task.id, $event)"
                    @update:default-task-cycle-day="forwardTaskCycleDay(task.id, $event)"
                  />
                </div>
              </template>
            </div>
          </div>
        </div>
      </div>

      <div v-for="title in titleRows" :key="title.id">
        <button
          type="button"
          class="preview-title-row"
          :class="{
            'preview-title-row-active': selectedTaskId === title.id,
            'preview-title-row-expanded': !isTitleCollapsed(title.id),
          }"
          :data-testid="`editor-task-preview-title-${title.id}`"
          :aria-expanded="!isTitleCollapsed(title.id)"
          @click="handleTitleClick(title.id)"
        >
          <span class="preview-title-row-main">
            <span class="preview-title-dot" />
            <span class="preview-title-label">{{ title.name }}</span>
          </span>
          <ChevronDown
            class="preview-title-chevron"
            :class="{ 'preview-title-chevron-collapsed': isTitleCollapsed(title.id) }"
          />
        </button>

        <div
          class="preview-title-panel"
          :class="{ 'preview-title-panel-collapsed': isTitleCollapsed(title.id) }"
          :data-testid="`editor-task-preview-panel-${title.id}`"
        >
          <div class="preview-title-panel-inner">
            <div class="preview-title-panel-content">
              <div v-if="groupedTasksByTitle[title.id]?.length" class="space-y-2">
                <template v-for="task in groupedTasksByTitle[title.id]" :key="task.id">
                  <div
                    class="preview-task-wrap"
                    :class="{
                      'preview-task-wrap-clickable': selectedTaskId !== task.id,
                      'preview-task-wrap-active': selectedTaskId === task.id,
                    }"
                    @click="selectedTaskId !== task.id && $emit('select-task', task.id)"
                  >
                    <EditorUiPreviewPanel
                      :task-name="task.name"
                      :task-description="task.description || ''"
                      :default-task-cycle="resolveTaskCycle(task)"
                      :default-task-cycle-value="resolveTaskCycleValue(task)"
                      :default-task-cycle-mode="resolveTaskCycleMode(task)"
                      :default-task-cycle-day="resolveTaskCycleDay(task)"
                      :editable-cycle="editAllTasks || selectedTaskId === task.id"
                      :show-enabled-toggle="task.showEnabledToggle"
                      :default-enabled="resolveTaskEnabled(task)"
                      :task-tone="task.taskTone"
                      :require-bound-input="requireBoundInput"
                      :show-task-cycle="showTaskCycle"
                      :embedded="true"
                      :readonly="!editAllTasks && selectedTaskId !== task.id"
                      :active="selectedTaskId === task.id"
                      :indent-level="task.indentLevel"
                      :ui-schema="selectedTaskId === task.id && !editAllTasks ? selectedTaskUiSchema : parseUiSchema(task.data.uiData ?? {})"
                      :selected-ui-field-id="selectedTaskId === task.id ? selectedUiFieldId : null"
                      :input-entries="selectedTaskId === task.id && !editAllTasks ? selectedTaskInputEntries : sharedInputEntries"
                      @select-ui-field="$emit('select-ui-field', $event)"
                      @update-input="forwardUpdateInput"
                      @update:default-enabled="forwardTaskEnabled(task.id, $event)"
                      @update:default-task-cycle-value="forwardTaskCycleValue(task.id, $event)"
                      @update:default-task-cycle-day="forwardTaskCycleDay(task.id, $event)"
                    />
                  </div>
                </template>
              </div>

              <div v-else class="rounded-[16px] border border-dashed border-(--app-border) px-4 py-3 text-sm text-(--app-text-soft)">
                当前标题下还没有任务。
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-if="!titleRows.length && !ungroupedTasks.length" class="rounded-[16px] border border-dashed border-(--app-border) px-4 py-4 text-sm text-(--app-text-soft)">
        还没有任务可预览。
      </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { ChevronDown } from 'lucide-vue-next';
import type { TaskCycle } from '@/types/bindings/TaskCycle';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import { parseUiSchema, type EditorUiSchema } from '@/views/script-editor/editorSchema';
import type { EditorInputEntry } from '@/views/script-editor/editorVariables';
import { parseTaskCycleValue } from '@/views/script-editor/editorTaskMeta';
import EditorUiPreviewPanel from '@/views/script-editor/EditorUiPreviewPanel.vue';

defineOptions({ name: 'EditorTaskTablePreview' });

const props = withDefaults(defineProps<{
  tasks: ScriptTaskTable[];
  selectedTaskId: string | null;
  selectedTaskUiSchema: EditorUiSchema;
  selectedTaskInputEntries: EditorInputEntry[];
  sharedInputEntries?: EditorInputEntry[];
  taskEnabledById?: Record<string, boolean>;
  taskCycleById?: Record<string, TaskCycle>;
  selectedUiFieldId: string | null;
  selectedTaskCycleValue: string;
  selectedTaskCycleMode: 'named' | 'weekDay' | 'monthDay';
  selectedTaskCycleDay: number;
  editAllTasks?: boolean;
  requireBoundInput?: boolean;
  showHeader?: boolean;
  showTaskCycle?: boolean;
}>(), {
  editAllTasks: false,
  requireBoundInput: false,
  sharedInputEntries: () => [],
  taskEnabledById: () => ({}),
  taskCycleById: () => ({}),
  showHeader: true,
  showTaskCycle: true,
});

const emit = defineEmits<{
  'select-task': [taskId: string];
  'select-ui-field': [fieldId: string];
  'update-input': [entryId: string, field: 'stringValue' | 'booleanValue', value: string | boolean];
  'update:default-enabled': [value: boolean];
  'update:task-enabled': [taskId: string, value: boolean];
  'update:default-task-cycle-value': [value: string];
  'update:default-task-cycle-day': [value: number];
  'update:task-cycle': [taskId: string, value: TaskCycle];
}>();

const ungroupedTitileId = '844533e9-03b7-4ac2-a423-6d608ba94e10';

const visibleTasks = computed(() =>
  props.tasks.filter((task) => !task.isDeleted && !task.isHidden),
);
const sortedTasks = computed(() => [...visibleTasks.value].sort((left, right) => left.index - right.index));
const titleRows = computed(() => sortedTasks.value.filter((task) => task.rowType === 'title'));
const collapsedTitleIds = ref<string[]>([]);
const collapsibleTitleIds = computed(() => [
  ...titleRows.value.map((title) => title.id),
  ...(ungroupedTasks.value.length ? [ungroupedTitileId] : []),
]);
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
const taskCount = computed(() => visibleTasks.value.filter((task) => task.rowType === 'task').length);
const isTitleCollapsed = (titleId: string) => collapsedTitleIds.value.includes(titleId);

const resolveTaskEnabled = (task: ScriptTaskTable) =>
  Object.prototype.hasOwnProperty.call(props.taskEnabledById, task.id)
    ? Boolean(props.taskEnabledById[task.id])
    : task.defaultEnabled;

const resolveTaskCycle = (task: ScriptTaskTable): TaskCycle =>
  Object.prototype.hasOwnProperty.call(props.taskCycleById, task.id)
    ? props.taskCycleById[task.id]
    : task.defaultTaskCycle;

const resolveTaskCycleValue = (task: ScriptTaskTable) => {
  if (!props.editAllTasks && props.selectedTaskId === task.id) {
    return props.selectedTaskCycleValue;
  }

  const taskCycle = resolveTaskCycle(task);
  if (typeof taskCycle === 'string') {
    return taskCycle;
  }
  return 'weekDay' in taskCycle ? 'weekDay' : 'monthDay';
};

const resolveTaskCycleMode = (task: ScriptTaskTable): 'named' | 'weekDay' | 'monthDay' => {
  if (!props.editAllTasks && props.selectedTaskId === task.id) {
    return props.selectedTaskCycleMode;
  }

  const taskCycle = resolveTaskCycle(task);
  if (typeof taskCycle === 'string') {
    return 'named';
  }
  return 'weekDay' in taskCycle ? 'weekDay' : 'monthDay';
};

const resolveTaskCycleDay = (task: ScriptTaskTable) => {
  if (!props.editAllTasks && props.selectedTaskId === task.id) {
    return props.selectedTaskCycleDay;
  }

  const taskCycle = resolveTaskCycle(task);
  if (typeof taskCycle === 'string') {
    return 1;
  }
  return 'weekDay' in taskCycle ? taskCycle.weekDay : taskCycle.monthDay;
};

const forwardUpdateInput = (
  entryId: string,
  field: 'stringValue' | 'booleanValue',
  value: string | boolean,
) => {
  emit('update-input', entryId, field, value);
};

const forwardTaskEnabled = (taskId: string, value: boolean) => {
  emit('update:default-enabled', value);
  emit('update:task-enabled', taskId, value);
};

const forwardTaskCycleValue = (taskId: string, value: string) => {
  emit('update:default-task-cycle-value', value);
  emit('update:task-cycle', taskId, parseTaskCycleValue(value));
};

const forwardTaskCycleDay = (taskId: string, value: number) => {
  emit('update:default-task-cycle-day', value);
  const task = props.tasks.find((item) => item.id === taskId);
  if (!task) {
    return;
  }

  const mode = resolveTaskCycleMode(task);
  if (mode === 'weekDay') {
    emit('update:task-cycle', taskId, { weekDay: Math.max(1, Math.min(7, Math.trunc(value))) });
    return;
  }
  if (mode === 'monthDay') {
    emit('update:task-cycle', taskId, { monthDay: Math.max(1, Math.min(31, Math.trunc(value))) });
  }
};

const toggleTitleCollapsed = (titleId: string) => {
  collapsedTitleIds.value = isTitleCollapsed(titleId)
    ? collapsedTitleIds.value.filter((id) => id !== titleId)
    : [...collapsedTitleIds.value, titleId];
};

const handleTitleClick = (titleId: string) => {
  emit('select-task', titleId);
  toggleTitleCollapsed(titleId);
};

watch(collapsibleTitleIds, (titleIds) => {
  const nextIds = new Set(titleIds);
  collapsedTitleIds.value = collapsedTitleIds.value.filter((id) => nextIds.has(id));
}, { immediate: true });

watch(() => props.selectedTaskId, (taskId) => {
  if (!taskId) {
    return;
  }

  const selectedTask = sortedTasks.value.find((task) => task.id === taskId && task.rowType === 'task');
  if (!selectedTask) {
    return;
  }

  const titleId = selectedTask.sectionId ?? ungroupedTitileId;
  if (!isTitleCollapsed(titleId)) {
    return;
  }

  collapsedTitleIds.value = collapsedTitleIds.value.filter((id) => id !== titleId);
});

</script>

<style scoped>
.preview-scroll-shell {
  min-height: 0;
  overflow: hidden;
}

.preview-scroll-body {
  height: 100%;
  padding-right: 0.65rem;
  margin-right: -0.65rem;
  scrollbar-gutter: stable;
  scrollbar-width: 14pxs;
}

.preview-scroll-body::-webkit-scrollbar {
  width: 14px;
}

.preview-scroll-body::-webkit-scrollbar-thumb {
  border: 3px solid transparent;
  border-radius: 999px;
  background-clip: content-box;
}

.preview-scroll-body::-webkit-scrollbar-track {
  background: transparent;
}

.preview-title-row {
  display: flex;
  width: 100%;
  align-items: center;
  justify-content: space-between;
  border-radius: 16px;
  border: 1px solid var(--app-border);
  background: var(--app-panel-muted);
  padding: 1rem 1rem;
  text-align: left;
  font-weight: 700;
  color: var(--app-text-strong);
  transition: border-color 0.16s ease, background 0.16s ease, box-shadow 0.16s ease;
}

.preview-title-row-expanded {
  border-color: var(--app-accent);
  background: var(--app-accent);
  color: var(--color-primary-content);
}

.preview-title-row-main {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 0.65rem;
}

.preview-title-row-active {
  border-color: color-mix(in srgb, var(--app-accent) 34%, var(--app-border));
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--app-accent) 18%, transparent);
}

.preview-title-row-expanded.preview-title-row-active {
  border-color: var(--color-primary);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, white 20%, transparent);
}

.preview-title-label {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preview-title-dot {
  width: 0.7rem;
  height: 0.7rem;
  border-radius: 999px;
  background: color-mix(in srgb, var(--app-accent) 78%, white);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--app-accent) 14%, transparent);
}

.preview-title-row-expanded .preview-title-dot {
  background: currentColor;
  box-shadow: 0 0 0 4px color-mix(in srgb, currentColor 18%, transparent);
}

.preview-title-chevron {
  flex-shrink: 0;
  color: var(--app-text-faint);
  transition: transform 0.2s ease, color 0.16s ease;
}

.preview-title-row:hover .preview-title-chevron,
.preview-title-row-active .preview-title-chevron {
  color: var(--app-text-strong);
}

.preview-title-row-expanded .preview-title-chevron,
.preview-title-row-expanded:hover .preview-title-chevron,
.preview-title-row-expanded.preview-title-row-active .preview-title-chevron {
  /* color: var(--color-primary-content); */
  color: var(--color-primary-content);
}

.preview-title-chevron-collapsed {
  transform: rotate(-90deg);
}

.preview-title-panel {
  display: grid;
  grid-template-rows: 1fr;
  opacity: 1;
  transition: grid-template-rows 0.22s ease, opacity 0.18s ease;
}

.preview-title-panel-collapsed {
  grid-template-rows: 0fr;
  opacity: 0;
}

.preview-title-panel-inner {
  min-height: 0;
  overflow: hidden;
}

.preview-title-panel-content {
  padding-top: 0.5rem;
}

.preview-task-wrap-clickable {
  border-radius: 18px;
  cursor: pointer;
  transition: background 0.16s ease, box-shadow 0.16s ease;
}

.preview-task-wrap-clickable:hover {
  background: color-mix(in srgb, var(--app-accent) 8%, var(--app-panel-muted));
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--app-accent) 14%, transparent);
}

.preview-task-wrap-active {
  border-radius: 18px;
  background: color-mix(in srgb, var(--app-accent) 10%, transparent);
}
</style>
