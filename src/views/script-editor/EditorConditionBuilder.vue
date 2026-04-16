<template>
  <div class="editor-condition-card" :class="{ 'editor-condition-nested': depth > 0 }">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="flex min-w-0 flex-1 flex-wrap items-center gap-3">
      <EditorSelectField
          :model-value="modelValue.type"
          :options="conditionTypeOptions"
          placeholder="条件类型"
          class="min-w-[180px]"
          :test-id="rootTestId('type')"
          @update:model-value="changeType(String($event || 'rawExpr'))"
        />
        <span class="truncate text-xs text-[var(--app-text-faint)]">{{ conditionSummary }}</span>
      </div>

      <button
        v-if="removable"
        class="app-button app-button-danger app-toolbar-button"
        type="button"
        @click="$emit('remove')"
      >
        删除条件
      </button>
    </div>

    <div class="mt-4 space-y-3">
      <template v-if="modelValue.type === 'rawExpr'">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">表达式</span>
          <input
            :value="modelValue.expr"
            class="app-input"
            :data-testid="rootTestId('raw-expr')"
            @input="replaceNode({ ...modelValue, expr: ($event.target as HTMLInputElement).value })"
          />
        </label>
      </template>

      <template v-else-if="modelValue.type === 'group'">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">组合逻辑</span>
          <EditorSelectField
            :model-value="modelValue.op"
            :options="logicOpOptions"
            placeholder="组合逻辑"
            :test-id="rootTestId('logic-op')"
            @update:model-value="updateGroupOp(String($event || 'And'))"
          />
        </label>

        <div class="flex flex-wrap gap-2">
          <button
            v-for="option in addableConditionTypes"
            :key="option.value"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            @click="addGroupItem(option.value)"
          >
            添加{{ option.label }}
          </button>
        </div>

        <div class="space-y-3">
          <EditorConditionBuilder
            v-for="(item, index) in modelValue.items"
            :key="`${item.type}-${index}`"
            :model-value="item"
            :depth="depth + 1"
            removable
            :variable-options="variableOptions"
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
            @update:model-value="updateGroupItem(index, $event)"
            @remove="removeGroupItem(index)"
          />
        </div>

        <EmptyState
          v-if="!modelValue.items.length"
          title="还没有子条件"
          description="先添加表达式、颜色、任务状态等子条件。"
        />
      </template>

      <template v-else-if="modelValue.type === 'execNumCompare'">
        <div class="editor-inline-grid">
          <div class="editor-inline-label">目标类型</div>
          <div class="editor-inline-content">
            <EditorSelectField
              :model-value="modelValue.target.type"
              :options="stateTargetTypeOptions"
              placeholder="目标类型"
              @update:model-value="updateExecTargetType(String($event || 'task'))"
            />
          </div>

          <div class="editor-inline-label">目标资源</div>
          <div class="editor-inline-content md:col-span-3">
            <EditorSelectField
              :model-value="modelValue.target.id || null"
              :options="resolvedExecTargetOptions"
              placeholder="选择任务或策略"
              @update:model-value="updateExecTargetId(String($event || ''))"
            />
          </div>

          <div class="editor-inline-label">比较方式</div>
          <div class="editor-inline-content">
            <EditorSelectField
              :model-value="modelValue.op"
              :options="execCompareOpOptions"
              placeholder="比较方式"
              @update:model-value="updateExecCompareOp(String($event || 'ge'))"
            />
          </div>
        </div>

        <div class="flex flex-wrap gap-2">
          <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="createExecTargetReference">
            <AppIcon name="plus" :size="14" />
            新建{{ modelValue.target.type === 'task' ? '任务' : '策略' }}
          </button>
          <button
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            :disabled="!modelValue.target.id"
            @click="jumpToExecTargetReference"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位编辑
          </button>
        </div>
      </template>

      <template v-else-if="modelValue.type === 'taskStatus'">
        <div class="editor-inline-grid">
          <div class="editor-inline-label">状态动作</div>
          <div class="editor-inline-content">
            <EditorSelectField
              :model-value="modelValue.a.type"
              :options="taskControlTypeOptions"
              placeholder="状态动作"
              @update:model-value="updateTaskStatusField('type', String($event || 'setState'))"
            />
          </div>

          <div class="editor-inline-label">目标类型</div>
          <div class="editor-inline-content">
            <EditorSelectField
              :model-value="modelValue.a.target.type"
              :options="stateTargetTypeOptions"
              placeholder="目标类型"
              @update:model-value="updateTaskTargetType(String($event || 'task'))"
            />
          </div>

          <div class="editor-inline-label">目标资源</div>
          <div class="editor-inline-content md:col-span-3">
            <EditorSelectField
              :model-value="modelValue.a.target.id || null"
              :options="resolvedTaskStatusTargetOptions"
              placeholder="选择任务或策略"
              @update:model-value="updateTaskTargetId(String($event || ''))"
            />
          </div>

          <div class="editor-inline-label">状态类型</div>
          <div class="editor-inline-content">
            <EditorSelectField
              :model-value="modelValue.a.status.type"
              :options="filteredStateStatusTypeOptions"
              placeholder="状态类型"
              @update:model-value="updateTaskStatusType(String($event || 'done'))"
            />
          </div>
        </div>

        <div class="flex flex-wrap gap-2">
          <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="createTaskStatusTargetReference">
            <AppIcon name="plus" :size="14" />
            新建{{ modelValue.a.target.type === 'task' ? '任务' : '策略' }}
          </button>
          <button
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            :disabled="!modelValue.a.target.id"
            @click="jumpToTaskStatusTargetReference"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位编辑
          </button>
        </div>

        <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
          <input
            :checked="Boolean(modelValue.a.status.value)"
            type="checkbox"
            class="h-4 w-4"
            style="accent-color: var(--app-accent)"
            @change="updateTaskStatusValue(($event.target as HTMLInputElement).checked)"
          />
          <span class="text-sm text-[var(--app-text-soft)]">状态值为真</span>
        </label>
      </template>

      <template v-else-if="modelValue.type === 'varCompare'">
        <div class="editor-inline-grid">
          <div class="editor-inline-label">变量名称</div>
          <div class="editor-inline-content md:col-span-3">
            <EditorSelectField
              :model-value="modelValue.var_name || null"
              :options="variableOptions"
              :show-description="true"
              placeholder="从变量列表中选择"
              :test-id="rootTestId('var-name')"
              @update:model-value="updateVarCompareField('var_name', String($event || ''))"
            />
          </div>

          <div class="editor-inline-label">比较方式</div>
          <div class="editor-inline-content">
            <EditorSelectField
              :model-value="modelValue.op"
              :options="compareOpOptions"
              placeholder="比较方式"
              @update:model-value="updateVarCompareField('op', String($event || 'eq'))"
            />
          </div>

          <div class="editor-inline-label">值类型</div>
          <div class="editor-inline-content">
            <EditorSelectField
              :model-value="currentVarValueDraft.kind"
              :options="varValueTypeOptions"
              placeholder="值类型"
              @update:model-value="updateVarCompareValueKind(String($event || 'string'))"
            />
          </div>
        </div>

        <div v-if="createVariable || (selectedVarCompareOption && jumpToVariable)" class="flex flex-wrap gap-2">
          <button v-if="createVariable" class="app-button app-button-ghost app-toolbar-button" type="button" @click="createVarCompareVariable">
            <AppIcon name="plus" :size="14" />
            新建变量
          </button>
          <button
            v-if="selectedVarCompareOption && jumpToVariable"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            @click="jumpToVarCompareVariable"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位变量
          </button>
        </div>

        <EditorVariableMetaCard
          v-if="selectedVarCompareOption"
          :variable="selectedVarCompareOption"
          :input-entry="selectedVarCompareInputEntry"
          editable
          @update-input="(entryId, field, value) => emit('update-input', entryId, field, value)"
        />

        <label v-if="currentVarValueDraft.kind === 'bool'" class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
          <input
            :checked="currentVarValueDraft.boolValue"
            type="checkbox"
            class="h-4 w-4"
            style="accent-color: var(--app-accent)"
            @change="updateVarCompareBool(($event.target as HTMLInputElement).checked)"
          />
          <span class="text-sm text-[var(--app-text-soft)]">比较值为真</span>
        </label>

        <label v-else class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">比较值</span>
          <input
            :value="currentVarValueDraft.textValue"
            class="app-input"
            :type="currentVarValueDraft.kind === 'string' ? 'text' : 'number'"
            @input="updateVarCompareText(($event.target as HTMLInputElement).value)"
          />
        </label>
      </template>

      <template v-else-if="modelValue.type === 'policySetResult'">
        <div class="editor-inline-grid">
          <div class="editor-inline-label">结果变量</div>
          <div class="editor-inline-content md:col-span-3">
            <EditorSelectField
              :model-value="modelValue.result_var || null"
              :options="resolvedPolicySetResultVarOptions"
              :show-description="true"
              placeholder="选择策略集结果变量"
              :test-id="rootTestId('policy-set-result-var')"
              @update:model-value="updatePolicySetResultField('result_var', String($event || ''))"
            />
          </div>

          <div class="editor-inline-label">判断字段</div>
          <div class="editor-inline-content">
            <EditorSelectField
              :model-value="modelValue.field"
              :options="policySetResultFieldOptions"
              placeholder="结果字段"
              :test-id="rootTestId('policy-set-result-field')"
              @update:model-value="updatePolicySetResultFieldType(String($event || 'policyId'))"
            />
          </div>

          <div class="editor-inline-label">比较方式</div>
          <div class="editor-inline-content">
            <EditorSelectField
              :model-value="modelValue.op"
              :options="policySetResultCompareOptions"
              placeholder="比较方式"
              :test-id="rootTestId('policy-set-result-op')"
              @update:model-value="updatePolicySetResultCompareOp(String($event || 'eq'))"
            />
          </div>
        </div>

        <div class="flex flex-wrap gap-2">
          <button v-if="createVariable" class="app-button app-button-ghost app-toolbar-button" type="button" @click="createPolicySetResultVariable">
            <AppIcon name="plus" :size="14" />
            新建结果变量
          </button>
          <button
            v-if="selectedPolicySetResultVarOption && jumpToVariable"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            @click="jumpToPolicySetResultVariable"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位变量
          </button>
        </div>

        <label v-if="modelValue.field === 'matched'" class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
          <input
            :checked="Boolean(modelValue.value_bool)"
            type="checkbox"
            class="h-4 w-4"
            style="accent-color: var(--app-accent)"
            @change="updatePolicySetResultBool(($event.target as HTMLInputElement).checked)"
          />
          <span class="text-sm text-[var(--app-text-soft)]">比较值为真</span>
        </label>

        <div v-else class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">比较对象</span>
          <EditorSelectField
            :model-value="modelValue.value_id || null"
            :options="resolvedPolicySetResultTargetOptions"
            placeholder="选择资源"
            :test-id="rootTestId('policy-set-result-target-id')"
            @update:model-value="updatePolicySetResultValueId(String($event || ''))"
          />
        </div>

        <div class="rounded-[14px] border border-[var(--app-border)] bg-white/40 px-4 py-3 text-sm leading-6 text-[var(--app-text-soft)]">
          该节点比较策略集处理结果对象里的明确字段。运行时结果会同时写出 `policySetId`、`policyGroupId`、`policyId`
          和动作序列签名，前端只展示名称，保存时仍然只存 id。
        </div>
      </template>

      <template v-else-if="modelValue.type === 'policyCondition'">
        <div class="space-y-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
          <div class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输入图像变量</span>
            <EditorSelectField
              :model-value="modelValue.input_var || null"
              :options="resolvedPolicyConditionInputOptions"
              :show-description="true"
              placeholder="留空则使用当前策略图像上下文"
              :test-id="rootTestId('policy-condition-input-var')"
              @update:model-value="updatePolicyConditionInput($event ? String($event) : null)"
            />
          </div>

          <div v-if="createVariable || (selectedPolicyConditionInputOption && jumpToVariable)" class="flex flex-wrap gap-2">
            <button v-if="createVariable" class="app-button app-button-ghost app-toolbar-button" type="button" @click="createPolicyConditionInputVariable">
              <AppIcon name="plus" :size="14" />
              新建图像变量
            </button>
            <button
              v-if="selectedPolicyConditionInputOption && jumpToVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="jumpToPolicyConditionInputVariable"
            >
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>

          <EditorPolicyConditionRuleBuilder
            :model-value="modelValue.rule"
            :test-id-prefix="rootTestId('policy-condition-rule')"
            @update:model-value="updatePolicyConditionRule"
          />
        </div>
      </template>

      <template v-else-if="modelValue.type === 'colorCompare'">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">OCR 目标文本</span>
          <input
            :value="modelValue.txt_target"
            class="app-input"
            @input="updateColorField('txt_target', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
          <input
            :checked="modelValue.is_font"
            type="checkbox"
            class="h-4 w-4"
            style="accent-color: var(--app-accent)"
            @change="updateColorField('is_font', ($event.target as HTMLInputElement).checked)"
          />
          <span class="text-sm text-[var(--app-text-soft)]">比较字体颜色</span>
        </label>

        <div class="grid gap-3 md:grid-cols-3">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">R</span>
            <input :value="String(modelValue.r)" class="app-input" type="number" min="0" max="255" @input="updateColorNumber('r', ($event.target as HTMLInputElement).value)" />
          </label>
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">G</span>
            <input :value="String(modelValue.g)" class="app-input" type="number" min="0" max="255" @input="updateColorNumber('g', ($event.target as HTMLInputElement).value)" />
          </label>
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">B</span>
            <input :value="String(modelValue.b)" class="app-input" type="number" min="0" max="255" @input="updateColorNumber('b', ($event.target as HTMLInputElement).value)" />
          </label>
        </div>
      </template>

      <div v-else class="rounded-[14px] border border-[var(--app-border)] bg-white/40 px-3 py-3 text-sm text-[var(--app-text-soft)]">
        当前条件类型暂未提供专用表单。
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import EditorPolicyConditionRuleBuilder from '@/views/script-editor/EditorPolicyConditionRuleBuilder.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import EditorVariableMetaCard from '@/views/script-editor/EditorVariableMetaCard.vue';
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { PolicyConditionRule } from '@/types/bindings/PolicyConditionRule';
import type { EditorReferenceKind, EditorReferenceOption } from '@/views/script-editor/editorReferences';
import { withResolvedReferenceOption } from '@/views/script-editor/editorReferences';
import { buildVariableCatalogKey, type EditorInputEntry, type EditorInputType, type EditorVariableOption } from '@/views/script-editor/editorVariables';
import {
  buildVarValue,
  compareOpOptions,
  conditionTypeOptions,
  createConditionNode,
  describeConditionNode,
  logicOpOptions,
  policySetResultCompareOptions,
  policySetResultFieldOptions,
  parseVarValueDraft,
  stateStatusTypeOptions,
  stateTargetTypeOptions,
  taskControlTypeOptions,
  varValueTypeOptions,
} from '@/views/script-editor/editorCondition';
import type { VarValueKind } from '@/views/script-editor/editorCondition';

