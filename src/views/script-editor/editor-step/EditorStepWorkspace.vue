<template>
  <div class="grid min-h-0 gap-4 xl:grid-rows-[auto_minmax(0,1fr)]">
    <datalist v-if="variableSuggestions.length" :id="variableDatalistId">
      <option v-for="option in variableSuggestions" :key="option" :value="option" />
    </datalist>

    <EditorStepBreadcrumb
      :steps="steps"
      :active-branch-path="activeBranchPath"
      :selected-step-path="selectedStepPath"
      @navigate-branch="$emit('navigate-branch', $event)"
      @select-step-path="$emit('select-step-path', $event)"
    />

    <div class="grid min-h-0 gap-4 xl:grid-cols-[minmax(0,360px)_minmax(0,1fr)]">
      <div class="min-h-0 overflow-y-auto pr-1 custom-scrollbar">
        <EditorStepList
          v-if="currentContainerSteps.length"
          :steps="currentContainerSteps"
          :selected-index="currentSelectedIndex"
          @select="selectCurrentBranchStep"
          @remove="$emit('remove-step', $event)"
          @reorder="handleReorder"
        />

        <EmptyState
          v-else
          title="还没有步骤"
          description="当前层级还是空的。"
        />
      </div>

      <div class="min-h-0 overflow-y-auto pr-1 custom-scrollbar">
        <div v-if="selectedStep" class="space-y-4">
          <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <div class="grid gap-4 xl:grid-cols-[minmax(0,1fr)_minmax(260px,320px)] xl:items-start">
              <div class="min-w-0">
                <div class="flex flex-wrap items-center gap-2">
                  <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ describeStep(selectedStep) }}</p>
                  <span class="rounded-full bg-white/50 px-3 py-1 text-xs text-[var(--app-text-soft)]">{{ selectedStep.op }}</span>
                </div>
                <p v-if="describeStepMeta(selectedStep) !== describeStep(selectedStep)" class="mt-2 text-xs text-[var(--app-text-faint)]">
                  {{ describeStepMeta(selectedStep) }}
                </p>
              </div>

              <label class="space-y-2">
                <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">步骤标题</span>
                <input :value="selectedStep.label || ''" class="app-input" @input="updateStepLabel(($event.target as HTMLInputElement).value)" />
              </label>
            </div>
          </div>

          <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <div class="space-y-3">
              <EditorStepActionPanel
                v-if="selectedStep.op === STEP_OP.action && selectedAction"
                :selected-action="selectedAction"
                :action-exec-max="selectedStep.exec_max"
                :variable-datalist-id="variableDatalistId"
                :writable-catalog-variable-options="writableCatalogVariableOptions"
                :result-catalog-variable-options="resultCatalogVariableOptions"
                :label-index-options="labelIndexOptions"
                :label-select-placeholder="labelSelectPlaceholder"
                :label-select-hint="labelSelectHint"
                :selected-capture-output-target="selectedCaptureOutputTarget"
                :selected-capture-output-input-entry="selectedCaptureOutputInputEntry"
                :selected-action-input-target="selectedActionInputTarget"
                :task-reference-options="taskReferenceOptions"
                :policy-reference-options="policyReferenceOptions"
                :task-ui-variable-options="taskUiVariableOptions"
                :click-mode-options="clickModeOptions"
                :swipe-mode-options="swipeModeOptions"
                :create-variable="createVariable"
                :jump-to-variable="jumpToVariable"
                :create-policy="createPolicyTarget"
                :jump-to-policy="jumpToPolicyTarget"
                :create-task="createTaskTarget"
                :jump-to-task="jumpToTaskTarget"
                @update-input="(entryId, field, value) => updateInput?.(entryId, field, value)"
                @update-exec-max="updateActionExecMax"
                @update-field="updateActionField"
                @update-mode="updateActionMode"
                @update-point-field="updateActionPointField"
                @update-number-field="updateActionNumberField"
                @update-text-field="updateActionTextField"
                @create-variable="handleCreateActionVariable"
                @jump-to-variable="handleJumpToDataVariable"
                @create-policy-target="handleCreatePolicyTarget"
                @jump-policy-target="jumpToPolicyTarget"
                @create-drop-set-task="handleCreateDropSetTask"
                @jump-drop-set-task="jumpToTaskTarget"
              />

              <EditorStepFlowPanel
                v-else-if="selectedStep.op === STEP_OP.flowControl && selectedFlow"
                :selected-flow="selectedFlow"
                :flow-with-condition="flowWithCondition"
                :flow-condition="flowCondition"
                :has-else-branch="hasElseBranch"
                :branch-summary="flowBranchSummary"
                :flow-type-options="flowTypeOptions"
                :readable-catalog-variable-options="readableCatalogVariableOptions"
                :variable-input-entries="inputEntries"
                :variable-reference-options="variableOptions"
                :task-reference-options="taskReferenceOptions"
                :policy-reference-options="policyReferenceOptions"
                :policy-group-reference-options="policyGroupReferenceOptions"
                :policy-set-reference-options="policySetReferenceOptions"
                :create-reference="createReference"
                :jump-to-reference="jumpToReference"
                :create-variable="createVariable"
                :jump-to-variable="jumpToVariable"
                @update-input="(entryId, field, value) => updateInput?.(entryId, field, value)"
                @update-number-field="updateNumberField"
                @update-field="updateFlowField"
                @update-flow-type="updateFlowType"
                @update-flow-condition="updateFlowCondition"
                @toggle-else-branch="toggleElseBranch"
              />

              <EditorStepDataPanel
                v-else-if="selectedStep.op === STEP_OP.dataHanding && selectedData"
                :selected-data="selectedData"
                :selected-set-var-target="selectedSetVarTarget"
                :selected-set-var-input-entry="selectedSetVarInputEntry"
                :selected-get-var-target="selectedGetVarTarget"
                :selected-get-var-input-entry="selectedGetVarInputEntry"
                :selected-filter-input-target="selectedFilterInputTarget"
                :selected-filter-input-entry="selectedFilterInputEntry"
                :selected-filter-output-target="selectedFilterOutputTarget"
                :selected-filter-output-input-entry="selectedFilterOutputInputEntry"
                :selected-color-compare-input-target="selectedColorCompareInputTarget"
                :selected-color-compare-input-entry="selectedColorCompareInputEntry"
                :selected-color-compare-output-target="selectedColorCompareOutputTarget"
                :selected-color-compare-output-input-entry="selectedColorCompareOutputInputEntry"
                :selected-set-var-kind="selectedSetVarKind"
                :set-var-uses-expression="setVarUsesExpression"
                :set-var-can-switch-mode="setVarCanSwitchMode"
                :effective-set-var-kind="effectiveSetVarKind"
                :set-var-draft="setVarDraft"
                :get-var-has-default="getVarHasDefault"
                :get-var-draft="getVarDraft"
                :writable-catalog-variable-options="writableCatalogVariableOptions"
                :readable-catalog-variable-options="readableCatalogVariableOptions"
                :filter-mode-options="filterModeOptions"
                :color-compare-method-options="colorCompareMethodOptions"
                :filter-branch-target="filterBranchTarget"
                :variable-datalist-id="variableDatalistId"
                :create-variable="createVariable"
                :jump-to-variable="jumpToVariable"
                @update-input="(entryId, field, value) => updateInput?.(entryId, field, value)"
                @update-set-var-target="updateSetVarTarget"
                @update-set-var-mode="updateSetVarMode"
                @update-set-var-type="updateSetVarType"
                @update-set-var-text="updateSetVarText"
                @update-set-var-bool="updateSetVarBool"
                @update-data-field="updateDataField"
                @update-data-nullable-field="updateDataNullableField"
                @toggle-get-var-default="toggleGetVarDefault"
                @update-get-var-type="updateGetVarType"
                @update-get-var-text="updateGetVarText"
                @update-get-var-bool="updateGetVarBool"
                @update-filter-mode="updateFilterMode"
                @update-color-compare-channel="updateColorCompareChannel"
                @update-color-compare-threshold="updateColorCompareThreshold"
                @update-color-compare-method="updateColorCompareMethod"
                @update-color-compare-boolean="updateColorCompareBoolean"
                @create-variable="handleCreateDataVariable"
                @jump-to-variable="handleJumpToDataVariable"
                @navigate-branch="$emit('navigate-branch', $event)"
              />

              <EditorStepTaskControlPanel
                v-else-if="selectedStep.op === STEP_OP.taskControl && selectedTaskControl"
                :selected-task-control="selectedTaskControl"
                :task-control-type-options="taskControlTypeOptions"
                :state-target-type-options="stateTargetTypeOptions"
                :state-status-type-options="filteredStateStatusTypeOptions"
                :task-reference-options="taskReferenceOptions"
                :policy-reference-options="policyReferenceOptions"
                :create-reference="createReference"
                :jump-to-reference="jumpToReference"
                @update-type="updateTaskControlType"
                @update-target-type="updateTaskControlTargetType"
                @add-target-id="addTaskControlTargetId"
                @remove-target-id="removeTaskControlTargetId"
                @update-status-type="updateTaskControlStatusType"
                @update-status-value="updateTaskControlStatusValue"
              />

              <EditorStepVisionPanel
                v-else-if="selectedStep.op === STEP_OP.vision && selectedVision?.type === VISION_TYPE.visionSearch"
                :selected-vision="selectedVision"
                :variable-datalist-id="variableDatalistId"
                :writable-catalog-variable-options="writableCatalogVariableOptions"
                :selected-vision-output-target="selectedVisionOutputTarget"
                :selected-vision-output-input-entry="selectedVisionOutputInputEntry"
                :vision-branch-target="visionBranchTarget"
                :create-variable="createVariable"
                :jump-to-variable="jumpToVariable"
                @update-input="(entryId, field, value) => updateInput?.(entryId, field, value)"
                @update-field="updateVisionField"
                @update-rule="updateVisionRule"
                @create-variable="handleCreateVisionVariable"
                @jump-to-variable="handleJumpToDataVariable"
                @navigate-branch="$emit('navigate-branch', $event)"
              />

              <p v-else class="text-sm leading-6 text-[var(--app-text-soft)]">
                当前步骤暂未提供专用表单，必要时可从右上角打开底层结构调试。
              </p>
            </div>
          </div>

          <div v-if="branchTargets.length" class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <p class="text-sm font-semibold text-[var(--app-text-strong)]">可进入层级</p>
            <div class="mt-3 grid gap-3">
              <button
                v-for="target in branchTargets"
                :key="target.key"
                class="app-list-item"
                :class="{ 'app-list-item-active': isSameBranchPath(activeBranchPath, target.path) }"
                type="button"
                :data-testid="`editor-branch-${target.key}`"
                @click="$emit('navigate-branch', target.path)"
              >
                <div class="flex items-center justify-between gap-3">
                  <div class="min-w-0">
                    <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ target.label }}</p>
                    <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ target.count }} 个步骤</p>
                  </div>
                  <span class="text-xs text-[var(--app-text-faint)]">进入</span>
                </div>
              </button>
            </div>
          </div>
        </div>

        <EmptyState
          v-else
          title="选择一个步骤"
          description="右侧默认展示步骤概览，选中后可调整标题、关键字段和嵌套关系。"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import type { Action } from '@/types/bindings/Action';
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { DataHanding } from '@/types/bindings/DataHanding';
import type { SearchRule } from '@/types/bindings/SearchRule';
import type { FlowControl } from '@/types/bindings/FlowControl';
import type { TaskControl } from '@/types/bindings/TaskControl';
import type { Step } from '@/types/bindings/Step';
import type { VisionNode } from '@/types/bindings/VisionNode';
import type { EditorReferenceKind, EditorReferenceOption, EditorTaskUiVariableOption } from '@/views/script-editor/editorReferences';
import EditorStepActionPanel from '@/views/script-editor/editor-step/EditorStepActionPanel.vue';
import EditorStepBreadcrumb from '@/views/script-editor/editor-step/EditorStepBreadcrumb.vue';
import EditorStepDataPanel from '@/views/script-editor/editor-step/EditorStepDataPanel.vue';
import EditorStepFlowPanel from '@/views/script-editor/editor-step/EditorStepFlowPanel.vue';
import EditorStepList from '@/views/script-editor/editor-step/EditorStepList.vue';
import EditorStepTaskControlPanel from '@/views/script-editor/editor-step/EditorStepTaskControlPanel.vue';
import EditorStepVisionPanel from '@/views/script-editor/editor-step/EditorStepVisionPanel.vue';
import { createConditionNode } from '@/views/script-editor/editorCondition';
import {
  ACTION_MODE,
  ACTION_TYPE,
  COLOR_COMPARE_METHOD_TYPE,
  DATA_TYPE,
  FILTER_MODE_TYPE,
  FLOW_TYPE,
  STATE_STATUS_TYPE,
  STATE_TARGET_TYPE,
  STEP_OP,
  TASK_CONTROL_TYPE,
  VISION_TYPE,
} from '@/views/script-editor/editor-step/editorStepKinds';
import { describeStep, describeStepMeta } from '@/views/script-editor/editor-step/editorStepTemplates';
import {
  buildVarValue,
  parseVarValueDraft,
  type VarValueKind,
} from '@/views/script-editor/editorVarValue';
import {
  buildVariableCatalogKey,
  getInputTypeLabel,
  getVariableDisplayKey,
  getVariableValueTypeLabel,
  type EditorInputEntry,
  type EditorInputType,
  type EditorVariableOption,
} from '@/views/script-editor/editorVariables';
import {
  buildStepPath,
  getBranchSteps,
  getParentBranchPath,
  getStepByPath,
  isSameBranchPath,
  type StepBranchPath,
  type StepPath,
} from '@/views/script-editor/editor-step/editorStepTree';
import { cloneJson } from '@/views/script-editor/editorSchema';

