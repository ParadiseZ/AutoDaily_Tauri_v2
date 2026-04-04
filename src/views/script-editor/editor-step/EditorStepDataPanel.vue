<template>
  <div class="space-y-3">
    <template v-if="selectedData.type === DATA_TYPE.setVar">
      <div class="space-y-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
        <div class="editor-inline-grid">
          <div class="editor-inline-label">目标变量</div>
          <div class="editor-inline-content md:col-span-3">
            <EditorSelectField
              :model-value="selectedData.name || null"
              :options="writableCatalogVariableOptions"
              placeholder="从变量列表中选择"
              test-id="editor-set-var-name"
              @update:model-value="$emit('update-set-var-target', String($event || ''))"
            />
          </div>
        </div>

        <EditorVariableMetaCard v-if="selectedSetVarTarget" :variable="selectedSetVarTarget" />
      </div>

      <div v-if="selectedSetVarTarget && setVarCanSwitchMode" class="flex justify-end">
        <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('update-set-var-mode', setVarUsesExpression ? 'value' : 'expr')">
          {{ setVarUsesExpression ? '改为直接值' : '改用表达式' }}
        </button>
      </div>

      <template v-if="selectedSetVarTarget && !setVarUsesExpression">
        <div v-if="!selectedSetVarKind" class="editor-inline-grid">
          <div class="editor-inline-label">值类型</div>
          <div class="editor-inline-content md:col-span-3">
            <EditorSelectField
              :model-value="effectiveSetVarKind"
              :options="varValueTypeOptions"
              placeholder="值类型"
              test-id="editor-set-var-type"
              @update:model-value="$emit('update-set-var-type', String($event || 'string'))"
            />
          </div>
        </div>

        <label v-if="effectiveSetVarKind === 'bool'" class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
          <input
            :checked="setVarDraft.boolValue"
            type="checkbox"
            class="h-4 w-4"
            data-testid="editor-set-var-bool"
            style="accent-color: var(--app-accent)"
            @change="$emit('update-set-var-bool', ($event.target as HTMLInputElement).checked)"
          />
          <span class="text-sm text-[var(--app-text-soft)]">值为真</span>
        </label>
        <label v-else class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">值</span>
          <input
            :value="setVarDraft.textValue"
            class="app-input"
            :type="effectiveSetVarKind === 'string' ? 'text' : 'number'"
            data-testid="editor-set-var-value"
            @input="$emit('update-set-var-text', ($event.target as HTMLInputElement).value)"
          />
        </label>
      </template>

      <div
        v-else-if="selectedSetVarTarget && !selectedSetVarKind"
        class="rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4 text-sm leading-6 text-[var(--app-text-soft)]"
      >
        当前变量类型不适合直接写固定值，请使用表达式。
      </div>

      <label v-if="selectedSetVarTarget && setVarUsesExpression" class="space-y-2">
        <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">表达式</span>
        <input
          :value="selectedData.expr ?? ''"
          class="app-input"
          @input="$emit('update-data-nullable-field', 'expr', ($event.target as HTMLInputElement).value)"
        />
      </label>
    </template>

    <template v-else-if="selectedData.type === DATA_TYPE.getVar">
      <div class="space-y-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
        <div class="editor-inline-grid">
          <div class="editor-inline-label">读取变量</div>
          <div class="editor-inline-content md:col-span-3">
            <EditorSelectField
              :model-value="selectedData.name || null"
              :options="readableCatalogVariableOptions"
              placeholder="从变量列表中选择"
              test-id="editor-get-var-name"
              @update:model-value="$emit('update-data-field', 'name', String($event || ''))"
            />
          </div>
        </div>

        <EditorVariableMetaCard v-if="selectedGetVarTarget" :variable="selectedGetVarTarget" />
      </div>
      <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
        <input
          :checked="getVarHasDefault"
          type="checkbox"
          class="h-4 w-4"
          style="accent-color: var(--app-accent)"
          @change="$emit('toggle-get-var-default', ($event.target as HTMLInputElement).checked)"
        />
        <span class="text-sm text-[var(--app-text-soft)]">启用默认值</span>
      </label>
      <template v-if="getVarHasDefault">
        <div class="editor-inline-grid">
          <div class="editor-inline-label">默认值类型</div>
          <div class="editor-inline-content md:col-span-3">
            <EditorSelectField
              :model-value="getVarDraft.kind"
              :options="varValueTypeOptions"
              placeholder="默认值类型"
              test-id="editor-get-var-type"
              @update:model-value="$emit('update-get-var-type', String($event || 'string'))"
            />
          </div>
        </div>
        <label v-if="getVarDraft.kind === 'bool'" class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
          <input
            :checked="getVarDraft.boolValue"
            type="checkbox"
            class="h-4 w-4"
            data-testid="editor-get-var-bool"
            style="accent-color: var(--app-accent)"
            @change="$emit('update-get-var-bool', ($event.target as HTMLInputElement).checked)"
          />
          <span class="text-sm text-[var(--app-text-soft)]">默认值为真</span>
        </label>
        <label v-else class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">默认值</span>
          <input
            :value="getVarDraft.textValue"
            class="app-input"
            :type="getVarDraft.kind === 'string' ? 'text' : 'number'"
            data-testid="editor-get-var-value"
            @input="$emit('update-get-var-text', ($event.target as HTMLInputElement).value)"
          />
        </label>
      </template>
    </template>

    <template v-else-if="selectedData.type === DATA_TYPE.filter">
      <div class="grid gap-3 md:grid-cols-2">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输入变量</span>
          <input :value="selectedData.input_var" :list="variableDatalistId" class="app-input" @input="$emit('update-data-field', 'input_var', ($event.target as HTMLInputElement).value)" />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输出变量</span>
          <input :value="selectedData.out_name" :list="variableDatalistId" class="app-input" @input="$emit('update-data-field', 'out_name', ($event.target as HTMLInputElement).value)" />
        </label>
        <div class="editor-inline-grid">
          <div class="editor-inline-label">过滤模式</div>
          <div class="editor-inline-content">
            <EditorSelectField
              :model-value="selectedData.mode.type"
              :options="filterModeOptions"
              placeholder="过滤模式"
              @update:model-value="$emit('update-filter-mode', String($event || FILTER_MODE_TYPE.filter))"
            />
          </div>
        </div>
        <div class="rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-3">
          <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">命中后行为</p>
          <div class="mt-2 flex items-center justify-between gap-3">
            <span class="text-sm text-[var(--app-text-soft)]">{{ filterBranchTarget?.count ?? 0 }} 个步骤</span>
            <button
              v-if="filterBranchTarget"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="$emit('navigate-branch', filterBranchTarget.path)"
            >
              进入步骤
            </button>
          </div>
        </div>
        <label class="space-y-2 md:col-span-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">逻辑表达式</span>
          <input :value="selectedData.logic_expr" class="app-input" @input="$emit('update-data-field', 'logic_expr', ($event.target as HTMLInputElement).value)" />
        </label>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import type { DataHanding } from '@/types/bindings/DataHanding';
