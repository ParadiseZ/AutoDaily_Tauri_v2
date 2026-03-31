<template>
  <div class="editor-condition-card" :class="{ 'editor-condition-nested': depth > 0 }">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="flex min-w-0 flex-1 flex-wrap items-center gap-3">
        <AppSelect
          :model-value="modelValue.type"
          :options="conditionTypeOptions"
          placeholder="条件类型"
          class="min-w-[180px]"
          :test-id="rootTestId('type')"
          @update:model-value="changeType(String($event || 'rawExpr'))"
        />
        <span class="truncate text-xs text-[var(--app-text-faint)]">{{ describeConditionNode(modelValue) }}</span>
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
          <AppSelect
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
            :variable-datalist-id="variableDatalistId"
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
        <div class="grid gap-3 md:grid-cols-[180px_minmax(0,1fr)]">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标类型</span>
            <AppSelect
              :model-value="modelValue.a.type"
              :options="stateTargetTypeOptions"
              placeholder="目标类型"
              @update:model-value="updateExecTargetType(String($event || 'task'))"
            />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标 ID</span>
            <input
              :value="modelValue.a.id"
              class="app-input"
              @input="updateExecTargetId(($event.target as HTMLInputElement).value)"
            />
          </label>
        </div>
      </template>

      <template v-else-if="modelValue.type === 'taskStatus'">
        <div class="grid gap-3 md:grid-cols-2">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">状态动作</span>
            <AppSelect
              :model-value="modelValue.a.type"
              :options="taskControlTypeOptions"
              placeholder="状态动作"
              @update:model-value="updateTaskStatusField('type', String($event || 'getState'))"
            />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标类型</span>
            <AppSelect
              :model-value="modelValue.a.target.type"
              :options="stateTargetTypeOptions"
              placeholder="目标类型"
              @update:model-value="updateTaskTargetType(String($event || 'task'))"
            />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标 ID</span>
            <input
              :value="modelValue.a.target.id"
              class="app-input"
              @input="updateTaskTargetId(($event.target as HTMLInputElement).value)"
            />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">状态类型</span>
            <AppSelect
              :model-value="modelValue.a.status.type"
              :options="stateStatusTypeOptions"
              placeholder="状态类型"
              @update:model-value="updateTaskStatusType(String($event || 'done'))"
            />
          </label>
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
        <div class="grid gap-3 md:grid-cols-2">
          <label class="space-y-2 md:col-span-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">变量名</span>
          <input
            :value="modelValue.var_name"
            :list="variableDatalistId || undefined"
            class="app-input"
            @input="updateVarCompareField('var_name', ($event.target as HTMLInputElement).value)"
          />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">比较方式</span>
            <AppSelect
              :model-value="modelValue.op"
              :options="compareOpOptions"
              placeholder="比较方式"
              @update:model-value="updateVarCompareField('op', String($event || 'eq'))"
            />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">值类型</span>
            <AppSelect
              :model-value="currentVarValueDraft.kind"
              :options="varValueTypeOptions"
              placeholder="值类型"
              @update:model-value="updateVarCompareValueKind(String($event || 'string'))"
            />
          </label>
        </div>

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
import AppSelect from '@/components/shared/AppSelect.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import {
  buildVarValue,
  compareOpOptions,
  conditionTypeOptions,
  createConditionNode,
  describeConditionNode,
  logicOpOptions,
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
    variableDatalistId?: string | null;
  }>(),
  {
    depth: 0,
    removable: false,
    testIdPrefix: null,
    variableDatalistId: null,
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: ConditionNode];
  remove: [];
}>();

const addableConditionTypes = computed(() => conditionTypeOptions.filter((option) => option.value !== 'group' || props.depth < 2));
const varCompareKindPreference = ref<VarValueKind | null>(null);
const currentVarValueDraft = computed(() =>
  props.modelValue.type === 'varCompare'
    ? parseVarValueDraft(props.modelValue.value, varCompareKindPreference.value ?? undefined)
    : parseVarValueDraft(''),
);

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
    a: {
      ...props.modelValue.a,
      type,
    },
  } as ConditionNode);
};

const updateExecTargetId = (id: string) => {
  if (props.modelValue.type !== 'execNumCompare') return;
  replaceNode({
    ...props.modelValue,
    a: {
      ...props.modelValue.a,
      id,
    },
  } as ConditionNode);
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
  replaceNode({
    ...props.modelValue,
    a: {
      ...props.modelValue.a,
      target: {
        ...props.modelValue.a.target,
        type,
      },
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
</style>
