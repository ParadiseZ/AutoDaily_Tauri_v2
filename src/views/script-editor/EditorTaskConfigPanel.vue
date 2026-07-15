<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <div class="overflow-x-auto">
      <div class="editor-panel-tabs min-w-max">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          class="editor-panel-tab"
          :class="{ 'editor-panel-tab-active': activePanel === tab.id }"
          :data-testid="`editor-tab-${tab.id}`"
          @click="$emit('update:active-panel', tab.id)"
        >
          {{ tab.label }}
        </button>
      </div>
    </div>

    <template v-if="task">
      <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
        <div v-if="activePanel === 'basic'" class="grid gap-3 md:grid-cols-2">
          <div class="grid gap-2 md:col-span-2">
            <p class="text-xs font-medium uppercase tracking-[0.14em]">任务说明</p>
            <input
              :value="taskDescription"
              class="app-input"
              type="text"
              placeholder="选填，用于补充任务说明"
              data-testid="editor-task-description"
              @input="$emit('update:task-description', ($event.target as HTMLInputElement).value)"
            />
          </div>

          <div class="grid gap-3 md:col-span-2 md:grid-cols-[72px_minmax(0,1fr)] md:items-center">
            <p class="text-xs font-medium uppercase tracking-[0.14em]">行类型</p>
            <EditorSelectField
              :model-value="taskRowType"
              :options="taskRowTypeOptions"
              placeholder="选择行类型"
              test-id="editor-task-row-type"
              @update:model-value="$emit('update:task-row-type', $event as 'task' | 'title')"
            />
          </div>

          <label class="flex items-center gap-3 rounded-[18px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3 text-sm text-(--app-text-soft) md:col-span-2">
            <input
              :checked="taskHidden"
              class="h-4 w-4 accent-(--app-accent)"
              type="checkbox"
              data-testid="editor-task-hidden"
              @change="$emit('update:task-hidden', ($event.target as HTMLInputElement).checked)"
            />
            <span>UI中隐藏</span>
          </label>
        </div>

        <EditorVariableListPanel
          v-else-if="activePanel === 'inputs'"
          :entries="inputEntries"
          :selected-input-id="selectedInputId"
          :input-error="inputError"
          :entry-reference-state="entryReferenceState"
          @add="$emit('add-input')"
          @select="$emit('select-input', $event)"
          @remove="$emit('remove-input', $event)"
        />

        <div v-else-if="activePanel === 'ui'" class="space-y-4">
          <div class="flex items-center justify-between gap-3">
            <p class="text-sm font-semibold text-(--app-text-strong)">界面字段</p>
          </div>

          <div class="flex flex-wrap gap-2">
            <button
              v-for="template in uiFieldTemplates"
              :key="template.id"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              :data-testid="`editor-ui-template-${template.id}`"
              @click="$emit('add-ui-field', template.id)"
            >
              {{ template.label }}
            </button>
          </div>

          <div v-if="uiSchema.fields.length" class="space-y-2">
            <article
              v-for="(field, index) in uiSchema.fields"
              :key="field.id"
              class="app-list-item"
              :class="{ 'app-list-item-active': selectedUiFieldId === field.id }"
              @click="$emit('select-ui-field', field.id)"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <p class="truncate text-sm font-semibold text-(--app-text-strong)">{{ field.label || '未命名字段' }}</p>
                  <p class="mt-1 text-xs text-(--app-text-faint)">
                    {{ getUiControlLabel(field.control) }} · {{ field.inputKey || '未绑定' }}
                  </p>
                </div>
                <div class="flex items-center gap-2">
                  <span class="rounded-full border border-(--app-border) px-2 py-1 text-[11px] text-(--app-text-soft)">
                    {{ index + 1 }}
                  </span>
                  <button class="app-icon-button app-crash-icon app-icon-button-sec" type="button" aria-label="删除" title="删除" @click.stop="$emit('remove-ui-field', field.id)">
                    <Trash2 class="h-4 w-4" />
                  </button>
                </div>
              </div>
            </article>
          </div>

          <EmptyState
            v-else
            title="点击上方按钮以添加UI内容"
          />
        </div>

        <EditorStepTemplateLibrary
          v-else
          :restrict-sequence-templates="restrictSequenceTemplates"
          test-id-prefix="editor-step-template"
          @select="$emit('append-template-step', $event)"
        />
      </div>
    </template>

    <EmptyState
      v-else
      title="还没有可编辑任务"
      description="先在左侧创建任务，再在这里配置基本信息、输入和步骤。"
    />
  </SurfacePanel>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Trash2 } from '@lucide/vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { TaskRowType } from '@/types/bindings/TaskRowType';
import type { TaskTone } from '@/types/bindings/TaskTone';
import type { TaskTriggerMode } from '@/types/bindings/TaskTriggerMode';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import EditorStepTemplateLibrary from '@/views/script-editor/EditorStepTemplateLibrary.vue';
import EditorVariableListPanel from '@/views/script-editor/EditorVariableListPanel.vue';
import { getUiControlLabel, uiFieldTemplates } from '@/views/script-editor/editorSchema';
import type { EditorPanelId, EditorUiSchema, UiFieldControl } from '@/views/script-editor/editorSchema';
import { taskRowTypeOptions } from '@/views/script-editor/editorTaskMeta';
import type { EditorInputEntry } from '@/views/script-editor/editorVariables';

const props = defineProps<{
  task: ScriptTaskTable | null;
  activePanel: EditorPanelId;
  taskName: string;
  taskDescription: string;
  taskRowType: TaskRowType;
  taskTriggerMode: TaskTriggerMode;
  taskHidden: boolean;
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
  inputEntries: EditorInputEntry[];
  inputError: string | null;
  entryReferenceState?: Record<string, { referenced: boolean }>;
  uiSchema: EditorUiSchema;
  selectedInputId: string | null;
  selectedUiFieldId: string | null;
  restrictSequenceTemplates?: boolean;
}>();

defineEmits<{
  'update:active-panel': [panel: EditorPanelId];
  'update:task-name': [value: string];
  'update:task-description': [value: string];
  'update:task-row-type': [value: TaskRowType];
  'update:task-trigger-mode': [value: TaskTriggerMode];
  'update:task-hidden': [value: boolean];
  'update:record-schedule': [value: boolean];
  'update:section-id': [value: string | null];
  'update:indent-level': [value: number];
  'update:default-task-cycle-value': [value: string];
  'update:default-task-cycle-day': [value: number];
  'update:task-exec-max': [value: number];
  'update:show-enabled-toggle': [value: boolean];
  'update:default-enabled': [value: boolean];
  'update:task-tone': [value: TaskTone];
  'add-input': [];
  'select-input': [entryId: string];
  'remove-input': [entryId: string];
  'add-ui-field': [control: UiFieldControl];
  'select-ui-field': [fieldId: string];
  'remove-ui-field': [fieldId: string];
  'append-template-step': [templateId: string];
  'open-raw': [section: 'inputs' | 'ui' | 'steps'];
}>();

const tabs = computed<Array<{ id: EditorPanelId; label: string }>>(() => {
  if (props.taskRowType === 'title') {
    return [{ id: 'basic', label: '基本' }];
  }

  return [
    { id: 'basic', label: '基本' },
    { id: 'inputs', label: `变量 ${props.inputEntries.length}` },
    { id: 'ui', label: `界面 ${props.uiSchema.fields.length}` },
    { id: 'steps', label: `步骤 ${props.task?.data.steps.length ?? 0}` },
  ];
});

</script>
