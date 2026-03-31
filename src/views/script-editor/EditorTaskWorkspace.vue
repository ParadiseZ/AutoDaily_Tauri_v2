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

      <div v-if="activePanel === 'inputs'" class="min-h-0 overflow-y-auto pr-1 custom-scrollbar">
          <div v-if="selectedInputEntry" class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <div class="flex items-start justify-between gap-3">
              <div>
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">变量详情</p>
                <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ selectedInputEntry.key || '未设置键' }}</p>
              </div>
              <button class="app-button app-button-danger app-toolbar-button" type="button" @click="$emit('remove-input', selectedInputEntry.id)">
                删除变量
              </button>
            </div>

            <div class="mt-4 grid gap-3">
              <label class="space-y-2">
                <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">名称</span>
                <input
                  :value="selectedInputEntry.name"
                  class="app-input"
                  placeholder="例如：扫荡次数"
                  @input="$emit('update-input', selectedInputEntry.id, 'name', ($event.target as HTMLInputElement).value)"
                />
              </label>

              <label class="space-y-2">
                <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">键</span>
                <input
                  :value="selectedInputEntry.key"
                  class="app-input"
                  placeholder="例如：activitySweepCount"
                  :data-testid="selectedInputIndex === 0 ? 'editor-input-key-0' : undefined"
                  @input="$emit('update-input', selectedInputEntry.id, 'key', ($event.target as HTMLInputElement).value)"
                />
              </label>

              <div class="grid gap-3 md:grid-cols-2">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">类型</span>
                  <AppSelect
                    :model-value="selectedInputEntry.type"
                    :options="inputTypeOptions"
                    placeholder="选择类型"
                    :test-id="selectedInputIndex === 0 ? 'editor-input-type-0' : undefined"
                    @update:model-value="$emit('update-input', selectedInputEntry.id, 'type', String($event))"
                  />
                </label>

                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">作用域</span>
                  <AppSelect
                    :model-value="selectedInputEntry.namespace"
                    :options="scopeOptions"
                    placeholder="选择作用域"
                    @update:model-value="$emit('update-input', selectedInputEntry.id, 'namespace', String($event))"
                  />
                </label>
              </div>

              <label class="space-y-2">
                <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">备注</span>
                <input
                  :value="selectedInputEntry.description"
                  class="app-input"
                  placeholder="用于后续检索、绑定和变量引用"
                  @input="$emit('update-input', selectedInputEntry.id, 'description', ($event.target as HTMLInputElement).value)"
                />
              </label>

              <template v-if="selectedInputEntry.namespace === 'input'">
                <label v-if="selectedInputEntry.type === 'bool'" class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
                  <input
                    :checked="selectedInputEntry.booleanValue"
                    type="checkbox"
                    class="h-4 w-4"
                    :data-testid="selectedInputIndex === 0 ? 'editor-input-bool-0' : undefined"
                    style="accent-color: var(--app-accent)"
                    @change="$emit('update-input', selectedInputEntry.id, 'booleanValue', ($event.target as HTMLInputElement).checked)"
                  />
                  <span class="text-sm text-[var(--app-text-soft)]">默认启用</span>
                </label>

                <label v-else class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">默认值</span>
                  <textarea
                    v-if="selectedInputEntry.type === 'json'"
                    :value="selectedInputEntry.stringValue"
                    class="app-textarea min-h-[120px]"
                    spellcheck="false"
                    @input="$emit('update-input', selectedInputEntry.id, 'stringValue', ($event.target as HTMLTextAreaElement).value)"
                  />
                  <input
                    v-else
                    :value="selectedInputEntry.stringValue"
                    class="app-input"
                    :type="selectedInputEntry.type === 'string' ? 'text' : 'number'"
                    :data-testid="selectedInputIndex === 0 ? 'editor-input-value-0' : undefined"
                    @input="$emit('update-input', selectedInputEntry.id, 'stringValue', ($event.target as HTMLInputElement).value)"
                  />
                </label>
              </template>

              <div
                v-else
                class="rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4 text-sm leading-6 text-[var(--app-text-soft)]"
              >
                {{ selectedInputEntry.namespace === 'runtime' ? 'Runtime 变量只定义结构和来源，不在这里设置默认值。' : 'System 变量由运行时注入，只在这里保留元数据。' }}
              </div>
            </div>
          </div>

          <EmptyState
            v-else
            title="选择一个变量"
            description="中间列表选中变量后，右侧才会显示名称、键、类型、作用域和值。"
          />
      </div>

      <div v-else-if="activePanel === 'ui'" class="grid min-h-0 gap-4 xl:grid-rows-[auto_minmax(0,1fr)]">
        <div
          class="rounded-[22px] border border-[var(--app-border)] bg-[linear-gradient(160deg,rgba(255,255,255,0.92),rgba(243,247,255,0.9))] px-5 py-5 shadow-[var(--app-shadow-soft)]"
        >
          <div class="space-y-3">
            <div class="flex flex-wrap items-center gap-3">
              <label class="editor-ui-chip editor-ui-chip-static">
                <input type="checkbox" checked disabled />
                <span>启用</span>
              </label>
              <span class="editor-ui-task-name">{{ task.name }}</span>
              <button
                v-if="uiSchema.layout === 'vertical'"
                class="app-button app-button-ghost app-toolbar-button"
                type="button"
                @click="uiPreviewExpanded = !uiPreviewExpanded"
              >
                {{ uiPreviewExpanded ? '收起' : '展开' }}
              </button>
            </div>

            <div
              v-if="uiSchema.layout === 'horizontal' || uiPreviewExpanded"
              :class="uiSchema.layout === 'vertical' ? 'grid gap-3' : 'flex flex-wrap items-center gap-3'"
            >
              <button
                v-for="field in uiSchema.fields"
                :key="field.id"
                class="editor-ui-preview-item"
                :class="{ 'editor-ui-preview-item-active': selectedUiFieldId === field.id, 'editor-ui-preview-item-vertical': uiSchema.layout === 'vertical' }"
                type="button"
                @click="$emit('select-ui-field', field.id)"
              >
                <template v-if="field.control === 'checkbox'">
                  <input type="checkbox" :checked="Boolean(resolvePreviewValue(field))" disabled />
                  <span class="editor-ui-preview-text">{{ field.label || '未命名字段' }}</span>
                </template>

                <template v-else-if="field.control === 'number'">
                  <span v-if="field.label" class="editor-ui-preview-text">{{ field.label }}</span>
                  <span class="editor-ui-inline-value">{{ resolveNumberPreview(field) }}</span>
                </template>

                <template v-else-if="field.control === 'select'">
                  <span v-if="field.label" class="editor-ui-preview-text">{{ field.label }}</span>
                  <span class="editor-ui-inline-value editor-ui-inline-select">
                    <span>{{ resolveSelectPreview(field) }}</span>
                    <span class="editor-ui-inline-caret">v</span>
                  </span>
                </template>

                <template v-else-if="field.control === 'radio'">
                  <span v-if="field.label" class="editor-ui-preview-text">{{ field.label }}</span>
                  <span class="editor-ui-inline-options">
                    <span
                      v-for="option in parseFieldOptions(field)"
                      :key="option"
                      class="editor-ui-inline-pill"
                      :class="{ 'editor-ui-inline-pill-active': resolvePreviewValue(field) === option }"
                    >
                      {{ option }}
                    </span>
                  </span>
                </template>

                <template v-else>
                  <span class="editor-ui-inline-value">{{ resolveTextPreview(field) }}</span>
                </template>
              </button>
            </div>
          </div>

          <EmptyState
            v-if="!uiSchema.fields.length"
            title="还没有可预览的字段"
            description="在中间插入 checkbox、radio、number 等模板后，这里会按排布方向实时预览。"
          />
        </div>

        <div class="min-h-0 overflow-y-auto pr-1 custom-scrollbar">
            <div v-if="selectedUiField" class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
              <div class="flex items-start justify-between gap-3">
                <div>
                  <p class="text-sm font-semibold text-[var(--app-text-strong)]">字段详情</p>
                  <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ getUiControlLabel(selectedUiField.control) }}</p>
                </div>
                <button class="app-button app-button-danger app-toolbar-button" type="button" @click="$emit('remove-ui-field', selectedUiField.id)">
                  删除字段
                </button>
              </div>

              <div class="mt-4 grid gap-3">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">字段名</span>
                  <input
                    :value="selectedUiField.label"
                    class="app-input"
                    :data-testid="selectedUiFieldIndex === 0 ? 'editor-ui-field-label-0' : undefined"
                    @input="$emit('update-ui-field', selectedUiField.id, 'label', ($event.target as HTMLInputElement).value)"
                  />
                </label>

                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">绑定输入</span>
                  <AppSelect
                    :model-value="selectedUiField.variableId || null"
                    :options="bindOptions"
                    placeholder="未绑定"
                    :test-id="selectedUiFieldIndex === 0 ? 'editor-ui-field-bind-0' : undefined"
                    @update:model-value="selectUiBinding(selectedUiField.id, String($event ?? ''))"
                  />
                </label>

                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">说明</span>
                  <input
                    :value="selectedUiField.description"
                    class="app-input"
                    @input="$emit('update-ui-field', selectedUiField.id, 'description', ($event.target as HTMLInputElement).value)"
                  />
                </label>

                <label v-if="selectedUiField.control === 'text' || selectedUiField.control === 'number'" class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">占位提示</span>
                  <input
                    :value="selectedUiField.placeholder"
                    class="app-input"
                    @input="$emit('update-ui-field', selectedUiField.id, 'placeholder', ($event.target as HTMLInputElement).value)"
                  />
                </label>

                <label v-if="selectedUiField.control === 'radio' || selectedUiField.control === 'select'" class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">选项</span>
                  <textarea
                    :value="selectedUiField.optionsText"
                    class="app-textarea min-h-[100px]"
                    placeholder="每行一个选项"
                    @input="$emit('update-ui-field', selectedUiField.id, 'optionsText', ($event.target as HTMLTextAreaElement).value)"
                  />
                </label>
              </div>
            </div>

            <EmptyState
              v-else
              title="选择一个字段"
              description="点击中间字段列表或上方预览项，下面会切换到当前字段的可编辑内容。"
            />
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
import { computed, ref } from 'vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { Step } from '@/types/bindings/Step';
import EditorStepWorkspace from '@/views/script-editor/EditorStepWorkspace.vue';
import {
  getUiControlLabel,
  type EditorPanelId,
  type EditorUiField,
  type EditorUiSchema,
} from '@/views/script-editor/editorSchema';
import {
  buildInputJson,
  editorInputTypeOptions,
  type EditorInputEntry,
  type EditorVariableOption,
} from '@/views/script-editor/editorVariables';
import type { StepBranchPath, StepPath } from '@/views/script-editor/editorStepTree';