defineOptions({ name: 'EditorConditionBuilder' });

const props = withDefaults(
  defineProps<{
    modelValue: ConditionNode;
    depth?: number;
    removable?: boolean;
    testIdPrefix?: string | null;
    variableOptions?: Array<{
      label: string;
      value: string;
      description?: string;
    }>;
    variableReferenceOptions?: EditorVariableOption[];
    variableInputEntries?: EditorInputEntry[];
    taskReferenceOptions?: EditorReferenceOption[];
    policyReferenceOptions?: EditorReferenceOption[];
    policyGroupReferenceOptions?: EditorReferenceOption[];
    policySetReferenceOptions?: EditorReferenceOption[];
    createReference?: (kind: EditorReferenceKind) => Promise<string>;
    jumpToReference?: (kind: EditorReferenceKind, id: string) => void;
    createVariable?: (namespace?: 'input' | 'runtime', inputType?: EditorInputType) => Promise<string>;
    jumpToVariable?: (option: EditorVariableOption) => void;
  }>(),
  {
    depth: 0,
    removable: false,
    testIdPrefix: null,
    variableOptions: () => [],
    variableReferenceOptions: () => [],
    variableInputEntries: () => [],
    taskReferenceOptions: () => [],
    policyReferenceOptions: () => [],
    policyGroupReferenceOptions: () => [],
    policySetReferenceOptions: () => [],
    createReference: undefined,
    jumpToReference: undefined,
    createVariable: undefined,
    jumpToVariable: undefined,
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: ConditionNode];
  remove: [];
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
}>();

