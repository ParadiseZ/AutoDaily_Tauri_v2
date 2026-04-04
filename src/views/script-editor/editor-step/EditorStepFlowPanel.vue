<template>
  <div class="space-y-3">
    <template v-if="selectedFlow.type === FLOW_TYPE.waitMs">
      <label class="space-y-2">
        <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">等待毫秒</span>
        <input :value="String(selectedFlow.ms ?? 1000)" class="app-input" type="number" @input="$emit('update-number-field', 'ms', ($event.target as HTMLInputElement).value)" />
      </label>
    </template>

    <template v-else-if="selectedFlow.type === FLOW_TYPE.link">
      <label class="space-y-2">
        <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标任务</span>
        <input :value="selectedFlow.target || ''" class="app-input" @input="$emit('update-field', 'target', ($event.target as HTMLInputElement).value)" />
      </label>
    </template>

    <template v-else-if="selectedFlow.type === FLOW_TYPE.continue || selectedFlow.type === FLOW_TYPE.break">
      <div class="rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4 text-sm leading-6 text-[var(--app-text-soft)]">
        {{ selectedFlow.type === FLOW_TYPE.continue ? '该步骤会立即开始下一轮循环。' : '该步骤会立即跳出当前循环。' }}
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
          test-id-prefix="editor-condition"
          @update:model-value="$emit('update-flow-condition', $event)"
        />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { FlowControl } from '@/types/bindings/FlowControl';
import EditorConditionBuilder from '@/views/script-editor/EditorConditionBuilder.vue';
import { FLOW_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';

defineOptions({ name: 'EditorStepFlowPanel' });

defineProps<{
  selectedFlow: FlowControl;
  flowWithCondition: { type: string; con: ConditionNode } | null;
  flowCondition: ConditionNode | null;
  hasElseBranch: boolean;
  branchSummary: string;
  flowTypeOptions: Array<{ label: string; value: string; description: string }>;
  readableCatalogVariableOptions: Array<{ label: string; value: string; description: string }>;
}>();

defineEmits<{
  'update-number-field': [field: string, value: string];
  'update-field': [field: string, value: string];
  'update-flow-type': [type: string];
  'update-flow-condition': [condition: ConditionNode];
  'toggle-else-branch': [];
}>();
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
