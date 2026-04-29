<template>
  <div class="space-y-3">
    <template v-if="selectedData.type === DATA_TYPE.setVar">
      <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-white/35 px-4 py-4">
        <div class="editor-inline-grid">
          <div class="editor-inline-label">目标名称</div>
          <div class="editor-inline-content md:col-span-3">
            <EditorSelectField
              :model-value="selectedData.name || null"
              :options="writableCatalogVariableOptions"
              :show-description="true"
              placeholder="从变量列表中选择"
              test-id="editor-set-var-name"
              @update:model-value="$emit('update-set-var-target', String($event || ''))"
            />
          </div>
        </div>

        <div v-if="createVariable || (selectedSetVarTarget && jumpToVariable)" class="flex flex-wrap gap-2">
          <button v-if="createVariable" class="app-button app-button-ghost app-toolbar-button" type="button" data-testid="editor-set-var-create" @click="$emit('create-variable', 'setVar')">
            <AppIcon name="plus" :size="14" />
            新建变量
          </button>
          <button
            v-if="selectedSetVarTarget && jumpToVariable"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            data-testid="editor-set-var-locate"
            @click="$emit('jump-to-variable', selectedSetVarTarget)"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位变量
          </button>
        </div>

        <EditorVariableMetaCard
          v-if="selectedSetVarTarget"
          :variable="selectedSetVarTarget"
          :input-entry="selectedSetVarInputEntry"
          editable
          @update-input="(entryId, field, value) => emit('update-input', entryId, field, value)"
        />
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

        <label v-if="effectiveSetVarKind === 'bool'" class="flex items-center gap-3 rounded-[16px] border border-(--app-border) px-4 py-3">
          <input
            :checked="setVarDraft.boolValue"
            type="checkbox"
            class="h-4 w-4"
            data-testid="editor-set-var-bool"
            style="accent-color: var(--app-accent)"
            @change="$emit('update-set-var-bool', ($event.target as HTMLInputElement).checked)"
          />
          <span class="text-sm text-(--app-text-soft)">值为真</span>
        </label>
        <label v-else class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">值</span>
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
        class="rounded-[16px] border border-(--app-border) bg-white/35 px-4 py-4 text-sm leading-6 text-(--app-text-soft)"
      >
        当前变量类型不适合直接写固定值，请使用表达式。
      </div>

      <label v-if="selectedSetVarTarget && setVarUsesExpression" class="space-y-2">
        <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">表达式</span>
        <input
          :value="selectedData.expr ?? ''"
          class="app-input"
          @input="$emit('update-data-nullable-field', 'expr', ($event.target as HTMLInputElement).value)"
        />
      </label>
    </template>

    <template v-else-if="selectedData.type === DATA_TYPE.getVar">
      <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-white/35 px-4 py-4">
        <div class="editor-inline-grid">
          <div class="editor-inline-label">读取名称</div>
          <div class="editor-inline-content md:col-span-3">
            <EditorSelectField
              :model-value="selectedData.name || null"
              :options="readableCatalogVariableOptions"
              :show-description="true"
              placeholder="从变量列表中选择"
              test-id="editor-get-var-name"
              @update:model-value="$emit('update-data-field', 'name', String($event || ''))"
            />
          </div>
        </div>

        <div v-if="createVariable || (selectedGetVarTarget && jumpToVariable)" class="flex flex-wrap gap-2">
          <button v-if="createVariable" class="app-button app-button-ghost app-toolbar-button" type="button" data-testid="editor-get-var-create" @click="$emit('create-variable', 'getVar')">
            <AppIcon name="plus" :size="14" />
            新建变量
          </button>
          <button
            v-if="selectedGetVarTarget && jumpToVariable"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            data-testid="editor-get-var-locate"
            @click="$emit('jump-to-variable', selectedGetVarTarget)"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位变量
          </button>
        </div>

        <EditorVariableMetaCard
          v-if="selectedGetVarTarget"
          :variable="selectedGetVarTarget"
          :input-entry="selectedGetVarInputEntry"
          editable
          @update-input="(entryId, field, value) => emit('update-input', entryId, field, value)"
        />
      </div>
      <label class="flex items-center gap-3 rounded-[16px] border border-(--app-border) px-4 py-3">
        <input
          :checked="getVarHasDefault"
          type="checkbox"
          class="h-4 w-4"
          style="accent-color: var(--app-accent)"
          @change="$emit('toggle-get-var-default', ($event.target as HTMLInputElement).checked)"
        />
        <span class="text-sm text-(--app-text-soft)">启用默认值</span>
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
        <label v-if="getVarDraft.kind === 'bool'" class="flex items-center gap-3 rounded-[16px] border border-(--app-border) px-4 py-3">
          <input
            :checked="getVarDraft.boolValue"
            type="checkbox"
            class="h-4 w-4"
            data-testid="editor-get-var-bool"
            style="accent-color: var(--app-accent)"
            @change="$emit('update-get-var-bool', ($event.target as HTMLInputElement).checked)"
          />
          <span class="text-sm text-(--app-text-soft)">默认值为真</span>
        </label>
        <label v-else class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">默认值</span>
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
        <div class="space-y-3">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">输入名称</span>
            <EditorSelectField
              :model-value="selectedData.input_var || null"
              :options="resolvedFilterInputOptions"
              :show-description="true"
              placeholder="从变量列表中选择"
              test-id="editor-filter-input-var"
              @update:model-value="$emit('update-data-field', 'input_var', String($event || ''))"
            />
          </label>
          <div v-if="createVariable || (selectedFilterInputTarget && jumpToVariable)" class="flex flex-wrap gap-2">
            <button
              v-if="createVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              data-testid="editor-filter-input-create"
              @click="$emit('create-variable', 'filterInput')"
            >
              <AppIcon name="plus" :size="14" />
              新建变量
            </button>
            <button
              v-if="selectedFilterInputTarget && jumpToVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              data-testid="editor-filter-input-locate"
              @click="$emit('jump-to-variable', selectedFilterInputTarget)"
            >
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>
          <EditorVariableMetaCard
            v-if="selectedFilterInputTarget"
            :variable="selectedFilterInputTarget"
            :input-entry="selectedFilterInputEntry"
            editable
            @update-input="(entryId, field, value) => emit('update-input', entryId, field, value)"
          />
        </div>

        <div class="space-y-3">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">输出名称</span>
            <EditorSelectField
              :model-value="selectedData.out_name || null"
              :options="resolvedFilterOutputOptions"
              :show-description="true"
              placeholder="选择或创建输出变量"
              test-id="editor-filter-output-var"
              @update:model-value="$emit('update-data-field', 'out_name', String($event || ''))"
            />
          </label>
          <div v-if="createVariable || (selectedFilterOutputTarget && jumpToVariable)" class="flex flex-wrap gap-2">
            <button
              v-if="createVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              data-testid="editor-filter-output-create"
              @click="$emit('create-variable', 'filterOutput')"
            >
              <AppIcon name="plus" :size="14" />
              新建 Runtime 变量
            </button>
            <button
              v-if="selectedFilterOutputTarget && jumpToVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              data-testid="editor-filter-output-locate"
              @click="$emit('jump-to-variable', selectedFilterOutputTarget)"
            >
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>
          <EditorVariableMetaCard
            v-if="selectedFilterOutputTarget"
            :variable="selectedFilterOutputTarget"
            :input-entry="selectedFilterOutputInputEntry"
            editable
            @update-input="(entryId, field, value) => emit('update-input', entryId, field, value)"
          />
        </div>
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
        <div class="rounded-[16px] border border-(--app-border) bg-white/35 px-4 py-3">
          <p class="text-[11px] uppercase tracking-[0.12em] text-(--app-text-faint)">命中后行为</p>
          <div class="mt-2 flex items-center justify-between gap-3">
            <span class="text-sm text-(--app-text-soft)">{{ filterBranchTarget?.count ?? 0 }} 个步骤</span>
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
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">逻辑表达式</span>
          <input :value="selectedData.logic_expr" class="app-input" @input="$emit('update-data-field', 'logic_expr', ($event.target as HTMLInputElement).value)" />
        </label>
      </div>
    </template>

    <template v-else-if="selectedData.type === DATA_TYPE.colorCompare">
      <div class="grid gap-3 md:grid-cols-2">
        <div class="space-y-3">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">输入结果集</span>
            <EditorSelectField
              :model-value="selectedData.input_var || null"
              :options="resolvedColorCompareInputOptions"
              :show-description="true"
              placeholder="选择 OCR 或过滤结果变量"
              test-id="editor-color-compare-input-var"
              @update:model-value="$emit('update-data-field', 'input_var', String($event || ''))"
            />
          </label>
          <div v-if="createVariable || (selectedColorCompareInputTarget && jumpToVariable)" class="flex flex-wrap gap-2">
            <button
              v-if="createVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              data-testid="editor-color-compare-input-create"
              @click="$emit('create-variable', 'colorCompareInput')"
            >
              <AppIcon name="plus" :size="14" />
              新建变量
            </button>
            <button
              v-if="selectedColorCompareInputTarget && jumpToVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              data-testid="editor-color-compare-input-locate"
              @click="$emit('jump-to-variable', selectedColorCompareInputTarget)"
            >
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>
          <EditorVariableMetaCard
            v-if="selectedColorCompareInputTarget"
            :variable="selectedColorCompareInputTarget"
            :input-entry="selectedColorCompareInputEntry"
            editable
            @update-input="(entryId, field, value) => emit('update-input', entryId, field, value)"
          />
        </div>

        <div class="space-y-3">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">输出结果集</span>
            <EditorSelectField
              :model-value="selectedData.out_var || null"
              :options="resolvedColorCompareOutputOptions"
              :show-description="true"
              placeholder="选择或创建输出变量"
              test-id="editor-color-compare-output-var"
              @update:model-value="$emit('update-data-field', 'out_var', String($event || ''))"
            />
          </label>
          <div v-if="createVariable || (selectedColorCompareOutputTarget && jumpToVariable)" class="flex flex-wrap gap-2">
            <button
              v-if="createVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              data-testid="editor-color-compare-output-create"
              @click="$emit('create-variable', 'colorCompareOutput')"
            >
              <AppIcon name="plus" :size="14" />
              新建 Runtime 变量
            </button>
            <button
              v-if="selectedColorCompareOutputTarget && jumpToVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              data-testid="editor-color-compare-output-locate"
              @click="$emit('jump-to-variable', selectedColorCompareOutputTarget)"
            >
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>
          <EditorVariableMetaCard
            v-if="selectedColorCompareOutputTarget"
            :variable="selectedColorCompareOutputTarget"
            :input-entry="selectedColorCompareOutputInputEntry"
            editable
            @update-input="(entryId, field, value) => emit('update-input', entryId, field, value)"
          />
        </div>

        <label class="space-y-2 md:col-span-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">目标文字</span>
          <input
            :value="selectedData.target_text ?? ''"
            class="app-input"
            placeholder="留空则比较输入结果集中的全部 OCR 区域"
            data-testid="editor-color-compare-target-text"
            @input="$emit('update-data-nullable-field', 'target_text', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <label class="flex items-center gap-3 rounded-[16px] border border-(--app-border) px-4 py-3 md:col-span-2">
          <input
            :checked="selectedData.is_font"
            type="checkbox"
            class="h-4 w-4"
            data-testid="editor-color-compare-is-font"
            style="accent-color: var(--app-accent)"
            @change="$emit('update-color-compare-boolean', 'is_font', ($event.target as HTMLInputElement).checked)"
          />
          <span class="text-sm text-(--app-text-soft)">比较字体颜色</span>
        </label>

        <div class="editor-inline-grid md:col-span-2">
          <div class="editor-inline-label">比较方法</div>
          <div class="editor-inline-content">
            <EditorSelectField
              :model-value="selectedData.method.type"
              :options="colorCompareMethodOptions"
              placeholder="比较方法"
              test-id="editor-color-compare-method"
              @update:model-value="$emit('update-color-compare-method', String($event || 'oklabDistance'))"
            />
          </div>

          <div class="editor-inline-label">阈值</div>
          <div class="editor-inline-content">
            <input
              :value="String(selectedData.method.threshold)"
              class="app-input"
              type="number"
              min="0"
              max="1"
              step="0.01"
              data-testid="editor-color-compare-threshold"
              @input="$emit('update-color-compare-threshold', ($event.target as HTMLInputElement).value)"
            />
          </div>
        </div>

        <div class="grid gap-3 md:col-span-2 md:grid-cols-3">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">R</span>
            <input
              :value="String(selectedData.target_color.r)"
              class="app-input"
              type="number"
              min="0"
              max="255"
              data-testid="editor-color-compare-r"
              @input="$emit('update-color-compare-channel', 'r', ($event.target as HTMLInputElement).value)"
            />
          </label>
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">G</span>
            <input
              :value="String(selectedData.target_color.g)"
              class="app-input"
              type="number"
              min="0"
              max="255"
              data-testid="editor-color-compare-g"
              @input="$emit('update-color-compare-channel', 'g', ($event.target as HTMLInputElement).value)"
            />
          </label>
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">B</span>
            <input
              :value="String(selectedData.target_color.b)"
              class="app-input"
              type="number"
              min="0"
              max="255"
              data-testid="editor-color-compare-b"
              @input="$emit('update-color-compare-channel', 'b', ($event.target as HTMLInputElement).value)"
            />
          </label>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import type { DataHanding } from '@/types/bindings/DataHanding';
