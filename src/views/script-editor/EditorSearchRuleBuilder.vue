<template>
  <div
    ref="rootEl"
    class="app-rule-card"
    :class="{
      'app-rule-card-nested': depth > 0,
      'app-rule-card-root': isRootGroup,
      'app-rule-card-group': currentRule.type === SEARCH_RULE_TYPE.group,
    }"
  >
    <div class="flex flex-wrap items-center justify-between gap-3 w-full">
      <div class="flex flex-wrap items-center gap-3 flex-1 min-w-0">
        <span class="app-rule-badge shrink-0">{{ headerLabel }}</span>

        <div v-if="!isRootGroup" class="flex flex-wrap items-center gap-3 flex-1 min-w-0">
          <EditorSelectField
            :model-value="currentRule.type"
            :options="searchRuleTypeOptions"
            placeholder="规则类型"
            class="shrink-0"
            :test-id="rootTestId('type')"
            @update:model-value="changeType(String($event || SEARCH_RULE_TYPE.txt))"
          />

          <!-- 行内输入框，使布局更紧凑 -->
          <div v-if="currentRule.type !== SEARCH_RULE_TYPE.group" class="flex-1 min-w-[150px] max-w-[400px]">
            <input
              v-if="currentRule.type === SEARCH_RULE_TYPE.txt"
              ref="textInputEl"
              :value="currentRule.pattern"
              class="app-input"
              :class="{ 'app-input-invalid': isPatternMissing }"
              placeholder="请输入文本"
              :aria-invalid="isPatternMissing"
              data-search-rule-text-input="true"
              :data-testid="rootTestId('txt')"
              @input="replaceRule({ ...currentRule, pattern: ($event.target as HTMLInputElement).value })"
            />
            <div v-else-if="currentRule.type === SEARCH_RULE_TYPE.detLabel" class="space-y-2">
              <AppSelect
                :model-value="currentRule.idx ?? null"
                :options="resolvedLabelIndexOptions"
                :placeholder="labelSelectPlaceholder"
                :disabled="!(resolvedLabelIndexOptions.length)"
                searchable
                search-placeholder="搜索标签"
                :max-menu-height="320"
                :test-id="rootTestId('det-label-idx')"
                @update:model-value="replaceRule({ ...currentRule, idx: Number($event ?? 0) || 0 })"
              />
              <p v-if="labelSelectHint" class="text-xs leading-5 text-amber-700">{{ labelSelectHint }}</p>
            </div>
          </div>
        </div>

        <span v-else-if="currentRule.type === SEARCH_RULE_TYPE.group" class="truncate text-xs text-(--app-text-faint)">
          {{ describeSearchRule(currentRule) }}
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

    <!-- 逻辑组自身的配置与子规则列表 -->
    <div v-if="currentRule.type === SEARCH_RULE_TYPE.group" class="mt-4 space-y-3">
      <!-- 逻辑组自身的配置区域，直接平铺于卡片中 -->
      <div class="flex flex-wrap items-start gap-x-6 gap-y-4">
        <label class="space-y-2 shrink-0">
          <span class="text-xs font-semibold uppercase tracking-[0.12em] text-(--app-text-faint)">逻辑类型</span>
          <EditorSelectField
            :model-value="currentRule.op"
            :options="logicOpOptions"
            placeholder="组合逻辑"
            :test-id="rootTestId('logic-op')"
            @update:model-value="updateGroupField('op', String($event || LOGIC_OP.And))"
          />
        </label>

        <label class="space-y-2 shrink-0">
          <span class="text-xs font-semibold uppercase tracking-[0.12em] text-(--app-text-faint)">作用域</span>
          <EditorSelectField
            :model-value="currentRule.scope"
            :options="searchScopeOptions"
            placeholder="作用域"
            :test-id="rootTestId('scope')"
            @update:model-value="updateGroupField('scope', String($event || SEARCH_SCOPE.Global))"
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
              @click="addGroupItem(String(option.value))"
            >
              {{ option.label }}
            </button>
          </div>
        </div>
      </div>

      <!-- 子规则列表与父级配置之间由分割线清晰隔开 -->
      <div v-if="currentRule.items.length" class="border-t border-(--app-border) my-4 pt-4 space-y-3">
        <EditorSearchRuleBuilder
          v-for="(item, index) in currentRule.items"
          :key="`${item.type}-${index}`"
          :model-value="item"
          :depth="depth + 1"
          :test-id-prefix="rootTestId(`item-${index}`)"
          removable
          :label-index-options="labelIndexOptions"
          :label-select-placeholder="labelSelectPlaceholder"
          :label-select-hint="labelSelectHint"
          @update:model-value="updateGroupItem(index, $event)"
          @remove="removeGroupItem(index)"
        />
      </div>

      <EmptyState v-else title="还没有子规则" description="先在当前逻辑组里添加文本、标签或子组。" class="mt-4" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref } from 'vue';