const addableConditionTypes = computed(() => conditionTypeOptions.filter((option) => option.value !== 'group' || props.depth < 2));
const varCompareKindPreference = ref<VarValueKind | null>(null);
const filteredStateStatusTypeOptions = computed(() =>
  props.modelValue.type === 'taskStatus' && props.modelValue.a.target.type !== 'task'
    ? stateStatusTypeOptions.filter((option) => option.value !== 'enabled')
    : stateStatusTypeOptions,
);
const resolvedExecTargetOptions = computed(() =>
  props.modelValue.type === 'execNumCompare'
    ? withResolvedReferenceOption(
        props.modelValue.target.type === 'task' ? props.taskReferenceOptions : props.policyReferenceOptions,
        props.modelValue.target.id,
        props.modelValue.target.type === 'task' ? 'task' : 'policy',
      )
    : [],
);
const execCompareOpOptions = computed(() =>
  compareOpOptions.filter((option) => ['eq', 'ne', 'lt', 'le', 'gt', 'ge'].includes(option.value)),
);
const resolvedTaskStatusTargetOptions = computed(() =>
  props.modelValue.type === 'taskStatus'
    ? withResolvedReferenceOption(
        props.modelValue.a.target.type === 'task' ? props.taskReferenceOptions : props.policyReferenceOptions,
        props.modelValue.a.target.id,
        props.modelValue.a.target.type === 'task' ? 'task' : 'policy',
      )
    : [],
);
const resolvedPolicySetResultVarOptions = computed(() => {
  const resultVarOptions = props.variableReferenceOptions
    .filter((option) => option.valueType === 'json' || option.valueType === 'object' || option.valueType === 'list')
    .map((option) => ({
      label: option.label,
      value: option.key,
      description: option.description,
    }));

  if (props.modelValue.type !== 'policySetResult') {
    return resultVarOptions;
  }

  const node = props.modelValue;

  if (!node.result_var || resultVarOptions.some((option) => option.value === node.result_var)) {
    return resultVarOptions;
  }

  return [
    {
      label: `未解析变量 ${node.result_var}`,
      value: node.result_var,
      description: `保留历史变量 ${node.result_var}`,
    },
    ...resultVarOptions,
  ];
});
const currentVarValueDraft = computed(() =>
  props.modelValue.type === 'varCompare'
    ? parseVarValueDraft(props.modelValue.value, varCompareKindPreference.value ?? undefined)
    : parseVarValueDraft(''),
);
const selectedVarCompareOption = computed(() => {
  const node = props.modelValue;
  if (node.type !== 'varCompare') {
    return null;
  }

  return props.variableReferenceOptions.find((option) => option.key === node.var_name) ?? null;
});
const selectedVarCompareInputEntry = computed(() => {
  const option = selectedVarCompareOption.value;
  if (!option) {
    return null;
  }

  return props.variableInputEntries.find((entry) => buildVariableCatalogKey(entry.key, entry.namespace) === option.key) ?? null;
});
const selectedPolicySetResultVarOption = computed(() => {
  const node = props.modelValue;
  if (node.type !== 'policySetResult') {
    return null;
  }

  return props.variableReferenceOptions.find((option) => option.key === node.result_var) ?? null;
});
const selectedPolicyConditionInputOption = computed(() => {
  const node = props.modelValue;
  if (node.type !== 'policyCondition' || !node.input_var) {
    return null;
  }

  return props.variableReferenceOptions.find((option) => option.key === node.input_var) ?? null;
});
const imageVariableReferenceOptions = computed(() =>
  props.variableReferenceOptions
    .filter((option) => option.valueType === 'image')
    .map((option) => ({
      label: option.label,
      value: option.key,
      description: option.description,
    })),
);
const resolvedPolicyConditionInputOptions = computed(() => {
  if (props.modelValue.type !== 'policyCondition') {
    return imageVariableReferenceOptions.value;
  }

  const current = props.modelValue.input_var ?? '';
  if (!current || imageVariableReferenceOptions.value.some((option) => option.value === current)) {
    return imageVariableReferenceOptions.value;
  }

  return [
    {
      label: `未解析变量 ${current}`,
      value: current,
      description: `保留历史输入 ${current}`,
    },
    ...imageVariableReferenceOptions.value,
  ];
});
const resolvedPolicySetResultTargetOptions = computed(() => {
  if (props.modelValue.type !== 'policySetResult') {
    return [];
  }

  const selectedId = props.modelValue.value_id;
  const sourceOptions =
    props.modelValue.field === 'policySetId'
      ? props.policySetReferenceOptions
      : props.modelValue.field === 'policyGroupId'
        ? props.policyGroupReferenceOptions
        : props.policyReferenceOptions;

  return withResolvedReferenceOption(
    sourceOptions,
    selectedId,
    props.modelValue.field === 'policySetId'
      ? 'policySet'
      : props.modelValue.field === 'policyGroupId'
        ? 'policyGroup'
        : 'policy',
  );
});
const conditionSummary = computed(() => {
  if (props.modelValue.type !== 'policySetResult') {
    return describeConditionNode(props.modelValue);
  }

  const node = props.modelValue;
  return `策略集结果 · ${getPolicySetResultFieldLabel(node.field)}`;
});

