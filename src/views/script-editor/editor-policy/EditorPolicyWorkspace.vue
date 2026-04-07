<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <template v-if="policy">
      <div class="flex items-start justify-between gap-3">
        <div class="space-y-1">
          <p class="text-xs uppercase tracking-[0.18em] text-[var(--app-text-faint)]">Workspace</p>
          <h2 class="text-xl font-semibold text-[var(--app-text-strong)]">{{ workspaceTitle }}</h2>
        </div>
      </div>

      <div v-if="activePanel === 'basic'" class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
        <div class="grid gap-4 xl:grid-cols-2">
          <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <p class="text-xs uppercase tracking-[0.12em] text-[var(--app-text-faint)]">当前位置</p>
            <input
              :value="String(policy.data.curPos)"
              class="app-input mt-2"
              type="number"
              @input="$emit('update:number-field', 'curPos', ($event.target as HTMLInputElement).value)"
            />
          </div>

          <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <label class="flex items-center gap-3 text-sm text-[var(--app-text-soft)]">
              <input
                :checked="policy.data.skipFlag"
                class="h-4 w-4 accent-[var(--app-accent)]"
                type="checkbox"
                @change="$emit('update:boolean-field', 'skipFlag', ($event.target as HTMLInputElement).checked)"
              />
              <span>命中后跳过后续执行</span>
            </label>
          </div>

          <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <p class="text-xs uppercase tracking-[0.12em] text-[var(--app-text-faint)]">当前执行次数</p>
            <input
              :value="String(policy.data.execCur)"
              class="app-input mt-2"
              type="number"
              @input="$emit('update:number-field', 'execCur', ($event.target as HTMLInputElement).value)"
            />
          </div>

          <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <p class="text-xs uppercase tracking-[0.12em] text-[var(--app-text-faint)]">最大执行次数</p>
            <input
              :value="String(policy.data.execMax)"
              class="app-input mt-2"
              type="number"
              @input="$emit('update:number-field', 'execMax', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </div>
      </div>

      <div v-else-if="activePanel === 'condition'" class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
        <EditorSearchRuleBuilder
          :model-value="policy.data.cond"
          force-group-root
          test-id-prefix="editor-policy-condition"
          @update:model-value="$emit('update:condition', $event)"
        />
      </div>

      <EditorStepWorkspace
        v-else
        :steps="steps"
        :selected-step-path="selectedStepPath"
        :active-branch-path="activeBranchPath"
        :variable-options="variableOptions"
        :catalog-variable-options="catalogVariableOptions"
        :label-index-options="labelIndexOptions"
        :label-select-placeholder="labelSelectPlaceholder"
        :label-select-hint="labelSelectHint"
        :task-reference-options="taskReferenceOptions"
        :policy-reference-options="policyReferenceOptions"
        :create-reference="createReference"
        :jump-to-reference="jumpToReference"
        :create-variable="createVariable"
        :jump-to-variable="jumpToVariable"
        @select-step-path="$emit('select-step-path', $event)"
        @navigate-branch="$emit('navigate-branch', $event)"
        @reorder-step="(from, to) => $emit('reorder-step', from, to)"
        @remove-step="$emit('remove-step', $event)"
        @update-step="(index, step) => $emit('update-step', index, step)"
      />
    </template>

    <EmptyState
      v-else
      title="没有选中策略"
      description="先从左侧选择策略，右侧才会显示策略信息、命中条件和行为步骤。"
    />
  </SurfacePanel>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import type { SearchRule } from '@/types/bindings/SearchRule';
import type { Step } from '@/types/bindings/Step';
import EditorSearchRuleBuilder from '@/views/script-editor/EditorSearchRuleBuilder.vue';
import type { EditorReferenceKind, EditorReferenceOption } from '@/views/script-editor/editorReferences';
import EditorStepWorkspace from '@/views/script-editor/editor-step/EditorStepWorkspace.vue';
import type { PolicyEditorPanelId } from '@/views/script-editor/editor-policy/editorPolicy';
import type { StepBranchPath, StepPath } from '@/views/script-editor/editor-step/editorStepTree';
import type { EditorInputType, EditorVariableOption } from '@/views/script-editor/editorVariables';

const props = defineProps<{
  policy: PolicyTable | null;
  activePanel: PolicyEditorPanelId;
  steps: Step[];
  selectedStepPath: StepPath | null;
  activeBranchPath: StepBranchPath;
  variableOptions: EditorVariableOption[];
  catalogVariableOptions: EditorVariableOption[];
  labelIndexOptions: Array<{ label: string; value: number; description?: string; disabled?: boolean }>;
  labelSelectPlaceholder: string;
  labelSelectHint?: string | null;
  taskReferenceOptions: EditorReferenceOption[];
  policyReferenceOptions: EditorReferenceOption[];
  createReference: (kind: EditorReferenceKind) => Promise<string>;
  jumpToReference: (kind: EditorReferenceKind, id: string) => void;
  createVariable?: (namespace?: 'input' | 'runtime', inputType?: EditorInputType) => Promise<string>;
  jumpToVariable?: (option: EditorVariableOption) => void;
}>();

defineEmits<{
  'update:number-field': [field: 'curPos' | 'execCur' | 'execMax', value: string];
  'update:boolean-field': [field: 'skipFlag', value: boolean];
  'update:condition': [value: SearchRule];
  'select-step-path': [path: StepPath];
  'navigate-branch': [branchPath: StepBranchPath];
  'reorder-step': [from: number, to: number];
  'remove-step': [index: number];
  'update-step': [index: number, step: Step];
}>();

const workspaceTitle = computed(() => {
  if (props.activePanel === 'condition') return '命中条件';
  if (props.activePanel === 'before') return '全局行为';
  if (props.activePanel === 'after') return '命中行为';
  return '策略信息';
});
</script>