type NestedGroupKey = 'sequence' | 'then' | 'else' | 'flow' | 'visionThen' | 'filterThen';

const props = withDefaults(
  defineProps<{
    steps: Step[];
    selectedStepPath: StepPath | null;
    activeBranchPath: StepBranchPath;
    inputEntries?: EditorInputEntry[];
    variableOptions: EditorVariableOption[];
    catalogVariableOptions: EditorVariableOption[];
    labelIndexOptions?: Array<{ label: string; value: number; description?: string; disabled?: boolean }>;
    labelSelectPlaceholder?: string;
    labelSelectHint?: string | null;
    taskReferenceOptions: EditorReferenceOption[];
    policyReferenceOptions: EditorReferenceOption[];
    taskUiVariableOptions?: EditorTaskUiVariableOption[];
    policyGroupReferenceOptions: EditorReferenceOption[];
    policySetReferenceOptions: EditorReferenceOption[];
    createReference: (kind: EditorReferenceKind) => Promise<string>;
    jumpToReference: (kind: EditorReferenceKind, id: string) => void;
    createVariable?: (
      namespace?: 'input' | 'runtime',
      inputType?: EditorInputType,
      options?: { preferredKey?: string; name?: string; select?: boolean; silent?: boolean },
    ) => Promise<string>;
    jumpToVariable?: (option: EditorVariableOption) => void;
    updateInput?: (
      entryId: string,
      field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue',
      value: string | boolean,
    ) => void;
  }>(),
  {
    inputEntries: () => [],
    labelIndexOptions: () => [],
    labelSelectPlaceholder: '请先设置文字检测模型标签文件',
    labelSelectHint: null,
  },
);