const props = defineProps<{
  task: ScriptTaskTable | null;
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

const uiPreviewExpanded = ref(true);
const emit = defineEmits<{
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
  'remove-input': [entryId: string];
  'select-input': [entryId: string];
  'select-ui-field': [fieldId: string];
  'update-ui-field': [fieldId: string, field: 'label' | 'key' | 'variableId' | 'inputKey' | 'description' | 'placeholder' | 'optionsText', value: string];
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

const inputTypeOptions = editorInputTypeOptions;
const scopeOptions = [
  { label: 'Input', value: 'input', description: '用户可配置并持久化的输入变量。' },
  { label: 'Runtime', value: 'runtime', description: '步骤执行过程中的运行时变量。' },
  { label: 'System', value: 'system', description: '运行时注入的只读系统变量。' },
];
const selectedInputEntry = computed(() => props.inputEntries.find((entry) => entry.id === props.selectedInputId) ?? null);
const selectedInputIndex = computed(() =>
  selectedInputEntry.value ? props.inputEntries.findIndex((entry) => entry.id === selectedInputEntry.value?.id) : -1,
);

const selectedUiField = computed(() => props.uiSchema.fields.find((field) => field.id === props.selectedUiFieldId) ?? null);
const selectedUiFieldIndex = computed(() =>
  selectedUiField.value ? props.uiSchema.fields.findIndex((field) => field.id === selectedUiField.value?.id) : -1,
);

const bindOptions = computed(() => [
  { label: '未绑定', value: null, description: '纯展示字段或说明文本。' },
  ...props.variableOptions
    .filter((entry) => entry.uiBindable)
    .map((entry) => ({
      label: entry.label || entry.key || '未命名输入',
      value: entry.id,
      description: `${entry.key} · ${entry.namespace} · ${entry.valueType}`,
    })),
]);

const selectUiBinding = (fieldId: string, variableId: string) => {
  const matched = props.variableOptions.find((item) => item.id === variableId) ?? null;
  emit('update-ui-field', fieldId, 'variableId', variableId);
  emit('update-ui-field', fieldId, 'inputKey', matched?.key.startsWith('input.') ? matched.key.slice('input.'.length) : '');
};

const findBoundInputEntry = (field: EditorUiField) => {
  if (field.variableId) {
    const byId = props.inputEntries.find((entry) => entry.id === field.variableId);
    if (byId) {
      return byId;
    }
  }

  return props.inputEntries.find((entry) => entry.key === field.inputKey) ?? null;
};

const resolvePreviewValue = (field: EditorUiField) => {
  try {
    const inputs = buildInputJson(props.inputEntries);
    const entry = findBoundInputEntry(field);
    return entry ? inputs[entry.key] ?? null : null;
  } catch {
    return null;
  }
};

const parseFieldOptions = (field: EditorUiField) =>
  field.optionsText
    .split('\n')
    .map((item) => item.trim())
    .filter(Boolean);

const resolveNumberPreview = (field: EditorUiField) => {
  const value = resolvePreviewValue(field);
  return value === null || value === undefined || value === '' ? '0' : String(value);
};

const resolveSelectPreview = (field: EditorUiField) => {
  const options = parseFieldOptions(field);
  const value = resolvePreviewValue(field);
  if (value !== null && value !== undefined && String(value).trim()) {
    return String(value);
  }
  return options[0] ?? '请选择';
};

const resolveTextPreview = (field: EditorUiField) => {
  const value = resolvePreviewValue(field);
  if (value !== null && value !== undefined && String(value).trim()) {
    return String(value);
  }
  return field.label || field.placeholder || field.description || '文本';
};
</script>

<style scoped>
.editor-ui-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.65rem;
  border-radius: 18px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.72);
  padding: 0.8rem 0.95rem;
  text-align: left;
  transition: border-color 0.16s ease, background 0.16s ease;
}

