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
          :variable-reference-options="variableReferenceOptions"
          :variable-input-entries="variableInputEntries"
          :task-reference-options="taskReferenceOptions"
          :policy-reference-options="policyReferenceOptions"
          :create-reference="createReference"
          :jump-to-reference="jumpToReference"
          :create-variable="createVariable"
          :jump-to-variable="jumpToVariable"
          test-id-prefix="editor-condition"
          @update:model-value="$emit('update-flow-condition', $event)"
        />
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { FlowControl } from '@/types/bindings/FlowControl';
import EditorConditionBuilder from '@/views/script-editor/EditorConditionBuilder.vue';
import type { EditorReferenceKind, EditorReferenceOption } from '@/views/script-editor/editorReferences';
import { withResolvedReferenceOption } from '@/views/script-editor/editorReferences';
import { FLOW_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';
import type { EditorInputEntry, EditorVariableOption } from '@/views/script-editor/editorVariables';

defineOptions({ name: 'EditorStepFlowPanel' });

const emit = defineEmits<{
  'update-number-field': [field: string, value: string];
  'update-field': [field: string, value: string];
  'update-flow-type': [type: string];
  'update-flow-condition': [condition: ConditionNode];
  'toggle-else-branch': [];
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
  createReference: (kind: EditorReferenceKind) => Promise<string>;
  jumpToReference: (kind: EditorReferenceKind, id: string) => void;
  createVariable?: (namespace?: 'input' | 'runtime') => Promise<string>;
  jumpToVariable?: (option: EditorVariableOption) => void;
}>();

const resolvedTaskReferenceOptions = computed(() =>
  withResolvedReferenceOption(props.taskReferenceOptions, selectedLinkTarget.value, 'task'),
);
const selectedLinkTarget = computed(() => (props.selectedFlow.type === FLOW_TYPE.link ? props.selectedFlow.target : ''));

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
