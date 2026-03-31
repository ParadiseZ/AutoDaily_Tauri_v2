<template>
  <div class="space-y-4">
    <div class="grid gap-3 xl:grid-cols-[minmax(0,1fr)_240px]">
      <div class="rounded-[16px] border border-[var(--app-border)] bg-white/40 px-4 py-4">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输出变量</span>
          <input :value="selectedVision.out_var || ''" :list="variableDatalistId" class="app-input" @input="$emit('update-field', 'out_var', ($event.target as HTMLInputElement).value)" />
        </label>
      </div>

      <div class="rounded-[16px] border border-[var(--app-border)] bg-white/40 px-4 py-4">
        <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">命中后行为</p>
        <div class="mt-2 flex items-center justify-between gap-3">
          <span class="text-sm text-[var(--app-text-soft)]">{{ visionBranchTarget?.count ?? 0 }} 个步骤</span>
          <button
            v-if="visionBranchTarget"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            @click="$emit('navigate-branch', visionBranchTarget.path)"
          >
            进入步骤
          </button>
        </div>
      </div>
    </div>

    <div class="rounded-[16px] border border-[var(--app-border)] bg-white/40 px-4 py-4">
      <p class="text-sm font-semibold text-[var(--app-text-strong)]">搜索规则</p>
      <div class="mt-3">
        <EditorSearchRuleBuilder
          :model-value="selectedVision.rule"
          force-group-root
          test-id-prefix="editor-search-rule"
          @update:model-value="$emit('update-rule', $event)"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { SearchRule } from '@/types/bindings/SearchRule';
import type { VisionNode } from '@/types/bindings/VisionNode';
import EditorSearchRuleBuilder from '@/views/script-editor/EditorSearchRuleBuilder.vue';
import type { StepBranchPath } from '@/views/script-editor/editor-step/editorStepTree';

defineOptions({ name: 'EditorStepVisionPanel' });

defineProps<{
  selectedVision: VisionNode & { out_var?: string; rule: SearchRule };
  variableDatalistId: string;
  visionBranchTarget: { count: number; path: StepBranchPath } | null;
}>();

defineEmits<{
  'update-field': [field: string, value: string];
  'update-rule': [rule: SearchRule];
  'navigate-branch': [branchPath: StepBranchPath];
}>();
</script>