watch(
  () => props.modelValue.type,
  (type) => {
    if (type !== 'varCompare') {
      varCompareKindPreference.value = null;
    }
  },
  { immediate: true },
);

const rootTestId = (suffix: string) => (props.testIdPrefix ? `${props.testIdPrefix}-${suffix}` : undefined);

const replaceNode = (value: ConditionNode) => {
  emit('update:modelValue', value);
};

const changeType = (type: string) => {
  replaceNode(createConditionNode(type));
};

const updateGroupOp = (op: string) => {
  if (props.modelValue.type !== 'group') return;
  replaceNode({
    ...props.modelValue,
    op: op as ConditionNode & { op: never }['op'],
  } as ConditionNode);
};

const addGroupItem = (type: string) => {
  if (props.modelValue.type !== 'group') return;
  replaceNode({
    ...props.modelValue,
    items: [...props.modelValue.items, createConditionNode(type)],
  });
};

const updateGroupItem = (index: number, value: ConditionNode) => {
  if (props.modelValue.type !== 'group') return;
  replaceNode({
    ...props.modelValue,
    items: props.modelValue.items.map((item, itemIndex) => (itemIndex === index ? value : item)),
  });
};

const removeGroupItem = (index: number) => {
  if (props.modelValue.type !== 'group') return;
  replaceNode({
    ...props.modelValue,
    items: props.modelValue.items.filter((_, itemIndex) => itemIndex !== index),
  });
};

