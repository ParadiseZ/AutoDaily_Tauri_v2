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
          :task-reference-options="taskReferenceOptions"
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
        <EditorOverviewPanel v-if="selectedStep">
          <EditorOverviewSection title="步骤概览" heading-tag="h1" width="wide">
            <div class="step-badge-row">
              <span class="step-badge step-badge-strong">{{ selectedStepKindLabel }}</span>
              <span class="step-badge">{{ `当前层级第 ${currentSelectedIndexDisplay} 步` }}</span>
              <span class="step-badge">{{ `本层共 ${currentContainerSteps.length} 步` }}</span>
            </div>

            <EditorOverviewField label="步骤标题">
              <input :value="selectedStep.label || ''" class="app-input" @input="updateStepLabel(($event.target as HTMLInputElement).value)" />
            </EditorOverviewField>
          </EditorOverviewSection>

          <EditorOverviewSection
            :title="selectedStepConfigTitle"
            :description="selectedStepConfigDescription"
            width="wide"
          >
            <div class="space-y-3">
              <EditorStepActionPanel
                v-if="selectedStep.op === STEP_OP.action && selectedAction"
                :selected-action="selectedAction"
                :action-exec-max="selectedStep.exec_max"
                :variable-datalist-id="variableDatalistId"
                :writable-catalog-variable-options="writableCatalogVariableOptions"
                :result-catalog-variable-options="resultCatalogVariableOptions"
                :text-variable-options="textVariableOptions"
                :number-variable-options="numberVariableOptions"
                :label-index-options="labelIndexOptions"
                :label-select-placeholder="labelSelectPlaceholder"
                :label-select-hint="labelSelectHint"
                :selected-capture-output-target="selectedCaptureOutputTarget"
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
                @update-swipe-target-field="updateSwipeTargetField"
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
                :branch-targets="flowBranchTargets"
                :active-branch-path="activeBranchPath"
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
                @update-boolean-field="updateFlowBooleanField"
                @update-flow-type="updateFlowType"
                @update-flow-condition="updateFlowCondition"
                @toggle-else-branch="toggleElseBranch"
                @navigate-branch="$emit('navigate-branch', $event)"
              />

              <EditorStepDataPanel
                v-else-if="selectedStep.op === STEP_OP.dataHanding && selectedData"
                :selected-data="selectedData"
                :selected-set-var-target="selectedSetVarTarget"
                :selected-set-var-input-entry="selectedSetVarInputEntry"
                :selected-get-var-target="selectedGetVarTarget"
                :selected-get-var-input-entry="selectedGetVarInputEntry"
                :selected-filter-input-target="selectedFilterInputTarget"
                :selected-filter-output-target="selectedFilterOutputTarget"
                :selected-color-compare-input-target="selectedColorCompareInputTarget"
                :selected-color-compare-output-target="selectedColorCompareOutputTarget"
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
                @update-region-point="updateDataRegionPoint"
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
                v-else-if="selectedStep.op === STEP_OP.vision && selectedVision"
                :selected-vision="selectedVision"
                :variable-datalist-id="variableDatalistId"
                :writable-catalog-variable-options="writableCatalogVariableOptions"
                :readable-catalog-variable-options="readableCatalogVariableOptions"
                :label-index-options="labelIndexOptions"
                :label-select-placeholder="labelSelectPlaceholder"
                :label-select-hint="labelSelectHint"
                :selected-vision-input-target="selectedVisionInputTarget"
                :selected-vision-output-target="selectedVisionOutputTarget"
                :selected-vision-det-input-target="selectedVisionDetInputTarget"
                :selected-vision-ocr-input-target="selectedVisionOcrInputTarget"
                :selected-vision-det-output-target="selectedVisionDetOutputTarget"
                :selected-vision-ocr-output-target="selectedVisionOcrOutputTarget"
                :vision-branch-target="visionBranchTarget"
                :create-variable="createVariable"
                :jump-to-variable="jumpToVariable"
                @update-field="updateVisionField"
                @update-nullable-field="updateVisionNullableField"
                @update-number-field="updateVisionNumberField"
                @update-rule="updateVisionRule"
                @create-variable="handleCreateVisionVariable"
                @jump-to-variable="handleJumpToDataVariable"
                @navigate-branch="$emit('navigate-branch', $event)"
              />

              <div
                v-else-if="selectedStep.op === STEP_OP.sequence && sequenceBranchTarget"
                class="space-y-4 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4"
              >
                <div class="space-y-2">
                  <p class="text-sm font-semibold text-(--app-text-strong)">动作序列</p>
                  <p class="text-sm leading-6 text-(--app-text-soft)">
                    这里只用于收拢设备动作与显式等待。运行时会优先尝试合并为单条 ADB sequence，以减少多次下发带来的耗时。
                  </p>
                </div>
                <div class="rounded-[14px] border border-dashed border-(--app-border) px-4 py-4 text-sm text-(--app-text-soft)">
                  允许的子步骤：固定设备动作与固定等待。当前编辑器提供：启动应用、停止应用、固定坐标点击/滑动、百分比点击/滑动、返回、等待；不支持截图、重启、文字/标签定位，以及依赖变量的等待。
                </div>
                <button
                  class="app-button app-button-primary app-toolbar-button"
                  type="button"
                  data-testid="editor-branch-sequence"
                  @click="$emit('navigate-branch', sequenceBranchTarget.path)"
                >
                  编辑序列步骤
                  <span class="text-xs text-white/80">{{ sequenceBranchTarget.count }}</span>
                </button>
              </div>

              <p v-else class="text-sm leading-6 text-(--app-text-soft)">
                当前步骤暂未提供专用表单，必要时可从右上角打开底层结构调试。
              </p>
            </div>
          </EditorOverviewSection>
        </EditorOverviewPanel>

        <EmptyState
          v-else
          title="选择一个步骤"
          description="右侧会先给出步骤概览，再展示对应的配置表单。"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import type { Action } from '@/types/bindings/Action';
