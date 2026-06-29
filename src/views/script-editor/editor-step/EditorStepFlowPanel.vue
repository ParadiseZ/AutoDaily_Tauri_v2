<template>
  <div class="space-y-3">
    <template v-if="selectedFlow.type === FLOW_TYPE.waitMs">
      <div class="space-y-4 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">等待来源</span>
          <EditorSelectField
            :model-value="waitBindingMode"
            :options="waitBindingModeOptions"
            placeholder="选择等待来源"
            test-id="editor-flow-wait-binding-mode"
            @update:model-value="updateWaitBindingMode(String($event || 'fixed'))"
          />
        </label>

        <EditorVariableBindingField
          v-if="waitBindingMode === 'input'"
          label="输入变量"
          :model-value="selectedFlow.input_var || null"
          :options="resolvedWaitInputOptions"
          placeholder="绑定输入毫秒变量"
          test-id="editor-flow-wait-input-var"
          create-label="新建毫秒变量"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedWaitInputOption && jumpToVariable)"
          :locate-disabled="!selectedWaitInputOption || !jumpToVariable"
          @update:model-value="$emit('update-field', 'input_var', String($event || ''))"
          @create="createWaitInputVariable"
          @locate="jumpToSelectedWaitInputVariable"
        />

        <EditorVariableBindingField
          v-else-if="waitBindingMode === 'runtime'"
          label="运行时变量"
          :model-value="selectedFlow.runtime_var || null"
          :options="resolvedWaitRuntimeOptions"
          placeholder="绑定 OCR 结果变量"
          test-id="editor-flow-wait-runtime-var"
          create-label="新建 OCR 变量"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedWaitRuntimeOption && jumpToVariable)"
          :locate-disabled="!selectedWaitRuntimeOption || !jumpToVariable"
          @update:model-value="$emit('update-field', 'runtime_var', String($event || ''))"
          @create="createWaitRuntimeVariable"
          @locate="jumpToSelectedWaitRuntimeVariable"
        />

        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">兜底等待毫秒</span>
          <input :value="String(selectedFlow.ms ?? 1000)" class="app-input" type="number" @input="$emit('update-number-field', 'ms', ($event.target as HTMLInputElement).value)" />
        </label>

        <p class="text-xs leading-6 text-(--app-text-soft)">
          绑定 `input` 时直接读取毫秒值；绑定 `runtime` 时会从 OCR 结果里提取 `00:00` 或 `00:00:00`，解析失败时回退到这里的毫秒值。
        </p>
      </div>
    </template>

    <template v-else-if="selectedFlow.type === FLOW_TYPE.link">
      <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
        <div class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">目标任务</span>
          <EditorSelectField
            :model-value="selectedLinkTarget || null"
            :options="resolvedTaskReferenceOptions"
            placeholder="选择跳转任务"
            @update:model-value="$emit('update-field', 'target', String($event || ''))"
          />
        </div>
        <div class="flex flex-wrap gap-2">
          <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="createTaskReferenceAndBind">
            <AppIcon name="plus" :size="14" />
            新建任务
          </button>
          <button
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            :disabled="!selectedLinkTarget"
            @click="jumpToLinkedTask"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位编辑
          </button>
        </div>
      </div>
    </template>

    <template
      v-else-if="
        selectedFlow.type === FLOW_TYPE.addPolicies ||
        selectedFlow.type === FLOW_TYPE.bindPolicyGroup ||
        selectedFlow.type === FLOW_TYPE.bindPolicy
      "
    >
      <div class="space-y-4 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
        <div class="grid gap-4 xl:grid-cols-2">
          <div class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">{{ bindingSourceTitle }}</span>
            <EditorSelectField
              :model-value="bindingSourceId || null"
              :options="bindingSourceReferenceOptions"
              :placeholder="bindingSourcePlaceholder"
              :test-id="bindingSourceTestId"
              @update:model-value="$emit('update-field', 'source', String($event || ''))"
            />
            <div class="flex flex-wrap gap-2">
              <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="createBindingSourceReference">
                <AppIcon name="plus" :size="14" />
                新建
              </button>
              <button
                class="app-button app-button-ghost app-toolbar-button"
                type="button"
                :disabled="!bindingSourceId"
                @click="jumpToBindingSource"
              >
                <AppIcon name="locate-fixed" :size="14" />
                定位
              </button>
            </div>
          </div>

          <div class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">{{ bindingTargetTitle }}</span>
            <EditorSelectField
              :model-value="bindingTargetId || null"
              :options="bindingTargetReferenceOptions"
              :placeholder="bindingTargetPlaceholder"
              :test-id="bindingTargetTestId"
              @update:model-value="$emit('update-field', 'target', String($event || ''))"
            />
            <div class="flex flex-wrap gap-2">
              <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="createBindingTargetReference">
                <AppIcon name="plus" :size="14" />
                新建
              </button>
              <button
                class="app-button app-button-ghost app-toolbar-button"
                type="button"
                :disabled="!bindingTargetId"
                @click="jumpToBindingTarget"
              >
                <AppIcon name="locate-fixed" :size="14" />
                定位
              </button>
            </div>
          </div>
        </div>

        <label class="flex items-center gap-3 rounded-[16px] border border-(--app-border) bg-white/55 px-4 py-3">
          <input
            :checked="bindingTopValue"
            type="checkbox"
            class="h-4 w-4"
            :data-testid="bindingTopTestId"
            style="accent-color: var(--app-accent)"
            @change="$emit('update-boolean-field', 'top', ($event.target as HTMLInputElement).checked)"
          />
          <div class="space-y-1">
            <p class="text-sm font-medium text-(--app-text-strong)">添加到顶部</p>
            <p class="text-xs leading-6 text-(--app-text-soft)">开启后，源对象会插到目标对象当前顺序的最前面。</p>
          </div>
        </label>

        <label class="flex items-center gap-3 rounded-[16px] border border-(--app-border) bg-white/55 px-4 py-3">
          <input
            :checked="bindingReverseValue"
            type="checkbox"
            class="h-4 w-4"
            :data-testid="bindingReverseTestId"
            style="accent-color: var(--app-accent)"
            @change="$emit('update-boolean-field', 'reverse', ($event.target as HTMLInputElement).checked)"
          />
          <div class="space-y-1">
            <p class="text-sm font-medium text-(--app-text-strong)">插入前反转</p>
            <p class="text-xs leading-6 text-(--app-text-soft)">{{ bindingReverseDescription }}</p>
          </div>
        </label>

        <div class="rounded-[14px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4 text-xs leading-6 text-(--app-text-soft)">
          {{ bindingHelpText }}
        </div>
      </div>
    </template>

    <template v-else-if="selectedFlow.type === FLOW_TYPE.handlePolicySet || selectedFlow.type === FLOW_TYPE.handlePolicy">
      <div class="space-y-4 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
        <div class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">{{ targetTitle }}</span>
          <div class="grid gap-2 md:grid-cols-[minmax(0,1fr)_auto]">
            <EditorSelectField
              :model-value="pendingTargetId"
              :options="availableTargetReferenceOptions"
              :placeholder="targetPlaceholder"
              :test-id="selectedFlow.type === FLOW_TYPE.handlePolicySet ? 'editor-flow-policy-set-pending' : 'editor-flow-policy-pending'"
              @update:model-value="pendingTargetId = String($event || '')"
            />
            <button
              class="app-button app-button-primary app-toolbar-button justify-center"
              type="button"
              :data-testid="selectedFlow.type === FLOW_TYPE.handlePolicySet ? 'editor-flow-policy-set-add' : 'editor-flow-policy-add'"
              :disabled="!pendingTargetId"
              @click="appendTarget"
            >
              添加
            </button>
          </div>
        </div>

        <div v-if="resolvedTargets.length" class="space-y-2">
          <article
            v-for="target in resolvedTargets"
            :key="target.id"
            :data-testid="selectedFlow.type === FLOW_TYPE.handlePolicySet ? `editor-flow-policy-set-target-${target.id}` : `editor-flow-policy-target-${target.id}`"
            class="flex items-center justify-between gap-3 rounded-[14px] border border-(--app-border) bg-white/55 px-3 py-3"
          >
            <div class="min-w-0">
              <p class="truncate text-sm font-semibold text-(--app-text-strong)">{{ target.label }}</p>
              <p class="mt-1 text-xs text-(--app-text-faint)">{{ target.description }}</p>
            </div>
            <div class="flex items-center gap-2">
              <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="jumpToTarget(target.id)">
                <AppIcon name="locate-fixed" :size="14" />
                定位
              </button>
              <button class="app-icon-button app-crash-icon app-icon-button-sec" type="button" aria-label="移除" title="移除" @click="removeTarget(target.id)">
                <Trash2 class="h-4 w-4" />
              </button>
            </div>
          </article>
        </div>

        <div v-else class="rounded-[14px] border border-dashed border-(--app-border) px-4 py-4 text-sm text-(--app-text-soft)">
          {{ emptyTargetText }}
        </div>

        <div class="space-y-4">
          <EditorVariableBindingField
            label="输入图像变量"
            :model-value="selectedFlowInput || null"
            :options="resolvedFlowInputOptions"
            placeholder="选择截图或图像变量"
            :test-id="selectedFlow.type === FLOW_TYPE.handlePolicySet ? 'editor-flow-policy-set-input-var' : 'editor-flow-policy-input-var'"
            create-label="新建图像变量"
            :show-create="Boolean(createVariable)"
            :show-locate="Boolean(selectedFlowInputOption && jumpToVariable)"
            :locate-disabled="!selectedFlowInputOption || !jumpToVariable"
            @update:model-value="$emit('update-field', 'input_var', String($event || ''))"
            @create="createFlowInputVariable"
            @locate="jumpToFlowInputVariable"
          />

          <EditorVariableBindingField
            label="输出结果变量"
            :model-value="selectedFlowOutput || null"
            :options="resolvedFlowOutputOptions"
            placeholder="选择 JSON 结果变量"
            :test-id="selectedFlow.type === FLOW_TYPE.handlePolicySet ? 'editor-flow-policy-set-out-var' : 'editor-flow-policy-out-var'"
            create-label="新建结果变量"
            :show-create="Boolean(createVariable)"
            :show-locate="Boolean(selectedFlowOutputOption && jumpToVariable)"
            :locate-disabled="!selectedFlowOutputOption || !jumpToVariable"
            @update:model-value="$emit('update-field', 'out_var', String($event || ''))"
            @create="createFlowOutputVariable"
            @locate="jumpToFlowOutputVariable"
          />
        </div>

        <div class="rounded-[14px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4 text-xs leading-6 text-(--app-text-soft)">
          输出 JSON 约定：顶层摘要字段为 `matched`、`policySetId`、`policyGroupId`、`policyId`，逐轮明细写入 `rounds`。
          每个 round 内再保存 `pageFingerprints`、`actionSignatures`、`actions`，其中 `actions` 按 `actionIndex` 标识单轮中的动作顺序。
        </div>
      </div>
    </template>

    <template v-else-if="selectedFlow.type === FLOW_TYPE.continue || selectedFlow.type === FLOW_TYPE.break || selectedFlow.type === FLOW_TYPE.stopScript">
      <div class="rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4 text-sm leading-6 text-(--app-text-soft)">
        {{
          selectedFlow.type === FLOW_TYPE.continue
            ? '该步骤会立即开始下一轮循环。'
            : selectedFlow.type === FLOW_TYPE.break
              ? '该步骤会立即跳出当前循环。'
              : '该步骤会立即结束当前脚本执行。'
        }}
      </div>
    </template>

    <template v-else-if="selectedFlow.type === FLOW_TYPE.forEach">
      <div class="space-y-4 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
        <EditorVariableBindingField
          label="结果集变量"
          :model-value="selectedFlow.input_var || null"
          :options="resolvedForEachInputOptions"
          placeholder="绑定要遍历的结果集变量"
          test-id="editor-flow-for-each-input-var"
          create-label="新建结果集变量"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedForEachInputOption && jumpToVariable)"
          :locate-disabled="!selectedForEachInputOption || !jumpToVariable"
          @update:model-value="$emit('update-field', 'input_var', String($event || ''))"
          @create="createForEachInputVariable"
          @locate="jumpToSelectedForEachInputVariable"
        />

        <div class="grid gap-3 md:grid-cols-2">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">元素变量</span>
            <input
              :value="selectedFlow.item_var || ''"
              class="app-input"
              @input="$emit('update-field', 'item_var', ($event.target as HTMLInputElement).value)"
            />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">索引变量</span>
            <input
              :value="selectedFlow.index_var || ''"
              class="app-input"
              @input="$emit('update-field', 'index_var', ($event.target as HTMLInputElement).value)"
            />
          </label>
        </div>
      </div>
    </template>

    <template v-else-if="selectedFlow.type === FLOW_TYPE.repeat">
      <div class="space-y-4 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
        <EditorVariableBindingField
          label="次数变量"
          :model-value="selectedFlow.count_expr || null"
          :options="resolvedRepeatCountOptions"
          placeholder="绑定数字变量"
          test-id="editor-flow-repeat-count-var"
          create-label="新建次数变量"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedRepeatCountOption && jumpToVariable)"
          :locate-disabled="!selectedRepeatCountOption || !jumpToVariable"
          @update:model-value="$emit('update-field', 'count_expr', String($event || ''))"
          @create="createRepeatCountVariable"
          @locate="jumpToSelectedRepeatCountVariable"
        />

        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">索引变量</span>
          <input
            :value="selectedFlow.index_var || ''"
            class="app-input"
            placeholder="runtime.repeatIndex"
            @input="$emit('update-field', 'index_var', ($event.target as HTMLInputElement).value)"
          />
        </label>
      </div>
    </template>

    <template v-else-if="flowWithCondition && flowCondition">
      <div class="space-y-4 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
        <div class="flex flex-wrap items-start justify-between gap-3">
          <!-- 去掉步骤内切换其它类型步骤的功能 -->
          <!-- <EditorSelectField
            :model-value="flowWithCondition.type"
            :options="flowTypeOptions"
            placeholder="流程类型"
            class="min-w-[180px] flex-1"
            @update:model-value="$emit('update-flow-type', String($event || FLOW_TYPE.if))"
          /> -->

          <div class="flex flex-wrap items-center gap-2" v-if="branchTargets.length">
            <button
              v-for="target in branchTargets"
              :key="target.key"
              class="editor-branch-pill"
              :class="{ 'editor-branch-pill-active': isActiveBranch(target.path) }"
              type="button"
              :data-testid="`editor-branch-${target.key}`"
              @click="$emit('navigate-branch', target.path)"
            >
              <span>{{ target.label }}</span>
              <span class="editor-branch-pill-count">{{ target.count }}</span>
            </button>

            <button
              v-if="flowWithCondition.type === FLOW_TYPE.if"
              class="app-button app-button-primary app-toolbar-button justify-center"
              type="button"
              @click="$emit('toggle-else-branch')"
            >
              {{ hasElseBranch ? '移除 Else' : '添加 Else' }}
            </button>
          </div>
        </div>

        <div v-if="flowWithCondition.type !== FLOW_TYPE.if && branchSummary" class="text-xs text-(--app-text-faint)">
          {{ branchSummary }}
        </div>

        <EditorConditionBuilder
          :model-value="flowCondition"
          :variable-options="readableCatalogVariableOptions"
          :variable-reference-options="variableReferenceOptions"
          :variable-input-entries="variableInputEntries"
          :task-reference-options="taskReferenceOptions"
          :policy-reference-options="policyReferenceOptions"
          :policy-group-reference-options="policyGroupReferenceOptions"
          :policy-set-reference-options="policySetReferenceOptions"
          :create-reference="createReference"
          :jump-to-reference="jumpToReference"
          :create-variable="createVariable"
          :jump-to-variable="jumpToVariable"
          @update-input="(entryId, field, value) => emit('update-input', entryId, field, value)"
          test-id-prefix="editor-condition"
          @update:model-value="$emit('update-flow-condition', $event)"
        />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import EditorVariableBindingField from '@/views/script-editor/EditorVariableBindingField.vue';
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { FlowControl } from '@/types/bindings/FlowControl';
import EditorConditionBuilder from '@/views/script-editor/EditorConditionBuilder.vue';
import type { EditorReferenceKind, EditorReferenceOption } from '@/views/script-editor/editorReferences';
import { withResolvedReferenceOption } from '@/views/script-editor/editorReferences';
import { FLOW_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';
import { isSameBranchPath, type StepBranchPath } from '@/views/script-editor/editor-step/editorStepTree';
import { getVariableOptionSummary, type EditorInputEntry, type EditorInputType, type EditorVariableOption } from '@/views/script-editor/editorVariables';

type BindingFlow = Extract<
  FlowControl,
  { type: 'addPolicies' } | { type: 'bindPolicyGroup' } | { type: 'bindPolicy' }
>;

defineOptions({ name: 'EditorStepFlowPanel' });

const emit = defineEmits<{
  'update-number-field': [field: string, value: string];
  'update-field': [field: string, value: string];
  'update-boolean-field': [field: string, value: boolean];
  'update-flow-type': [type: string];
  'update-flow-condition': [condition: ConditionNode];
  'toggle-else-branch': [];
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
  'navigate-branch': [path: StepBranchPath];
}>();

const props = defineProps<{
  selectedFlow: FlowControl;
  flowWithCondition: { type: string; con: ConditionNode } | null;
  flowCondition: ConditionNode | null;
  hasElseBranch: boolean;
  branchSummary: string;
  branchTargets: Array<{ key: 'then' | 'else' | 'flow'; label: string; count: number; path: StepBranchPath }>;
  activeBranchPath: StepBranchPath;
  flowTypeOptions: Array<{ label: string; value: string; description: string }>;
  readableCatalogVariableOptions: Array<{ label: string; value: string; description: string }>;
  variableInputEntries?: EditorInputEntry[];
  variableReferenceOptions: EditorVariableOption[];
  taskReferenceOptions: EditorReferenceOption[];
  policyReferenceOptions: EditorReferenceOption[];
  policyGroupReferenceOptions: EditorReferenceOption[];
  policySetReferenceOptions: EditorReferenceOption[];
  createReference: (kind: EditorReferenceKind) => Promise<string>;
  jumpToReference: (kind: EditorReferenceKind, id: string) => void;
  createVariable?: (namespace?: 'input' | 'runtime', inputType?: EditorInputType, options?: { preferredKey?: string; name?: string; select?: boolean; silent?: boolean; focusEditor?: boolean }) => Promise<string>;
  jumpToVariable?: (option: EditorVariableOption) => void;
}>();

const isActiveBranch = (branchPath: StepBranchPath) => isSameBranchPath(branchPath, props.activeBranchPath);

const resolvedTaskReferenceOptions = computed(() =>
  withResolvedReferenceOption(props.taskReferenceOptions, selectedLinkTarget.value, 'task'),
);
const selectedLinkTarget = computed(() => (props.selectedFlow.type === FLOW_TYPE.link ? props.selectedFlow.target : ''));
const pendingTargetId = ref('');
const jsonVariableOptions = computed(() =>
  props.variableReferenceOptions
    .filter((option) => ['json', 'object', 'list'].includes(option.valueType))
    .map((option) => ({ label: option.label, value: option.key, description: getVariableOptionSummary(option) })),
);
const numberVariableOptions = computed(() =>
  props.variableReferenceOptions
    .filter((option) => ['int', 'float', 'string'].includes(option.valueType))
    .map((option) => ({ label: option.label, value: option.key, description: getVariableOptionSummary(option) })),
);
const imageVariableOptions = computed(() =>
  props.variableReferenceOptions
    .filter((option) => option.valueType === 'image')
    .map((option) => ({ label: option.label, value: option.key, description: getVariableOptionSummary(option) })),
);
const runtimeWaitVariableOptions = computed(() => {
  const options = props.variableReferenceOptions
    .filter((option) => option.namespace === 'runtime' && ['json', 'list', 'object'].includes(option.valueType))
    .map((option) => ({ label: option.label, value: option.key, description: getVariableOptionSummary(option) }));

  if (options.some((option) => option.value === 'runtime.ocrResults')) {
    return options;
  }

  return [
    {
      label: 'OCR 结果',
      value: 'runtime.ocrResults',
      description: '默认 OCR 输出变量，WaitMs 会从其中提取 00:00 或 00:00:00。',
    },
    ...options,
  ];
});
const waitBindingModeOptions = [
  { label: '固定毫秒', value: 'fixed', description: '始终按下方毫秒值等待。' },
  { label: '输入变量', value: 'input', description: '从 input 变量读取等待毫秒。' },
  { label: 'OCR 结果', value: 'runtime', description: '从 runtime OCR 结果里提取倒计时文本。' },
];
const waitBindingMode = computed(() => {
  if (props.selectedFlow.type !== FLOW_TYPE.waitMs) {
    return 'fixed';
  }
  if (props.selectedFlow.runtime_var?.trim()) {
    return 'runtime';
  }
  if (props.selectedFlow.input_var?.trim()) {
    return 'input';
  }
  return 'fixed';
});
const isBindingFlow = computed(
  () =>
    props.selectedFlow.type === FLOW_TYPE.addPolicies ||
    props.selectedFlow.type === FLOW_TYPE.bindPolicyGroup ||
    props.selectedFlow.type === FLOW_TYPE.bindPolicy,
);
const bindingFlow = computed<BindingFlow | null>(() =>
  isBindingFlow.value ? (props.selectedFlow as BindingFlow) : null,
);
const bindingSourceKind = computed<EditorReferenceKind | null>(() => {
  if (props.selectedFlow.type === FLOW_TYPE.addPolicies) return 'policySet';
  if (props.selectedFlow.type === FLOW_TYPE.bindPolicyGroup) return 'policyGroup';
  if (props.selectedFlow.type === FLOW_TYPE.bindPolicy) return 'policy';
  return null;
});
const bindingTargetKind = computed<EditorReferenceKind | null>(() => {
  if (props.selectedFlow.type === FLOW_TYPE.addPolicies) return 'policySet';
  if (props.selectedFlow.type === FLOW_TYPE.bindPolicyGroup) return 'policySet';
  if (props.selectedFlow.type === FLOW_TYPE.bindPolicy) return 'policyGroup';
  return null;
});
const bindingSourceId = computed(() => bindingFlow.value?.source ?? '');
const bindingTargetId = computed(() => bindingFlow.value?.target ?? '');
const bindingTopValue = computed(() => Boolean(bindingFlow.value?.top));
const bindingReverseValue = computed(() => Boolean(bindingFlow.value?.reverse));
const bindingSourceTitle = computed(() => {
  if (props.selectedFlow.type === FLOW_TYPE.addPolicies) return '源策略集';
  if (props.selectedFlow.type === FLOW_TYPE.bindPolicyGroup) return '源策略组';
  return '源策略';
});
const bindingTargetTitle = computed(() => {
  if (props.selectedFlow.type === FLOW_TYPE.bindPolicy) return '目标策略组';
  return '目标策略集';
});
const bindingSourcePlaceholder = computed(() => `选择${bindingSourceTitle.value}`);
const bindingTargetPlaceholder = computed(() => `选择${bindingTargetTitle.value}`);
const bindingSourceTestId = computed(() => {
  if (props.selectedFlow.type === FLOW_TYPE.addPolicies) return 'editor-flow-add-policies-source';
  if (props.selectedFlow.type === FLOW_TYPE.bindPolicyGroup) return 'editor-flow-bind-policy-group-source';
  return 'editor-flow-bind-policy-source';
});
const bindingTargetTestId = computed(() => {
  if (props.selectedFlow.type === FLOW_TYPE.addPolicies) return 'editor-flow-add-policies-target';
  if (props.selectedFlow.type === FLOW_TYPE.bindPolicyGroup) return 'editor-flow-bind-policy-group-target';
  return 'editor-flow-bind-policy-target';
});
const bindingTopTestId = computed(() => {
  if (props.selectedFlow.type === FLOW_TYPE.addPolicies) return 'editor-flow-add-policies-top';
  if (props.selectedFlow.type === FLOW_TYPE.bindPolicyGroup) return 'editor-flow-bind-policy-group-top';
  return 'editor-flow-bind-policy-top';
});
const bindingReverseTestId = computed(() => {
  if (props.selectedFlow.type === FLOW_TYPE.addPolicies) return 'editor-flow-add-policies-reverse';
  if (props.selectedFlow.type === FLOW_TYPE.bindPolicyGroup) return 'editor-flow-bind-policy-group-reverse';
  return 'editor-flow-bind-policy-reverse';
});
const bindingReverseDescription = computed(() => {
  if (props.selectedFlow.type === FLOW_TYPE.addPolicies) {
    return '会先按源策略集当前顺序展开策略组，再整体反转后插入目标策略集。';
  }
  if (props.selectedFlow.type === FLOW_TYPE.bindPolicyGroup) {
    return '单个策略组本身不会变成多个对象，但保留这个开关以统一绑定语义。';
  }
  return '单个策略本身不会变成多个对象，但保留这个开关以统一绑定语义。';
});
const bindingHelpText = computed(() => {
  if (props.selectedFlow.type === FLOW_TYPE.addPolicies) {
    return '运行时会读取源策略集当前可见的策略组顺序，再按 top / reverse 规则插入到目标策略集。后续处理目标策略集时会直接使用这份展开结果。';
  }
  if (props.selectedFlow.type === FLOW_TYPE.bindPolicyGroup) {
    return '运行时会把源策略组插入目标策略集。若源策略组之后又被追加了策略，目标策略集在执行时也会读到最新顺序。';
  }
  return '运行时会把源策略插入目标策略组。后续处理引用该策略组的策略集时，会使用插入后的最终策略顺序。';
});
const resolveReferenceOptions = (
  currentId: string,
  options: EditorReferenceOption[],
  unresolvedLabel: string,
) => {
  if (!currentId || options.some((option) => option.value === currentId)) {
    return options;
  }

  return [
    {
      label: `当前绑定不存在：${currentId}`,
      value: currentId,
      description: unresolvedLabel,
    },
    ...options,
  ];
};
const bindingSourceReferenceOptions = computed(() => {
  if (bindingSourceKind.value === 'policySet') {
    return resolveReferenceOptions(bindingSourceId.value, props.policySetReferenceOptions, '策略集目录里找不到该源绑定，保存时仍会保留当前值。');
  }
  if (bindingSourceKind.value === 'policyGroup') {
    return resolveReferenceOptions(bindingSourceId.value, props.policyGroupReferenceOptions, '策略组目录里找不到该源绑定，保存时仍会保留当前值。');
  }
  if (bindingSourceKind.value === 'policy') {
    return resolveReferenceOptions(bindingSourceId.value, props.policyReferenceOptions, '策略目录里找不到该源绑定，保存时仍会保留当前值。');
  }
  return [];
});
const bindingTargetReferenceOptions = computed(() => {
  if (bindingTargetKind.value === 'policySet') {
    return resolveReferenceOptions(bindingTargetId.value, props.policySetReferenceOptions, '策略集目录里找不到该目标绑定，保存时仍会保留当前值。');
  }
  if (bindingTargetKind.value === 'policyGroup') {
    return resolveReferenceOptions(bindingTargetId.value, props.policyGroupReferenceOptions, '策略组目录里找不到该目标绑定，保存时仍会保留当前值。');
  }
  return [];
});
const selectedFlowInput = computed(() =>
  props.selectedFlow.type === FLOW_TYPE.handlePolicySet || props.selectedFlow.type === FLOW_TYPE.handlePolicy ? props.selectedFlow.input_var : '',
);
const selectedFlowOutput = computed(() =>
  props.selectedFlow.type === FLOW_TYPE.handlePolicySet || props.selectedFlow.type === FLOW_TYPE.handlePolicy ? props.selectedFlow.out_var : '',
);
const selectedFlowInputOption = computed(() =>
  props.variableReferenceOptions.find((option) => option.key === selectedFlowInput.value) ?? null,
);
const selectedFlowOutputOption = computed(() =>
  props.variableReferenceOptions.find((option) => option.key === selectedFlowOutput.value) ?? null,
);
const selectedWaitInputOption = computed(() => {
  const flow = props.selectedFlow;
  if (flow.type !== FLOW_TYPE.waitMs) {
    return null;
  }
  return flow.input_var
    ? props.variableReferenceOptions.find((option) => option.key === flow.input_var) ?? null
    : null;
});
const selectedWaitRuntimeOption = computed(() => {
  const flow = props.selectedFlow;
  if (flow.type !== FLOW_TYPE.waitMs) {
    return null;
  }
  return flow.runtime_var
    ? props.variableReferenceOptions.find((option) => option.key === flow.runtime_var) ?? null
    : null;
});
const selectedForEachInputOption = computed(() => {
  const flow = props.selectedFlow;
  if (flow.type !== FLOW_TYPE.forEach) {
    return null;
  }
  return flow.input_var
    ? props.variableReferenceOptions.find((option) => option.key === flow.input_var) ?? null
    : null;
});
const selectedRepeatCountOption = computed(() => {
  const flow = props.selectedFlow;
  if (flow.type !== FLOW_TYPE.repeat) {
    return null;
  }
  return flow.count_expr
    ? props.variableReferenceOptions.find((option) => option.key === flow.count_expr) ?? null
    : null;
});
const resolvedFlowInputOptions = computed(() => {
  if (!selectedFlowInput.value || imageVariableOptions.value.some((option) => option.value === selectedFlowInput.value)) {
    return imageVariableOptions.value;
  }

  return [
    {
      label: `当前绑定不存在：${selectedFlowInput.value}`,
      value: selectedFlowInput.value,
      description: '变量目录里找不到该绑定，保存时仍会保留当前值。',
    },
    ...imageVariableOptions.value,
  ];
});
const resolvedFlowOutputOptions = computed(() => {
  if (!selectedFlowOutput.value || jsonVariableOptions.value.some((option) => option.value === selectedFlowOutput.value)) {
    return jsonVariableOptions.value;
  }

  return [
    {
      label: `当前绑定不存在：${selectedFlowOutput.value}`,
      value: selectedFlowOutput.value,
      description: '变量目录里找不到该绑定，保存时仍会保留当前值。',
    },
    ...jsonVariableOptions.value,
  ];
});
const resolvedForEachInputOptions = computed(() => {
  const flow = props.selectedFlow;
  if (flow.type !== FLOW_TYPE.forEach) {
    return jsonVariableOptions.value;
  }

  if (!flow.input_var || jsonVariableOptions.value.some((option) => option.value === flow.input_var)) {
    return jsonVariableOptions.value;
  }

  return [
    {
      label: `当前绑定不存在：${flow.input_var}`,
      value: flow.input_var,
      description: '变量目录里找不到该结果集绑定，保存时仍会保留当前值。',
    },
    ...jsonVariableOptions.value,
  ];
});
const resolvedRepeatCountOptions = computed(() => {
  const flow = props.selectedFlow;
  if (flow.type !== FLOW_TYPE.repeat) {
    return numberVariableOptions.value;
  }

  if (!flow.count_expr || numberVariableOptions.value.some((option) => option.value === flow.count_expr)) {
    return numberVariableOptions.value;
  }

  return [
    {
      label: `当前绑定不存在：${flow.count_expr}`,
      value: flow.count_expr,
      description: '变量目录里找不到该次数绑定，保存时仍会保留当前值。',
    },
    ...numberVariableOptions.value,
  ];
});
const resolvedWaitInputOptions = computed(() => {
  const flow = props.selectedFlow;
  if (flow.type !== FLOW_TYPE.waitMs) {
    return numberVariableOptions.value;
  }

  if (!flow.input_var || numberVariableOptions.value.some((option) => option.value === flow.input_var)) {
    return numberVariableOptions.value;
  }

  return [
    {
      label: `当前绑定不存在：${flow.input_var}`,
      value: flow.input_var,
      description: '变量目录里找不到该输入绑定，保存时仍会保留当前值。',
    },
    ...numberVariableOptions.value,
  ];
});
const resolvedWaitRuntimeOptions = computed(() => {
  const flow = props.selectedFlow;
  if (flow.type !== FLOW_TYPE.waitMs) {
    return runtimeWaitVariableOptions.value;
  }

  if (!flow.runtime_var || runtimeWaitVariableOptions.value.some((option) => option.value === flow.runtime_var)) {
    return runtimeWaitVariableOptions.value;
  }

  return [
    {
      label: `当前绑定不存在：${flow.runtime_var}`,
      value: flow.runtime_var,
      description: '变量目录里找不到该运行时绑定，保存时仍会保留当前值。',
    },
    ...runtimeWaitVariableOptions.value,
  ];
});
const availableTargetReferenceOptions = computed(() => {
  if (props.selectedFlow.type === FLOW_TYPE.handlePolicySet) {
    const selectedIds = new Set(props.selectedFlow.target);
    return props.policySetReferenceOptions.filter((option) => !selectedIds.has(option.value));
  }

  if (props.selectedFlow.type === FLOW_TYPE.handlePolicy) {
    const selectedIds = new Set(props.selectedFlow.target);
    return props.policyReferenceOptions.filter((option) => !selectedIds.has(option.value));
  }

  return [];
});
const resolvedTargets = computed(() => {
  if (props.selectedFlow.type === FLOW_TYPE.handlePolicySet) {
    return props.selectedFlow.target.map((id) => {
      const matched = props.policySetReferenceOptions.find((option) => option.value === id);
      return {
        id,
        label: matched?.label || '未解析策略集',
        description: matched?.description || `保留历史引用 ${id}`,
      };
    });
  }

  if (props.selectedFlow.type === FLOW_TYPE.handlePolicy) {
    return props.selectedFlow.target.map((id) => {
      const matched = props.policyReferenceOptions.find((option) => option.value === id);
      return {
        id,
        label: matched?.label || '未解析策略',
        description: matched?.description || `保留历史引用 ${id}`,
      };
    });
  }

  return [];
});
const emptyTargetText = computed(() =>
  props.selectedFlow.type === FLOW_TYPE.handlePolicySet ? '还没有绑定策略集，运行时不会执行任何匹配。' : '还没有绑定策略，运行时不会执行任何匹配。',
);
const targetTitle = computed(() => (props.selectedFlow.type === FLOW_TYPE.handlePolicySet ? '目标策略集' : '目标策略'));
const targetPlaceholder = computed(() => (props.selectedFlow.type === FLOW_TYPE.handlePolicySet ? '选择策略集后添加' : '选择策略后添加'));

const createTaskReferenceAndBind = async () => {
  const id = await props.createReference('task');
  emit('update-field', 'target', id);
};

const createBindingSourceReference = async () => {
  if (!bindingSourceKind.value) {
    return;
  }
  const id = await props.createReference(bindingSourceKind.value);
  emit('update-field', 'source', id);
};

const createBindingTargetReference = async () => {
  if (!bindingTargetKind.value) {
    return;
  }
  const id = await props.createReference(bindingTargetKind.value);
  emit('update-field', 'target', id);
};

const jumpToBindingSource = () => {
  if (!bindingSourceKind.value || !bindingSourceId.value) {
    return;
  }
  props.jumpToReference(bindingSourceKind.value, bindingSourceId.value);
};

const jumpToBindingTarget = () => {
  if (!bindingTargetKind.value || !bindingTargetId.value) {
    return;
  }
  props.jumpToReference(bindingTargetKind.value, bindingTargetId.value);
};

const jumpToLinkedTask = () => {
  if (!selectedLinkTarget.value) {
    return;
  }
  props.jumpToReference('task', selectedLinkTarget.value);
};

const appendTarget = () => {
  if ((props.selectedFlow.type !== FLOW_TYPE.handlePolicySet && props.selectedFlow.type !== FLOW_TYPE.handlePolicy) || !pendingTargetId.value) {
    return;
  }

  emit('update-field', 'target', JSON.stringify([...props.selectedFlow.target, pendingTargetId.value]));
  pendingTargetId.value = '';
};

const createRepeatCountVariable = async () => {
  if (!props.createVariable || props.selectedFlow.type !== FLOW_TYPE.repeat) {
    return;
  }
  const key = await props.createVariable('input', 'int', {
    preferredKey: 'repeatCount',
    name: '循环次数',
    focusEditor: true,
  });
  if (key) {
    emit('update-field', 'count_expr', key);
  }
};

const createWaitInputVariable = async () => {
  if (!props.createVariable || props.selectedFlow.type !== FLOW_TYPE.waitMs) {
    return;
  }
  const key = await props.createVariable('input', 'int', {
    preferredKey: 'waitMs',
    name: '等待毫秒',
    focusEditor: true,
  });
  if (key) {
    emit('update-field', 'input_var', key);
  }
};

const createWaitRuntimeVariable = async () => {
  if (!props.createVariable || props.selectedFlow.type !== FLOW_TYPE.waitMs) {
    return;
  }
  const key = await props.createVariable('runtime', 'json', {
    preferredKey: 'ocrResults',
    name: 'OCR结果',
    focusEditor: true,
  });
  if (key) {
    emit('update-field', 'runtime_var', key);
  }
};

const createForEachInputVariable = async () => {
  if (!props.createVariable || props.selectedFlow.type !== FLOW_TYPE.forEach) {
    return;
  }
  const key = await props.createVariable('runtime', 'json', {
    preferredKey: 'items',
    name: '结果集',
    focusEditor: true,
  });
  if (key) {
    emit('update-field', 'input_var', key);
  }
};

const updateWaitBindingMode = (mode: string) => {
  if (props.selectedFlow.type !== FLOW_TYPE.waitMs) {
    return;
  }

  if (mode === 'input') {
    emit('update-field', 'runtime_var', '');
    return;
  }

  if (mode === 'runtime') {
    emit('update-field', 'input_var', '');
    if (!props.selectedFlow.runtime_var?.trim()) {
      emit('update-field', 'runtime_var', 'runtime.ocrResults');
    }
    return;
  }

  emit('update-field', 'input_var', '');
  emit('update-field', 'runtime_var', '');
};

const removeTarget = (targetId: string) => {
  if (props.selectedFlow.type !== FLOW_TYPE.handlePolicySet && props.selectedFlow.type !== FLOW_TYPE.handlePolicy) {
    return;
  }

  emit('update-field', 'target', JSON.stringify(props.selectedFlow.target.filter((item) => item !== targetId)));
};

const jumpToTarget = (targetId: string) => {
  props.jumpToReference(props.selectedFlow.type === FLOW_TYPE.handlePolicySet ? 'policySet' : 'policy', targetId);
};

const createFlowInputVariable = async () => {
  if (!props.createVariable) {
    return;
  }

  const key = await props.createVariable('runtime', 'image', {
    preferredKey: 'captureResult',
    name: '截图结果',
    focusEditor: true,
  });
  if (!key) {
    return;
  }
  emit('update-field', 'input_var', key);
};

const createFlowOutputVariable = async () => {
  if (!props.createVariable) {
    return;
  }

  const key = await props.createVariable('runtime', 'json', {
    preferredKey: props.selectedFlow.type === FLOW_TYPE.handlePolicySet ? 'policySetResult' : 'policyResult',
    name: props.selectedFlow.type === FLOW_TYPE.handlePolicySet ? '策略集结果' : '策略结果',
    focusEditor: true,
  });
  if (!key) {
    return;
  }
  emit('update-field', 'out_var', key);
};

const jumpToFlowInputVariable = () => {
  if (!selectedFlowInputOption.value || !props.jumpToVariable) {
    return;
  }
  props.jumpToVariable(selectedFlowInputOption.value);
};

const jumpToFlowOutputVariable = () => {
  if (!selectedFlowOutputOption.value || !props.jumpToVariable) {
    return;
  }
  props.jumpToVariable(selectedFlowOutputOption.value);
};
const jumpToSelectedWaitInputVariable = () => {
  if (!selectedWaitInputOption.value || !props.jumpToVariable) {
    return;
  }
  props.jumpToVariable(selectedWaitInputOption.value);
};
const jumpToSelectedWaitRuntimeVariable = () => {
  if (!selectedWaitRuntimeOption.value || !props.jumpToVariable) {
    return;
  }
  props.jumpToVariable(selectedWaitRuntimeOption.value);
};
const jumpToSelectedForEachInputVariable = () => {
  if (!selectedForEachInputOption.value || !props.jumpToVariable) {
    return;
  }
  props.jumpToVariable(selectedForEachInputOption.value);
};
const jumpToSelectedRepeatCountVariable = () => {
  if (!selectedRepeatCountOption.value || !props.jumpToVariable) {
    return;
  }
  props.jumpToVariable(selectedRepeatCountOption.value);
};
</script>

<style scoped>
.editor-inline-grid {
  display: grid;
  gap: 0.75rem;
}

@media (min-width: 1280px) {
  .editor-inline-grid {
    grid-template-columns: 78px minmax(0, 1fr) 78px minmax(0, 1fr);
    align-items: center;
  }
}

.editor-inline-label {
  display: flex;
  align-items: center;
  min-height: 44px;
  color: var(--app-text-faint);
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.editor-inline-content {
  min-height: 44px;
}

.editor-branch-pill {
  display: inline-flex;
  align-items: center;
  gap: 0.55rem;
  border-radius: 999px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.58);
  padding: 0.5rem 0.8rem;
  color: var(--app-text-soft);
  font-size: 0.86rem;
  font-weight: 600;
  transition: border-color 0.16s ease, background 0.16s ease, color 0.16s ease;
}

.editor-branch-pill:hover {
  border-color: color-mix(in srgb, var(--app-accent) 48%, white);
  color: var(--app-text-strong);
}

.editor-branch-pill-active {
  border-color: color-mix(in srgb, var(--app-accent) 70%, white);
  background: color-mix(in srgb, var(--app-accent) 14%, white);
  color: var(--app-text-strong);
}

.editor-branch-pill-count {
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.72);
  padding: 0.1rem 0.45rem;
  font-size: 0.74rem;
  color: var(--app-text-faint);
}
</style>
