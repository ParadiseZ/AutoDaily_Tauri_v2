<template>
  <div class="space-y-3">
    <template v-if="selectedFlow.type === FLOW_TYPE.waitMs">
      <label class="space-y-2">
        <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">等待毫秒</span>
        <input :value="String(selectedFlow.ms ?? 1000)" class="app-input" type="number" @input="$emit('update-number-field', 'ms', ($event.target as HTMLInputElement).value)" />
      </label>
    </template>

    <template v-else-if="selectedFlow.type === FLOW_TYPE.link">
      <div class="space-y-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
        <div class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标任务</span>
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

    <template v-else-if="selectedFlow.type === FLOW_TYPE.handlePolicySet || selectedFlow.type === FLOW_TYPE.handlePolicy">
      <div class="space-y-4 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
        <div class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">{{ targetTitle }}</span>
          <div class="grid gap-2 md:grid-cols-[minmax(0,1fr)_auto]">
            <EditorSelectField
              :model-value="pendingTargetId"
              :options="availableTargetReferenceOptions"
              :placeholder="targetPlaceholder"
              :test-id="selectedFlow.type === FLOW_TYPE.handlePolicySet ? 'editor-flow-policy-set-pending' : 'editor-flow-policy-pending'"
              @update:model-value="pendingTargetId = String($event || '')"
            />
            <button
              class="app-button app-button-primary app-toolbar-button"
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
            class="flex items-center justify-between gap-3 rounded-[14px] border border-[var(--app-border)] bg-white/55 px-3 py-3"
          >
            <div class="min-w-0">
              <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ target.label }}</p>
              <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ target.description }}</p>
            </div>
            <div class="flex items-center gap-2">
              <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="jumpToTarget(target.id)">
                <AppIcon name="locate-fixed" :size="14" />
                定位
              </button>
              <button class="app-button app-button-danger app-toolbar-button" type="button" @click="removeTarget(target.id)">
                移除
              </button>
            </div>
          </article>
        </div>

        <div v-else class="rounded-[14px] border border-dashed border-[var(--app-border)] px-4 py-4 text-sm text-[var(--app-text-soft)]">
          还没有绑定策略集，运行时不会执行任何匹配。
        </div>

        <div class="grid gap-4 xl:grid-cols-2">
          <div class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输入图像变量</span>
            <EditorSelectField
              :model-value="selectedFlowInput || null"
              :options="resolvedFlowInputOptions"
              :show-description="true"
              placeholder="选择截图或图像变量"
              :test-id="selectedFlow.type === FLOW_TYPE.handlePolicySet ? 'editor-flow-policy-set-input-var' : 'editor-flow-policy-input-var'"
              @update:model-value="$emit('update-field', 'input_var', String($event || ''))"
            />
            <div class="flex flex-wrap gap-2">
              <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="createFlowInputVariable">
                <AppIcon name="plus" :size="14" />
                新建图像变量
              </button>
              <button
                class="app-button app-button-ghost app-toolbar-button"
                type="button"
                :disabled="!selectedFlowInputOption"
                @click="jumpToFlowInputVariable"
              >
                <AppIcon name="locate-fixed" :size="14" />
                定位变量
              </button>
            </div>
          </div>

          <div class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输出结果变量</span>
            <EditorSelectField
              :model-value="selectedFlowOutput || null"
              :options="resolvedFlowOutputOptions"
              :show-description="true"
              placeholder="选择 JSON 结果变量"
              :test-id="selectedFlow.type === FLOW_TYPE.handlePolicySet ? 'editor-flow-policy-set-out-var' : 'editor-flow-policy-out-var'"
              @update:model-value="$emit('update-field', 'out_var', String($event || ''))"
            />
            <div class="flex flex-wrap gap-2">
              <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="createFlowOutputVariable">
                <AppIcon name="plus" :size="14" />
                新建结果变量
              </button>
              <button
                class="app-button app-button-ghost app-toolbar-button"
                type="button"
                :disabled="!selectedFlowOutputOption"
                @click="jumpToFlowOutputVariable"
              >
                <AppIcon name="locate-fixed" :size="14" />
                定位变量
              </button>
            </div>
          </div>
        </div>

        <div class="rounded-[14px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4 text-xs leading-6 text-[var(--app-text-soft)]">
          输出 JSON 约定：顶层摘要字段为 `matched`、`policySetId`、`policyGroupId`、`policyId`，逐轮明细写入 `rounds`。
          每个 round 内再保存 `pageFingerprints`、`actionSignatures`、`actions`，其中 `actions` 按 `actionIndex` 标识单轮中的动作顺序。
        </div>
      </div>
    </template>

    <template v-else-if="selectedFlow.type === FLOW_TYPE.continue || selectedFlow.type === FLOW_TYPE.break">
      <div class="rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4 text-sm leading-6 text-[var(--app-text-soft)]">
        {{ selectedFlow.type === FLOW_TYPE.continue ? '该步骤会立即开始下一轮循环。' : '该步骤会立即跳出当前循环。' }}
      </div>
    </template>

    <template v-else-if="selectedFlow.type === FLOW_TYPE.forEach">
      <div class="space-y-4 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输入集合变量</span>
          <EditorSelectField
            :model-value="selectedFlow.input_var || null"
            :options="resolvedForEachInputOptions"
            :show-description="true"
            placeholder="选择数组或 JSON 变量"
            test-id="editor-flow-for-each-input-var"
            @update:model-value="$emit('update-field', 'input_var', String($event || ''))"
          />
        </label>

        <div class="grid gap-3 md:grid-cols-2">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">元素变量</span>
            <input
              :value="selectedFlow.item_var || ''"
              class="app-input"
              @input="$emit('update-field', 'item_var', ($event.target as HTMLInputElement).value)"
            />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">索引变量</span>
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
      <div class="space-y-4 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">循环次数表达式</span>
          <input
            :value="selectedFlow.count_expr || ''"
            class="app-input"
            placeholder="例如：input.count"
            @input="$emit('update-field', 'count_expr', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">索引变量</span>
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
      <div class="grid gap-3 xl:grid-cols-[minmax(0,1fr)_220px]">
        <div class="editor-inline-grid">
          <div class="editor-inline-label">流程类型</div>
          <div class="editor-inline-content xl:col-span-3">
            <EditorSelectField
              :model-value="flowWithCondition.type"
              :options="flowTypeOptions"
              placeholder="流程类型"
              @update:model-value="$emit('update-flow-type', String($event || FLOW_TYPE.if))"
            />
          </div>
        </div>

        <div class="rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-3">
          <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">分支概览</p>
          <div class="mt-2 flex flex-wrap items-center justify-between gap-3">
            <span class="text-sm text-[var(--app-text-soft)]">{{ branchSummary }}</span>
            <button
              v-if="flowWithCondition.type === FLOW_TYPE.if"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="$emit('toggle-else-branch')"
            >
              {{ hasElseBranch ? '移除 Else' : '添加 Else' }}
            </button>
          </div>
        </div>
      </div>

      <div class="space-y-2">
        <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">条件</p>
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
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { FlowControl } from '@/types/bindings/FlowControl';
import EditorConditionBuilder from '@/views/script-editor/EditorConditionBuilder.vue';
import type { EditorReferenceKind, EditorReferenceOption } from '@/views/script-editor/editorReferences';
import { withResolvedReferenceOption } from '@/views/script-editor/editorReferences';
import { FLOW_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';
import type { EditorInputEntry, EditorInputType, EditorVariableOption } from '@/views/script-editor/editorVariables';

defineOptions({ name: 'EditorStepFlowPanel' });

const emit = defineEmits<{
  'update-number-field': [field: string, value: string];
  'update-field': [field: string, value: string];
  'update-flow-type': [type: string];
  'update-flow-condition': [condition: ConditionNode];
  'toggle-else-branch': [];
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
}>();

const props = defineProps<{
  selectedFlow: FlowControl;
  flowWithCondition: { type: string; con: ConditionNode } | null;
  flowCondition: ConditionNode | null;
  hasElseBranch: boolean;
  branchSummary: string;
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
  createVariable?: (namespace?: 'input' | 'runtime', inputType?: EditorInputType, options?: { preferredKey?: string; name?: string; select?: boolean; silent?: boolean }) => Promise<string>;
  jumpToVariable?: (option: EditorVariableOption) => void;
}>();

const resolvedTaskReferenceOptions = computed(() =>
  withResolvedReferenceOption(props.taskReferenceOptions, selectedLinkTarget.value, 'task'),
);
const selectedLinkTarget = computed(() => (props.selectedFlow.type === FLOW_TYPE.link ? props.selectedFlow.target : ''));
const pendingTargetId = ref('');
const jsonVariableOptions = computed(() =>
  props.variableReferenceOptions
    .filter((option) => ['json', 'object', 'list'].includes(option.valueType))
    .map((option) => ({ label: option.label, value: option.key, description: option.description })),
);
const imageVariableOptions = computed(() =>
  props.variableReferenceOptions
    .filter((option) => option.valueType === 'image')
    .map((option) => ({ label: option.label, value: option.key, description: option.description })),
);
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
const resolvedFlowInputOptions = computed(() => {
  if (!selectedFlowInput.value || imageVariableOptions.value.some((option) => option.value === selectedFlowInput.value)) {
    return imageVariableOptions.value;
  }

  return [
    {
      label: `未解析变量 ${selectedFlowInput.value}`,
      value: selectedFlowInput.value,
      description: `保留历史输入 ${selectedFlowInput.value}`,
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
      label: `未解析变量 ${selectedFlowOutput.value}`,
      value: selectedFlowOutput.value,
      description: `保留历史输出 ${selectedFlowOutput.value}`,
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
      label: `未解析变量 ${flow.input_var}`,
      value: flow.input_var,
      description: `保留历史输入 ${flow.input_var}`,
    },
    ...jsonVariableOptions.value,
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
const targetTitle = computed(() => (props.selectedFlow.type === FLOW_TYPE.handlePolicySet ? '目标策略集' : '目标策略'));
const targetPlaceholder = computed(() => (props.selectedFlow.type === FLOW_TYPE.handlePolicySet ? '选择策略集后添加' : '选择策略后添加'));

const createTaskReferenceAndBind = async () => {
  const id = await props.createReference('task');
  emit('update-field', 'target', id);
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
    preferredKey: props.selectedFlow.type === FLOW_TYPE.handlePolicySet ? 'policySetImage' : 'policyImage',
    name: props.selectedFlow.type === FLOW_TYPE.handlePolicySet ? '策略集输入图像' : '策略输入图像',
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
</style>