import type { CompareOp } from '@/types/bindings/CompareOp';
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { DataHanding } from '@/types/bindings/DataHanding';
import type { SearchRule } from '@/types/bindings/SearchRule';
import type { FlowControl } from '@/types/bindings/FlowControl';
import type { TaskControl } from '@/types/bindings/TaskControl';
import type { Step } from '@/types/bindings/Step';
import type { VisionNode } from '@/types/bindings/VisionNode';
import type { EditorReferenceKind, EditorReferenceOption, EditorTaskUiVariableOption } from '@/views/script-editor/editorReferences';
import EditorOverviewField from '@/views/script-editor/EditorOverviewField.vue';
import EditorOverviewPanel from '@/views/script-editor/EditorOverviewPanel.vue';
import EditorOverviewSection from '@/views/script-editor/EditorOverviewSection.vue';
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
  CONDITION_TYPE,
  createColorCompareMethod,
  createFilterMode,
  createStateStatus,
  createStateTarget,
  createStateTargetList,
  createStepList,
  createStringList,
  DATA_TYPE,
  FILTER_MODE_TYPE,
  FLOW_TYPE,
  STATE_STATUS_TYPE,
  STATE_TARGET_TYPE,
  STEP_OP,
  TASK_CONTROL_TYPE,
  VISION_TYPE,
} from '@/views/script-editor/editor-step/editorStepKinds';
//import { describeStep, describeStepMeta } from '@/views/script-editor/editor-step/editorStepTemplates';
import { describeStep } from '@/views/script-editor/editor-step/editorStepTemplates';
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
type EditableVisionNode = {
  type: VisionNode['type'];
  input_var?: string;
  out_var?: string;
  det_res_var?: string | null;
  ocr_res_var?: string | null;
  out_det_var?: string | null;
  out_ocr_var?: string | null;
  target_value?: string | null;
  op?: CompareOp;
  expected_count?: number;
  then_steps?: Step[];
  rule?: SearchRule;
};

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
    labelSelectPlaceholder: '请先设置图像检测模型标签文件',
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
const textVariableOptions = computed(() => {
  const options = new Map<string, VariableSelectOption>();

  for (const entry of props.inputEntries.filter((item) => item.namespace === 'input' && item.type === 'string')) {
    const option = createDraftVariableSelectOption(entry, 'read');
    if (option) {
      options.set(option.value, option);
    }
  }

  for (const item of props.variableOptions.filter((option) => option.namespace === 'input' && option.readable && option.valueType === 'string')) {
    if (!options.has(item.key)) {
      options.set(item.key, createVariableSelectOption(item));
    }
  }

  return Array.from(options.values());
});

