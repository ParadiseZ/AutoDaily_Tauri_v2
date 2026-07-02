<template>
  <div class="space-y-3">
    <template v-if="selectedData.type === DATA_TYPE.setVar">
      <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
        <EditorVariableBindingField
          label="目标变量"
          :model-value="selectedData.name || null"
          :options="resolvedSetVarOptions"
          placeholder="从变量列表中选择"
          test-id="editor-set-var-name"
          locate-test-id="editor-set-var-locate"
          :show-locate="Boolean(selectedSetVarTarget && jumpToVariable)"
          :locate-disabled="!selectedSetVarTarget || !jumpToVariable"
          @update:model-value="$emit('update-set-var-target', String($event || ''))"
          @locate="selectedSetVarTarget ? $emit('jump-to-variable', selectedSetVarTarget) : undefined"
        />
      </div>

      <label v-if="selectedSetVarTarget?.valueType === 'bool'" class="flex items-center gap-3 rounded-[16px] border border-(--app-border) px-4 py-3">
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
        <label
          v-else-if="selectedSetVarTarget && ['string', 'int', 'float'].includes(selectedSetVarTarget.valueType)"
          class="space-y-2"
        >
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">值</span>
          <input
            :value="setVarDraft.textValue"
            class="app-input"
            :type="selectedSetVarTarget.valueType === 'string' ? 'text' : 'number'"
            data-testid="editor-set-var-value"
            @input="$emit('update-set-var-text', ($event.target as HTMLInputElement).value)"
          />
        </label>
        <div v-else-if="selectedSetVarTarget && ['json', 'list', 'object'].includes(selectedSetVarTarget.valueType)" class="space-y-2">
          <div class="flex items-center justify-between gap-3">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">JSON 值</span>
            <span class="text-xs text-(--app-text-faint)">列表必须是 `[]`，对象必须是 `{}`。</span>
          </div>
          <EditorCodeField
            :model-value="setVarJsonText"
            placeholder="请输入 JSON"
            :min-height="180"
            test-id="editor-set-var-json"
            @update:model-value="$emit('update-set-var-json', $event)"
          />
          <div
            v-if="setVarJsonError"
            class="rounded-[14px] border border-amber-500/30 bg-amber-500/10 px-4 py-3 text-sm text-amber-200"
            data-testid="editor-set-var-json-error"
          >
            {{ setVarJsonError }}
          </div>
        </div>
        <div
          v-else-if="selectedSetVarTarget"
          class="rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4 text-sm leading-6 text-(--app-text-soft)"
        >
          当前变量类型不支持在这里直接赋值。
        </div>
    </template>

    <template v-else-if="selectedData.type === DATA_TYPE.clearVars">
      <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
        <div class="space-y-1">
          <p class="text-sm font-semibold text-(--app-text-strong)">要清空的变量</p>
          <p class="text-sm leading-6 text-(--app-text-soft)">
            Input / Runtime 都可以选。图像变量会移除当前引用；列表会清成 `[]`；对象和通用 JSON 会清成 `{}`。
          </p>
        </div>

        <div v-if="clearableVariableOptions.length" class="clear-var-grid">
          <label
            v-for="option in clearableVariableOptions"
            :key="option.value"
            class="clear-var-option"
            :data-testid="`editor-clear-vars-option-${option.value}`"
          >
            <input
              :checked="selectedClearVarNames.includes(option.value)"
              type="checkbox"
              class="h-4 w-4"
              style="accent-color: var(--app-accent)"
              @change="$emit('toggle-clear-var', option.value, ($event.target as HTMLInputElement).checked)"
            />
            <span class="clear-var-copy">
              <span class="clear-var-label">{{ option.label }}</span>
              <span class="clear-var-description">{{ option.description }}</span>
            </span>
          </label>
        </div>

        <div v-else class="rounded-[14px] border border-dashed border-(--app-border) px-4 py-4 text-sm text-(--app-text-soft)">
          当前没有可清空的变量。
        </div>
      </div>
    </template>

    <template v-else-if="selectedData.type === DATA_TYPE.getVar">
      <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
        <EditorVariableBindingField
          label="读取名称"
          :model-value="selectedData.name || null"
          :options="readableCatalogVariableOptions"
          placeholder="从变量列表中选择"
          test-id="editor-get-var-name"
          create-test-id="editor-get-var-create"
          locate-test-id="editor-get-var-locate"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedGetVarTarget && jumpToVariable)"
          :locate-disabled="!selectedGetVarTarget || !jumpToVariable"
          @update:model-value="$emit('update-data-field', 'name', String($event || ''))"
          @create="$emit('create-variable', 'getVar')"
          @locate="selectedGetVarTarget ? $emit('jump-to-variable', selectedGetVarTarget) : undefined"
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

    <template v-else-if="selectedData.type === DATA_TYPE.rhai">
      <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
        <EditorVariableBindingField
          label="输出变量"
          :model-value="selectedData.out_var || null"
          :options="resolvedRhaiOutputOptions"
          placeholder="可选，接收代码块最后返回值"
          test-id="editor-rhai-output-var"
          create-label="新建 Runtime 变量"
          create-test-id="editor-rhai-output-create"
          locate-test-id="editor-rhai-output-locate"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedRhaiOutputTarget && jumpToVariable)"
          :locate-disabled="!selectedRhaiOutputTarget || !jumpToVariable"
          @update:model-value="$emit('update-data-nullable-field', 'out_var', String($event || ''))"
          @create="$emit('create-variable', 'rhaiOutput')"
          @locate="selectedRhaiOutputTarget ? $emit('jump-to-variable', selectedRhaiOutputTarget) : undefined"
        />
      </div>

      <div class="space-y-2">
        <div class="flex items-center justify-between gap-3">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">Rhai 代码</span>
          <span class="text-xs text-(--app-text-faint)">可读写 `input` / `runtime`；动作与视觉等能力请调用 `click(point(...))`、`wait_ms(...)`、`detect(...)` 这类内建函数</span>
        </div>
        <EditorCodeField
          :model-value="selectedData.code"
          placeholder="// 例如：\nruntime.count = (runtime.count ?? 0) + 1;\nclick(point(120, 320));\nwait_ms(1000);\nruntime.count"
          :min-height="260"
          test-id="editor-rhai-code"
          @update:model-value="$emit('update-data-field', 'code', $event)"
        />
      </div>
    </template>

    <template v-else-if="selectedData.type === DATA_TYPE.filter">
      <div class="grid gap-3 md:grid-cols-2">
        <EditorVariableBindingField
          label="输入名称"
          :model-value="selectedData.input_var || null"
          :options="resolvedFilterInputOptions"
          placeholder="从变量列表中选择"
          test-id="editor-filter-input-var"
          create-test-id="editor-filter-input-create"
          locate-test-id="editor-filter-input-locate"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedFilterInputTarget && jumpToVariable)"
          :locate-disabled="!selectedFilterInputTarget || !jumpToVariable"
          @update:model-value="$emit('update-data-field', 'input_var', String($event || ''))"
          @create="$emit('create-variable', 'filterInput')"
          @locate="selectedFilterInputTarget ? $emit('jump-to-variable', selectedFilterInputTarget) : undefined"
        />

        <EditorVariableBindingField
          label="输出名称"
          :model-value="selectedData.out_name || null"
          :options="resolvedFilterOutputOptions"
          placeholder="选择或创建输出变量"
          test-id="editor-filter-output-var"
          create-label="新建 Runtime 变量"
          create-test-id="editor-filter-output-create"
          locate-test-id="editor-filter-output-locate"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedFilterOutputTarget && jumpToVariable)"
          :locate-disabled="!selectedFilterOutputTarget || !jumpToVariable"
          @update:model-value="$emit('update-data-field', 'out_name', String($event || ''))"
          @create="$emit('create-variable', 'filterOutput')"
          @locate="selectedFilterOutputTarget ? $emit('jump-to-variable', selectedFilterOutputTarget) : undefined"
        />
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
        <div class="rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3">
          <p class="text-[11px] uppercase tracking-[0.12em] text-(--app-text-faint)">命中后行为</p>
          <div class="mt-2 flex items-center justify-between gap-3">
            <span class="text-sm text-(--app-text-soft)">{{ filterBranchTarget?.count ?? 0 }} 个步骤</span>
            <button
              v-if="filterBranchTarget"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              data-testid="editor-branch-filterThen"
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
        <div class="md:col-span-2 grid gap-3 md:grid-cols-2">
          <RegionPointEditor
            label="区域左上"
            :point="selectedData.region_top_left"
            @update-mode="$emit('update-region-point', 'region_top_left', 'mode', $event)"
            @update-x="$emit('update-region-point', 'region_top_left', 'x', $event)"
            @update-y="$emit('update-region-point', 'region_top_left', 'y', $event)"
          />
          <RegionPointEditor
            label="区域右下"
            :point="selectedData.region_bottom_right"
            @update-mode="$emit('update-region-point', 'region_bottom_right', 'mode', $event)"
            @update-x="$emit('update-region-point', 'region_bottom_right', 'x', $event)"
            @update-y="$emit('update-region-point', 'region_bottom_right', 'y', $event)"
          />
        </div>
      </div>
    </template>

    <template v-else-if="selectedData.type === DATA_TYPE.colorCompare">
      <div class="grid gap-3 md:grid-cols-2">
        <EditorVariableBindingField
          label="输入结果集"
          :model-value="selectedData.input_var || null"
          :options="resolvedColorCompareInputOptions"
          placeholder="选择 OCR 或待筛选结果变量"
          test-id="editor-color-compare-input-var"
          create-test-id="editor-color-compare-input-create"
          locate-test-id="editor-color-compare-input-locate"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedColorCompareInputTarget && jumpToVariable)"
          :locate-disabled="!selectedColorCompareInputTarget || !jumpToVariable"
          @update:model-value="$emit('update-data-field', 'input_var', String($event || ''))"
          @create="$emit('create-variable', 'colorCompareInput')"
          @locate="selectedColorCompareInputTarget ? $emit('jump-to-variable', selectedColorCompareInputTarget) : undefined"
        />

        <EditorVariableBindingField
          label="输出结果集"
          :model-value="selectedData.out_var || null"
          :options="resolvedColorCompareOutputOptions"
          placeholder="选择或创建筛选结果变量"
          test-id="editor-color-compare-output-var"
          create-label="新建 Runtime 变量"
          create-test-id="editor-color-compare-output-create"
          locate-test-id="editor-color-compare-output-locate"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedColorCompareOutputTarget && jumpToVariable)"
          :locate-disabled="!selectedColorCompareOutputTarget || !jumpToVariable"
          @update:model-value="$emit('update-data-field', 'out_var', String($event || ''))"
          @create="$emit('create-variable', 'colorCompareOutput')"
          @locate="selectedColorCompareOutputTarget ? $emit('jump-to-variable', selectedColorCompareOutputTarget) : undefined"
        />

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

        <div class="md:col-span-2 grid gap-3 md:grid-cols-2">
          <RegionPointEditor
            label="区域左上"
            :point="selectedData.region_top_left"
            @update-mode="$emit('update-region-point', 'region_top_left', 'mode', $event)"
            @update-x="$emit('update-region-point', 'region_top_left', 'x', $event)"
            @update-y="$emit('update-region-point', 'region_top_left', 'y', $event)"
          />
          <RegionPointEditor
            label="区域右下"
            :point="selectedData.region_bottom_right"
            @update-mode="$emit('update-region-point', 'region_bottom_right', 'mode', $event)"
            @update-x="$emit('update-region-point', 'region_bottom_right', 'x', $event)"
            @update-y="$emit('update-region-point', 'region_bottom_right', 'y', $event)"
          />
        </div>

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
import { computed, defineComponent, h, type PropType } from 'vue';
import EditorCodeField from '@/views/script-editor/EditorCodeField.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import EditorVariableBindingField from '@/views/script-editor/EditorVariableBindingField.vue';
import type { DataHanding } from '@/types/bindings/DataHanding';
import { DATA_TYPE, FILTER_MODE_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';
import { varValueTypeOptions, type VarValueDraft } from '@/views/script-editor/editorVarValue';
import type { StepBranchPath } from '@/views/script-editor/editor-step/editorStepTree';
import type { EditorInputType, EditorVariableOption } from '@/views/script-editor/editorVariables';

defineOptions({ name: 'EditorStepDataPanel' });

const props = defineProps<{
  selectedData: DataHanding;
  selectedSetVarTarget: EditorVariableOption | null;
  selectedGetVarTarget: EditorVariableOption | null;
  selectedRhaiOutputTarget?: EditorVariableOption | null;
  selectedFilterInputTarget?: EditorVariableOption | null;
  selectedFilterOutputTarget?: EditorVariableOption | null;
  selectedColorCompareInputTarget?: EditorVariableOption | null;
  selectedColorCompareOutputTarget?: EditorVariableOption | null;
  setVarDraft: VarValueDraft;
  setVarJsonText: string;
  setVarJsonError: string | null;
  getVarHasDefault: boolean;
  getVarDraft: VarValueDraft;
  writableCatalogVariableOptions: Array<{ label: string; value: string; description: string }>;
  readableCatalogVariableOptions: Array<{ label: string; value: string; description: string }>;
  clearableVariableOptions: Array<{ label: string; value: string; description: string }>;
  selectedClearVarNames: string[];
  filterModeOptions: Array<{ label: string; value: string; description: string }>;
  colorCompareMethodOptions: Array<{ label: string; value: string; description: string }>;
  filterBranchTarget: { count: number; path: StepBranchPath } | null;
  variableDatalistId: string;
  createVariable?: (namespace?: 'input' | 'runtime', inputType?: EditorInputType) => Promise<string>;
  jumpToVariable?: (option: EditorVariableOption) => void;
}>();

const emit = defineEmits<{
  'update-set-var-target': [value: string];
  'update-set-var-text': [value: string];
  'update-set-var-bool': [value: boolean];
  'update-set-var-json': [value: string];
  'toggle-clear-var': [value: string, checked: boolean];
  'update-data-field': [field: string, value: string];
  'update-data-nullable-field': [field: string, value: string];
  'update-region-point': [field: 'region_top_left' | 'region_bottom_right', key: 'mode' | 'x' | 'y', value: string];
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
  'create-variable': [target: 'getVar' | 'rhaiOutput' | 'filterInput' | 'filterOutput' | 'colorCompareInput' | 'colorCompareOutput'];
  'jump-to-variable': [option: EditorVariableOption];
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
}>();

type SelectOption = { label: string; value: string; description: string; disabled?: boolean };
type RegionPoint = { mode: 'point' | 'percent'; p: { x: number; y: number } };

const regionModeOptions = [
  { label: '坐标', value: 'point', description: '使用设备像素坐标。' },
  { label: '百分比', value: 'percent', description: '按设备宽高换算百分比。' },
];

const RegionPointEditor = defineComponent({
  name: 'RegionPointEditor',
  props: {
    label: { type: String, required: true },
    point: { type: Object as PropType<RegionPoint>, required: true },
  },
  emits: ['update-mode', 'update-x', 'update-y'],
  setup(componentProps, { emit: componentEmit }) {
    return () =>
      h('div', { class: 'space-y-3 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3' }, [
        h('p', { class: 'text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)' }, componentProps.label),
        h(EditorSelectField, {
          modelValue: componentProps.point.mode,
          options: regionModeOptions,
          placeholder: '坐标模式',
          'onUpdate:modelValue': (value: unknown) => componentEmit('update-mode', String(value || 'point')),
        }),
        h('div', { class: 'grid grid-cols-2 gap-3' }, [
          h('input', {
            value: String(componentProps.point.p?.x ?? 0),
            class: 'app-input',
            type: 'number',
            placeholder: 'x',
            onInput: (event: Event) => componentEmit('update-x', (event.target as HTMLInputElement).value),
          }),
          h('input', {
            value: String(componentProps.point.p?.y ?? 0),
            class: 'app-input',
            type: 'number',
            placeholder: 'y',
            onInput: (event: Event) => componentEmit('update-y', (event.target as HTMLInputElement).value),
          }),
        ]),
      ]);
  },
});

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

const resolvedSetVarOptions = computed(() =>
  props.selectedData.type === DATA_TYPE.setVar
    ? withCurrentVariableOption(props.writableCatalogVariableOptions, props.selectedData.name)
    : props.writableCatalogVariableOptions,
);

const resolvedRhaiOutputOptions = computed(() =>
  props.selectedData.type === DATA_TYPE.rhai
    ? withCurrentVariableOption(props.writableCatalogVariableOptions, props.selectedData.out_var ?? '')
    : props.writableCatalogVariableOptions,
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

.clear-var-grid {
  display: grid;
  gap: 0.75rem;
}

.clear-var-option {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  border: 1px solid var(--app-border);
  border-radius: 14px;
  padding: 0.85rem 0.95rem;
  background: color-mix(in srgb, var(--app-panel-muted) 86%, white 14%);
}

.clear-var-copy {
  display: flex;
  min-width: 0;
  flex: 1 1 auto;
  flex-direction: column;
  gap: 0.2rem;
}

.clear-var-label {
  color: var(--app-text-strong);
  font-size: 0.92rem;
  font-weight: 600;
}

.clear-var-description {
  color: var(--app-text-faint);
  font-size: 0.76rem;
  line-height: 1.4;
}
</style>
