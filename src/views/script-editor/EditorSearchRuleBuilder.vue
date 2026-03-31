<template>
  <div
    class="editor-search-rule-card"
    :class="{
      'editor-search-rule-nested': depth > 0,
      'editor-search-rule-root': isRootGroup,
      'editor-search-rule-group': currentRule.type === SEARCH_RULE_TYPE.group,
    }"
  >
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="min-w-0 flex-1 space-y-1">
        <div class="flex flex-wrap items-center gap-2">
          <span class="editor-search-rule-badge">{{ headerLabel }}</span>
          <span class="truncate text-xs text-[var(--app-text-faint)]">{{ describeSearchRule(currentRule) }}</span>
        </div>

        <div v-if="!isRootGroup" class="flex min-w-0 flex-1 flex-wrap items-center gap-3">
          <AppSelect
            :model-value="currentRule.type"
            :options="searchRuleTypeOptions"
            placeholder="规则类型"
            class="min-w-[180px]"
            :test-id="rootTestId('type')"
            @update:model-value="changeType(String($event || SEARCH_RULE_TYPE.keyword))"
          />
        </div>
      </div>

      <button v-if="removable && !isRootGroup" class="app-button app-button-danger app-toolbar-button" type="button" @click="$emit('remove')">
        删除规则
      </button>
    </div>

    <div class="mt-4 space-y-3">
      <template v-if="currentRule.type === SEARCH_RULE_TYPE.keyword || currentRule.type === SEARCH_RULE_TYPE.regex">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">
            {{ currentRule.type === SEARCH_RULE_TYPE.keyword ? '关键字' : '正则表达式' }}
          </span>
          <input
            :value="currentRule.pattern"
            class="app-input"
            :data-testid="rootTestId(currentRule.type === SEARCH_RULE_TYPE.keyword ? 'keyword' : 'regex')"
            @input="replaceRule({ ...currentRule, pattern: ($event.target as HTMLInputElement).value })"
          />
        </label>
      </template>

      <template v-else-if="currentRule.type === SEARCH_RULE_TYPE.yoloIdx">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">检测索引</span>
          <input
            :value="String(currentRule.idx)"
            class="app-input"
            type="number"
            :data-testid="rootTestId('yolo-idx')"
            @input="replaceRule({ ...currentRule, idx: Number(($event.target as HTMLInputElement).value) || 0 })"
          />
        </label>
      </template>

      <template v-else-if="currentRule.type === SEARCH_RULE_TYPE.group">
        <div class="rounded-[16px] border border-[var(--app-border)] bg-white/45 px-4 py-4">
          <div class="grid gap-3 md:grid-cols-2">
            <label class="space-y-2">
              <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">组合逻辑</span>
              <AppSelect
                :model-value="currentRule.op"
                :options="logicOpOptions"
                placeholder="组合逻辑"
                :test-id="rootTestId('logic-op')"
                @update:model-value="updateGroupField('op', String($event || LOGIC_OP.And))"
              />
            </label>

            <label class="space-y-2">
              <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">作用域</span>
              <AppSelect
                :model-value="currentRule.scope"
                :options="searchScopeOptions"
                placeholder="作用域"
                :test-id="rootTestId('scope')"
                @update:model-value="updateGroupField('scope', String($event || SEARCH_SCOPE.Global))"
              />
            </label>
          </div>

          <div class="mt-4 flex flex-wrap gap-2">
            <button
              v-for="option in addableRuleTypes"
              :key="option.value"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="addGroupItem(String(option.value))"
            >
              添加{{ option.label }}
            </button>
          </div>
        </div>

        <div class="space-y-3">
          <EditorSearchRuleBuilder
            v-for="(item, index) in currentRule.items"
            :key="`${item.type}-${index}`"
            :model-value="item"
            :depth="depth + 1"
            :test-id-prefix="rootTestId(`item-${index}`)"
            removable
            @update:model-value="updateGroupItem(index, $event)"
            @remove="removeGroupItem(index)"
          />
        </div>

        <EmptyState v-if="!currentRule.items.length" title="还没有子规则" description="先在当前逻辑组里添加关键字、正则、YOLO 或子组。" />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import type { SearchRule } from '@/types/bindings/SearchRule';
import { LOGIC_OP, SEARCH_RULE_TYPE, SEARCH_SCOPE } from '@/views/script-editor/editorStepKinds';
import {
  createSearchRule,
  describeSearchRule,
  ensureRootGroupRule,
  logicOpOptions,
  searchRuleTypeOptions,
  searchScopeOptions,
} from '@/views/script-editor/editorSearchRule';

defineOptions({ name: 'EditorSearchRuleBuilder' });

const props = withDefaults(
  defineProps<{
    modelValue: SearchRule;
    depth?: number;
    removable?: boolean;
    testIdPrefix?: string | null;
    forceGroupRoot?: boolean;
  }>(),
  {
    depth: 0,
    removable: false,
    testIdPrefix: null,
    forceGroupRoot: false,
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: SearchRule];
  remove: [];
}>();

const isRootGroup = computed(() => Boolean(props.forceGroupRoot && props.depth === 0));
const currentRule = computed(() => (isRootGroup.value ? ensureRootGroupRule(props.modelValue) : props.modelValue));
const headerLabel = computed(() => {
  if (isRootGroup.value) return '根逻辑组';
  if (currentRule.value.type === SEARCH_RULE_TYPE.group) return '逻辑组';
  return '规则';
});
const addableRuleTypes = computed(() => searchRuleTypeOptions.filter((option) => option.value !== SEARCH_RULE_TYPE.group || props.depth < 2));
const rootTestId = (suffix: string) => (props.testIdPrefix ? `${props.testIdPrefix}-${suffix}` : undefined);

const replaceRule = (value: SearchRule) => {
  emit('update:modelValue', value);
};

const changeType = (type: string) => {
  replaceRule(createSearchRule(type));
};

const updateGroupField = (field: 'op' | 'scope', value: string) => {
  if (currentRule.value.type !== SEARCH_RULE_TYPE.group) return;
  replaceRule({
    ...currentRule.value,
    [field]: value,
  } as SearchRule);
};

const addGroupItem = (type: string) => {
  if (currentRule.value.type !== SEARCH_RULE_TYPE.group) return;
  replaceRule({
    ...currentRule.value,
    items: [...currentRule.value.items, createSearchRule(type)],
  });
};

const updateGroupItem = (index: number, value: SearchRule) => {
  if (currentRule.value.type !== SEARCH_RULE_TYPE.group) return;
  replaceRule({
    ...currentRule.value,
    items: currentRule.value.items.map((item, itemIndex) => (itemIndex === index ? value : item)),
  });
};

const removeGroupItem = (index: number) => {
  if (currentRule.value.type !== SEARCH_RULE_TYPE.group) return;
  replaceRule({
    ...currentRule.value,
    items: currentRule.value.items.filter((_, itemIndex) => itemIndex !== index),
  });
};
</script>

<style scoped>
.editor-search-rule-card {
  border-radius: 18px;
  border: 1px solid var(--app-border);
  background: color-mix(in srgb, var(--app-panel-muted) 88%, white);
  padding: 1rem;
}

.editor-search-rule-nested {
  background: rgba(255, 255, 255, 0.56);
}

.editor-search-rule-root {
  border-style: solid;
  border-width: 1px;
  box-shadow: inset 0 0 0 1px rgba(70, 110, 255, 0.08);
}

.editor-search-rule-group {
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.86), rgba(245, 248, 255, 0.7)),
    color-mix(in srgb, var(--app-panel-muted) 88%, white);
}

.editor-search-rule-badge {
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