const numberVariableOptions = computed(() => {
  const options = new Map<string, VariableSelectOption>();

  for (const entry of props.inputEntries.filter((item) => item.namespace === 'input' && item.type === 'int')) {
    const option = createDraftVariableSelectOption(entry, 'read');
    if (option) {
      options.set(option.value, option);
    }
  }

  for (const item of props.variableOptions.filter((option) => option.namespace === 'input' && option.readable && option.valueType === 'int')) {
    if (!options.has(item.key)) {
      options.set(item.key, createVariableSelectOption(item));
    }
  }

  return Array.from(options.values());
});

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
  { label: '混合目标', value: ACTION_MODE.mixed, description: '起点和终点分别选择文字或标签。' },
];

const flowTypeOptions = [
  { label: '条件分支', value: FLOW_TYPE.if, description: 'Then / Else 分支。' },
  { label: 'While', value: FLOW_TYPE.while, description: '满足条件时循环。' },
  { label: '遍历循环', value: FLOW_TYPE.forEach, description: '遍历输入集合，并向子步骤暴露元素变量。' },
  { label: '次数循环', value: FLOW_TYPE.repeat, description: '按绑定的数字变量循环 N 次，并暴露索引变量。' },
  { label: '跳过脚本', value: FLOW_TYPE.stopScript, description: '立即结束当前脚本执行。' },
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
const currentSelectedIndexDisplay = computed(() =>
  currentSelectedIndex.value === null ? '-' : String(currentSelectedIndex.value + 1),
);
const selectedStepKindLabel = computed(() => {
  if (!selectedStep.value) {
    return '未选择步骤';
  }
  if (selectedStep.value.op === STEP_OP.action) {
    return selectedAction.value ? `动作 · ${selectedAction.value.ac}` : '动作';
  }
  if (selectedStep.value.op === STEP_OP.flowControl) {
    return selectedFlow.value ? `流程 · ${selectedFlow.value.type}` : '流程';
  }
  if (selectedStep.value.op === STEP_OP.dataHanding) {
    return selectedData.value ? `数据 · ${selectedData.value.type}` : '数据';
  }
  if (selectedStep.value.op === STEP_OP.taskControl) {
    return selectedTaskControl.value ? `状态 · ${selectedTaskControl.value.type}` : '状态';
  }
  if (selectedStep.value.op === STEP_OP.vision) {
    return selectedVision.value ? `视觉 · ${selectedVision.value.type}` : '视觉';
  }
  return '容器 · sequence';
});
const selectedStepSummary = computed(() => {
  if (!selectedStep.value) {
    return '';
  }
  if (selectedStep.value.op === STEP_OP.flowControl) {
    return '流程步骤既要编辑当前条件，也要继续进入 then / else / flow 这些子层级维护嵌套步骤。';
  }
  if (selectedStep.value.op === STEP_OP.dataHanding) {
    return '数据步骤负责变量写入、过滤和结果整理。优先保持输入输出变量命名稳定，后续排查会轻很多。';
  }
  if (selectedStep.value.op === STEP_OP.taskControl) {
    return '状态步骤会直接影响任务或策略的启停与完成状态，改动前先确认目标范围和副作用。';
  }
  if (selectedStep.value.op === STEP_OP.vision) {
    return '视觉步骤通常依赖检测结果变量和命中后分支，先把输出变量和后续跳转关系说明清楚。';
  }
  if (selectedStep.value.op === STEP_OP.sequence) {
    return 'Sequence 更像容器节点，价值在于收拢可被合并下发的一组设备动作。';
  }
  return '动作步骤直接决定设备行为，建议优先检查目标、模式和变量绑定是否明确。';
});
const selectedStepConfigTitle = computed(() => {
  if (!selectedStep.value) {
    return '步骤配置';
  }
  if (selectedStep.value.op === STEP_OP.action) return '动作配置';
  if (selectedStep.value.op === STEP_OP.flowControl) return '流程配置';
  if (selectedStep.value.op === STEP_OP.dataHanding) return '数据配置';
  if (selectedStep.value.op === STEP_OP.taskControl) return '状态配置';
  if (selectedStep.value.op === STEP_OP.vision) return '视觉配置';
  return '序列配置';
});
const selectedStepConfigDescription = computed(() => {
  if (!selectedStep.value) {
    return '';
  }
  if (selectedStep.value.op === STEP_OP.sequence) {
    return 'Sequence 节点字段较少，重点是进入子层级继续编辑允许合并的动作步骤。';
  }
  /* return '详情区现在按概览与配置拆开，和输入、界面等 tab 保持同一套阅读节奏。'; */
  return '';
});

const selectedAction = computed<Action | null>(() => (selectedStep.value?.op === STEP_OP.action ? selectedStep.value.a : null));
const selectedFlow = computed<FlowControl | null>(() => (selectedStep.value?.op === STEP_OP.flowControl ? selectedStep.value.a : null));
const selectedData = computed<DataHanding | null>(() => (selectedStep.value?.op === STEP_OP.dataHanding ? selectedStep.value.a : null));
const selectedTaskControl = computed<TaskControl | null>(() => (selectedStep.value?.op === STEP_OP.taskControl ? selectedStep.value.a : null));
const selectedVision = computed<EditableVisionNode | null>(() =>
  selectedStep.value?.op === STEP_OP.vision ? (selectedStep.value.a as EditableVisionNode) : null,
);
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
const isVarValueKind = (value: string): value is VarValueKind =>
  value === 'int' || value === 'float' || value === 'bool' || value === 'string';
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
const selectedColorCompareInputTarget = computed(() =>
  currentColorCompareInputName.value ? props.variableOptions.find((item) => item.key === currentColorCompareInputName.value) ?? null : null,
);
const selectedColorCompareOutputTarget = computed(() =>
  currentColorCompareOutputName.value ? props.variableOptions.find((item) => item.key === currentColorCompareOutputName.value) ?? null : null,
);
const currentVisionInputName = computed(() =>
  selectedVision.value?.type === VISION_TYPE.detect ||
  selectedVision.value?.type === VISION_TYPE.ocr ||
  selectedVision.value?.type === VISION_TYPE.countCompare
    ? selectedVision.value.input_var ?? ''
    : '',
);
const currentVisionOutputName = computed(() =>
  selectedVision.value?.type === VISION_TYPE.detect ||
  selectedVision.value?.type === VISION_TYPE.ocr ||
  selectedVision.value?.type === VISION_TYPE.countCompare ||
  selectedVision.value?.type === VISION_TYPE.visionSearch
    ? selectedVision.value.out_var ?? ''
    : '',
);
const selectedVisionInputTarget = computed(() =>
  currentVisionInputName.value ? props.variableOptions.find((item) => item.key === currentVisionInputName.value) ?? null : null,
);
const selectedVisionOutputTarget = computed(() =>
  currentVisionOutputName.value ? props.variableOptions.find((item) => item.key === currentVisionOutputName.value) ?? null : null,
);
const selectedVisionDetInputTarget = computed(() =>
  {
    const vision = selectedVision.value;
    return vision?.type === VISION_TYPE.visionSearch && vision.det_res_var
      ? props.variableOptions.find((item) => item.key === vision.det_res_var) ?? null
      : null;
  },
);
const selectedVisionOcrInputTarget = computed(() =>
  {
    const vision = selectedVision.value;
    return vision?.type === VISION_TYPE.visionSearch && vision.ocr_res_var
      ? props.variableOptions.find((item) => item.key === vision.ocr_res_var) ?? null
      : null;
  },
);
const selectedVisionDetOutputTarget = computed(() =>
  {
    const vision = selectedVision.value;
    return vision?.type === VISION_TYPE.visionSearch && vision.out_det_var
      ? props.variableOptions.find((item) => item.key === vision.out_det_var) ?? null
      : null;
  },
);
const selectedVisionOcrOutputTarget = computed(() =>
  {
    const vision = selectedVision.value;
    return vision?.type === VISION_TYPE.visionSearch && vision.out_ocr_var
      ? props.variableOptions.find((item) => item.key === vision.out_ocr_var) ?? null
      : null;
  },
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
    return [{ key: 'sequence', label: '动作序列', count: selectedStep.value.steps.length, path: { parentStepPath: props.selectedStepPath, branch: 'sequence' } }];
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

  if (selectedFlow.value?.type === FLOW_TYPE.while || selectedFlow.value?.type === FLOW_TYPE.forEach || selectedFlow.value?.type === FLOW_TYPE.repeat) {
    return [{ key: 'flow', label: '循环体', count: selectedFlow.value.flow.length, path: { parentStepPath: props.selectedStepPath, branch: 'flow' } }];
  }

  if (selectedVision.value?.type === VISION_TYPE.visionSearch || selectedVision.value?.type === VISION_TYPE.countCompare) {
    return [{ key: 'visionThen', label: '命中后执行', count: selectedVision.value.then_steps?.length ?? 0, path: { parentStepPath: props.selectedStepPath, branch: 'visionThen' } }];
  }

  if (selectedData.value?.type === DATA_TYPE.filter) {
    return [{ key: 'filterThen', label: '过滤命中后', count: selectedData.value.then_steps.length, path: { parentStepPath: props.selectedStepPath, branch: 'filterThen' } }];
  }

  return [];
});
const sequenceBranchTarget = computed(() => branchTargets.value.find((target) => target.key === 'sequence') ?? null);
const visionBranchTarget = computed(() => branchTargets.value.find((target) => target.key === 'visionThen') ?? null);
const filterBranchTarget = computed(() => branchTargets.value.find((target) => target.key === 'filterThen') ?? null);
const flowBranchTargets = computed(() =>
  branchTargets.value.filter((target): target is { key: 'then' | 'else' | 'flow'; label: string; count: number; path: StepBranchPath } =>
    target.key === 'then' || target.key === 'else' || target.key === 'flow'),
);

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
    step.a = {
      ...(step.a ?? {}),
      [field]: field === 'enable_filter' ? value === 'true' : value,
    } as Action;
  });
};

