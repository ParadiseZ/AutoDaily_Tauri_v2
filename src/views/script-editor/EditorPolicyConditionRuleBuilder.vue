<template>
  <div
    class="editor-search-rule-card"
    :class="{
      'editor-search-rule-nested': depth > 0,
      'editor-search-rule-group': currentRule.type === POLICY_CONDITION_RULE_TYPE.group,
    }"
  >
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="min-w-0 flex-1 space-y-1">
        <div class="flex flex-wrap items-center gap-2">
          <span class="editor-search-rule-badge">{{ currentRule.type === POLICY_CONDITION_RULE_TYPE.group ? '规则组' : '精判规则' }}</span>
          <span class="truncate text-xs text-[var(--app-text-faint)]">{{ describePolicyConditionRule(currentRule) }}</span>
        </div>

        <div class="flex min-w-0 flex-1 flex-wrap items-center gap-3">
          <EditorSelectField
            :model-value="currentRule.type"
            :options="policyConditionRuleTypeOptions"
            placeholder="规则类型"
            class="min-w-[180px]"
            :test-id="rootTestId('type')"
            @update:model-value="changeType(String($event || POLICY_CONDITION_RULE_TYPE.regex))"
          />
        </div>
      </div>

      <button v-if="removable" class="app-button app-button-danger app-toolbar-button" type="button" @click="$emit('remove')">
        删除规则
      </button>
    </div>

    <div class="mt-4 space-y-3">
      <template v-if="currentRule.type === POLICY_CONDITION_RULE_TYPE.regex">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">正则表达式</span>
          <input
            :value="currentRule.pattern"
            class="app-input"
            :data-testid="rootTestId('regex')"
            @input="replaceRule({ ...currentRule, pattern: ($event.target as HTMLInputElement).value })"
          />
        </label>
      </template>

      <template v-else-if="currentRule.type === POLICY_CONDITION_RULE_TYPE.relative">
        <div class="rounded-[16px] border border-[var(--app-border)] bg-white/45 px-4 py-4">
          <div class="editor-inline-grid">
            <div class="editor-inline-label">锚点类型</div>
            <div class="editor-inline-content">
              <EditorSelectField
                :model-value="currentRule.anchor_type"
                :options="relativeAnchorTypeOptions"
                placeholder="选择锚点类型"
                :test-id="rootTestId('relative-anchor-type')"
                @update:model-value="replaceRule({ ...currentRule, anchor_type: String($event || 'ocrText') as typeof currentRule.anchor_type })"
              />
            </div>

            <div class="editor-inline-label">方向</div>
            <div class="editor-inline-content">
              <EditorSelectField
                :model-value="currentRule.direction"
                :options="relativeDirectionOptions"
                placeholder="选择方向"
                :test-id="rootTestId('relative-direction')"
                @update:model-value="replaceRule({ ...currentRule, direction: String($event || 'right') as typeof currentRule.direction })"
              />
            </div>

            <div class="editor-inline-label">目标类型</div>
            <div class="editor-inline-content">
              <EditorSelectField
                :model-value="currentRule.target_kind"
                :options="relativeTargetKindOptions"
                placeholder="选择目标类型"
                :test-id="rootTestId('relative-target-kind')"
                @update:model-value="replaceRule({ ...currentRule, target_kind: String($event || 'ocrText') as typeof currentRule.target_kind })"
              />
            </div>

            <div class="editor-inline-label">值类型</div>
            <div class="editor-inline-content">
              <EditorSelectField
                :model-value="currentRule.value_type"
                :options="relativeValueTypeOptions"
                placeholder="选择值类型"
                :test-id="rootTestId('relative-value-type')"
                @update:model-value="replaceRule({ ...currentRule, value_type: String($event || 'text') as typeof currentRule.value_type })"
              />
            </div>

            <div class="editor-inline-label">比较</div>
            <div class="editor-inline-content">
              <EditorSelectField
                :model-value="currentRule.compare"
                :options="relativeCompareOptions"
                placeholder="选择比较方式"
                :test-id="rootTestId('relative-compare')"
                @update:model-value="replaceRule({ ...currentRule, compare: String($event || 'eq') as typeof currentRule.compare })"
              />
            </div>
          </div>

          <div class="mt-4 grid gap-4 md:grid-cols-2">
            <label class="grid gap-2">
              <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">
                {{ currentRule.anchor_type === 'ocrText' ? '锚点文字' : '锚点标签索引' }}
              </span>
              <input
                v-if="currentRule.anchor_type === 'ocrText'"
                :value="currentRule.anchor_text"
                class="app-input"
                :data-testid="rootTestId('relative-anchor-text')"
                @input="replaceRule({ ...currentRule, anchor_text: ($event.target as HTMLInputElement).value })"
              />
              <input
                v-else
                :value="String(currentRule.anchor_idx)"
                class="app-input"
                type="number"
                :data-testid="rootTestId('relative-anchor-idx')"
                @input="replaceRule({ ...currentRule, anchor_idx: Number(($event.target as HTMLInputElement).value) || 0 })"
              />
            </label>

            <label class="grid gap-2">
              <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">比较值</span>
              <input
                :value="currentRule.value"
                class="app-input"
                :data-testid="rootTestId('relative-value')"
                @input="replaceRule({ ...currentRule, value: ($event.target as HTMLInputElement).value })"
              />
            </label>
          </div>
        </div>
      </template>

      <template v-else-if="currentRule.type === POLICY_CONDITION_RULE_TYPE.group">
        <div class="rounded-[16px] border border-[var(--app-border)] bg-white/45 px-4 py-4">
          <div class="editor-inline-grid">
            <div class="editor-inline-label">组合逻辑</div>
            <div class="editor-inline-content">
              <EditorSelectField
                :model-value="currentRule.op"
                :options="logicOpOptions"
                placeholder="组合逻辑"
                :test-id="rootTestId('logic-op')"
                @update:model-value="updateGroupOp(String($event || LOGIC_OP.And))"
              />
            </div>
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
          <EditorPolicyConditionRuleBuilder
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

        <EmptyState v-if="!currentRule.items.length" title="还没有子规则" description="先添加正则、相对位置或子组。" />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import type { PolicyConditionRule } from '@/types/bindings/PolicyConditionRule';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import { LOGIC_OP } from '@/views/script-editor/editor-step/editorStepKinds';