const emit = defineEmits<{
  'select-step-path': [path: StepPath];
  'navigate-branch': [branchPath: StepBranchPath];
  'reorder-step': [from: number, to: number];
  'remove-step': [index: number];
  'update-step': [index: number, step: Step];
}>();

const variableDatalistId = 'editor-variable-suggestions';
const variableSuggestions = computed(() =>
  Array.from(new Set(props.variableOptions.map((item) => item.key).filter(Boolean))),
);
type VariableSelectOption = {
  label: string;
  value: string;
  description: string;
  disabled?: boolean;
};
const formatVariableScopeLabel = (namespace: EditorVariableOption['namespace']) => {
  if (namespace === 'runtime') return 'Runtime';
  if (namespace === 'system') return 'System';
  return 'Input';
};
const createVariableSelectOption = (item: EditorVariableOption): VariableSelectOption => {
  const keyLabel = getVariableDisplayKey(item.key, item.namespace);
  const shouldShowKey = keyLabel && keyLabel !== item.label;
  const parts = [shouldShowKey ? keyLabel : null, formatVariableScopeLabel(item.namespace), getVariableValueTypeLabel(item.valueType)].filter(Boolean);
  return {
    label: item.label || item.key,
    value: item.key,
    description: parts.join(' · '),
  };
};
const createDraftVariableSelectOption = (
  entry: EditorInputEntry,
  capability: 'read' | 'write',
): VariableSelectOption | null => {
  if (capability === 'write' && entry.namespace === 'system') {
    return null;
  }

  const trimmedKey = entry.key.trim();
  const hasKey = Boolean(trimmedKey);
  const parts = [
    hasKey && entry.name && entry.name !== trimmedKey ? trimmedKey : !hasKey ? '未设置键' : null,
    formatVariableScopeLabel(entry.namespace),
    getInputTypeLabel(entry.type),
    !hasKey ? '需先填写键' : null,
  ].filter(Boolean);

  return {
    label: entry.name || trimmedKey || '未命名变量',
    value: hasKey ? buildVariableCatalogKey(trimmedKey, entry.namespace) : `__draft__${entry.id}`,
    description: parts.join(' · '),
    disabled: !hasKey,
  };
};
const buildVariableSelectOptions = (capability: 'read' | 'write') => {
  const options = new Map<string, VariableSelectOption>();

  for (const entry of props.inputEntries) {
    const option = createDraftVariableSelectOption(entry, capability);
    if (!option) {
      continue;
    }
    options.set(option.value, option);
  }

  for (const item of props.variableOptions.filter((option) => (capability === 'write' ? option.writable : option.readable))) {
    if (!options.has(item.key)) {
      options.set(item.key, createVariableSelectOption(item));
    }
  }

  return Array.from(options.values());
};
const readableCatalogVariableOptions = computed(() => buildVariableSelectOptions('read'));
const writableCatalogVariableOptions = computed(() => buildVariableSelectOptions('write'));
const resultCatalogVariableOptions = computed(() =>
  props.variableOptions
    .filter((item) => item.readable && ['json', 'list', 'object'].includes(item.valueType))
    .map((item) => createVariableSelectOption(item)),
);