const updateActionExecMax = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action) return;
    step.exec_max = Math.max(0, Number(value) || 0);
  });
};

const createClickAction = (mode: typeof ACTION_MODE.point | typeof ACTION_MODE.percent | typeof ACTION_MODE.txt | typeof ACTION_MODE.labelIdx): Action => {
  const offset = selectedAction.value?.ac === ACTION_TYPE.click
    ? {
        offset_x: selectedAction.value.offset_x ?? 0,
        offset_y: selectedAction.value.offset_y ?? 0,
      }
    : { offset_x: 0, offset_y: 0 };
  switch (mode) {
    case ACTION_MODE.percent:
      return { ac: ACTION_TYPE.click, ...offset, mode: ACTION_MODE.percent, p: { x: 0.5, y: 0.5 } };
    case ACTION_MODE.txt:
      return { ac: ACTION_TYPE.click, ...offset, mode: ACTION_MODE.txt, input_var: currentActionInputName.value || 'runtime.searchHits', txt: '开始', txt_expr: null, enable_filter: true };
    case ACTION_MODE.labelIdx:
      return { ac: ACTION_TYPE.click, ...offset, mode: ACTION_MODE.labelIdx, input_var: currentActionInputName.value || 'runtime.detResults', idx: 0, idx_expr: null, enable_filter: true };
    default:
      return { ac: ACTION_TYPE.click, ...offset, mode: ACTION_MODE.point, p: { x: 640, y: 360 } };
  }
};