import EditorVariableMetaCard from '@/views/script-editor/EditorVariableMetaCard.vue';
import { DATA_TYPE, FILTER_MODE_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';
import { varValueTypeOptions, type VarValueDraft, type VarValueKind } from '@/views/script-editor/editorVarValue';
import type { StepBranchPath } from '@/views/script-editor/editor-step/editorStepTree';
import type { EditorInputEntry, EditorInputType, EditorVariableOption } from '@/views/script-editor/editorVariables';

defineOptions({ name: 'EditorStepDataPanel' });

const props = defineProps<{
  selectedData: DataHanding;
  selectedSetVarTarget: EditorVariableOption | null;
  selectedSetVarInputEntry?: EditorInputEntry | null;
  selectedGetVarTarget: EditorVariableOption | null;
  selectedGetVarInputEntry?: EditorInputEntry | null;
  selectedFilterInputTarget?: EditorVariableOption | null;
  selectedFilterInputEntry?: EditorInputEntry | null;
  selectedFilterOutputTarget?: EditorVariableOption | null;
  selectedFilterOutputInputEntry?: EditorInputEntry | null;
  selectedColorCompareInputTarget?: EditorVariableOption | null;
  selectedColorCompareInputEntry?: EditorInputEntry | null;
  selectedColorCompareOutputTarget?: EditorVariableOption | null;
  selectedColorCompareOutputInputEntry?: EditorInputEntry | null;
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
  colorCompareMethodOptions: Array<{ label: string; value: string; description: string }>;
  filterBranchTarget: { count: number; path: StepBranchPath } | null;
  variableDatalistId: string;
  createVariable?: (namespace?: 'input' | 'runtime', inputType?: EditorInputType) => Promise<string>;
  jumpToVariable?: (option: EditorVariableOption) => void;
}>();

const emit = defineEmits<{
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
  'update-color-compare-channel': [channel: 'r' | 'g' | 'b', value: string];
  'update-color-compare-threshold': [value: string];
  'update-color-compare-method': [value: string];
  'update-color-compare-boolean': [field: 'is_font', value: boolean];
  'navigate-branch': [branchPath: StepBranchPath];
  'create-variable': [target: 'setVar' | 'getVar' | 'filterInput' | 'filterOutput' | 'colorCompareInput' | 'colorCompareOutput'];
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

const resolvedFilterInputOptions = computed(() =>
  props.selectedData.type === DATA_TYPE.filter
    ? withCurrentVariableOption(props.readableCatalogVariableOptions, props.selectedData.input_var)
    : props.readableCatalogVariableOptions,
);

const resolvedFilterOutputOptions = computed(() =>
  props.selectedData.type === DATA_TYPE.filter
    ? withCurrentVariableOption(props.writableCatalogVariableOptions, props.selectedData.out_name)
    : props.writableCatalogVariableOptions,
);

const resolvedColorCompareInputOptions = computed(() =>
  props.selectedData.type === DATA_TYPE.colorCompare
    ? withCurrentVariableOption(props.readableCatalogVariableOptions, props.selectedData.input_var)
    : props.readableCatalogVariableOptions,
);

const resolvedColorCompareOutputOptions = computed(() =>
  props.selectedData.type === DATA_TYPE.colorCompare
    ? withCurrentVariableOption(props.writableCatalogVariableOptions, props.selectedData.out_var)
    : props.writableCatalogVariableOptions,
);
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