const clickModeOptions = [
  { label: '坐标', value: ACTION_MODE.point, description: '绝对坐标点击。' },
  { label: '百分比', value: ACTION_MODE.percent, description: '相对坐标点击。' },
  { label: '文字', value: ACTION_MODE.txt, description: '按 OCR 文本点击。' },
  { label: '标签', value: ACTION_MODE.labelIdx, description: '按视觉标签点击。' },
];

const swipeModeOptions = [
  { label: '坐标', value: ACTION_MODE.point, description: '绝对坐标滑动。' },
  { label: '百分比', value: ACTION_MODE.percent, description: '相对坐标滑动。' },
  { label: '文字', value: ACTION_MODE.txt, description: '按 OCR 文本滑动。' },
  { label: '标签', value: ACTION_MODE.labelIdx, description: '按视觉标签滑动。' },
];

const flowTypeOptions = [
  { label: '条件分支', value: FLOW_TYPE.if, description: 'Then / Else 分支。' },
  { label: 'While', value: FLOW_TYPE.while, description: '满足条件时循环。' },
  { label: '遍历循环', value: FLOW_TYPE.forEach, description: '遍历输入集合，并向子步骤暴露元素变量。' },
];

const filterModeOptions = [
  { label: '过滤', value: FILTER_MODE_TYPE.filter, description: '保留符合条件的元素。' },
  { label: '映射', value: FILTER_MODE_TYPE.map, description: '将输入映射为新结果。' },
];
const colorCompareMethodOptions = [
  { label: 'OKLab 距离', value: COLOR_COMPARE_METHOD_TYPE.oklabDistance, description: '在 OKLab 空间比较颜色距离。' },
];
const taskControlTypeOptions = [
  { label: '设置状态', value: TASK_CONTROL_TYPE.setState, description: '写入目标状态。' },
];

const stateTargetTypeOptions = [
  { label: '任务', value: STATE_TARGET_TYPE.task, description: '针对任务状态。' },
  { label: '策略', value: STATE_TARGET_TYPE.policy, description: '针对策略状态。' },
];

const stateStatusTypeOptions = [
  { label: '启用', value: STATE_STATUS_TYPE.enabled, description: 'enabled 状态。' },
  { label: '完成', value: STATE_STATUS_TYPE.done, description: 'done 状态。' },
  { label: '跳过', value: STATE_STATUS_TYPE.skip, description: 'skip 状态。' },
];

const currentContainerSteps = computed(() => getBranchSteps(props.steps, props.activeBranchPath));
const selectedStep = computed(() => getStepByPath(props.steps, props.selectedStepPath));
const currentSelectedIndex = computed(() => {
  if (!props.selectedStepPath?.length) return null;
  const branchPath = getParentBranchPath(props.selectedStepPath);
  if (!isSameBranchPath(branchPath, props.activeBranchPath)) return null;
  return props.selectedStepPath[props.selectedStepPath.length - 1]?.index ?? null;
});

const selectedAction = computed<Action | null>(() => (selectedStep.value?.op === STEP_OP.action ? selectedStep.value.a : null));
const selectedFlow = computed<FlowControl | null>(() => (selectedStep.value?.op === STEP_OP.flowControl ? selectedStep.value.a : null));
const selectedData = computed<DataHanding | null>(() => (selectedStep.value?.op === STEP_OP.dataHanding ? selectedStep.value.a : null));
const selectedTaskControl = computed<TaskControl | null>(() => (selectedStep.value?.op === STEP_OP.taskControl ? selectedStep.value.a : null));
const selectedVision = computed<VisionNode | null>(() => (selectedStep.value?.op === STEP_OP.vision ? selectedStep.value.a : null));
const filteredStateStatusTypeOptions = computed(() =>
  selectedTaskControl.value?.target.type !== STATE_TARGET_TYPE.task
    ? stateStatusTypeOptions.filter((option) => option.value !== STATE_STATUS_TYPE.enabled)
    : stateStatusTypeOptions,
);
const setVarKindPreference = ref<VarValueKind | null>(null);
const getVarKindPreference = ref<VarValueKind | null>(null);
const setVarDraft = computed(() =>
  selectedData.value?.type === DATA_TYPE.setVar
    ? parseVarValueDraft(selectedData.value.val, setVarKindPreference.value ?? undefined)
    : parseVarValueDraft(''),
);
const getVarHasDefault = computed(() => Boolean(selectedData.value?.type === DATA_TYPE.getVar && selectedData.value.default_val !== null));
const getVarDraft = computed(() =>
  selectedData.value?.type === DATA_TYPE.getVar
    ? parseVarValueDraft(selectedData.value.default_val, getVarKindPreference.value ?? undefined)
    : parseVarValueDraft(''),
);
const mapVariableTypeToVarKind = (valueType: EditorVariableOption['valueType']): VarValueKind | null => {
  switch (valueType) {
    case 'int':
      return 'int';
    case 'float':
      return 'float';
    case 'bool':
      return 'bool';
    case 'string':
      return 'string';
    default:
      return null;
  }
};
const createDefaultVarValueDraft = (kind: VarValueKind) =>
  parseVarValueDraft(kind === 'string' ? '' : kind === 'bool' ? false : 0, kind);
const currentSetVarName = computed(() =>
  selectedData.value?.type === DATA_TYPE.setVar ? selectedData.value.name : '',
);
const currentGetVarName = computed(() =>
  selectedData.value?.type === DATA_TYPE.getVar ? selectedData.value.name : '',
);
const selectedSetVarTarget = computed(() =>
  currentSetVarName.value ? props.variableOptions.find((item) => item.key === currentSetVarName.value) ?? null : null,
);
const selectedGetVarTarget = computed(() =>
  currentGetVarName.value ? props.variableOptions.find((item) => item.key === currentGetVarName.value) ?? null : null,
);
const findInputEntryByVariableKey = (key: string) =>
  props.inputEntries.find((entry) => buildVariableCatalogKey(entry.key, entry.namespace) === key) ?? null;