const createSwipeAction = (mode: typeof ACTION_MODE.point | typeof ACTION_MODE.percent | typeof ACTION_MODE.txt | typeof ACTION_MODE.labelIdx | typeof ACTION_MODE.mixed): Action => {
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
        from_expr: null,
        to_expr: null,
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
    case ACTION_MODE.mixed:
      return {
        ac: ACTION_TYPE.swipe,
        mode: ACTION_MODE.mixed,
        duration: 300 as never,
        from: {
          source: ACTION_MODE.labelIdx,
          input_var: 'runtime.detResults',
          idx: 0,
        },
        to: {
          source: ACTION_MODE.txt,
          input_var: 'runtime.ocrResults',
          value: '结束',
          value_expr: null,
        },
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
  if (mode !== ACTION_MODE.point && mode !== ACTION_MODE.percent && mode !== ACTION_MODE.txt && mode !== ACTION_MODE.labelIdx && mode !== ACTION_MODE.mixed) {
    return;
  }
  if (!selectedAction.value) return;
  if (selectedAction.value.ac === ACTION_TYPE.click) {
    if (mode === ACTION_MODE.mixed) return;
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

const updateSwipeTargetField = (target: 'from' | 'to', field: string, value: string | number | null) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action || step.a.ac !== ACTION_TYPE.swipe || step.a.mode !== ACTION_MODE.mixed) return;
    const current = { ...(step.a[target] as Record<string, unknown>) };
    if (field === 'source') {
      step.a = {
        ...step.a,
        [target]:
          value === ACTION_MODE.labelIdx
            ? { source: ACTION_MODE.labelIdx, input_var: 'runtime.detResults', idx: 0 }
            : { source: ACTION_MODE.txt, input_var: 'runtime.ocrResults', value: '', value_expr: null },
      } as Action;
      return;
    }
    step.a = {
      ...step.a,
      [target]: {
        ...current,
        [field]: field === 'idx' ? Math.max(0, toNumber(String(value))) : value,
      },
    } as Action;
  });
};

