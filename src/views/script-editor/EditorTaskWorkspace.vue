<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <template v-if="task">
      <div class="flex items-start justify-between gap-3">
        <div class="space-y-1">
          <p class="text-xs uppercase tracking-[0.18em] text-[var(--app-text-faint)]">Workspace</p>
          <h2 class="text-xl font-semibold text-[var(--app-text-strong)]">{{ workspaceTitle }}</h2>
        </div>
        <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('open-raw', rawSection)">
          查看底层结构
        </button>
      </div>

      <EditorInputDetailsPanel
        v-if="activePanel === 'inputs'"
        :selected-input-entry="selectedInputEntry"
        :selected-input-index="selectedInputIndex"
        @update-input="forwardUpdateInput"
        @remove-input="$emit('remove-input', $event)"
      />

      <div v-else-if="activePanel === 'ui'" class="grid min-h-0 gap-4 xl:grid-rows-[auto_minmax(0,1fr)]">
        <EditorTaskTablePreview
          :tasks="tasks"
          :selected-task-id="task.id"
          :selected-task-ui-schema="uiSchema"
          :selected-task-input-entries="inputEntries"
          :selected-ui-field-id="selectedUiFieldId"
          :selected-task-cycle-value="defaultTaskCycleValue"
          :selected-task-cycle-mode="defaultTaskCycleMode"
          :selected-task-cycle-day="defaultTaskCycleDay"
          @select-task="$emit('select-task', $event)"
          @select-ui-field="$emit('select-ui-field', $event)"
          @update-input="forwardUpdateInput"
          @update:default-enabled="$emit('update:default-enabled', $event)"
          @update:default-task-cycle-value="$emit('update:default-task-cycle-value', $event)"
          @update:default-task-cycle-day="$emit('update:default-task-cycle-day', $event)"
        />

        <EditorUiFieldDetailsPanel
          v-if="task.rowType === 'task'"
          :selected-ui-field="selectedUiField"
          :selected-ui-field-index="selectedUiFieldIndex"
          :variable-options="variableOptions"
          @update-ui-field="forwardUpdateUiField"
          @remove-ui-field="$emit('remove-ui-field', $event)"
        />

        <div
          v-else
          class="rounded-[18px] border border-dashed border-[var(--app-border)] bg-[var(--app-panel-muted)] px-5 py-5 text-sm text-[var(--app-text-soft)]"
        >
          选择普通任务后，这里会显示当前任务的字段详情。
        </div>
      </div>

      <EditorStepWorkspace
        v-else-if="activePanel === 'steps'"
        :steps="steps"
        :selected-step-path="selectedStepPath"
        :active-branch-path="activeBranchPath"
        :input-entries="inputEntries"
        :variable-options="variableOptions"
        :catalog-variable-options="catalogVariableOptions"
        :label-index-options="labelIndexOptions"
        :label-select-placeholder="labelSelectPlaceholder"
        :label-select-hint="labelSelectHint"
        :task-reference-options="taskReferenceOptions"
        :policy-reference-options="policyReferenceOptions"
        :task-ui-variable-options="taskUiVariableOptions"
        :policy-group-reference-options="policyGroupReferenceOptions"
        :policy-set-reference-options="policySetReferenceOptions"
        :create-reference="createReference"
        :jump-to-reference="jumpToReference"
        :create-variable="createVariable"
        :jump-to-variable="jumpToVariable"
        :update-input="forwardUpdateInput"
        @select-step-path="$emit('select-step-path', $event)"
        @navigate-branch="$emit('navigate-branch', $event)"
        @reorder-step="(from, to) => $emit('reorder-step', from, to)"
        @remove-step="$emit('remove-step', $event)"
        @update-step="(index, step) => $emit('update-step', index, step)"
      />

      <EditorTaskOverviewPanel
        v-else
        :task="task"
        :task-name="task.name"
        :task-trigger-mode="taskTriggerMode"
        :record-schedule="recordSchedule"
        :section-id="sectionId"
        :indent-level="indentLevel"
        :default-task-cycle-value="defaultTaskCycleValue"
        :default-task-cycle-mode="defaultTaskCycleMode"
        :default-task-cycle-day="defaultTaskCycleDay"
        :task-exec-max="taskExecMax"
        :show-enabled-toggle="showEnabledToggle"
        :default-enabled="defaultEnabled"
        :task-tone="taskTone"
        :title-options="titleOptions"
        @update:task-name="$emit('update:task-name', $event)"
        @update:task-trigger-mode="$emit('update:task-trigger-mode', $event)"
        @update:record-schedule="$emit('update:record-schedule', $event)"
        @update:section-id="$emit('update:section-id', $event)"
        @update:indent-level="$emit('update:indent-level', $event)"
        @update:default-task-cycle-value="$emit('update:default-task-cycle-value', $event)"
        @update:default-task-cycle-day="$emit('update:default-task-cycle-day', $event)"
        @update:task-exec-max="$emit('update:task-exec-max', $event)"
        @update:show-enabled-toggle="$emit('update:show-enabled-toggle', $event)"
        @update:default-enabled="$emit('update:default-enabled', $event)"
        @update:task-tone="$emit('update:task-tone', $event)"
      />
    </template>

    <EmptyState
      v-else
      title="没有选中任务"
      description="先从左侧选择任务，右侧工作区才会显示步骤概览和 UI 预览。"
    />
  </SurfacePanel>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { Step } from '@/types/bindings/Step';