const selectedSetVarInputEntry = computed(() => (selectedSetVarTarget.value ? findInputEntryByVariableKey(selectedSetVarTarget.value.key) : null));
const selectedGetVarInputEntry = computed(() => (selectedGetVarTarget.value ? findInputEntryByVariableKey(selectedGetVarTarget.value.key) : null));
const currentCaptureOutputName = computed(() =>
  selectedAction.value?.ac === ACTION_TYPE.capture ? selectedAction.value.output_var ?? '' : '',
);
const currentActionInputName = computed(() => {
  if (!selectedAction.value) {
    return '';
  }

  if (selectedAction.value.ac !== ACTION_TYPE.click && selectedAction.value.ac !== ACTION_TYPE.swipe) {
    return '';
  }

  return selectedAction.value.mode === ACTION_MODE.txt || selectedAction.value.mode === ACTION_MODE.labelIdx
    ? (selectedAction.value as { input_var?: string }).input_var ?? ''
    : '';
});
const selectedCaptureOutputTarget = computed(() =>
  currentCaptureOutputName.value ? props.variableOptions.find((item) => item.key === currentCaptureOutputName.value) ?? null : null,
);
const selectedCaptureOutputInputEntry = computed(() =>
  selectedCaptureOutputTarget.value ? findInputEntryByVariableKey(selectedCaptureOutputTarget.value.key) : null,
);
const selectedActionInputTarget = computed(() =>
  currentActionInputName.value ? props.variableOptions.find((item) => item.key === currentActionInputName.value) ?? null : null,
);
const currentFilterInputName = computed(() => (selectedData.value?.type === DATA_TYPE.filter ? selectedData.value.input_var : ''));
const currentFilterOutputName = computed(() => (selectedData.value?.type === DATA_TYPE.filter ? selectedData.value.out_name : ''));
const currentColorCompareInputName = computed(() =>
  selectedData.value?.type === DATA_TYPE.colorCompare ? selectedData.value.input_var : '',
);
const currentColorCompareOutputName = computed(() =>
  selectedData.value?.type === DATA_TYPE.colorCompare ? selectedData.value.out_var : '',
);
const selectedFilterInputTarget = computed(() =>
  currentFilterInputName.value ? props.variableOptions.find((item) => item.key === currentFilterInputName.value) ?? null : null,
);
const selectedFilterOutputTarget = computed(() =>
  currentFilterOutputName.value ? props.variableOptions.find((item) => item.key === currentFilterOutputName.value) ?? null : null,
);
const selectedFilterInputEntry = computed(() => (selectedFilterInputTarget.value ? findInputEntryByVariableKey(selectedFilterInputTarget.value.key) : null));
const selectedFilterOutputInputEntry = computed(() =>
  selectedFilterOutputTarget.value ? findInputEntryByVariableKey(selectedFilterOutputTarget.value.key) : null,
);
const selectedColorCompareInputTarget = computed(() =>
  currentColorCompareInputName.value ? props.variableOptions.find((item) => item.key === currentColorCompareInputName.value) ?? null : null,
);
const selectedColorCompareOutputTarget = computed(() =>
  currentColorCompareOutputName.value ? props.variableOptions.find((item) => item.key === currentColorCompareOutputName.value) ?? null : null,
);
const selectedColorCompareInputEntry = computed(() =>
  selectedColorCompareInputTarget.value ? findInputEntryByVariableKey(selectedColorCompareInputTarget.value.key) : null,
);
const selectedColorCompareOutputInputEntry = computed(() =>
  selectedColorCompareOutputTarget.value ? findInputEntryByVariableKey(selectedColorCompareOutputTarget.value.key) : null,
);
const currentVisionOutputName = computed(() =>
  selectedVision.value?.type === VISION_TYPE.visionSearch ? selectedVision.value.out_var ?? '' : '',
);
const selectedVisionOutputTarget = computed(() =>
  currentVisionOutputName.value ? props.variableOptions.find((item) => item.key === currentVisionOutputName.value) ?? null : null,
);
const selectedVisionOutputInputEntry = computed(() =>
  selectedVisionOutputTarget.value ? findInputEntryByVariableKey(selectedVisionOutputTarget.value.key) : null,
);
const selectedSetVarKind = computed(() => (selectedSetVarTarget.value ? mapVariableTypeToVarKind(selectedSetVarTarget.value.valueType) : null));
const setVarUsesExpression = computed(() => {
  if (selectedData.value?.type !== DATA_TYPE.setVar) {
    return false;
  }

  if (selectedSetVarTarget.value && !selectedSetVarKind.value) {
    return true;
  }

  return Boolean(selectedData.value.expr);
});
const effectiveSetVarKind = computed<VarValueKind>(() => selectedSetVarKind.value ?? setVarDraft.value.kind);
const setVarCanSwitchMode = computed(() => Boolean(selectedSetVarTarget.value && selectedSetVarKind.value));

watch(
  () => props.selectedStepPath?.map((segment) => `${segment.branch}:${segment.index}`).join('/') ?? '',
  () => {
    setVarKindPreference.value = null;
    getVarKindPreference.value = null;
  },
  { immediate: true },
);

const flowWithCondition = computed(() => {
  if (!selectedFlow.value) return null;
  const type = selectedFlow.value.type;
  if ((type === FLOW_TYPE.if || type === FLOW_TYPE.while) && selectedFlow.value.con) {
    return {
      type,
      con: selectedFlow.value.con,
    };
  }
  return null;
});

const flowCondition = computed<ConditionNode | null>(() => flowWithCondition.value?.con ?? null);
const hasElseBranch = computed(() => Boolean(selectedFlow.value?.type === FLOW_TYPE.if && selectedFlow.value.else_steps));
const flowBranchSummary = computed(() => {
  if (!flowWithCondition.value || !selectedFlow.value) return '';
  return flowWithCondition.value.type === FLOW_TYPE.if
    ? `Then ${(((selectedFlow.value as { then?: unknown[] } | null)?.then)?.length ?? 0)} · Else ${(((selectedFlow.value as { else_steps?: unknown[] } | null)?.else_steps)?.length ?? 0)}`
    : `循环 ${(((selectedFlow.value as { flow?: unknown[] } | null)?.flow)?.length ?? 0)} 步`;
});

