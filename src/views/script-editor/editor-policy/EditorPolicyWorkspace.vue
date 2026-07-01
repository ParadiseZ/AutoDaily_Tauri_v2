<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <template v-if="policy">
      <EditorInputDetailsPanel
        v-if="activePanel === 'inputs'"
        :selected-input-entry="selectedInputEntry"
        :selected-input-index="selectedInputIndex"
        @update-input="(entryId, field, value) => $emit('update-input', entryId, field, value)"
      />

      <div v-if="activePanel === 'basic'" class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
        <div class="space-y-3">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">命中条件</span>
          <EditorSearchRuleBuilder
            :model-value="policy.data.cond"
            force-group-root
            test-id-prefix="editor-policy-condition"
            :label-index-options="labelIndexOptions"
            :label-select-placeholder="labelSelectPlaceholder"
            :label-select-hint="labelSelectHint"
            @update:model-value="$emit('update:condition', $event)"
          />
        </div>
      </div>

      <EditorStepWorkspace
        v-else-if="activePanel === 'before' || activePanel === 'after'"
        :steps="steps"
        :selected-step-path="selectedStepPath"
        :active-branch-path="activeBranchPath"
        :input-entries="inputEntries"
        :variable-options="variableOptions"
        :catalog-variable-options="catalogVariableOptions"
        :label-index-options="labelIndexOptions"
        :label-select-placeholder="labelSelectPlaceholder"
        :label-select-hint="labelSelectHint"
        :task-reference-options="taskReferenceOptions"
        :task-description-map="taskDescriptionMap"
        :policy-reference-options="policyReferenceOptions"
        :policy-note-map="policyNoteMap"
        :task-ui-variable-options="taskUiVariableOptions"
        :policy-group-reference-options="policyGroupReferenceOptions"
        :policy-group-note-map="policyGroupNoteMap"
        :policy-set-reference-options="policySetReferenceOptions"
        :policy-set-note-map="policySetNoteMap"
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
      description="先从左侧选择策略，右侧才会显示策略变量、命中条件和行为步骤。"
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
import EditorInputDetailsPanel from '@/views/script-editor/EditorInputDetailsPanel.vue';
import EditorSearchRuleBuilder from '@/views/script-editor/EditorSearchRuleBuilder.vue';
import type { EditorReferenceKind, EditorReferenceOption, EditorTaskUiVariableOption } from '@/views/script-editor/editorReferences';
import EditorStepWorkspace from '@/views/script-editor/editor-step/EditorStepWorkspace.vue';
import type { PolicyEditorPanelId } from '@/views/script-editor/editor-policy/editorPolicy';
import type { StepBranchPath, StepPath } from '@/views/script-editor/editor-step/editorStepTree';
import type { EditorInputEntry, EditorInputType, EditorVariableOption } from '@/views/script-editor/editorVariables';

const props = defineProps<{
  policy: PolicyTable | null;
  activePanel: PolicyEditorPanelId;
  steps: Step[];
  selectedStepPath: StepPath | null;
  activeBranchPath: StepBranchPath;
  inputEntries: EditorInputEntry[];
  selectedInputId: string | null;
  variableOptions: EditorVariableOption[];
  catalogVariableOptions: EditorVariableOption[];
  labelIndexOptions: Array<{ label: string; value: number; description?: string; disabled?: boolean }>;
  labelSelectPlaceholder: string;
  labelSelectHint?: string | null;
  taskReferenceOptions: EditorReferenceOption[];
  taskDescriptionMap: Record<string, string>;
  policyReferenceOptions: EditorReferenceOption[];
  policyNoteMap: Record<string, string>;
  taskUiVariableOptions?: EditorTaskUiVariableOption[];
  policyGroupReferenceOptions: EditorReferenceOption[];
  policyGroupNoteMap: Record<string, string>;
  policySetReferenceOptions: EditorReferenceOption[];
  policySetNoteMap: Record<string, string>;
  createReference: (kind: EditorReferenceKind) => Promise<string>;
  jumpToReference: (kind: EditorReferenceKind, id: string) => void;
  createVariable?: (
    namespace?: 'input' | 'runtime',
    inputType?: EditorInputType,
    options?: { preferredKey?: string; name?: string; select?: boolean; silent?: boolean },
  ) => Promise<string>;
  jumpToVariable?: (option: EditorVariableOption) => void;
}>();

defineEmits<{
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
  'update:number-field': [field: 'curPos' | 'execMax', value: string];
  'update:boolean-field': [field: 'skipFlag', value: boolean];
  'update:condition': [value: SearchRule];
  'select-step-path': [path: StepPath];
  'navigate-branch': [branchPath: StepBranchPath];
  'reorder-step': [from: number, to: number];
  'remove-step': [index: number];
  'update-step': [index: number, step: Step];
}>();

const selectedInputEntry = computed(() => props.inputEntries.find((entry) => entry.id === props.selectedInputId) ?? null);
const selectedInputIndex = computed(() =>
  selectedInputEntry.value ? props.inputEntries.findIndex((entry) => entry.id === selectedInputEntry.value?.id) : -1,
);
</script>
