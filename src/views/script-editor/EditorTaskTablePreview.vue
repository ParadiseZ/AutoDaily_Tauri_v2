<template>
  <div class="rounded-[22px] border border-[var(--app-border)] bg-[linear-gradient(180deg,rgba(255,255,255,0.92),rgba(245,247,252,0.88))] px-5 py-5 shadow-[var(--app-shadow-soft)]">
    <div v-if="showHeader" class="flex items-center justify-between gap-3">
      <div>
        <p class="text-xs uppercase tracking-[0.18em] text-[var(--app-text-faint)]">Script Preview</p>
        <h3 class="text-lg font-semibold text-[var(--app-text-strong)]">整表任务预览</h3>
      </div>
      <span class="rounded-full border border-[var(--app-border)] bg-white/60 px-3 py-1 text-xs text-[var(--app-text-faint)]">
        {{ taskCount }} 个任务行
      </span>
    </div>

    <div class="min-h-0 space-y-4 overflow-y-auto pr-1 custom-scrollbar" :class="showHeader ? 'mt-4' : ''">
      <div v-if="ungroupedTasks.length" class="space-y-2">
        <p class="text-[11px] uppercase tracking-[0.14em] text-[var(--app-text-faint)]">未分组任务</p>
        <template v-for="task in ungroupedTasks" :key="task.id">
          <div
            class="preview-task-wrap"
            :class="{ 'preview-task-wrap-clickable': selectedTaskId !== task.id }"
            @click="selectedTaskId !== task.id && $emit('select-task', task.id)"
          >
            <EditorUiPreviewPanel
              :task-name="task.name"
              :default-task-cycle="task.defaultTaskCycle"
              :default-task-cycle-value="selectedTaskId === task.id ? selectedTaskCycleValue : undefined"
              :default-task-cycle-mode="selectedTaskId === task.id ? selectedTaskCycleMode : 'named'"
              :default-task-cycle-day="selectedTaskId === task.id ? selectedTaskCycleDay : 1"
              :editable-cycle="!editAllTasks && selectedTaskId === task.id"
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
              @update:default-task-cycle-value="$emit('update:default-task-cycle-value', $event)"
              @update:default-task-cycle-day="$emit('update:default-task-cycle-day', $event)"
            />
          </div>
        </template>
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
          <template v-for="task in groupedTasksByTitle[title.id]" :key="task.id">
            <div
              class="preview-task-wrap"
              :class="{ 'preview-task-wrap-clickable': selectedTaskId !== task.id }"
              @click="selectedTaskId !== task.id && $emit('select-task', task.id)"
            >
              <EditorUiPreviewPanel
                :task-name="task.name"
                :default-task-cycle="task.defaultTaskCycle"
                :default-task-cycle-value="selectedTaskId === task.id ? selectedTaskCycleValue : undefined"
                :default-task-cycle-mode="selectedTaskId === task.id ? selectedTaskCycleMode : 'named'"
                :default-task-cycle-day="selectedTaskId === task.id ? selectedTaskCycleDay : 1"
                :editable-cycle="!editAllTasks && selectedTaskId === task.id"
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
                @update:default-task-cycle-value="$emit('update:default-task-cycle-value', $event)"
                @update:default-task-cycle-day="$emit('update:default-task-cycle-day', $event)"
              />
            </div>
          </template>
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
import { computed } from 'vue';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import { parseUiSchema, type EditorUiSchema } from '@/views/script-editor/editorSchema';
import type { EditorInputEntry } from '@/views/script-editor/editorVariables';
import EditorUiPreviewPanel from '@/views/script-editor/EditorUiPreviewPanel.vue';

defineOptions({ name: 'EditorTaskTablePreview' });

const props = withDefaults(defineProps<{
  tasks: ScriptTaskTable[];
  selectedTaskId: string | null;
  selectedTaskUiSchema: EditorUiSchema;
  selectedTaskInputEntries: EditorInputEntry[];
  sharedInputEntries?: EditorInputEntry[];
  taskEnabledById?: Record<string, boolean>;
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
}>();

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

const resolveTaskEnabled = (task: ScriptTaskTable) =>
  Object.prototype.hasOwnProperty.call(props.taskEnabledById, task.id)
    ? Boolean(props.taskEnabledById[task.id])
    : task.defaultEnabled;

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

</script>

<style scoped>
.preview-title-row {
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

.preview-title-row-active {
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

.preview-task-wrap-clickable {
  cursor: pointer;
}
</style>