const updateExecTargetType = (type: string) => {
  if (props.modelValue.type !== 'execNumCompare') return;
  replaceNode({
    ...props.modelValue,
    target: {
      ...props.modelValue.target,
      type,
    },
  } as ConditionNode);
};

const updateExecTargetId = (id: string) => {
  if (props.modelValue.type !== 'execNumCompare') return;
  replaceNode({
    ...props.modelValue,
    target: {
      ...props.modelValue.target,
      id,
    },
  } as ConditionNode);
};

const updateExecCompareOp = (op: string) => {
  if (props.modelValue.type !== 'execNumCompare') return;
  replaceNode({
    ...props.modelValue,
    op: op as ConditionNode & { op: never }['op'],
  } as ConditionNode);
};

const createExecTargetReference = async () => {
  if (props.modelValue.type !== 'execNumCompare' || !props.createReference) return;
  updateExecTargetId(await props.createReference(props.modelValue.target.type === 'task' ? 'task' : 'policy'));
};

const jumpToExecTargetReference = () => {
  if (props.modelValue.type !== 'execNumCompare' || !props.jumpToReference || !props.modelValue.target.id) return;
  props.jumpToReference(props.modelValue.target.type === 'task' ? 'task' : 'policy', props.modelValue.target.id);
};

const updateTaskStatusField = (field: 'type', value: string) => {
  if (props.modelValue.type !== 'taskStatus') return;
  replaceNode({
    ...props.modelValue,
    a: {
      ...props.modelValue.a,
      [field]: value,
    },
  } as ConditionNode);
};