const branchTargets = computed<Array<{ key: NestedGroupKey; label: string; count: number; path: StepBranchPath }>>(() => {
  if (!selectedStep.value || !props.selectedStepPath) return [];

  if (selectedStep.value.op === STEP_OP.sequence) {
    return [{ key: 'sequence', label: '顺序步骤', count: selectedStep.value.steps.length, path: { parentStepPath: props.selectedStepPath, branch: 'sequence' } }];
  }

  if (selectedFlow.value?.type === FLOW_TYPE.if) {
      const targets: Array<{ key: NestedGroupKey; label: string; count: number; path: StepBranchPath }> = [
        { key: 'then', label: 'Then', count: selectedFlow.value.then.length, path: { parentStepPath: props.selectedStepPath, branch: 'then' } },
      ];
      if (selectedFlow.value.else_steps) {
        targets.push({ key: 'else', label: 'Else', count: selectedFlow.value.else_steps.length, path: { parentStepPath: props.selectedStepPath, branch: 'else' } });
      }
      return targets;
  }

  if (selectedFlow.value?.type === FLOW_TYPE.while || selectedFlow.value?.type === FLOW_TYPE.forEach) {
    return [{ key: 'flow', label: '循环体', count: selectedFlow.value.flow.length, path: { parentStepPath: props.selectedStepPath, branch: 'flow' } }];
  }

  if (selectedVision.value?.type === VISION_TYPE.visionSearch) {
    return [{ key: 'visionThen', label: '命中后执行', count: selectedVision.value.then_steps.length, path: { parentStepPath: props.selectedStepPath, branch: 'visionThen' } }];
  }

  if (selectedData.value?.type === DATA_TYPE.filter) {
    return [{ key: 'filterThen', label: '过滤命中后', count: selectedData.value.then_steps.length, path: { parentStepPath: props.selectedStepPath, branch: 'filterThen' } }];
  }

  return [];
});
const visionBranchTarget = computed(() => branchTargets.value.find((target) => target.key === 'visionThen') ?? null);
const filterBranchTarget = computed(() => branchTargets.value.find((target) => target.key === 'filterThen') ?? null);

const selectCurrentBranchStep = (index: number) => {
  emit('select-step-path', buildStepPath(props.activeBranchPath, index));
};

const handleReorder = (from: number, to: number) => {
  emit('reorder-step', from, to);
};

const updateSelectedStep = (mutator: (step: Step) => void) => {
  if (currentSelectedIndex.value === null || !selectedStep.value) return;
  const nextStep = cloneJson(selectedStep.value) as Step;
  mutator(nextStep);
  emit('update-step', currentSelectedIndex.value, nextStep);
};

const updateStepLabel = (value: string) => {
  updateSelectedStep((step) => {
    step.label = value;
  });
};

const updateActionField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action) return;
    step.a = { ...(step.a ?? {}), [field]: value } as Action;
  });
};

const updateActionExecMax = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action) return;
    step.exec_max = Math.max(0, Number(value) || 0);
  });
};

const createClickAction = (mode: string): Action => {
  switch (mode) {
    case ACTION_MODE.percent:
      return { ac: ACTION_TYPE.click, mode: ACTION_MODE.percent, p: { x: 0.5, y: 0.5 } };
    case ACTION_MODE.txt:
      return { ac: ACTION_TYPE.click, mode: ACTION_MODE.txt, input_var: currentActionInputName.value || 'runtime.ocrResults', txt: '开始' };
    case ACTION_MODE.labelIdx:
      return { ac: ACTION_TYPE.click, mode: ACTION_MODE.labelIdx, input_var: currentActionInputName.value || 'runtime.detResults', idx: 0 };
    default:
      return { ac: ACTION_TYPE.click, mode: ACTION_MODE.point, p: { x: 640, y: 360 } };
  }
};

const createSwipeAction = (mode: string): Action => {
  switch (mode) {
    case ACTION_MODE.percent:
      return {
        ac: ACTION_TYPE.swipe,
        mode: ACTION_MODE.percent,
        duration: 300 as never,
        from: { x: 0.5, y: 0.75 },
        to: { x: 0.5, y: 0.25 },
      };
    case ACTION_MODE.txt:
      return {
        ac: ACTION_TYPE.swipe,
        mode: ACTION_MODE.txt,
        duration: 300 as never,
        input_var: currentActionInputName.value || 'runtime.ocrResults',
        from: '开始',
        to: '结束',
      };
    case ACTION_MODE.labelIdx:
      return {
        ac: ACTION_TYPE.swipe,
        mode: ACTION_MODE.labelIdx,
        duration: 300 as never,
        input_var: currentActionInputName.value || 'runtime.detResults',
        from: 0,
        to: 1,
      };
    default:
      return {
        ac: ACTION_TYPE.swipe,
        mode: ACTION_MODE.point,
        duration: 300 as never,
        from: { x: 640, y: 560 },
        to: { x: 640, y: 180 },
      };
  }
};

const updateActionModel = (value: Action) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action) return;
    step.a = value;
  });
};

const updateActionMode = (mode: string) => {
  if (!selectedAction.value) return;
  if (selectedAction.value.ac === ACTION_TYPE.click) {
    updateActionModel(createClickAction(mode));
    return;
  }
  if (selectedAction.value.ac === ACTION_TYPE.swipe) {
    updateActionModel(createSwipeAction(mode));
  }
};

const toNumber = (value: string) => {
  const next = Number(value);
  return Number.isFinite(next) ? next : 0;
};

const updateActionPointField = (field: 'p' | 'from' | 'to', axis: 'x' | 'y', value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action) return;
    const action = step.a as Record<string, unknown>;
    const point = { ...((action[field] as Record<string, number> | undefined) ?? { x: 0, y: 0 }) };
    point[axis] = toNumber(value);
    step.a = { ...action, [field]: point } as Action;
  });
};

const updateActionNumberField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action) return;
    step.a = { ...(step.a as Record<string, unknown>), [field]: toNumber(value) } as Action;
  });
};

const createPolicyTarget = async () => props.createReference('policy');

const jumpToPolicyTarget = (id: string) => {
  if (id) {
    props.jumpToReference('policy', id);
  }
};

const handleCreatePolicyTarget = async () => {
  const id = await createPolicyTarget();
  updateActionField('target', id);
};

