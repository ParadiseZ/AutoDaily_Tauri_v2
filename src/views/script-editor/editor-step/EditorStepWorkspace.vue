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
                :variable-datalist-id="variableDatalistId"
                :click-mode-options="clickModeOptions"
                :swipe-mode-options="swipeModeOptions"
                @update-field="updateActionField"
                @update-mode="updateActionMode"
                @update-point-field="updateActionPointField"
                @update-number-field="updateActionNumberField"
                @update-text-field="updateActionTextField"
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
                :selected-get-var-target="selectedGetVarTarget"
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
                :filter-branch-target="filterBranchTarget"
                :variable-datalist-id="variableDatalistId"
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
                @navigate-branch="$emit('navigate-branch', $event)"
              />

              <EditorStepSequencePanel
                v-else-if="selectedStep.op === STEP_OP.sequence"
                :reverse="selectedStep.reverse"
                @update-reverse="updateSequenceReverse"
              />

              <EditorStepTaskControlPanel
                v-else-if="selectedStep.op === STEP_OP.taskControl && selectedTaskControl"
                :selected-task-control="selectedTaskControl"
                :task-control-type-options="taskControlTypeOptions"
                :state-target-type-options="stateTargetTypeOptions"
                :state-status-type-options="stateStatusTypeOptions"
                @update-type="updateTaskControlType"
                @update-target-type="updateTaskControlTargetType"
                @update-target-id="updateTaskControlTargetId"
                @update-status-type="updateTaskControlStatusType"
                @update-status-value="updateTaskControlStatusValue"
              />

              <EditorStepVisionPanel
                v-else-if="selectedStep.op === STEP_OP.vision && selectedVision?.type === VISION_TYPE.visionSearch"
                :selected-vision="selectedVision"
                :variable-datalist-id="variableDatalistId"
                :vision-branch-target="visionBranchTarget"
                @update-field="updateVisionField"
                @update-rule="updateVisionRule"
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
import EditorStepActionPanel from '@/views/script-editor/editor-step/EditorStepActionPanel.vue';
import EditorStepBreadcrumb from '@/views/script-editor/editor-step/EditorStepBreadcrumb.vue';
import EditorStepDataPanel from '@/views/script-editor/editor-step/EditorStepDataPanel.vue';
import EditorStepFlowPanel from '@/views/script-editor/editor-step/EditorStepFlowPanel.vue';
import EditorStepList from '@/views/script-editor/editor-step/EditorStepList.vue';
import EditorStepSequencePanel from '@/views/script-editor/editor-step/EditorStepSequencePanel.vue';
import EditorStepTaskControlPanel from '@/views/script-editor/editor-step/EditorStepTaskControlPanel.vue';
import EditorStepVisionPanel from '@/views/script-editor/editor-step/EditorStepVisionPanel.vue';
import { createConditionNode } from '@/views/script-editor/editorCondition';
import {
  ACTION_MODE,
  ACTION_TYPE,
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
import type { EditorVariableOption } from '@/views/script-editor/editorVariables';
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

const props = defineProps<{
  steps: Step[];
  selectedStepPath: StepPath | null;
  activeBranchPath: StepBranchPath;
  variableOptions: EditorVariableOption[];
  catalogVariableOptions: EditorVariableOption[];
}>();

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
const readableCatalogVariableOptions = computed(() =>
  props.catalogVariableOptions
    .filter((item) => item.readable)
    .map((item) => ({
      label: item.label || item.key,
      value: item.key,
      description: `${item.namespace} · ${item.valueType}`,
    })),
);
const writableCatalogVariableOptions = computed(() =>
  props.catalogVariableOptions
    .filter((item) => item.writable)
    .map((item) => ({
      label: item.label || item.key,
      value: item.key,
      description: `${item.namespace} · ${item.valueType}`,
    })),
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
  { label: 'For', value: FLOW_TYPE.for, description: '条件控制的遍历循环。' },
];

const filterModeOptions = [
  { label: '过滤', value: FILTER_MODE_TYPE.filter, description: '保留符合条件的元素。' },
  { label: '映射', value: FILTER_MODE_TYPE.map, description: '将输入映射为新结果。' },
];
const taskControlTypeOptions = [
  { label: '设置状态', value: TASK_CONTROL_TYPE.setState, description: '写入目标状态。' },
  { label: '读取状态', value: TASK_CONTROL_TYPE.getState, description: '读取目标状态。' },
];

const stateTargetTypeOptions = [
  { label: '任务', value: STATE_TARGET_TYPE.task, description: '针对任务状态。' },
  { label: '策略', value: STATE_TARGET_TYPE.policy, description: '针对策略状态。' },
];

const stateStatusTypeOptions = [
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
  currentSetVarName.value ? props.catalogVariableOptions.find((item) => item.key === currentSetVarName.value) ?? null : null,
);
const selectedGetVarTarget = computed(() =>
  currentGetVarName.value ? props.catalogVariableOptions.find((item) => item.key === currentGetVarName.value) ?? null : null,
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
  if ((type === FLOW_TYPE.if || type === FLOW_TYPE.while || type === FLOW_TYPE.for) && selectedFlow.value.con) {
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

  if (selectedFlow.value?.type === FLOW_TYPE.while || selectedFlow.value?.type === FLOW_TYPE.for) {
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

const updateSelectedStep = (mutator: (step: Step & { a?: Action | FlowControl | DataHanding | VisionNode }) => void) => {
  if (currentSelectedIndex.value === null || !selectedStep.value) return;
  const nextStep = cloneJson(selectedStep.value) as Step & { a?: Action | FlowControl | DataHanding | VisionNode };
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

const createClickAction = (mode: string): Action => {
  switch (mode) {
    case ACTION_MODE.percent:
      return { ac: ACTION_TYPE.click, mode: ACTION_MODE.percent, p: { x: 0.5, y: 0.5 } };
    case ACTION_MODE.txt:
      return { ac: ACTION_TYPE.click, mode: ACTION_MODE.txt, txt: '开始' };
    case ACTION_MODE.labelIdx:
      return { ac: ACTION_TYPE.click, mode: ACTION_MODE.labelIdx, idx: 0 };
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
        from: '开始',
        to: '结束',
      };
    case ACTION_MODE.labelIdx:
      return {
        ac: ACTION_TYPE.swipe,
        mode: ACTION_MODE.labelIdx,
        duration: 300 as never,
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

const updateActionTextField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action) return;
    step.a = { ...(step.a as Record<string, unknown>), [field]: value } as Action;
  });
};

const updateFlowField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl) return;
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
  const matched = props.catalogVariableOptions.find((item) => item.key === value) ?? null;
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

const updateSequenceReverse = (value: boolean) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.sequence) return;
    step.reverse = value;
  });
};