const updateFlowField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl) return;
    if ((step.a.type === FLOW_TYPE.handlePolicySet || step.a.type === FLOW_TYPE.handlePolicy) && field === 'target') {
      let nextTarget = createStringList();
      try {
        nextTarget = JSON.parse(value) as string[];
      } catch {
        nextTarget = createStringList();
      }
      step.a = { ...step.a, target: nextTarget } as FlowControl;
      return;
    }

    step.a = { ...(step.a ?? {}), [field]: value } as FlowControl;
  });
};

const updateFlowBooleanField = (field: string, value: boolean) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl) return;
    if (
      step.a.type !== FLOW_TYPE.addPolicies &&
      step.a.type !== FLOW_TYPE.bindPolicyGroup &&
      step.a.type !== FLOW_TYPE.bindPolicy
    ) {
      return;
    }
    if (field !== 'top' && field !== 'reverse') {
      return;
    }

    step.a = { ...step.a, [field]: value } as FlowControl;
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

const updateDataRegionPoint = (field: 'region_top_left' | 'region_bottom_right', key: 'mode' | 'x' | 'y', value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || (step.a.type !== DATA_TYPE.filter && step.a.type !== DATA_TYPE.colorCompare)) return;
    const current = (step.a[field] ?? { mode: ACTION_MODE.point, p: { x: 0, y: 0 } }) as { mode: 'point' | 'percent'; p: { x: number; y: number } };
    const nextMode = key === 'mode' && value === ACTION_MODE.percent ? ACTION_MODE.percent : key === 'mode' ? ACTION_MODE.point : current.mode;
    const nextPoint = {
      ...current.p,
      ...(key === 'x' || key === 'y' ? { [key]: toNumber(value) } : {}),
    };
    step.a = {
      ...step.a,
      [field]: {
        mode: nextMode,
        p: nextPoint,
      },
    } as DataHanding;
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
        : await props.createVariable('input', 'int', {
            preferredKey: 'newVar',
            name: '新变量',
          });
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