const createTaskTarget = async () => props.createReference('task');

const jumpToTaskTarget = (id: string) => {
  if (id) {
    props.jumpToReference('task', id);
  }
};

const handleCreateDropSetTask = async () => {
  const id = await createTaskTarget();
  updateActionField('task', id);
};

const updateActionTextField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action) return;
    step.a = { ...(step.a as Record<string, unknown>), [field]: value } as Action;
  });
};

const updateFlowField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl) return;
    if ((step.a.type === FLOW_TYPE.handlePolicySet || step.a.type === FLOW_TYPE.handlePolicy) && field === 'target') {
      let nextTarget: string[] = [];
      try {
        nextTarget = JSON.parse(value) as string[];
      } catch {
        nextTarget = [];
      }
      step.a = { ...step.a, target: nextTarget } as FlowControl;
      return;
    }

    step.a = { ...(step.a ?? {}), [field]: value } as FlowControl;
  });
};

const updateDataField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding) return;
    step.a = { ...(step.a ?? {}), [field]: value } as DataHanding;
  });
};

const updateDataNullableField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding) return;
    step.a = { ...(step.a ?? {}), [field]: value.trim() ? value : null } as DataHanding;
  });
};

const getSetVarDraftForKind = (kind: VarValueKind) => (setVarDraft.value.kind === kind ? setVarDraft.value : createDefaultVarValueDraft(kind));

const updateSetVarTarget = (value: string) => {
  const matched = props.variableOptions.find((item) => item.key === value) ?? null;
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.setVar) return;

    const nextKind = matched ? mapVariableTypeToVarKind(matched.valueType) : null;
    const nextExpr = matched && !nextKind ? step.a.expr ?? '' : step.a.expr;
    const nextVal = nextKind ? buildVarValue(getSetVarDraftForKind(nextKind)) : nextKind === null && matched ? null : step.a.val;

    step.a = {
      ...step.a,
      name: value,
      val: nextExpr ? null : nextVal,
      expr: matched && !nextKind ? nextExpr : step.a.expr,
    };
  });
};

const handleCreateDataVariable = async (
  target: 'setVar' | 'getVar' | 'filterInput' | 'filterOutput' | 'colorCompareInput' | 'colorCompareOutput',
) => {
  if (!props.createVariable) {
    return;
  }

  const key =
    target === 'filterOutput' || target === 'colorCompareOutput'
      ? await props.createVariable('runtime', 'json')
      : target === 'colorCompareInput'
        ? await props.createVariable('runtime', 'json')
        : target === 'filterInput'
        ? await props.createVariable('input', 'json')
        : await props.createVariable('input', 'int');
  if (!key) {
    return;
  }

  if (target === 'setVar') {
    updateSetVarTarget(key);
    return;
  }

  if (target === 'filterInput') {
    updateDataField('input_var', key);
    return;
  }

  if (target === 'filterOutput') {
    updateDataField('out_name', key);
    return;
  }

  if (target === 'colorCompareInput') {
    updateDataField('input_var', key);
    return;
  }

  if (target === 'colorCompareOutput') {
    updateDataField('out_var', key);
    return;
  }

  updateDataField('name', key);
};

const handleJumpToDataVariable = (option: EditorVariableOption) => {
  props.jumpToVariable?.(option);
};

const handleCreateActionVariable = async (target: 'captureOutput' | 'actionInput') => {
  if (!props.createVariable) {
    return;
  }

  if (target === 'actionInput') {
    const preferredMode =
      selectedAction.value?.ac === ACTION_TYPE.click || selectedAction.value?.ac === ACTION_TYPE.swipe
        ? selectedAction.value.mode
        : null;
    const key = await props.createVariable('runtime', 'json', {
      preferredKey: preferredMode === ACTION_MODE.labelIdx ? 'detResults' : 'ocrResults',
      name: preferredMode === ACTION_MODE.labelIdx ? '检测结果' : 'OCR结果',
    });
    if (!key) {
      return;
    }

    updateActionField('input_var', key);
    return;
  }

  if (target !== 'captureOutput') {
    return;
  }

  const key = await props.createVariable('runtime', 'image');
  if (!key) {
    return;
  }

  updateActionField('output_var', key);
};

const updateSetVarMode = (mode: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.setVar) return;
    const forcedKind = selectedSetVarKind.value;
    const nextDraft = forcedKind ? getSetVarDraftForKind(forcedKind) : setVarDraft.value;
    step.a = {
      ...step.a,
      val: mode === 'expr' ? null : buildVarValue(nextDraft),
      expr: mode === 'expr' ? (step.a.expr || 'true') : null,
    };
  });
};

const updateSetVarType = (kind: string) => {
  setVarKindPreference.value = kind as VarValueKind;
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.setVar) return;
    const nextDraft = createDefaultVarValueDraft(kind as VarValueKind);
    step.a = {
      ...step.a,
      val: buildVarValue(nextDraft),
      expr: null,
    };
  });
};

const updateSetVarText = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.setVar) return;
    const nextKind = effectiveSetVarKind.value;
    step.a = {
      ...step.a,
      val: buildVarValue({
        ...getSetVarDraftForKind(nextKind),
        textValue: value,
      }),
      expr: null,
    };
  });
};

const updateSetVarBool = (value: boolean) => {
  setVarKindPreference.value = 'bool';
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.setVar) return;
    step.a = {
      ...step.a,
      val: buildVarValue({
        ...getSetVarDraftForKind('bool'),
        kind: 'bool',
        boolValue: value,
        textValue: value ? 'true' : 'false',
      }),
      expr: null,
    };
  });
};

const toggleGetVarDefault = (enabled: boolean) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.getVar) return;
    step.a = {
      ...step.a,
      default_val: enabled ? buildVarValue(getVarDraft.value) : null,
    };
  });
};

const updateGetVarType = (kind: string) => {
  getVarKindPreference.value = kind as VarValueKind;
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.getVar) return;
    step.a = {
      ...step.a,
      default_val: buildVarValue({
        kind: kind as 'int' | 'float' | 'bool' | 'string',
        textValue: kind === 'string' ? '' : '0',
        boolValue: false,
      }),
    };
  });
};

