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
          <div class="grid gap-3 md:col-span-2 md:grid-cols-[72px_minmax(0,1fr)] md:items-center">
            <span class="text-xs font-medium uppercase tracking-[0.14em] text-(--app-text-faint)">行类型</span>
            <EditorSelectField
              :model-value="taskRowType"
              :options="taskRowTypeOptions"
              placeholder="选择行类型"
              test-id="editor-task-row-type"
              @update:model-value="$emit('update:task-row-type', $event as 'task' | 'title')"
            />
          </div>

          <div class="rounded-[18px] border border-dashed border-(--app-border) px-4 py-4 text-sm text-(--app-text-soft) md:col-span-2">
            任务行专属设置已移动到右侧“任务概览”工作区，方便结合整表预览一起调整。
          </div>

          <label class="flex items-center gap-3 rounded-[18px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3 text-sm text-(--app-text-soft) md:col-span-2">
            <input
              :checked="taskHidden"
              class="h-4 w-4 accent-(--app-accent)"
              type="checkbox"
              data-testid="editor-task-hidden"
              @change="$emit('update:task-hidden', ($event.target as HTMLInputElement).checked)"
            />
            <span>在工作台中隐藏当前任务</span>
          </label>
        </div>

        <div v-else-if="activePanel === 'inputs'" class="space-y-4">
          <div class="flex items-center justify-between gap-3">
            <div class="flex gap-2">
              <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('open-raw', 'inputs')">
                JSON
              </button>
              <button class="app-button app-button-primary app-toolbar-button" type="button" data-testid="editor-input-add" @click="$emit('add-input')">
                添加输入
              </button>
            </div>
          </div>

          <div v-if="inputEntries.length" class="space-y-2">
            <article
              v-for="(entry, index) in inputEntries"
              :key="entry.id"
              class="app-list-item cursor-pointer"
              :class="{ 'app-list-item-active': selectedInputId === entry.id }"
              :data-testid="`editor-input-item-${index}`"
              @click="$emit('select-input', entry.id)"
            >
              <div class="flex items-start justify-between gap-3">
                <div class="min-w-0">
                  <div class="flex items-center gap-2">
                    <p class="truncate text-sm font-semibold text-(--app-text-strong)">{{ entry.name || entry.key || '未命名变量' }}</p>
                    <span class="rounded-full border border-(--app-border) px-2 py-1 text-[11px] text-(--app-text-faint)">
                      {{ getScopeLabel(entry.namespace) }}
                    </span>
                  </div>
                  <p class="mt-1 text-xs text-(--app-text-faint)">{{ entry.key || '未设置键' }} · {{ getInputTypeLabel(entry.type) }}</p>
                </div>
                <button
                  class="app-button app-button-danger app-toolbar-button shrink-0"
                  type="button"
                  :data-testid="`editor-input-remove-${index}`"
                  @click.stop="$emit('remove-input', entry.id)"
                >
                  删除
                </button>
              </div>
            </article>
          </div>

          <EmptyState
            v-else
            title="还没有变量"
            description="先添加变量，再在右侧编辑名称、键、类型、作用域和值。"
          />

          <p v-if="inputError" class="text-sm text-red-700">{{ inputError }}</p>
        </div>

        <div v-else-if="activePanel === 'ui'" class="space-y-4">
          <div class="flex items-center justify-between gap-3">
            <p class="text-sm font-semibold text-(--app-text-strong)">界面字段</p>
            <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('open-raw', 'ui')">
              JSON
            </button>
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
                  <button class="app-button app-button-danger app-toolbar-button" type="button" @click.stop="$emit('remove-ui-field', field.id)">
                    删除
                  </button>
                </div>
              </div>
            </article>
          </div>

          <EmptyState
            v-else
            title="还没有 UI 字段"
            description="先从模板插入字段，再在右侧预览和细调内容。"
          />
        </div>

        <div v-else class="space-y-4">
          <div class="flex items-center justify-between gap-3">
            <div>
              <p class="text-sm font-semibold text-(--app-text-strong)">快速模板</p>
            </div>
            <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('open-raw', 'steps')">
              JSON
            </button>
          </div>

          <div class="space-y-3">
            <details v-for="group in templateGroups" :key="group.name" class="rounded-[18px] border border-(--app-border) bg-(--app-panel-muted)" :open="true">
              <summary class="cursor-pointer list-none px-4 py-3 text-sm font-semibold text-(--app-text-strong)">
                <div class="flex items-center justify-between gap-3">
                  <span>{{ group.name }}</span>
                  <span class="text-xs text-(--app-text-faint)">{{ group.items.length }}</span>
                </div>
              </summary>

              <div class="grid gap-2 px-3 pb-3 sm:grid-cols-2">
                <button
                  v-for="template in group.items"
                  :key="template.id"
                  class="editor-template-tile"
                  :data-testid="`editor-step-template-${template.id}`"
                  type="button"
                  @click="$emit('append-template-step', template.id)"
                >
                  <span class="editor-template-dot" />
                  <span class="truncate">{{ template.label }}</span>
                </button>
              </div>
            </details>
          </div>
        </div>
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
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { TaskRowType } from '@/types/bindings/TaskRowType';
import type { TaskTone } from '@/types/bindings/TaskTone';
import type { TaskTriggerMode } from '@/types/bindings/TaskTriggerMode';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import { editorStepTemplates } from '@/views/script-editor/editor-step/editorStepTemplates';
import { getUiControlLabel, uiFieldTemplates } from '@/views/script-editor/editorSchema';
import type { EditorPanelId, EditorUiSchema, UiFieldControl } from '@/views/script-editor/editorSchema';
import { taskRowTypeOptions } from '@/views/script-editor/editorTaskMeta';
import { getInputTypeLabel, type EditorInputEntry } from '@/views/script-editor/editorVariables';