const handleCreateActionVariable = async (target: 'captureOutput' | 'actionInput' | 'clickText' | 'swipeFromText' | 'swipeToText') => {
  if (!props.createVariable) {
    return;
  }

  if (target === 'clickText' || target === 'swipeFromText' || target === 'swipeToText') {
    const key = await props.createVariable('input', 'string', {
      preferredKey: target === 'clickText' ? 'targetText' : target === 'swipeFromText' ? 'swipeFromText' : 'swipeToText',
      name: target === 'clickText' ? '目标文字' : target === 'swipeFromText' ? '滑动起点文字' : '滑动终点文字',
    });
    if (!key) {
      return;
    }

    updateActionTextField(target === 'clickText' ? 'txt_expr' : target === 'swipeFromText' ? 'from_expr' : 'to_expr', key);
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
  if (!isVarValueKind(kind)) {
    return;
  }
  setVarKindPreference.value = kind;
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.setVar) return;
    const nextDraft = createDefaultVarValueDraft(kind);
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
  if (!isVarValueKind(kind)) {
    return;
  }
  getVarKindPreference.value = kind;
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.getVar) return;
    step.a = {
      ...step.a,
      default_val: buildVarValue({
        kind,
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
  if (value !== FILTER_MODE_TYPE.filter && value !== FILTER_MODE_TYPE.map) {
    return;
  }
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.filter) return;
    step.a = {
      ...step.a,
      mode: createFilterMode(value),
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
  if (value !== COLOR_COMPARE_METHOD_TYPE.oklabDistance) {
    return;
  }
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.colorCompare) return;
    step.a = {
      ...step.a,
      method: createColorCompareMethod(value, step.a.method.threshold),
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
  if (value !== TASK_CONTROL_TYPE.setState) {
    return;
  }
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    step.a = {
      ...step.a,
      type: value,
    };
  });
};

const updateTaskControlTargetType = (value: string) => {
  if (value !== STATE_TARGET_TYPE.task && value !== STATE_TARGET_TYPE.policy) {
    return;
  }
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    const nextTargetType = value;
    step.a = {
      ...step.a,
      target: createStateTarget(nextTargetType),
      targets: createStateTargetList(),
      status:
        nextTargetType === STATE_TARGET_TYPE.policy && step.a.status.type === STATE_STATUS_TYPE.enabled
          ? createStateStatus(STATE_STATUS_TYPE.done, step.a.status.value)
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
  if (value !== STATE_STATUS_TYPE.enabled && value !== STATE_STATUS_TYPE.done && value !== STATE_STATUS_TYPE.skip) {
    return;
  }
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    step.a = {
      ...step.a,
      status: createStateStatus(value, step.a.status.value),
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

const updateVisionNullableField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.vision) return;
    step.a = { ...(step.a ?? {}), [field]: value.trim() ? value : null } as VisionNode;
  });
};

const updateVisionNumberField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.vision) return;
    step.a = { ...(step.a ?? {}), [field]: Number(value) || 0 } as VisionNode;
  });
};