const updateGetVarText = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.getVar) return;
    step.a = {
      ...step.a,
      default_val: buildVarValue({
        ...getVarDraft.value,
        textValue: value,
      }),
    };
  });
};

const updateGetVarBool = (value: boolean) => {
  getVarKindPreference.value = 'bool';
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.getVar) return;
    step.a = {
      ...step.a,
      default_val: buildVarValue({
        ...getVarDraft.value,
        kind: 'bool',
        boolValue: value,
        textValue: value ? 'true' : 'false',
      }),
    };
  });
};

const updateFilterMode = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.filter) return;
    step.a = {
      ...step.a,
      mode: {
        type: value as typeof FILTER_MODE_TYPE.filter | typeof FILTER_MODE_TYPE.map,
      },
    };
  });
};

const updateColorCompareChannel = (channel: 'r' | 'g' | 'b', value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.colorCompare) return;
    step.a = {
      ...step.a,
      target_color: {
        ...step.a.target_color,
        [channel]: Math.max(0, Math.min(255, Number(value) || 0)),
      },
    };
  });
};

const updateColorCompareThreshold = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.colorCompare) return;
    const threshold = Math.max(0, Number(value) || 0);
    step.a = {
      ...step.a,
      method: {
        ...step.a.method,
        threshold,
      },
    };
  });
};

const updateColorCompareMethod = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.colorCompare) return;
    step.a = {
      ...step.a,
      method: {
        type: value as typeof COLOR_COMPARE_METHOD_TYPE.oklabDistance,
        threshold: step.a.method.threshold,
      },
    };
  });
};

const updateColorCompareBoolean = (field: 'is_font', value: boolean) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.colorCompare) return;
    step.a = {
      ...step.a,
      [field]: value,
    };
  });
};

const updateTaskControlType = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    step.a = {
      ...step.a,
      type: value as typeof TASK_CONTROL_TYPE.setState,
    };
  });
};

const updateTaskControlTargetType = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    const nextTargetType = value as typeof STATE_TARGET_TYPE.task | typeof STATE_TARGET_TYPE.policy;
    step.a = {
      ...step.a,
      target: {
        type: nextTargetType,
        id: '',
      },
      targets: [],
      status:
        nextTargetType === STATE_TARGET_TYPE.policy && step.a.status.type === STATE_STATUS_TYPE.enabled
          ? {
              ...step.a.status,
              type: STATE_STATUS_TYPE.done,
            }
          : step.a.status,
    };
  });
};

const addTaskControlTargetId = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    const id = value.trim();
    if (!id) return;
    const currentTargets = step.a.targets?.length ? step.a.targets : step.a.target.id ? [step.a.target] : [];
    if (currentTargets.some((target) => target.id === id && target.type === step.a.target.type)) {
      return;
    }
    const nextTargets = [
      ...currentTargets,
      {
        type: step.a.target.type,
        id,
      },
    ];
    step.a = {
      ...step.a,
      target: nextTargets[0] ?? { ...step.a.target, id: '' },
      targets: nextTargets,
    };
  });
};

const removeTaskControlTargetId = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    const currentTargets = step.a.targets?.length ? step.a.targets : step.a.target.id ? [step.a.target] : [];
    const nextTargets = currentTargets.filter((target) => target.id !== value);
    step.a = {
      ...step.a,
      target: nextTargets[0] ?? { ...step.a.target, id: '' },
      targets: nextTargets,
    };
  });
};

const updateTaskControlStatusType = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    step.a = {
      ...step.a,
      status: {
        ...step.a.status,
        type: value as typeof STATE_STATUS_TYPE.enabled | typeof STATE_STATUS_TYPE.done | typeof STATE_STATUS_TYPE.skip,
      },
    };
  });
};

const updateTaskControlStatusValue = (value: boolean) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    step.a = {
      ...step.a,
      status: {
        ...step.a.status,
        value,
      },
    };
  });
};

const updateVisionField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.vision) return;
    step.a = { ...(step.a ?? {}), [field]: value } as VisionNode;
  });
};

const handleCreateVisionVariable = async (target: 'visionOutput') => {
  if (!props.createVariable) {
    return;
  }

  if (target !== 'visionOutput') {
    return;
  }

  const key = await props.createVariable('runtime', 'json');
  if (!key) {
    return;
  }

  updateVisionField('out_var', key);
};

const updateVisionRule = (rule: SearchRule) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.vision || step.a.type !== VISION_TYPE.visionSearch) return;
    step.a = {
      ...step.a,
      rule,
    };
  });
};

const updateNumberField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl) return;
    step.a = { ...(step.a ?? {}), [field]: Number(value) } as FlowControl;
  });
};

const updateFlowCondition = (condition: ConditionNode) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl) return;
    if (step.a.type === FLOW_TYPE.if || step.a.type === FLOW_TYPE.while) {
      step.a.con = condition;
    }
  });
};

const updateFlowType = (type: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl) return;
    const currentCondition = flowCondition.value ?? createConditionNode('rawExpr');
    if (type === FLOW_TYPE.if) {
      const flowSteps = step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.forEach ? step.a.flow : [];
      step.a = {
        type: FLOW_TYPE.if,
        con: currentCondition,
        then: flowSteps,
        else_steps: null,
      } as FlowControl;
      return;
    }

    if (type === FLOW_TYPE.while) {
      const branchSteps =
        step.a.type === FLOW_TYPE.if ? step.a.then : step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.forEach ? step.a.flow : [];
      step.a = {
        type: FLOW_TYPE.while,
        con: currentCondition,
        flow: branchSteps,
      } as FlowControl;
      return;
    }

    if (type === FLOW_TYPE.forEach) {
      const branchSteps =
        step.a.type === FLOW_TYPE.if ? step.a.then : step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.forEach ? step.a.flow : [];
      step.a = {
        type: FLOW_TYPE.forEach,
        input_var: 'runtime.items',
        item_var: 'runtime.item',
        index_var: 'runtime.itemIndex',
        flow: branchSteps,
      } as FlowControl;
    }
  });
};

const toggleElseBranch = () => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl || step.a.type !== FLOW_TYPE.if) return;
    step.a.else_steps = step.a.else_steps ? null : [];
  });
};
</script>
