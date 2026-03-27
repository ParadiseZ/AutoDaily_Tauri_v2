<template>
  <div class="grid min-h-0 gap-4 xl:grid-rows-[auto_minmax(0,1fr)]">
    <EditorStepBreadcrumb
      :steps="steps"
      :active-branch-path="activeBranchPath"
      :selected-step-path="selectedStepPath"
      @navigate-branch="$emit('navigate-branch', $event)"
      @select-step-path="$emit('select-step-path', $event)"
    />

    <div class="grid min-h-0 gap-4 xl:grid-cols-[minmax(0,360px)_minmax(0,1fr)]">
      <div class="min-h-0 overflow-y-auto pr-1 custom-scrollbar">
        <div
          v-if="activeBranchPath.branch !== 'root'"
          class="mb-3 rounded-[16px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-3 text-sm text-[var(--app-text-soft)]"
        >
          中间的步骤模板会直接插入到当前层级。
        </div>

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
          description="先从中间选择一个模板，它会直接插入到当前层级。"
        />
      </div>

      <div class="min-h-0 overflow-y-auto pr-1 custom-scrollbar">
        <div v-if="selectedStep" class="space-y-4">
          <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <div class="flex items-start justify-between gap-3">
              <div>
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">步骤详情</p>
                <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ describeStep(selectedStep) }}</p>
              </div>
              <span class="rounded-full bg-white/50 px-3 py-1 text-xs text-[var(--app-text-soft)]">{{ selectedStep.op }}</span>
            </div>

            <label class="mt-4 block space-y-2">
              <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">步骤标题</span>
              <input :value="selectedStep.label || ''" class="app-input" @input="updateStepLabel(($event.target as HTMLInputElement).value)" />
            </label>
          </div>

          <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <p class="text-sm font-semibold text-[var(--app-text-strong)]">关键字段</p>
            <div class="mt-3 space-y-3">
              <template v-if="selectedStep.op === STEP_OP.action && selectedAction?.ac === ACTION_TYPE.capture">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输出变量</span>
                  <input :value="selectedAction.output_var || ''" class="app-input" @input="updateActionField('output_var', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

              <template
                v-else-if="
                  selectedStep.op === STEP_OP.action &&
                  (selectedAction?.ac === ACTION_TYPE.launchApp || selectedAction?.ac === ACTION_TYPE.stopApp)
                "
              >
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">包名</span>
                  <input :value="selectedAction.pkg_name || ''" class="app-input" @input="updateActionField('pkg_name', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.action && selectedAction?.ac === ACTION_TYPE.click">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">点击方式</span>
                  <AppSelect
                    :model-value="String(selectedAction.mode || ACTION_MODE.point)"
                    :options="clickModeOptions"
                    placeholder="点击方式"
                    @update:model-value="updateActionMode(String($event || ACTION_MODE.point))"
                  />
                </label>

                <div v-if="selectedAction.mode === ACTION_MODE.point || selectedAction.mode === ACTION_MODE.percent" class="grid gap-3 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">X</span>
                    <input
                      :value="String((selectedAction.p as { x?: number })?.x ?? '')"
                      class="app-input"
                      type="number"
                      @input="updateActionPointField('p', 'x', ($event.target as HTMLInputElement).value)"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">Y</span>
                    <input
                      :value="String((selectedAction.p as { y?: number })?.y ?? '')"
                      class="app-input"
                      type="number"
                      @input="updateActionPointField('p', 'y', ($event.target as HTMLInputElement).value)"
                    />
                  </label>
                </div>

                <label v-else-if="selectedAction.mode === ACTION_MODE.txt" class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标文字</span>
                  <input :value="String(selectedAction.txt ?? '')" class="app-input" @input="updateActionTextField('txt', ($event.target as HTMLInputElement).value)" />
                </label>

                <label v-else class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">标签索引</span>
                  <input :value="String(selectedAction.idx ?? 0)" class="app-input" type="number" @input="updateActionNumberField('idx', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.action && selectedAction?.ac === ACTION_TYPE.swipe">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">滑动方式</span>
                  <AppSelect
                    :model-value="String(selectedAction.mode || ACTION_MODE.point)"
                    :options="swipeModeOptions"
                    placeholder="滑动方式"
                    @update:model-value="updateActionMode(String($event || ACTION_MODE.point))"
                  />
                </label>

                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">持续时间 (ms)</span>
                  <input :value="String(selectedAction.duration ?? 300)" class="app-input" type="number" @input="updateActionNumberField('duration', ($event.target as HTMLInputElement).value)" />
                </label>

                <div v-if="selectedAction.mode === ACTION_MODE.point || selectedAction.mode === ACTION_MODE.percent" class="grid gap-3 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">起点 X</span>
                    <input
                      :value="String((selectedAction.from as { x?: number })?.x ?? '')"
                      class="app-input"
                      type="number"
                      @input="updateActionPointField('from', 'x', ($event.target as HTMLInputElement).value)"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">起点 Y</span>
                    <input
                      :value="String((selectedAction.from as { y?: number })?.y ?? '')"
                      class="app-input"
                      type="number"
                      @input="updateActionPointField('from', 'y', ($event.target as HTMLInputElement).value)"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">终点 X</span>
                    <input
                      :value="String((selectedAction.to as { x?: number })?.x ?? '')"
                      class="app-input"
                      type="number"
                      @input="updateActionPointField('to', 'x', ($event.target as HTMLInputElement).value)"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">终点 Y</span>
                    <input
                      :value="String((selectedAction.to as { y?: number })?.y ?? '')"
                      class="app-input"
                      type="number"
                      @input="updateActionPointField('to', 'y', ($event.target as HTMLInputElement).value)"
                    />
                  </label>
                </div>

                <div v-else-if="selectedAction.mode === ACTION_MODE.txt" class="grid gap-3 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">起点文字</span>
                    <input :value="String(selectedAction.from ?? '')" class="app-input" @input="updateActionTextField('from', ($event.target as HTMLInputElement).value)" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">终点文字</span>
                    <input :value="String(selectedAction.to ?? '')" class="app-input" @input="updateActionTextField('to', ($event.target as HTMLInputElement).value)" />
                  </label>
                </div>

                <div v-else class="grid gap-3 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">起点标签</span>
                    <input :value="String(selectedAction.from ?? 0)" class="app-input" type="number" @input="updateActionNumberField('from', ($event.target as HTMLInputElement).value)" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">终点标签</span>
                    <input :value="String(selectedAction.to ?? 1)" class="app-input" type="number" @input="updateActionNumberField('to', ($event.target as HTMLInputElement).value)" />
                  </label>
                </div>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.flowControl && selectedFlow?.type === FLOW_TYPE.waitMs">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">等待毫秒</span>
                  <input :value="String(selectedFlow.ms ?? 1000)" class="app-input" type="number" @input="updateNumberField('ms', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.flowControl && selectedFlow?.type === FLOW_TYPE.link">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标任务</span>
                  <input :value="selectedFlow.target || ''" class="app-input" @input="updateFlowField('target', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.flowControl && flowWithCondition">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">流程类型</span>
                  <AppSelect
                    :model-value="flowWithCondition.type"
                    :options="flowTypeOptions"
                    placeholder="流程类型"
                    @update:model-value="updateFlowType(String($event || FLOW_TYPE.if))"
                  />
                </label>

                <div class="flex flex-wrap items-center justify-between gap-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-3">
                  <span class="text-sm text-[var(--app-text-soft)]">
                    {{
                      flowWithCondition.type === FLOW_TYPE.if
                        ? `Then ${(((selectedFlow as { then?: unknown[] } | null)?.then)?.length ?? 0)} 步`
                        : `循环 ${(((selectedFlow as { flow?: unknown[] } | null)?.flow)?.length ?? 0)} 步`
                    }}
                  </span>
                  <button
                    v-if="flowWithCondition.type === FLOW_TYPE.if"
                    class="app-button app-button-ghost app-toolbar-button"
                    type="button"
                    @click="toggleElseBranch"
                  >
                    {{ hasElseBranch ? '移除 Else 分支' : '添加 Else 分支' }}
                  </button>
                </div>

                <EditorConditionBuilder
                  v-if="flowCondition"
                  :model-value="flowCondition"
                  test-id-prefix="editor-condition"
                  @update:model-value="updateFlowCondition"
                />
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.dataHanding && selectedData?.type === DATA_TYPE.setVar">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">变量名</span>
                  <input :value="selectedData.name || ''" class="app-input" @input="updateDataField('name', ($event.target as HTMLInputElement).value)" />
                </label>
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">默认值</span>
                  <input :value="String(selectedData.val ?? '')" class="app-input" @input="updateDataField('val', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.dataHanding && selectedData?.type === DATA_TYPE.getVar">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">变量名</span>
                  <input :value="selectedData.name || ''" class="app-input" @input="updateDataField('name', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.vision && selectedVision?.type === VISION_TYPE.visionSearch">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输出变量</span>
                  <input :value="selectedVision.out_var || ''" class="app-input" @input="updateVisionField('out_var', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

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
import { computed } from 'vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import type { Action } from '@/types/bindings/Action';
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { DataHanding } from '@/types/bindings/DataHanding';
import type { FlowControl } from '@/types/bindings/FlowControl';
import type { Step } from '@/types/bindings/Step';
import type { VisionNode } from '@/types/bindings/VisionNode';
import EditorConditionBuilder from '@/views/script-editor/EditorConditionBuilder.vue';
import EditorStepBreadcrumb from '@/views/script-editor/EditorStepBreadcrumb.vue';
import EditorStepList from '@/views/script-editor/EditorStepList.vue';
import { createConditionNode } from '@/views/script-editor/editorCondition';
import { ACTION_MODE, ACTION_TYPE, DATA_TYPE, FLOW_TYPE, STEP_OP, VISION_TYPE } from '@/views/script-editor/editorStepKinds';
import { describeStep } from '@/views/script-editor/editorStepTemplates';
import {
  buildStepPath,
  getBranchSteps,
  getParentBranchPath,
  getStepByPath,
  isSameBranchPath,
  type StepBranchPath,
  type StepPath,
} from '@/views/script-editor/editorStepTree';
import { cloneJson } from '@/views/script-editor/editorSchema';

type NestedGroupKey = 'sequence' | 'then' | 'else' | 'flow' | 'visionThen';

const props = defineProps<{
  steps: Step[];
  selectedStepPath: StepPath | null;
  activeBranchPath: StepBranchPath;
}>();

const emit = defineEmits<{
  'select-step-path': [path: StepPath];
  'navigate-branch': [branchPath: StepBranchPath];
  'reorder-step': [from: number, to: number];
  'remove-step': [index: number];
  'update-step': [index: number, step: Step];
}>();

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
const selectedVision = computed<VisionNode | null>(() => (selectedStep.value?.op === STEP_OP.vision ? selectedStep.value.a : null));

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

  return [];
});

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

const updateVisionField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.vision) return;
    step.a = { ...(step.a ?? {}), [field]: value } as VisionNode;
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
