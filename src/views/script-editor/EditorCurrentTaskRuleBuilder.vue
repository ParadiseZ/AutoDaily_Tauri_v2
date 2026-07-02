<template>
  <div
    class="app-rule-card"
    :class="{
      'app-rule-card-nested': depth > 0,
      'app-rule-card-root': isRootGroup,
      'app-rule-card-group': currentRule.type === 'group',
    }"
  >
    <div class="flex flex-wrap items-center justify-between gap-3 w-full">
      <div class="flex flex-wrap items-center gap-3 flex-1 min-w-0">
        <span class="app-rule-badge shrink-0">{{ headerLabel }}</span>

        <div v-if="!isRootGroup" class="flex flex-wrap items-center gap-3 flex-1 min-w-0">
          <EditorSelectField
            :model-value="currentRule.type"
            :options="currentTaskRuleTypeOptions"
            placeholder="规则类型"
            class="shrink-0"
            :test-id="rootTestId('type')"
            @update:model-value="changeType(String($event || 'task'))"
          />

          <div v-if="currentRule.type === 'task'" class="flex-1 min-w-[180px] max-w-[420px]">
            <AppSelect
              :model-value="currentRule.target || null"
              :options="resolvedTaskOptions"
              placeholder="搜索任务后选择"
              searchable
              search-placeholder="按名称搜索任务"
              :max-menu-height="320"
              :test-id="rootTestId('task-target')"
              @update:model-value="updateTaskTarget(String($event || ''))"
            />
          </div>
        </div>

        <span v-else-if="currentRule.type === 'group'" class="truncate text-xs text-(--app-text-faint)">
          {{ describeCurrentTaskRule(currentRule) }}
        </span>
      </div>

      <button
        v-if="removable && !isRootGroup"
        class="app-icon-button app-crash-icon app-icon-button-sec shrink-0"
        type="button"
        title="删除"
        aria-label="删除"
        @click="$emit('remove')"
      >
        <Trash2 class="h-4 w-4" />
      </button>
    </div>

    <div v-if="currentRule.type === 'group'" class="mt-4 space-y-3">
      <div class="flex flex-wrap items-start gap-x-6 gap-y-4">
        <label class="space-y-2 shrink-0">
          <span class="text-xs font-semibold uppercase tracking-[0.12em] text-(--app-text-faint)">逻辑类型</span>
          <EditorSelectField
            :model-value="currentRule.op"
            :options="logicOpOptions"
            placeholder="组合逻辑"
            :test-id="rootTestId('logic-op')"
            @update:model-value="updateGroupOp(String($event || 'Or'))"
          />
        </label>

        <div class="space-y-2 flex-1 min-w-[240px]">
          <span class="text-xs font-semibold uppercase tracking-[0.12em] text-(--app-text-faint) block">添加项</span>
          <div class="flex flex-wrap items-center gap-2">
            <button
              v-for="option in addableRuleTypes"
              :key="option.value"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="addGroupItem(option.value)"
            >
              {{ option.label }}
            </button>
          </div>
        </div>
      </div>

      <div v-if="currentRule.items.length" class="border-t border-(--app-border) my-4 pt-4 space-y-3">
        <EditorCurrentTaskRuleBuilder
          v-for="(item, index) in currentRule.items"
          :key="`${item.type}-${index}`"
          :model-value="item"
          :depth="depth + 1"
          removable
          :test-id-prefix="rootTestId(`item-${index}`)"
          :task-reference-options="taskReferenceOptions"
          :create-reference="createReference"
          :jump-to-reference="jumpToReference"
          @update:model-value="updateGroupItem(index, $event)"
          @remove="removeGroupItem(index)"
        />
      </div>

      <EmptyState v-else title="还没有子任务条件" description="先在当前逻辑组里添加任务或子组。" class="mt-4" />
    </div>

    <div v-else-if="currentRule.type === 'task'" class="mt-4 space-y-3">
      <div class="flex flex-wrap gap-2">
        <button
          v-if="createReference"
          class="app-button app-button-ghost app-toolbar-button"
          type="button"
          @click="createTaskReference"
        >
          <AppIcon name="plus" :size="14" />
          新建任务
        </button>
        <button
          v-if="jumpToReference"
          class="app-button app-button-ghost app-toolbar-button"
          type="button"
          :disabled="!currentRule.target"
          @click="jumpToTaskReference"
        >
          <AppIcon name="locate-fixed" :size="14" />
          定位编辑
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Trash2 } from 'lucide-vue-next';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import type { CurrentTaskRule } from '@/types/bindings/CurrentTaskRule';
import type { EditorReferenceKind, EditorReferenceOption } from '@/views/script-editor/editorReferences';
import { withResolvedReferenceOption } from '@/views/script-editor/editorReferences';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import {
  createCurrentTaskRule,
  currentTaskRuleTypeOptions,
  describeCurrentTaskRule,
} from '@/views/script-editor/editorCurrentTaskRule';
import { logicOpOptions } from '@/views/script-editor/editorCondition';

