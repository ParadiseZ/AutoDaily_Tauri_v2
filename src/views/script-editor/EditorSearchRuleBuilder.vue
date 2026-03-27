<template>
  <div class="editor-search-rule-card" :class="{ 'editor-search-rule-nested': depth > 0 }">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="flex min-w-0 flex-1 flex-wrap items-center gap-3">
        <AppSelect
          :model-value="modelValue.type"
          :options="searchRuleTypeOptions"
          placeholder="规则类型"
          class="min-w-[180px]"
          :test-id="rootTestId('type')"
          @update:model-value="changeType(String($event || SEARCH_RULE_TYPE.keyword))"
        />
        <span class="truncate text-xs text-[var(--app-text-faint)]">{{ describeSearchRule(modelValue) }}</span>
      </div>

      <button v-if="removable" class="app-button app-button-danger app-toolbar-button" type="button" @click="$emit('remove')">
        删除规则
      </button>
    </div>

    <div class="mt-4 space-y-3">
      <template v-if="modelValue.type === SEARCH_RULE_TYPE.keyword || modelValue.type === SEARCH_RULE_TYPE.regex">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">
            {{ modelValue.type === SEARCH_RULE_TYPE.keyword ? '关键字' : '正则表达式' }}
          </span>
          <input
            :value="modelValue.pattern"
            class="app-input"
            :data-testid="rootTestId(modelValue.type === SEARCH_RULE_TYPE.keyword ? 'keyword' : 'regex')"
            @input="replaceRule({ ...modelValue, pattern: ($event.target as HTMLInputElement).value })"
          />
        </label>
      </template>

      <template v-else-if="modelValue.type === SEARCH_RULE_TYPE.yoloIdx">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">检测索引</span>
          <input
            :value="String(modelValue.idx)"
            class="app-input"
            type="number"
            :data-testid="rootTestId('yolo-idx')"
            @input="replaceRule({ ...modelValue, idx: Number(($event.target as HTMLInputElement).value) || 0 })"
          />
        </label>
      </template>

      <template v-else-if="modelValue.type === SEARCH_RULE_TYPE.group">
        <div class="grid gap-3 md:grid-cols-2">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">组合逻辑</span>
            <AppSelect
              :model-value="modelValue.op"
              :options="logicOpOptions"
              placeholder="组合逻辑"
              :test-id="rootTestId('logic-op')"
              @update:model-value="updateGroupField('op', String($event || LOGIC_OP.And))"
            />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">作用域</span>
            <AppSelect
              :model-value="modelValue.scope"
              :options="searchScopeOptions"
              placeholder="作用域"
              :test-id="rootTestId('scope')"
              @update:model-value="updateGroupField('scope', String($event || SEARCH_SCOPE.Global))"
            />
          </label>
        </div>

        <div class="flex flex-wrap gap-2">
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

        <div class="space-y-3">
          <EditorSearchRuleBuilder
            v-for="(item, index) in modelValue.items"
            :key="`${item.type}-${index}`"
            :model-value="item"
            :depth="depth + 1"
            removable
            @update:model-value="updateGroupItem(index, $event)"
            @remove="removeGroupItem(index)"
          />
        </div>

        <EmptyState v-if="!modelValue.items.length" title="还没有子规则" description="先添加关键字、正则或 YOLO 索引规则。" />
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
import { createSearchRule, describeSearchRule, logicOpOptions, searchRuleTypeOptions, searchScopeOptions } from '@/views/script-editor/editorSearchRule';

defineOptions({ name: 'EditorSearchRuleBuilder' });

const props = withDefaults(
  defineProps<{
    modelValue: SearchRule;
    depth?: number;
    removable?: boolean;
    testIdPrefix?: string | null;
  }>(),
  {
    depth: 0,
    removable: false,
    testIdPrefix: null,
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: SearchRule];
  remove: [];
}>();

const addableRuleTypes = computed(() => searchRuleTypeOptions.filter((option) => option.value !== SEARCH_RULE_TYPE.group || props.depth < 2));
const rootTestId = (suffix: string) => (props.testIdPrefix ? `${props.testIdPrefix}-${suffix}` : undefined);

const replaceRule = (value: SearchRule) => {
  emit('update:modelValue', value);
};

const changeType = (type: string) => {
  replaceRule(createSearchRule(type));
};

const updateGroupField = (field: 'op' | 'scope', value: string) => {
  if (props.modelValue.type !== SEARCH_RULE_TYPE.group) return;
  replaceRule({
    ...props.modelValue,
    [field]: value,
  } as SearchRule);
};

const addGroupItem = (type: string) => {
  if (props.modelValue.type !== SEARCH_RULE_TYPE.group) return;
  replaceRule({
    ...props.modelValue,
    items: [...props.modelValue.items, createSearchRule(type)],
  });
};

const updateGroupItem = (index: number, value: SearchRule) => {
  if (props.modelValue.type !== SEARCH_RULE_TYPE.group) return;
  replaceRule({
    ...props.modelValue,
    items: props.modelValue.items.map((item, itemIndex) => (itemIndex === index ? value : item)),
  });
};

const removeGroupItem = (index: number) => {
  if (props.modelValue.type !== SEARCH_RULE_TYPE.group) return;
  replaceRule({
    ...props.modelValue,
    items: props.modelValue.items.filter((_, itemIndex) => itemIndex !== index),
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
</style>