const handleCreateVisionVariable = async (
  target:
    | 'visionInput'
    | 'visionOutput'
    | 'visionSearchDetInput'
    | 'visionSearchOcrInput'
    | 'visionSearchDetOutput'
    | 'visionSearchOcrOutput',
) => {
  if (!props.createVariable) {
    return;
  }

  const createOptions =
    target === 'visionInput'
      ? selectedVision.value?.type === VISION_TYPE.countCompare
        ? { preferredKey: 'ocrResults', name: 'OCR结果' }
        : { preferredKey: 'captureResult', name: '截图结果' }
      : target === 'visionOutput' && selectedVision.value?.type === VISION_TYPE.detect
      ? { preferredKey: 'detResults', name: '检测结果' }
      : target === 'visionOutput' && selectedVision.value?.type === VISION_TYPE.ocr
        ? { preferredKey: 'ocrResults', name: 'OCR结果' }
        : target === 'visionOutput' && selectedVision.value?.type === VISION_TYPE.countCompare
          ? { preferredKey: 'countMatched', name: '数量比较结果' }
        : target === 'visionOutput' && selectedVision.value?.type === VISION_TYPE.visionSearch
          ? { preferredKey: 'searchHits', name: '搜索命中' }
          : target === 'visionSearchDetInput'
            ? { preferredKey: 'detResults', name: '检测结果' }
            : target === 'visionSearchOcrInput'
              ? { preferredKey: 'ocrResults', name: 'OCR结果' }
              : target === 'visionSearchDetOutput'
                ? { preferredKey: 'filteredDetResults', name: '筛选检测结果' }
                : target === 'visionSearchOcrOutput'
                  ? { preferredKey: 'filteredOcrResults', name: '筛选OCR结果' }
                  : undefined;

  const key = await props.createVariable(
    'runtime',
    target === 'visionInput'
      ? selectedVision.value?.type === VISION_TYPE.countCompare
        ? 'json'
        : 'image'
      : selectedVision.value?.type === VISION_TYPE.countCompare
        ? 'bool'
        : 'json',
    createOptions,
  );
  if (!key) {
    return;
  }

  if (target === 'visionInput' || target === 'visionOutput') {
    updateVisionField(target === 'visionInput' ? 'input_var' : 'out_var', key);
    return;
  }

  updateVisionNullableField(
    target === 'visionSearchDetInput'
      ? 'det_res_var'
      : target === 'visionSearchOcrInput'
        ? 'ocr_res_var'
        : target === 'visionSearchDetOutput'
          ? 'out_det_var'
          : 'out_ocr_var',
    key,
  );
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
  if (type !== FLOW_TYPE.if && type !== FLOW_TYPE.while && type !== FLOW_TYPE.forEach && type !== FLOW_TYPE.repeat) {
    return;
  }
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl) return;
    const currentCondition = flowCondition.value ?? createConditionNode(CONDITION_TYPE.rawExpr);
    if (type === FLOW_TYPE.if) {
      const flowSteps =
        step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.forEach || step.a.type === FLOW_TYPE.repeat
          ? step.a.flow
          : createStepList();
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
        step.a.type === FLOW_TYPE.if
          ? step.a.then
          : step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.forEach || step.a.type === FLOW_TYPE.repeat
          ? step.a.flow
          : createStepList();
      step.a = {
        type: FLOW_TYPE.while,
        con: currentCondition,
        flow: branchSteps,
      } as FlowControl;
      return;
    }

    if (type === FLOW_TYPE.forEach) {
      const branchSteps =
        step.a.type === FLOW_TYPE.if
          ? step.a.then
          : step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.forEach || step.a.type === FLOW_TYPE.repeat
          ? step.a.flow
          : createStepList();
      step.a = {
        type: FLOW_TYPE.forEach,
        input_var: 'runtime.items',
        item_var: 'runtime.item',
        index_var: 'runtime.itemIndex',
        flow: branchSteps,
      } as FlowControl;
      return;
    }

    if (type === FLOW_TYPE.repeat) {
      const branchSteps =
        step.a.type === FLOW_TYPE.if
          ? step.a.then
          : step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.forEach || step.a.type === FLOW_TYPE.repeat
          ? step.a.flow
          : createStepList();
      step.a = {
        type: FLOW_TYPE.repeat,
        count_expr: '',
        index_var: 'runtime.repeatIndex',
        flow: branchSteps,
      } as FlowControl;
    }
  });
};

const toggleElseBranch = () => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl || step.a.type !== FLOW_TYPE.if) return;
    step.a.else_steps = step.a.else_steps ? null : createStepList();
  });
};
</script>

<style scoped>
@reference "../../../style.css";

.step-badge-row {
  @apply flex flex-wrap gap-2;
}

.step-badge {
  @apply rounded-full border border-(--app-border) bg-white/55 px-3 py-1 text-xs text-(--app-text-soft);
}

.step-badge-strong {
  @apply border-transparent bg-(--app-accent-soft) text-(--app-accent);
}

.step-summary-card {
  @apply rounded-[14px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4;
}

.sequence-note {
  @apply space-y-4 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4;
}

.sequence-note-tip {
  @apply rounded-[14px] border border-dashed border-(--app-border) px-4 py-4 text-sm text-(--app-text-soft);
}
</style>