const updateTaskControlType = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    step.a = {
      ...step.a,
      type: value as typeof TASK_CONTROL_TYPE.setState | typeof TASK_CONTROL_TYPE.getState,
    };
  });
};

const updateTaskControlTargetType = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    step.a = {
      ...step.a,
      target: {
        ...step.a.target,
        type: value as typeof STATE_TARGET_TYPE.task | typeof STATE_TARGET_TYPE.policy,
      },
    };
  });
};

const updateTaskControlTargetId = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    step.a = {
      ...step.a,
      target: {
        ...step.a.target,
        id: value,
      },
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
        type: value as typeof STATE_STATUS_TYPE.done | typeof STATE_STATUS_TYPE.skip,
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
    if (step.a.type === FLOW_TYPE.if || step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.for) {
      step.a.con = condition;
    }
  });
};

const updateFlowType = (type: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl) return;
    const currentCondition = flowCondition.value ?? createConditionNode('rawExpr');
    if (type === FLOW_TYPE.if) {
      const flowSteps = step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.for ? step.a.flow : [];
      step.a = {
        type: FLOW_TYPE.if,
        con: currentCondition,
        then: flowSteps,
        else_steps: null,
      } as FlowControl;
      return;
    }

    if (type === FLOW_TYPE.while || type === FLOW_TYPE.for) {
      const branchSteps =
        step.a.type === FLOW_TYPE.if ? step.a.then : step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.for ? step.a.flow : [];
      step.a = {
        type: type as typeof FLOW_TYPE.while | typeof FLOW_TYPE.for,
        con: currentCondition,
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