import type { EditorReferenceKind, EditorReferenceOption, EditorTaskUiVariableOption } from '@/views/script-editor/editorReferences';
import EditorInputDetailsPanel from '@/views/script-editor/EditorInputDetailsPanel.vue';
import EditorTaskOverviewPanel from '@/views/script-editor/EditorTaskOverviewPanel.vue';
import EditorStepWorkspace from '@/views/script-editor/editor-step/EditorStepWorkspace.vue';
import EditorTaskTablePreview from '@/views/script-editor/EditorTaskTablePreview.vue';
import EditorUiFieldDetailsPanel from '@/views/script-editor/EditorUiFieldDetailsPanel.vue';
import type { StepBranchPath, StepPath } from '@/views/script-editor/editor-step/editorStepTree';
import type { EditorPanelId, EditorUiSchema } from '@/views/script-editor/editorSchema';
import type { EditorInputEntry, EditorInputType, EditorVariableOption } from '@/views/script-editor/editorVariables';
import type { TaskTone } from '@/types/bindings/TaskTone';
import type { TaskTriggerMode } from '@/types/bindings/TaskTriggerMode';

defineOptions({ name: 'EditorTaskWorkspace' });

const props = defineProps<{
  task: ScriptTaskTable | null;
  tasks: ScriptTaskTable[];
  activePanel: EditorPanelId;
  taskTriggerMode: TaskTriggerMode;
  recordSchedule: boolean;
  sectionId: string | null;
  indentLevel: number;
  defaultTaskCycleValue: string;
  defaultTaskCycleMode: 'named' | 'weekDay' | 'monthDay';
  defaultTaskCycleDay: number;
  taskExecMax: number;
  showEnabledToggle: boolean;
  defaultEnabled: boolean;
  taskTone: TaskTone;
  titleOptions: Array<{ label: string; value: string | null; description?: string; disabled?: boolean }>;
  steps: Step[];
  selectedStepPath: StepPath | null;
  activeBranchPath: StepBranchPath;
  inputEntries: EditorInputEntry[];
  uiSchema: EditorUiSchema;
  selectedUiFieldId: string | null;
  variableOptions: EditorVariableOption[];
  catalogVariableOptions: EditorVariableOption[];
  labelIndexOptions: Array<{ label: string; value: number; description?: string; disabled?: boolean }>;
  labelSelectPlaceholder: string;
  labelSelectHint?: string | null;
  taskReferenceOptions: EditorReferenceOption[];
  policyReferenceOptions: EditorReferenceOption[];
  taskUiVariableOptions?: EditorTaskUiVariableOption[];
  policyGroupReferenceOptions: EditorReferenceOption[];
  policySetReferenceOptions: EditorReferenceOption[];
  createReference: (kind: EditorReferenceKind) => Promise<string>;
  jumpToReference: (kind: EditorReferenceKind, id: string) => void;
  createVariable?: (namespace?: 'input' | 'runtime', inputType?: EditorInputType, options?: { preferredKey?: string; name?: string; select?: boolean; silent?: boolean }) => Promise<string>;
  jumpToVariable?: (option: EditorVariableOption) => void;
  selectedInputId: string | null;
}>();

const emit = defineEmits<{
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
  'remove-input': [entryId: string];
  'update:task-name': [value: string];
  'select-input': [entryId: string];
  'select-ui-field': [fieldId: string];
  'select-task': [taskId: string];
  'update:task-trigger-mode': [value: TaskTriggerMode];
  'update:record-schedule': [value: boolean];
  'update:section-id': [value: string | null];
  'update:indent-level': [value: number];
  'update:default-task-cycle-value': [value: string];
  'update:default-task-cycle-day': [value: number];
  'update:task-exec-max': [value: number];
  'update:show-enabled-toggle': [value: boolean];
  'update:default-enabled': [value: boolean];
  'update:task-tone': [value: TaskTone];
  'update-ui-field': [fieldId: string, field: 'label' | 'key' | 'editable' | 'checkboxStyle' | 'variableId' | 'inputKey' | 'description' | 'placeholder' | 'optionsText' | 'min' | 'max' | 'step' | 'numericMode', value: string | boolean];
  'remove-ui-field': [fieldId: string];
  'select-step-path': [path: StepPath];
  'navigate-branch': [branchPath: StepBranchPath];
  'reorder-step': [from: number, to: number];
  'remove-step': [index: number];
  'update-step': [index: number, step: Step];
  'open-raw': [section: 'inputs' | 'ui' | 'steps'];
}>();

const workspaceTitle = computed(() => {
  if (props.activePanel === 'steps') return '步骤概览';
  if (props.activePanel === 'ui') return `UI 预览/${props.task?.name || '未命名任务'}`;
  if (props.activePanel === 'inputs') return '输入设置';
  return '任务概览';
});

const rawSection = computed(() => {
  if (props.activePanel === 'steps') return 'steps';
  if (props.activePanel === 'ui') return 'ui';
  return 'inputs';
});

const selectedInputEntry = computed(() => props.inputEntries.find((entry) => entry.id === props.selectedInputId) ?? null);
const selectedInputIndex = computed(() =>
  selectedInputEntry.value ? props.inputEntries.findIndex((entry) => entry.id === selectedInputEntry.value?.id) : -1,
);

const selectedUiField = computed(() => props.uiSchema.fields.find((field) => field.id === props.selectedUiFieldId) ?? null);
const selectedUiFieldIndex = computed(() =>
  selectedUiField.value ? props.uiSchema.fields.findIndex((field) => field.id === selectedUiField.value?.id) : -1,
);

const forwardUpdateInput = (
  entryId: string,
  field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue',
  value: string | boolean,
) => {
  emit('update-input', entryId, field, value);
};

const forwardUpdateUiField = (
  fieldId: string,
  field: 'label' | 'key' | 'editable' | 'checkboxStyle' | 'variableId' | 'inputKey' | 'description' | 'placeholder' | 'optionsText' | 'min' | 'max' | 'step' | 'numericMode',
  value: string | boolean,
) => {
  emit('update-ui-field', fieldId, field, value);
};
</script>
