<template>
  <div class="space-y-4">
    <div class="grid gap-3 xl:grid-cols-[minmax(0,1fr)_240px]">
      <div class="rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">输出名称</span>
          <EditorSelectField
            :model-value="selectedVision.out_var || null"
            :options="resolvedVisionOutputOptions"
            :show-description="true"
            placeholder="选择或创建输出变量"
            test-id="editor-vision-output-var"
            @update:model-value="$emit('update-field', 'out_var', String($event || ''))"
          />
        </label>
        <div v-if="createVariable || (selectedVisionOutputTarget && jumpToVariable)" class="mt-3 flex flex-wrap gap-2">
          <button
            v-if="createVariable"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            data-testid="editor-vision-output-create"
            @click="$emit('create-variable', 'visionOutput')"
          >
            <AppIcon name="plus" :size="14" />
            新建 Runtime 变量
          </button>
          <button
            v-if="selectedVisionOutputTarget && jumpToVariable"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            data-testid="editor-vision-output-locate"
            @click="$emit('jump-to-variable', selectedVisionOutputTarget)"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位变量
          </button>
        </div>
        <EditorVariableMetaCard
          v-if="selectedVisionOutputTarget"
          class="mt-3"
          :variable="selectedVisionOutputTarget"
          :input-entry="selectedVisionOutputInputEntry"
          editable
          @update-input="(entryId, field, value) => emit('update-input', entryId, field, value)"
        />
      </div>

      <div class="rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
        <div class="editor-inline-grid">
          <div class="editor-inline-label">命中后</div>
          <div class="editor-inline-content flex items-center justify-between gap-3">
            <span class="text-sm text-(--app-text-soft)">{{ visionBranchTarget?.count ?? 0 }} 个步骤</span>
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
    </div>

    <div class="rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
      <p class="text-sm font-semibold text-(--app-text-strong)">搜索规则</p>
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
import { computed } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import EditorVariableMetaCard from '@/views/script-editor/EditorVariableMetaCard.vue';
import type { SearchRule } from '@/types/bindings/SearchRule';
import type { VisionNode } from '@/types/bindings/VisionNode';
import EditorSearchRuleBuilder from '@/views/script-editor/EditorSearchRuleBuilder.vue';
import type { StepBranchPath } from '@/views/script-editor/editor-step/editorStepTree';
import type { EditorInputEntry, EditorInputType, EditorVariableOption } from '@/views/script-editor/editorVariables';

defineOptions({ name: 'EditorStepVisionPanel' });

const props = defineProps<{
  selectedVision: VisionNode & { out_var?: string; rule: SearchRule };
  variableDatalistId: string;
  writableCatalogVariableOptions?: Array<{ label: string; value: string; description: string; disabled?: boolean }>;
  selectedVisionOutputTarget?: EditorVariableOption | null;
  selectedVisionOutputInputEntry?: EditorInputEntry | null;
  visionBranchTarget: { count: number; path: StepBranchPath } | null;
  createVariable?: (namespace?: 'input' | 'runtime', inputType?: EditorInputType) => Promise<string>;
  jumpToVariable?: (option: EditorVariableOption) => void;
}>();

const emit = defineEmits<{
  'update-field': [field: string, value: string];
  'update-rule': [rule: SearchRule];
  'navigate-branch': [branchPath: StepBranchPath];
  'create-variable': [target: 'visionOutput'];
  'jump-to-variable': [option: EditorVariableOption];
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
}>();

type SelectOption = { label: string; value: string; description: string; disabled?: boolean };

const withCurrentVariableOption = (options: SelectOption[], value: string) => {
  const trimmedValue = value.trim();
  if (!trimmedValue || options.some((option) => option.value === trimmedValue)) {
    return options;
  }

  return [
    {
      label: trimmedValue,
      value: trimmedValue,
      description: '未解析变量',
    },
    ...options,
  ];
};

const resolvedVisionOutputOptions = computed(() =>
  withCurrentVariableOption(props.writableCatalogVariableOptions ?? [], props.selectedVision.out_var ?? ''),
);
</script>

<style scoped>
.editor-inline-grid {
  display: grid;
  gap: 0.75rem;
}

@media (min-width: 1280px) {
  .editor-inline-grid {
    grid-template-columns: 72px minmax(0, 1fr);
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