const updateTaskTargetType = (type: string) => {
  if (props.modelValue.type !== 'taskStatus') return;
  const nextTargetType = type as 'task' | 'policy';
  replaceNode({
    ...props.modelValue,
    a: {
      ...props.modelValue.a,
      target: {
        ...props.modelValue.a.target,
        type: nextTargetType,
      },
      status:
        nextTargetType === 'policy' && props.modelValue.a.status.type === 'enabled'
          ? {
              ...props.modelValue.a.status,
              type: 'done',
            }
          : props.modelValue.a.status,
    },
  } as ConditionNode);
};

const updateTaskTargetId = (id: string) => {
  if (props.modelValue.type !== 'taskStatus') return;
  replaceNode({
    ...props.modelValue,
    a: {
      ...props.modelValue.a,
      target: {
        ...props.modelValue.a.target,
        id,
      },
    },
  } as ConditionNode);
};

const createTaskStatusTargetReference = async () => {
  if (props.modelValue.type !== 'taskStatus' || !props.createReference) return;
  updateTaskTargetId(await props.createReference(props.modelValue.a.target.type === 'task' ? 'task' : 'policy'));
};

const jumpToTaskStatusTargetReference = () => {
  if (props.modelValue.type !== 'taskStatus' || !props.jumpToReference || !props.modelValue.a.target.id) return;
  props.jumpToReference(
    props.modelValue.a.target.type === 'task' ? 'task' : 'policy',
    props.modelValue.a.target.id,
  );
};