.editor-ui-chip:hover {
  border-color: rgba(70, 110, 255, 0.22);
}

.editor-ui-chip-active {
  border-color: var(--app-state-active-border);
  background: var(--app-state-active-bg);
}

.editor-ui-chip-static {
  background: rgba(255, 255, 255, 0.84);
}

.editor-ui-task-name {
  color: var(--app-text-strong);
  font-size: 1rem;
  font-weight: 600;
}

.editor-ui-preview-item {
  display: inline-flex;
  align-items: center;
  gap: 0.65rem;
  min-height: 44px;
  border-radius: 16px;
  border: 1px solid transparent;
  background: transparent;
  padding: 0.2rem 0.25rem;
  text-align: left;
  transition: border-color 0.16s ease, background 0.16s ease;
}

.editor-ui-preview-item:hover {
  border-color: rgba(70, 110, 255, 0.16);
  background: rgba(255, 255, 255, 0.4);
}

.editor-ui-preview-item-active {
  border-color: var(--app-state-active-border);
  background: var(--app-state-active-bg);
}

.editor-ui-preview-item-vertical {
  justify-content: flex-start;
  width: 100%;
}

.editor-ui-preview-text {
  color: var(--app-text-strong);
  font-size: 0.96rem;
  font-weight: 600;
}

.editor-ui-inline-value {
  min-width: 72px;
  border-radius: 12px;
  border: 1px solid var(--app-border);
  background: white;
  padding: 0.45rem 0.75rem;
  text-align: center;
  color: var(--app-text-strong);
}

.editor-ui-inline-select {
  display: inline-flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
  min-width: 120px;
}

.editor-ui-inline-caret {
  color: var(--app-text-faint);
  font-size: 0.78rem;
}

.editor-ui-inline-options {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.45rem;
}

.editor-ui-inline-pill {
  border-radius: 999px;
  border: 1px solid var(--app-border);
  background: white;
  padding: 0.3rem 0.7rem;
  font-size: 0.75rem;
  color: var(--app-text-soft);
}

.editor-ui-inline-pill-active {
  border-color: var(--app-state-active-border);
  background: var(--app-state-active-bg);
  color: var(--app-text-strong);
}

</style>
