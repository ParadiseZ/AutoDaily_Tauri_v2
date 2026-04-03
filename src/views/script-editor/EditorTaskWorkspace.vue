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

      <div v-else-if="activePanel === 'ui'" class="grid min-h-0 gap-4 xl:grid-rows-[auto_auto_minmax(0,1fr)]">
        <EditorTaskTablePreview :tasks="tasks" :selected-task-id="task.id" @select-task="$emit('select-task', $event)" />

        <EditorUiPreviewPanel
          v-if="task.rowType === 'task'"
          :task-name="task.name"
          :default-task-cycle="task.defaultTaskCycle"
          :show-enabled-toggle="task.showEnabledToggle"
          :default-enabled="task.defaultEnabled"
          :task-tone="task.taskTone"
          :ui-schema="uiSchema"
          :selected-ui-field-id="selectedUiFieldId"
          :input-entries="inputEntries"
          @select-ui-field="$emit('select-ui-field', $event)"
          @update-input="forwardUpdateInput"
        />

        <div
          v-else
          class="rounded-[18px] border border-dashed border-[var(--app-border)] bg-[var(--app-panel-muted)] px-5 py-5 text-sm text-[var(--app-text-soft)]"
        >
          标题行只用于整表分块显示，不包含单任务 UI 字段预览。
        </div>

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
        :variable-options="variableOptions"
        :catalog-variable-options="catalogVariableOptions"
        @select-step-path="$emit('select-step-path', $event)"
        @navigate-branch="$emit('navigate-branch', $event)"
        @reorder-step="(from, to) => $emit('reorder-step', from, to)"
        @remove-step="$emit('remove-step', $event)"
        @update-step="(index, step) => $emit('update-step', index, step)"
      />

      <div v-else class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-5 py-5">
        <p class="text-sm font-semibold text-[var(--app-text-strong)]">任务概览</p>
      </div>
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
import EditorInputDetailsPanel from '@/views/script-editor/EditorInputDetailsPanel.vue';
import EditorStepWorkspace from '@/views/script-editor/editor-step/EditorStepWorkspace.vue';
import EditorTaskTablePreview from '@/views/script-editor/EditorTaskTablePreview.vue';
import EditorUiFieldDetailsPanel from '@/views/script-editor/EditorUiFieldDetailsPanel.vue';
import EditorUiPreviewPanel from '@/views/script-editor/EditorUiPreviewPanel.vue';
import type { StepBranchPath, StepPath } from '@/views/script-editor/editor-step/editorStepTree';
import type { EditorPanelId, EditorUiSchema } from '@/views/script-editor/editorSchema';
import type { EditorInputEntry, EditorVariableOption } from '@/views/script-editor/editorVariables';

defineOptions({ name: 'EditorTaskWorkspace' });

const props = defineProps<{
  task: ScriptTaskTable | null;
  tasks: ScriptTaskTable[];
  activePanel: EditorPanelId;
  steps: Step[];
  selectedStepPath: StepPath | null;
  activeBranchPath: StepBranchPath;
  uiSchema: EditorUiSchema;
  selectedUiFieldId: string | null;
  inputEntries: EditorInputEntry[];
  variableOptions: EditorVariableOption[];
  catalogVariableOptions: EditorVariableOption[];
  selectedInputId: string | null;
}>();

const emit = defineEmits<{
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
  'remove-input': [entryId: string];
  'select-input': [entryId: string];
  'select-ui-field': [fieldId: string];
  'select-task': [taskId: string];
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
  if (props.activePanel === 'ui') return 'UI 预览';
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