const updateTaskStatusType = (type: string) => {
  if (props.modelValue.type !== 'taskStatus') return;
  replaceNode({
    ...props.modelValue,
    a: {
      ...props.modelValue.a,
      status: {
        ...props.modelValue.a.status,
        type,
      },
    },
  } as ConditionNode);
};

const updateTaskStatusValue = (value: boolean) => {
  if (props.modelValue.type !== 'taskStatus') return;
  replaceNode({
    ...props.modelValue,
    a: {
      ...props.modelValue.a,
      status: {
        ...props.modelValue.a.status,
        value,
      },
    },
  } as ConditionNode);
};

const updateVarCompareField = (field: 'var_name' | 'op', value: string) => {
  if (props.modelValue.type !== 'varCompare') return;
  replaceNode({
    ...props.modelValue,
    [field]: value,
  } as ConditionNode);
};

const createVarCompareVariable = async () => {
  if (props.modelValue.type !== 'varCompare' || !props.createVariable) return;
  const key = await props.createVariable('input', 'int');
  if (key) {
    updateVarCompareField('var_name', key);
  }
};

const jumpToVarCompareVariable = () => {
  if (!selectedVarCompareOption.value || !props.jumpToVariable) return;
  props.jumpToVariable(selectedVarCompareOption.value);
};

const updateVarCompareValueKind = (kind: string) => {
  if (props.modelValue.type !== 'varCompare') return;
  varCompareKindPreference.value = kind as VarValueKind;
  replaceNode({
    ...props.modelValue,
    value: buildVarValue({
      kind: kind as 'int' | 'float' | 'bool' | 'string',
      textValue: kind === 'string' ? '' : '0',
      boolValue: false,
    }),
  });
};

const updateVarCompareText = (value: string) => {
  if (props.modelValue.type !== 'varCompare') return;
  replaceNode({
    ...props.modelValue,
    value: buildVarValue({
      ...currentVarValueDraft.value,
      textValue: value,
    }),
  });
};

const updateVarCompareBool = (value: boolean) => {
  if (props.modelValue.type !== 'varCompare') return;
  varCompareKindPreference.value = 'bool';
  replaceNode({
    ...props.modelValue,
    value: buildVarValue({
      ...currentVarValueDraft.value,
      kind: 'bool',
      boolValue: value,
      textValue: value ? 'true' : 'false',
    }),
  });
};

