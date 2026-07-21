<template>
  <div
    class="app-rule-card"
    :class="{
      'app-rule-card-nested': depth > 0,
      'app-rule-card-root': depth === 0 && modelValue.type === 'group',
      'app-rule-card-group': modelValue.type === 'group',
    }"
    :data-testid="rootTestId('card')"
  >
    <div class="flex items-center justify-between gap-3">
      <span class="app-rule-badge shrink-0">{{ headerLabel }}</span>
      <button
        v-if="removable"
        class="app-icon-button app-crash-icon app-icon-button-sec shrink-0"
        type="button"
        title="删除"
        aria-label="删除"
        :data-testid="rootTestId('remove')"
        @click="$emit('remove')"
      >
        <Trash2 class="h-4 w-4" />
      </button>
    </div>

    <div class="mt-4 flex w-full max-w-[42rem] flex-col gap-4">

    <EditorOverviewField label="条件类型" width="compact">
      <EditorSelectField
        :model-value="modelValue.type"
        :options="resolvedConditionTypeOptions"
        placeholder="条件类型"
        :test-id="rootTestId('type')"
        @update:model-value="changeType(String($event || 'rawExpr'))"
      />
    </EditorOverviewField>

      <template v-if="modelValue.type === 'rawExpr'">
        <EditorOverviewField label="表达式" width="compact">
          <input
            :value="modelValue.expr"
            class="app-input"
            :data-testid="rootTestId('raw-expr')"
            @input="replaceNode({ ...modelValue, expr: ($event.target as HTMLInputElement).value })"
          />
        </EditorOverviewField>
      </template>

      <template v-else-if="modelValue.type === 'group'">
        <EditorOverviewField label="组合逻辑" width="radio">
          <EditorSelectField
            :model-value="modelValue.op"
            :options="logicOpOptions"
            placeholder="组合逻辑"
            :test-id="rootTestId('logic-op')"
            @update:model-value="updateGroupOp(String($event || 'And'))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="添加项">
          <div class="flex flex-wrap items-center gap-2">
            <button
              v-for="option in addableConditionTypes"
              :key="option.value"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="addGroupItem(option.value)"
            >
              {{ option.label }}
            </button>
          </div>
        </EditorOverviewField>

        <div v-if="modelValue.items.length" class="border-t border-(--app-border) my-4 pt-4 space-y-3">
          <EditorConditionBuilder
            v-for="(item, index) in modelValue.items"
            :key="`${item.type}-${index}`"
            :model-value="item"
            :depth="depth + 1"
            removable
            :test-id-prefix="rootTestId(`item-${index}`)"
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

        <div v-else class="rounded-[14px] border border-dashed border-(--app-border) px-4 py-3 text-sm text-(--app-text-faint) mt-4">
          还没有条件
        </div>
      </template>

      <template v-else-if="modelValue.type === 'execNumCompare'">
        <EditorOverviewField label="目标类型" width="compact">
          <EditorSelectField
            :model-value="modelValue.target.type"
            :options="stateTargetTypeOptions"
            placeholder="目标类型"
            @update:model-value="updateExecTargetType(String($event || 'task'))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="目标资源" width="compact">
          <EditorSelectField
            :model-value="modelValue.target.id || null"
            :options="resolvedExecTargetOptions"
            placeholder="选择任务或策略"
            @update:model-value="updateExecTargetId(String($event || ''))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="比较方式" width="compact">
          <EditorSelectField
            :model-value="modelValue.op"
            :options="execCompareOpOptions"
            placeholder="比较方式"
            @update:model-value="updateExecCompareOp(String($event || 'ge'))"
          />
        </EditorOverviewField>

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
        <!-- <EditorOverviewField label="状态动作" width="compact">
          <EditorSelectField
            :model-value="modelValue.a.type"
            :options="taskControlTypeOptions"
            placeholder="状态动作"
            @update:model-value="updateTaskStatusField('type', String($event || 'setState'))"
          />
        </EditorOverviewField> -->

        <EditorOverviewField label="目标类型" width="compact">
          <EditorSelectField
            :model-value="modelValue.a.target.type"
            :options="stateTargetTypeOptions"
            placeholder="目标类型"
            @update:model-value="updateTaskTargetType(String($event || 'task'))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="目标资源" width="compact">
          <EditorSelectField
            :model-value="modelValue.a.target.id || null"
            :options="resolvedTaskStatusTargetOptions"
            placeholder="选择任务或策略"
            @update:model-value="updateTaskTargetId(String($event || ''))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="状态类型">
          <EditorSelectField
            :model-value="modelValue.a.status.type"
            :options="filteredStateStatusTypeOptions"
            placeholder="状态类型"
            @update:model-value="updateTaskStatusType(String($event || 'done'))"
          />
        </EditorOverviewField>

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

        <EditorOverviewField label="状态值">
          <label class="flex items-center gap-3 rounded-[16px] border border-(--app-border) px-4 py-3">
            <input
              :checked="Boolean(modelValue.a.status.value)"
              type="checkbox"
              class="h-4 w-4"
              style="accent-color: var(--app-accent)"
              @change="updateTaskStatusValue(($event.target as HTMLInputElement).checked)"
            />
            <span class="text-sm text-(--app-text-soft)">状态值为真</span>
          </label>
        </EditorOverviewField>
      </template>

      <template v-else-if="modelValue.type === 'currentTaskIn'">
        <EditorOverviewField label="目标任务" width="compact">
          <EditorSelectField
            :model-value="modelValue.target || null"
            :options="resolvedCurrentTaskTargetOptions"
            placeholder="选择任务"
            :test-id="rootTestId('current-task-target')"
            @update:model-value="updateCurrentTaskTarget(String($event || ''))"
          />
        </EditorOverviewField>

        <div class="flex flex-wrap gap-2">
          <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="createCurrentTaskTargetReference">
            <AppIcon name="plus" :size="14" />
            新建任务
          </button>
          <button
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            :disabled="!modelValue.target"
            @click="jumpToCurrentTaskTargetReference"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位编辑
          </button>
        </div>

        <EditorOverviewField label="判断结果">
          <label class="flex items-center gap-3 rounded-[16px] border border-(--app-border) bg-white/55 px-4 py-3">
            <input
              :checked="modelValue.expected"
              type="checkbox"
              class="h-4 w-4"
              :data-testid="rootTestId('current-task-expected')"
              style="accent-color: var(--app-accent)"
              @change="updateCurrentTaskExpected(($event.target as HTMLInputElement).checked)"
            />
            <div class="space-y-1">
              <p class="text-sm font-medium text-(--app-text-strong)">是目标任务</p>
            </div>
          </label>
        </EditorOverviewField>
      </template>

      <template v-else-if="modelValue.type === 'varCompare'">
        <EditorOverviewField label="变量" width="compact">
          <EditorSelectField
            :model-value="modelValue.var_name || null"
            :options="variableOptions"
            placeholder="请选择"
            :test-id="rootTestId('var-name')"
            @update:model-value="updateVarCompareField('var_name', String($event || ''))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="比较" width="compact">
          <EditorSelectField
            :model-value="modelValue.op"
            :options="compareOpOptions"
            placeholder="比较方式"
            @update:model-value="updateVarCompareField('op', String($event || 'eq'))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="值类型" width="compact">
          <EditorSelectField
            :model-value="currentVarValueDraft.kind"
            :options="varValueTypeOptions"
            placeholder="值类型"
            @update:model-value="updateVarCompareValueKind(String($event || 'string'))"
          />
        </EditorOverviewField>

        <div v-if="createVariable || (selectedVarCompareOption && jumpToVariable)" class="flex flex-wrap gap-2">
          <button v-if="createVariable" class="app-button app-button-ghost app-toolbar-button" type="button" @click="createVarCompareVariable">
            <AppIcon name="plus" :size="14" />
            新建
          </button>
          <button
            v-if="selectedVarCompareOption && jumpToVariable"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            @click="jumpToVarCompareVariable"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位
          </button>
        </div>

        <!-- 去掉步骤内变量编辑功能，以保持页面简洁性 -->
        <!-- <EditorVariableMetaCard
          v-if="selectedVarCompareOption"
          :variable="selectedVarCompareOption"
          :input-entry="selectedVarCompareInputEntry"
          editable
          compact
          hide-header
          @update-input="(entryId, field, value) => emit('update-input', entryId, field, value)"
        /> -->

        <EditorOverviewField v-if="currentVarValueDraft.kind === 'bool'" label="比较值" width="compact">
          <label class="flex items-center gap-3 rounded-[16px] border border-(--app-border) px-4 py-3">
            <input
              :checked="currentVarValueDraft.boolValue"
              type="checkbox"
              class="h-4 w-4"
              style="accent-color: var(--app-accent)"
              @change="updateVarCompareBool(($event.target as HTMLInputElement).checked)"
            />
            <span class="text-sm text-(--app-text-soft)">比较值为真</span>
          </label>
        </EditorOverviewField>

        <EditorOverviewField v-else label="比较值" width="compact">
          <input
            ref="varCompareValueInputEl"
            :value="currentVarValueDraft.textValue"
            class="app-input"
            :class="{ 'app-input-invalid': !currentVarValueDraft.textValue || currentVarValueDraft.textValue.trim().length == 0 }"
            :type="currentVarValueDraft.kind === 'string' ? 'text' : 'number'"
            @input="updateVarCompareText(($event.target as HTMLInputElement).value)"
          />
        </EditorOverviewField>
      </template>

      <template v-else-if="modelValue.type === 'visionCountCompare'">
        <EditorOverviewField label="结果变量">
          <EditorSelectField width="compact"
            :model-value="modelValue.input_var || null"
            :options="resolvedVisionCountCompareInputOptions"
            placeholder="选择检测结果或 OCR 结果变量"
            :test-id="rootTestId('vision-count-compare-input-var')"
            @update:model-value="updateVisionCountCompareField('input_var', String($event || ''))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="比较" width="compact">
          <EditorSelectField
            :model-value="modelValue.op"
            :options="countCompareConditionOpOptions"
            placeholder="比较方式"
            :test-id="rootTestId('vision-count-compare-op')"
            @update:model-value="updateVisionCountCompareField('op', String($event || 'ge'))"
          />
        </EditorOverviewField>

        <div v-if="createVariable || (selectedVisionCountCompareInputOption && jumpToVariable)" class="flex flex-wrap gap-2">
          <button v-if="createVariable" class="app-button app-button-ghost app-toolbar-button" type="button" @click="createVisionCountCompareVariable">
            <AppIcon name="plus" :size="14" />
            新建结果变量
          </button>
          <button
            v-if="selectedVisionCountCompareInputOption && jumpToVariable"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            @click="jumpToVisionCountCompareVariable"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位变量
          </button>
        </div>

        <EditorOverviewField label="目标标签 / 文字" width="compact">
          <input
            :value="modelValue.target_value ?? ''"
            class="app-input"
            placeholder="留空则统计全部结果"
            :data-testid="rootTestId('vision-count-compare-target-value')"
            @input="updateVisionCountCompareNullableField('target_value', ($event.target as HTMLInputElement).value)"
          />
        </EditorOverviewField>

        <EditorOverviewField label="指定数量" width="compact">
          <input
            :value="String(modelValue.expected_count ?? 0)"
            class="app-input"
            type="number"
            :data-testid="rootTestId('vision-count-compare-expected-count')"
            @input="updateVisionCountCompareNumberField('expected_count', ($event.target as HTMLInputElement).value)"
          />
        </EditorOverviewField>
      </template>

      <template v-else-if="modelValue.type === 'policySetResult'">
        <EditorOverviewField label="结果变量">
          <EditorSelectField width="compact"
            :model-value="modelValue.result_var || null"
            :options="resolvedPolicySetResultVarOptions"
            :show-description="true"
            placeholder="选择策略集结果变量"
            :test-id="rootTestId('policy-set-result-var')"
            @update:model-value="updatePolicySetResultField('result_var', String($event || ''))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="判断字段" width="compact">
          <EditorSelectField
            :model-value="modelValue.field"
            :options="policySetResultFieldOptions"
            placeholder="结果字段"
            :test-id="rootTestId('policy-set-result-field')"
            @update:model-value="updatePolicySetResultFieldType(String($event || 'policyId'))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="比较方式" width="compact">
          <EditorSelectField
            :model-value="modelValue.op"
            :options="policySetResultCompareOptions"
            placeholder="比较方式"
            :test-id="rootTestId('policy-set-result-op')"
            @update:model-value="updatePolicySetResultCompareOp(String($event || 'eq'))"
          />
        </EditorOverviewField>

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

        <EditorOverviewField v-if="modelValue.field === 'matched'" label="比较值">
          <label class="flex items-center gap-3 rounded-[16px] border border-(--app-border) px-4 py-3">
            <input
              :checked="Boolean(modelValue.value_bool)"
              type="checkbox"
              class="h-4 w-4"
              style="accent-color: var(--app-accent)"
              @change="updatePolicySetResultBool(($event.target as HTMLInputElement).checked)"
            />
            <span class="text-sm text-(--app-text-soft)">比较值为真</span>
          </label>
        </EditorOverviewField>

        <EditorOverviewField v-else label="比较对象">
          <EditorSelectField
            width="compact"
            :model-value="modelValue.value_id || null"
            :options="resolvedPolicySetResultTargetOptions"
            placeholder="选择资源"
            :test-id="rootTestId('policy-set-result-target-id')"
            @update:model-value="updatePolicySetResultValueId(String($event || ''))"
          />
        </EditorOverviewField>

        <div class="rounded-[14px] border border-(--app-border) bg-white/40 px-4 py-3 text-sm leading-6 text-(--app-text-soft)">
          该节点比较策略集处理结果对象里的明确字段。运行时结果会同时写出 `policySetId`、`policyGroupId`、`policyId`
          和动作序列签名，前端只展示名称，保存时仍然只存 id。
        </div>
      </template>

      <div v-else class="rounded-[14px] border border-(--app-border) bg-white/40 px-3 py-3 text-sm text-(--app-text-soft)">
        当前条件类型暂未提供专用表单。
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { Trash2 } from '@lucide/vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import EditorOverviewField from '@/views/script-editor/EditorOverviewField.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { EditorReferenceKind, EditorReferenceOption } from '@/views/script-editor/editorReferences';
import { withResolvedReferenceOption } from '@/views/script-editor/editorReferences';
import { buildVariableCatalogKey, type EditorInputEntry, type EditorInputType, type EditorVariableOption } from '@/views/script-editor/editorVariables';
import {
  buildVarValue,
  compareOpOptions,
  conditionTypeOptions,
  createConditionNode,
  logicOpOptions,
  policySetResultCompareOptions,
  policySetResultFieldOptions,
  parseVarValueDraft,
  stateStatusTypeOptions,
  stateTargetTypeOptions,
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
    createVariable?: (namespace?: 'input' | 'runtime', inputType?: EditorInputType, options?: { preferredKey?: string; name?: string; select?: boolean; silent?: boolean; focusEditor?: boolean }) => Promise<string>;
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

const resolvedConditionTypeOptions = computed(() => [...conditionTypeOptions]);
const headerLabel = computed(() => {
  if (props.modelValue.type === 'group') {
    return props.depth === 0 ? '根逻辑组' : '逻辑组';
  }
  return '条件';
});
const addableConditionTypes = computed(() => conditionTypeOptions.filter((option) => option.value !== 'group' || props.depth < 2));
const varCompareKindPreference = ref<VarValueKind | null>(null);
const varCompareValueInputEl = ref<HTMLInputElement | null>(null);
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
const countCompareConditionOpOptions = computed(() =>
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
const resolvedCurrentTaskTargetOptions = computed(() =>
  props.modelValue.type === 'currentTaskIn'
    ? withResolvedReferenceOption(props.taskReferenceOptions, props.modelValue.target, 'task')
    : props.taskReferenceOptions,
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
const resolvedVisionCountCompareInputOptions = computed(() => {
  const node = props.modelValue;
  const inputOptions = props.variableReferenceOptions
    .filter((option) => option.valueType === 'json' || option.valueType === 'object' || option.valueType === 'list')
    .map((option) => ({
      label: option.label,
      value: option.key,
      description: option.description,
    }));

  if (node.type !== 'visionCountCompare') {
    return inputOptions;
  }

  if (!node.input_var || inputOptions.some((option) => option.value === node.input_var)) {
    return inputOptions;
  }

  return [
    {
      label: `未解析变量 ${node.input_var}`,
      value: node.input_var,
      description: `保留历史变量 ${node.input_var}`,
    },
    ...inputOptions,
  ];
});
const selectedVisionCountCompareInputOption = computed(() => {
  const node = props.modelValue;
  if (node.type !== 'visionCountCompare') {
    return null;
  }

  return props.variableReferenceOptions.find((option) => option.key === node.input_var) ?? null;
});
const selectedPolicySetResultVarOption = computed(() => {
  const node = props.modelValue;
  if (node.type !== 'policySetResult') {
    return null;
  }

  return props.variableReferenceOptions.find((option) => option.key === node.result_var) ?? null;
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

const changeType = async (type: string) => {
  if (!conditionTypeOptions.some((option) => option.value === type)) {
    return;
  }
  replaceNode(createConditionNode(type as ConditionNode['type']));
  if (type === 'varCompare') {
    await nextTick();
    varCompareValueInputEl.value?.focus();
  }
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
  if (!conditionTypeOptions.some((option) => option.value === type)) {
    return;
  }
  replaceNode({
    ...props.modelValue,
    items: [...props.modelValue.items, createConditionNode(type as ConditionNode['type'])],
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

const updateCurrentTaskTarget = (target: string) => {
  if (props.modelValue.type !== 'currentTaskIn') return;
  replaceNode({
    ...props.modelValue,
    target: target.trim() ? target : null,
  });
};

const updateCurrentTaskExpected = (expected: boolean) => {
  if (props.modelValue.type !== 'currentTaskIn') return;
  replaceNode({
    ...props.modelValue,
    expected,
  });
};

const createCurrentTaskTargetReference = async () => {
  if (props.modelValue.type !== 'currentTaskIn' || !props.createReference) return;
  updateCurrentTaskTarget(await props.createReference('task'));
};

const jumpToCurrentTaskTargetReference = () => {
  if (props.modelValue.type !== 'currentTaskIn' || !props.jumpToReference || !props.modelValue.target) return;
  props.jumpToReference('task', props.modelValue.target);
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
  const key = await props.createVariable('input', 'int', { focusEditor: true });
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

const updateVisionCountCompareField = (field: 'input_var' | 'op', value: string) => {
  if (props.modelValue.type !== 'visionCountCompare') return;
  replaceNode({
    ...props.modelValue,
    [field]: value,
  } as ConditionNode);
};

const updateVisionCountCompareNullableField = (field: 'target_value', value: string) => {
  if (props.modelValue.type !== 'visionCountCompare') return;
  replaceNode({
    ...props.modelValue,
    [field]: value.trim() ? value : null,
  } as ConditionNode);
};

const updateVisionCountCompareNumberField = (field: 'expected_count', value: string) => {
  if (props.modelValue.type !== 'visionCountCompare') return;
  const parsed = Number.parseInt(value, 10);
  replaceNode({
    ...props.modelValue,
    [field]: Number.isFinite(parsed) ? parsed : 0,
  } as ConditionNode);
};

const createVisionCountCompareVariable = async () => {
  if (props.modelValue.type !== 'visionCountCompare' || !props.createVariable) return;
  const key = await props.createVariable('runtime', 'json', { focusEditor: true });
  if (key) {
    updateVisionCountCompareField('input_var', key);
  }
};

const jumpToVisionCountCompareVariable = () => {
  if (!selectedVisionCountCompareInputOption.value || !props.jumpToVariable) return;
  props.jumpToVariable(selectedVisionCountCompareInputOption.value);
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
  const key = await props.createVariable('runtime', 'json', { focusEditor: true });
  if (key) {
    updatePolicySetResultField('result_var', key);
  }
};

const jumpToPolicySetResultVariable = () => {
  if (!selectedPolicySetResultVarOption.value || !props.jumpToVariable) return;
  props.jumpToVariable(selectedPolicySetResultVarOption.value);
};

</script>