const props = defineProps<{
  task: ScriptTaskTable | null;
  activePanel: EditorPanelId;
  taskName: string;
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
  uiSchema: EditorUiSchema;
  selectedInputId: string | null;
  selectedUiFieldId: string | null;
}>();

defineEmits<{
  'update:active-panel': [panel: EditorPanelId];
  'update:task-name': [value: string];
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
  'update-ui-layout': [value: 'horizontal' | 'vertical'];
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
    { id: 'inputs', label: `输入 ${props.inputEntries.length}` },
    { id: 'ui', label: `界面 ${props.uiSchema.fields.length}` },
    { id: 'steps', label: `步骤 ${props.task?.data.steps.length ?? 0}` },
  ];
});

const getScopeLabel = (scope: EditorInputEntry['namespace']) => {
  if (scope === 'runtime') return 'Runtime';
  if (scope === 'system') return 'System';
  return 'Input';
};

const templateGroups = computed(() => {
  const grouped = new Map<string, typeof editorStepTemplates>();
  for (const template of editorStepTemplates) {
    const bucket = grouped.get(template.group) ?? [];
    bucket.push(template);
    grouped.set(template.group, bucket);
  }

  const groupOrder = ['动作', '流程', '数据', '视觉', '状态', '容器', '兼容'];

  return Array.from(grouped.entries())
    .sort(([left], [right]) => {
      const leftIndex = groupOrder.indexOf(left);
      const rightIndex = groupOrder.indexOf(right);
      return (leftIndex === -1 ? Number.MAX_SAFE_INTEGER : leftIndex) - (rightIndex === -1 ? Number.MAX_SAFE_INTEGER : rightIndex);
    })
    .map(([name, items]) => ({ name, items }));
});
</script>

<style scoped>
.editor-panel-tabs {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  border-bottom: 1px solid var(--app-border);
}

.editor-panel-tab {
  position: relative;
  margin-bottom: -1px;
  border-bottom: 2px solid transparent;
  padding: 0.75rem 0.35rem 0.85rem;
  color: var(--app-text-faint);
  font-size: 0.93rem;
  font-weight: 600;
  transition: color 0.16s ease, border-color 0.16s ease;
}

.editor-panel-tab:hover {
  color: var(--app-text-soft);
}

.editor-panel-tab-active {
  border-bottom-color: var(--app-accent);
  color: var(--app-text-strong);
}

.editor-template-tile {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  width: 100%;
  border-radius: 14px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.52);
  padding: 0.7rem 0.8rem;
  text-align: left;
  color: var(--app-text-strong);
  transition: border-color 0.16s ease, background 0.16s ease;
}

.editor-template-tile:hover {
  border-color: rgba(70, 110, 255, 0.22);
}

.editor-template-dot {
  width: 0.65rem;
  height: 0.65rem;
  border-radius: 999px;
  background: color-mix(in srgb, var(--app-accent) 60%, white);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--app-accent) 14%, transparent);
}
</style>