const getPolicySetResultFieldLabel = (field: 'matched' | 'policySetId' | 'policyGroupId' | 'policyId') => {
  switch (field) {
    case 'matched':
      return 'matched';
    case 'policyGroupId':
      return 'policyGroupId';
    case 'policyId':
      return 'policyId';
    default:
      return 'policySetId';
  }
};

const updatePolicySetResultField = (field: 'result_var', value: string) => {
  if (props.modelValue.type !== 'policySetResult') return;
  replaceNode({
    ...props.modelValue,
    [field]: value,
  } as ConditionNode);
};

const updatePolicySetResultFieldType = (value: string) => {
  if (props.modelValue.type !== 'policySetResult') return;
  replaceNode({
    ...props.modelValue,
    field: value as 'matched' | 'policySetId' | 'policyGroupId' | 'policyId',
    value_id: value === 'matched' ? '' : props.modelValue.value_id,
  } as ConditionNode);
};

const updatePolicySetResultCompareOp = (value: string) => {
  if (props.modelValue.type !== 'policySetResult') return;
  replaceNode({
    ...props.modelValue,
    op: value as 'eq' | 'ne',
  } as ConditionNode);
};

const updatePolicySetResultBool = (value: boolean) => {
  if (props.modelValue.type !== 'policySetResult') return;
  replaceNode({
    ...props.modelValue,
    value_bool: value,
  });
};

const updatePolicySetResultValueId = (value: string) => {
  if (props.modelValue.type !== 'policySetResult') return;
  replaceNode({
    ...props.modelValue,
    value_id: value,
  });
};

const createPolicySetResultVariable = async () => {
  if (props.modelValue.type !== 'policySetResult' || !props.createVariable) return;
  const key = await props.createVariable('runtime', 'json');
  if (key) {
    updatePolicySetResultField('result_var', key);
  }
};

const jumpToPolicySetResultVariable = () => {
  if (!selectedPolicySetResultVarOption.value || !props.jumpToVariable) return;
  props.jumpToVariable(selectedPolicySetResultVarOption.value);
};

const updatePolicyConditionInput = (value: string | null) => {
  if (props.modelValue.type !== 'policyCondition') return;
  replaceNode({
    ...props.modelValue,
    input_var: value?.trim() ? value : null,
  });
};

const createPolicyConditionInputVariable = async () => {
  if (props.modelValue.type !== 'policyCondition' || !props.createVariable) return;
  const key = await props.createVariable('runtime', 'image');
  if (key) {
    updatePolicyConditionInput(key);
  }
};

const jumpToPolicyConditionInputVariable = () => {
  if (!selectedPolicyConditionInputOption.value || !props.jumpToVariable) return;
  props.jumpToVariable(selectedPolicyConditionInputOption.value);
};

const updatePolicyConditionRule = (rule: PolicyConditionRule) => {
  if (props.modelValue.type !== 'policyCondition') return;
  replaceNode({
    ...props.modelValue,
    rule,
  });
};

const updateColorField = (field: 'txt_target' | 'is_font', value: string | boolean) => {
  if (props.modelValue.type !== 'colorCompare') return;
  replaceNode({
    ...props.modelValue,
    [field]: value,
  } as ConditionNode);
};

const updateColorNumber = (field: 'r' | 'g' | 'b', value: string) => {
  if (props.modelValue.type !== 'colorCompare') return;
  replaceNode({
    ...props.modelValue,
    [field]: Math.max(0, Math.min(255, Number(value) || 0)),
  } as ConditionNode);
};
</script>

<style scoped>
.editor-condition-card {
  border-radius: 18px;
  border: 1px solid var(--app-border);
  background: color-mix(in srgb, var(--app-panel-muted) 88%, white);
  padding: 1rem;
}

.editor-condition-nested {
  background: rgba(255, 255, 255, 0.56);
}

.editor-inline-grid {
  display: grid;
  gap: 0.75rem;
}

@media (min-width: 768px) {
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