import EditorVariableMetaCard from '@/views/script-editor/EditorVariableMetaCard.vue';
import { DATA_TYPE, FILTER_MODE_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';
import { varValueTypeOptions, type VarValueDraft, type VarValueKind } from '@/views/script-editor/editorVarValue';
import type { StepBranchPath } from '@/views/script-editor/editor-step/editorStepTree';
import type { EditorVariableOption } from '@/views/script-editor/editorVariables';

defineOptions({ name: 'EditorStepDataPanel' });

defineProps<{
  selectedData: DataHanding;
  selectedSetVarTarget: EditorVariableOption | null;
  selectedGetVarTarget: EditorVariableOption | null;
  selectedSetVarKind: VarValueKind | null;
  setVarUsesExpression: boolean;
  setVarCanSwitchMode: boolean;
  effectiveSetVarKind: VarValueKind;
  setVarDraft: VarValueDraft;
  getVarHasDefault: boolean;
  getVarDraft: VarValueDraft;
  writableCatalogVariableOptions: Array<{ label: string; value: string; description: string }>;
  readableCatalogVariableOptions: Array<{ label: string; value: string; description: string }>;
  filterModeOptions: Array<{ label: string; value: string; description: string }>;
  filterBranchTarget: { count: number; path: StepBranchPath } | null;
  variableDatalistId: string;
}>();

defineEmits<{
  'update-set-var-target': [value: string];
  'update-set-var-mode': [mode: string];
  'update-set-var-type': [kind: string];
  'update-set-var-text': [value: string];
  'update-set-var-bool': [value: boolean];
  'update-data-field': [field: string, value: string];
  'update-data-nullable-field': [field: string, value: string];
  'toggle-get-var-default': [enabled: boolean];
  'update-get-var-type': [kind: string];
  'update-get-var-text': [value: string];
  'update-get-var-bool': [value: boolean];
  'update-filter-mode': [value: string];
  'navigate-branch': [branchPath: StepBranchPath];
}>();
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