import { Trash2 } from '@lucide/vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import type { SearchRule } from '@/types/bindings/SearchRule';
import { LOGIC_OP, SEARCH_RULE_TYPE, SEARCH_SCOPE } from '@/views/script-editor/editor-step/editorStepKinds';
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
    labelIndexOptions?: Array<{ label: string; value: number; description?: string; disabled?: boolean }>;
    labelSelectPlaceholder?: string;
    labelSelectHint?: string | null;
  }>(),
  {
    depth: 0,
    removable: false,
    testIdPrefix: null,
    forceGroupRoot: false,
    labelIndexOptions: () => [],
    labelSelectPlaceholder: '请先设置图像检测模型标签文件',
    labelSelectHint: null,
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: SearchRule];
  remove: [];
}>();

const isRootGroup = computed(() => Boolean(props.forceGroupRoot && props.depth === 0));
const currentRule = computed(() => (isRootGroup.value ? ensureRootGroupRule(props.modelValue) : props.modelValue));
const isPatternMissing = computed(() => currentRule.value.type === SEARCH_RULE_TYPE.txt && !currentRule.value.pattern.trim());
const headerLabel = computed(() => {
  if (isRootGroup.value) return '根逻辑组';
  if (currentRule.value.type === SEARCH_RULE_TYPE.group) return '逻辑组';
  return '召回规则';
});
const resolvedLabelIndexOptions = computed(() => {
  if (currentRule.value.type !== SEARCH_RULE_TYPE.detLabel) {
    return props.labelIndexOptions;
  }
  const currentIdx = currentRule.value.idx;
  if (props.labelIndexOptions.some((option) => option.value === currentIdx)) {
    return props.labelIndexOptions;
  }
  return [
    {
      label: `${currentIdx}: 未找到标签`,
      value: currentIdx,
      description: '标签文件中不存在该索引，保存时仍会保留当前 idx。',
    },
    ...props.labelIndexOptions,
  ];
});
const addableRuleTypes = computed(() => searchRuleTypeOptions.filter((option) => option.value !== SEARCH_RULE_TYPE.group || props.depth < 4));
const rootTestId = (suffix: string) => (props.testIdPrefix ? `${props.testIdPrefix}-${suffix}` : undefined);
const rootEl = ref<HTMLElement | null>(null);
const textInputEl = ref<HTMLInputElement | null>(null);

const replaceRule = (value: SearchRule) => {
  emit('update:modelValue', value);
};

const focusCurrentTextInput = async () => {
  await nextTick();
  textInputEl.value?.focus();
  textInputEl.value?.select();
};

const focusLastTextInput = async () => {
  await nextTick();
  const inputs = rootEl.value?.querySelectorAll<HTMLInputElement>('[data-search-rule-text-input="true"]');
  const lastInput = inputs?.item((inputs?.length ?? 1) - 1) ?? null;
  lastInput?.focus();
  lastInput?.select();
};

const changeType = async (type: string) => {
  if (!searchRuleTypeOptions.some((option) => option.value === type)) {
    return;
  }
  replaceRule(createSearchRule(type as SearchRule['type']));
  if (type === SEARCH_RULE_TYPE.txt) {
    await focusCurrentTextInput();
  }
};

const updateGroupField = (field: 'op' | 'scope', value: string) => {
  if (currentRule.value.type !== SEARCH_RULE_TYPE.group) return;
  replaceRule({
    ...currentRule.value,
    [field]: value,
  } as SearchRule);
};

const addGroupItem = async (type: string) => {
  if (currentRule.value.type !== SEARCH_RULE_TYPE.group) return;
  if (!searchRuleTypeOptions.some((option) => option.value === type)) {
    return;
  }
  replaceRule({
    ...currentRule.value,
    items: [...currentRule.value.items, createSearchRule(type as SearchRule['type'])],
  });
  if (type === SEARCH_RULE_TYPE.txt) {
    await focusLastTextInput();
  }
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