defineOptions({ name: 'EditorCurrentTaskRuleBuilder' });

const props = withDefaults(
  defineProps<{
    modelValue: CurrentTaskRule;
    depth?: number;
    removable?: boolean;
    testIdPrefix?: string | null;
    forceGroupRoot?: boolean;
    taskReferenceOptions?: EditorReferenceOption[];
    createReference?: (kind: EditorReferenceKind) => Promise<string>;
    jumpToReference?: (kind: EditorReferenceKind, id: string) => void;
  }>(),
  {
    depth: 0,
    removable: false,
    testIdPrefix: null,
    forceGroupRoot: false,
    taskReferenceOptions: () => [],
    createReference: undefined,
    jumpToReference: undefined,
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: CurrentTaskRule];
  remove: [];
}>();

const isRootGroup = computed(() => Boolean(props.forceGroupRoot && props.depth === 0 && props.modelValue.type === 'group'));
const currentRule = computed(() => props.modelValue);
const headerLabel = computed(() => {
  if (isRootGroup.value) return '根逻辑组';
  if (currentRule.value.type === 'group') return '逻辑组';
  return '当前任务';
});
const addableRuleTypes = computed(() => currentTaskRuleTypeOptions.filter((option) => option.value !== 'group' || props.depth < 2));
const resolvedTaskOptions = computed(() =>
  currentRule.value.type === 'task'
    ? withResolvedReferenceOption(props.taskReferenceOptions, currentRule.value.target, 'task')
    : props.taskReferenceOptions,
);
const rootTestId = (suffix: string) => (props.testIdPrefix ? `${props.testIdPrefix}-${suffix}` : undefined);

const replaceRule = (value: CurrentTaskRule) => {
  emit('update:modelValue', value);
};

const changeType = (type: string) => {
  if (!currentTaskRuleTypeOptions.some((option) => option.value === type)) {
    return;
  }
  replaceRule(createCurrentTaskRule(type as CurrentTaskRule['type']));
};

const updateTaskTarget = (target: string) => {
  if (currentRule.value.type !== 'task') {
    return;
  }
  replaceRule({
    ...currentRule.value,
    target,
  });
};

const createTaskReference = async () => {
  if (currentRule.value.type !== 'task' || !props.createReference) {
    return;
  }
  updateTaskTarget(await props.createReference('task'));
};

const jumpToTaskReference = () => {
  if (currentRule.value.type !== 'task' || !props.jumpToReference || !currentRule.value.target) {
    return;
  }
  props.jumpToReference('task', currentRule.value.target);
};

const updateGroupOp = (op: string) => {
  if (currentRule.value.type !== 'group') {
    return;
  }
  replaceRule({
    ...currentRule.value,
    op: op as Extract<CurrentTaskRule, { type: 'group' }>['op'],
  });
};

const addGroupItem = (type: string) => {
  if (currentRule.value.type !== 'group') {
    return;
  }
  if (!currentTaskRuleTypeOptions.some((option) => option.value === type)) {
    return;
  }
  replaceRule({
    ...currentRule.value,
    items: [...currentRule.value.items, createCurrentTaskRule(type as CurrentTaskRule['type'])],
  });
};

const updateGroupItem = (index: number, value: CurrentTaskRule) => {
  if (currentRule.value.type !== 'group') {
    return;
  }
  replaceRule({
    ...currentRule.value,
    items: currentRule.value.items.map((item, itemIndex) => (itemIndex === index ? value : item)),
  });
};

const removeGroupItem = (index: number) => {
  if (currentRule.value.type !== 'group') {
    return;
  }
  replaceRule({
    ...currentRule.value,
    items: currentRule.value.items.filter((_, itemIndex) => itemIndex !== index),
  });
};
</script>

<style scoped>
.app-rule-card {
  border-radius: 18px;
  border: 1px solid var(--app-border);
  background: color-mix(in srgb, var(--app-panel-muted) 88%, white);
  padding: 1rem;
}

.app-rule-card-nested {
  position: relative;
  margin-left: 0.4rem;
  background: rgba(255, 255, 255, 0.56);
}

.app-rule-card-root {
  border-style: solid;
  border-width: 1px;
  box-shadow: inset 0 0 0 1px rgba(70, 110, 255, 0.08);
}

.app-rule-card-group {
  border-left: 4px solid var(--app-accent);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.86), rgba(245, 248, 255, 0.7)),
    color-mix(in srgb, var(--app-panel-muted) 88%, white);
}

.app-rule-badge {
  border-radius: 999px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.72);
  padding: 0.22rem 0.65rem;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--app-text-soft);
}
</style>
