<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <div class="overflow-x-auto">
      <div class="editor-panel-tabs min-w-max">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          class="editor-panel-tab"
          :class="{ 'editor-panel-tab-active': activePanel === tab.id }"
          :data-testid="`editor-policy-tab-${tab.id}`"
          @click="$emit('update:active-panel', tab.id)"
        >
          {{ tab.label }}
        </button>
      </div>
    </div>

    <template v-if="policy">
      <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
        <div v-if="activePanel === 'basic'" class="space-y-4">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.14em] text-(--app-text-faint)">策略名称</span>
            <input :value="policyName" class="app-input" data-testid="editor-policy-name" @input="$emit('update:policy-name', ($event.target as HTMLInputElement).value)" />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.14em] text-(--app-text-faint)">备注</span>
            <textarea :value="policyNote" class="app-input min-h-[120px] resize-y" @input="$emit('update:policy-note', ($event.target as HTMLTextAreaElement).value)" />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.14em] text-(--app-text-faint)">日志输出</span>
            <input :value="policyLogPrint ?? ''" class="app-input" placeholder="可选，用于调试输出" @input="$emit('update:policy-log-print', ($event.target as HTMLInputElement).value)" />
          </label>

          <div class="grid gap-3 sm:grid-cols-2">
            <label class="space-y-2">
              <span class="text-xs font-medium uppercase tracking-[0.14em] text-(--app-text-faint)">多目标时选择第几个</span>
              <input :value="String(policy.data.curPos)" class="app-input" type="number" @input="$emit('update:number-field', 'curPos', ($event.target as HTMLInputElement).value)" />
              <span class="text-xs leading-5 text-(--app-text-faint)">注：按从上到下、从左到右排序后再按此序号点击；999 即最后一个。</span>
            </label>

            <label class="space-y-2">
              <span class="text-xs font-medium uppercase tracking-[0.14em] text-(--app-text-faint)">最大执行次数</span>
              <input :value="String(policy.data.execMax)" class="app-input" type="number" @input="$emit('update:number-field', 'execMax', ($event.target as HTMLInputElement).value)" />
            </label>

            <label class="flex items-center gap-3 rounded-[18px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3 text-sm text-(--app-text-soft)">
              <input
                :checked="policy.data.skipFlag"
                class="h-4 w-4 accent-(--app-accent)"
                type="checkbox"
                @change="$emit('update:boolean-field', 'skipFlag', ($event.target as HTMLInputElement).checked)"
              />
              <span>命中后跳过后续执行</span>
            </label>
          </div>
        </div>

        <EditorVariableListPanel
          v-else-if="activePanel === 'inputs'"
          :entries="inputEntries"
          :selected-input-id="selectedInputId"
          :entry-reference-state="entryReferenceState"
          @add="$emit('add-input')"
          @select="$emit('select-input', $event)"
          @remove="$emit('remove-input', $event)"
        />

        <EditorStepTemplateLibrary
          v-else
          :restrict-sequence-templates="restrictSequenceTemplates"
          test-id-prefix="editor-policy-step-template"
          @select="$emit('append-template-step', $event)"
        />
      </div>
    </template>
  </SurfacePanel>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import EditorStepTemplateLibrary from '@/views/script-editor/EditorStepTemplateLibrary.vue';
import EditorVariableListPanel from '@/views/script-editor/EditorVariableListPanel.vue';
import type { PolicyEditorPanelId } from '@/views/script-editor/editor-policy/editorPolicy';
import type { EditorInputEntry } from '@/views/script-editor/editorVariables';

const props = defineProps<{
  policy: PolicyTable | null;
  activePanel: PolicyEditorPanelId;
  policyName: string;
  policyNote: string;
  policyLogPrint: string | null;
  inputEntries: EditorInputEntry[];
  selectedInputId: string | null;
  entryReferenceState?: Record<string, { referenced: boolean }>;
  conditionCount: number;
  beforeCount: number;
  afterCount: number;
  restrictSequenceTemplates?: boolean;
}>();

defineEmits<{
  'update:active-panel': [panel: PolicyEditorPanelId];
  'update:policy-name': [value: string];
  'update:policy-note': [value: string];
  'update:policy-log-print': [value: string];
  'update:number-field': [field: 'curPos' | 'execMax', value: string];
  'update:boolean-field': [field: 'skipFlag', value: boolean];
  'append-template-step': [templateId: string];
  'add-input': [];
  'select-input': [entryId: string];
  'remove-input': [entryId: string];
}>();

const tabs = computed<Array<{ id: PolicyEditorPanelId; label: string }>>(() => [
  { id: 'basic', label: `基本 ${props.conditionCount}` },
  { id: 'after', label: `命中行为 ${props.afterCount}` },
  { id: 'inputs', label: `变量 ${props.inputEntries.length}` },
  { id: 'before', label: `前置行为 ${props.beforeCount}` },
]);
</script>