import {
  createPolicyConditionRule,
  describePolicyConditionRule,
  logicOpOptions,
  POLICY_CONDITION_RULE_TYPE,
  policyConditionRuleTypeOptions,
  relativeAnchorTypeOptions,
  relativeCompareOptions,
  relativeDirectionOptions,
  relativeTargetKindOptions,
  relativeValueTypeOptions,
} from '@/views/script-editor/editorPolicyConditionRule';

defineOptions({ name: 'EditorPolicyConditionRuleBuilder' });

const props = withDefaults(defineProps<{
  modelValue: PolicyConditionRule;
  depth?: number;
  removable?: boolean;
  testIdPrefix?: string | null;
}>(), {
  depth: 0,
  removable: false,
  testIdPrefix: null,
});

const emit = defineEmits<{
  'update:modelValue': [value: PolicyConditionRule];
  remove: [];
}>();

const currentRule = computed(() => props.modelValue);
const addableRuleTypes = computed(() => policyConditionRuleTypeOptions.filter((option) => option.value !== POLICY_CONDITION_RULE_TYPE.group || props.depth < 2));
const rootTestId = (suffix: string) => (props.testIdPrefix ? `${props.testIdPrefix}-${suffix}` : undefined);

const replaceRule = (value: PolicyConditionRule) => emit('update:modelValue', value);
const changeType = (type: string) => replaceRule(createPolicyConditionRule(type));

const updateGroupOp = (op: string) => {
  if (currentRule.value.type !== POLICY_CONDITION_RULE_TYPE.group) return;
  replaceRule({ ...currentRule.value, op: op as typeof currentRule.value.op });
};

const addGroupItem = (type: string) => {
  if (currentRule.value.type !== POLICY_CONDITION_RULE_TYPE.group) return;
  replaceRule({ ...currentRule.value, items: [...currentRule.value.items, createPolicyConditionRule(type)] });
};

const updateGroupItem = (index: number, value: PolicyConditionRule) => {
  if (currentRule.value.type !== POLICY_CONDITION_RULE_TYPE.group) return;
  replaceRule({
    ...currentRule.value,
    items: currentRule.value.items.map((item, itemIndex) => (itemIndex === index ? value : item)),
  });
};

const removeGroupItem = (index: number) => {
  if (currentRule.value.type !== POLICY_CONDITION_RULE_TYPE.group) return;
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
  position: relative;
  margin-left: 0.4rem;
  background: rgba(255, 255, 255, 0.56);
}

.editor-search-rule-nested::before {
  content: '';
  position: absolute;
  top: 14px;
  bottom: 14px;
  left: -10px;
  width: 2px;
  border-radius: 999px;
  background: rgba(70, 110, 255, 0.18);
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
