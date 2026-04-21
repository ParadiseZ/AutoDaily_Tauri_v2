<template>
  <div class="space-y-3">
    <div class="editor-inline-grid">
      <div class="editor-inline-label">最大执行次数</div>
      <div class="editor-inline-content">
        <input
          :value="String(actionExecMax)"
          class="app-input"
          type="number"
          min="0"
          data-testid="editor-action-exec-max"
          @input="$emit('update-exec-max', ($event.target as HTMLInputElement).value)"
        />
      </div>
      <div class="editor-inline-label">说明</div>
      <div class="editor-inline-content text-sm text-[var(--app-text-soft)]">
        `0` 表示无限次。
      </div>
    </div>

    <template v-if="selectedAction.ac === ACTION_TYPE.capture">
      <div class="space-y-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输出名称</span>
          <EditorSelectField
            :model-value="selectedAction.output_var || null"
            :options="resolvedCaptureOutputOptions"
            :show-description="true"
            placeholder="选择或创建输出变量"
            test-id="editor-capture-output-var"
            @update:model-value="$emit('update-field', 'output_var', String($event || ''))"
          />
        </label>
        <div v-if="createVariable || (selectedCaptureOutputTarget && jumpToVariable)" class="flex flex-wrap gap-2">
          <button
            v-if="createVariable"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            data-testid="editor-capture-output-create"
            @click="$emit('create-variable', 'captureOutput')"
          >
            <AppIcon name="plus" :size="14" />
            新建 Runtime 变量
          </button>
          <button
            v-if="selectedCaptureOutputTarget && jumpToVariable"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            data-testid="editor-capture-output-locate"
            @click="$emit('jump-to-variable', selectedCaptureOutputTarget)"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位变量
          </button>
        </div>
        <EditorVariableMetaCard
          v-if="selectedCaptureOutputTarget"
          :variable="selectedCaptureOutputTarget"
          :input-entry="selectedCaptureOutputInputEntry"
          editable
          @update-input="(entryId, field, value) => emit('update-input', entryId, field, value)"
        />
        <p class="text-xs leading-5 text-[var(--app-text-faint)]">
          当前运行时会把截图图像对象写入 runtime 变量，不再默认转成字符串或文件路径。
        </p>
      </div>
    </template>

    <template v-else-if="selectedAction.ac === ACTION_TYPE.launchApp || selectedAction.ac === ACTION_TYPE.stopApp">
      <div class="space-y-3">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">包名</span>
          <input :value="selectedAction.pkg_name || ''" class="app-input" @input="$emit('update-field', 'pkg_name', ($event.target as HTMLInputElement).value)" />
        </label>
        <label v-if="selectedAction.ac === ACTION_TYPE.launchApp" class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">Activity</span>
          <input :value="selectedAction.activity_name || ''" class="app-input" placeholder=".MainActivity" @input="$emit('update-field', 'activity_name', ($event.target as HTMLInputElement).value)" />
        </label>
      </div>
    </template>

    <template v-else-if="selectedAction.ac === ACTION_TYPE.click">
      <div class="editor-inline-grid">
        <div class="editor-inline-label">点击方式</div>
        <div class="editor-inline-content">
          <EditorSelectField
            :model-value="String(selectedAction.mode || ACTION_MODE.point)"
            :options="clickModeOptions"
            placeholder="点击方式"
            @update:model-value="$emit('update-mode', String($event || ACTION_MODE.point))"
          />
        </div>
      </div>

      <div v-if="selectedAction.mode === ACTION_MODE.point || selectedAction.mode === ACTION_MODE.percent" class="editor-inline-grid">
        <div class="editor-inline-label">X</div>
        <div class="editor-inline-content">
          <input
            :value="String((selectedAction.p as { x?: number })?.x ?? '')"
            aria-label="X"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'p', 'x', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="editor-inline-label">Y</div>
        <div class="editor-inline-content">
          <input
            :value="String((selectedAction.p as { y?: number })?.y ?? '')"
            aria-label="Y"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'p', 'y', ($event.target as HTMLInputElement).value)"
          />
        </div>
      </div>

      <template v-if="selectedAction.mode === ACTION_MODE.txt || selectedAction.mode === ACTION_MODE.labelIdx">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输入结果变量</span>
          <EditorSelectField
            :model-value="selectedActionInput || null"
            :options="resolvedActionInputOptions"
            :show-description="true"
            placeholder="选择 OCR / 检测 / 处理结果变量"
            test-id="editor-action-click-input-var"
            @update:model-value="$emit('update-field', 'input_var', String($event || ''))"
          />
        </label>
        <div class="flex flex-wrap gap-2">
          <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('create-variable', 'actionInput')">
            <AppIcon name="plus" :size="14" />
            新建结果变量
          </button>
          <button
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            :disabled="!selectedActionInputTarget || !jumpToVariable"
            @click="selectedActionInputTarget ? $emit('jump-to-variable', selectedActionInputTarget) : undefined"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位变量
          </button>
        </div>
      </template>

      <label v-if="selectedAction.mode === ACTION_MODE.txt" class="space-y-2">
        <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标文字</span>
        <input :value="String(selectedAction.txt ?? '')" class="app-input" @input="$emit('update-text-field', 'txt', ($event.target as HTMLInputElement).value)" />
      </label>

      <label v-else-if="selectedAction.mode === ACTION_MODE.labelIdx" class="space-y-2">
        <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">标签</span>
        <AppSelect
          :model-value="selectedAction.idx ?? null"
          :options="resolvedLabelIdxOptions"
          :placeholder="labelSelectPlaceholder"
          :disabled="!(labelIndexOptions?.length)"
          test-id="editor-action-click-label-idx"
          @update:model-value="$emit('update-number-field', 'idx', String($event ?? 0))"
        />
        <p v-if="labelSelectHint" class="text-xs leading-5 text-amber-700">{{ labelSelectHint }}</p>
      </label>
    </template>

    <template v-else-if="selectedAction.ac === ACTION_TYPE.swipe">
      <div class="editor-inline-grid">
        <div class="editor-inline-label">滑动方式</div>
        <div class="editor-inline-content">
          <EditorSelectField
            :model-value="String(selectedAction.mode || ACTION_MODE.point)"
            :options="swipeModeOptions"
            placeholder="滑动方式"
            @update:model-value="$emit('update-mode', String($event || ACTION_MODE.point))"
          />
        </div>
        <div class="editor-inline-label">持续时间</div>
        <div class="editor-inline-content">
          <input :value="String(selectedAction.duration ?? 300)" class="app-input" type="number" @input="$emit('update-number-field', 'duration', ($event.target as HTMLInputElement).value)" />
        </div>
      </div>

      <template v-if="selectedAction.mode === ACTION_MODE.txt || selectedAction.mode === ACTION_MODE.labelIdx">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输入结果变量</span>
          <EditorSelectField
            :model-value="selectedActionInput || null"
            :options="resolvedActionInputOptions"
            :show-description="true"
            placeholder="选择 OCR / 检测 / 处理结果变量"
            test-id="editor-action-swipe-input-var"
            @update:model-value="$emit('update-field', 'input_var', String($event || ''))"
          />
        </label>
        <div class="flex flex-wrap gap-2">
          <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('create-variable', 'actionInput')">
            <AppIcon name="plus" :size="14" />
            新建结果变量
          </button>
          <button
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            :disabled="!selectedActionInputTarget || !jumpToVariable"
            @click="selectedActionInputTarget ? $emit('jump-to-variable', selectedActionInputTarget) : undefined"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位变量
          </button>
        </div>
      </template>

      <div v-if="selectedAction.mode === ACTION_MODE.point || selectedAction.mode === ACTION_MODE.percent" class="editor-inline-grid">
        <div class="editor-inline-label">起点 X</div>
        <div class="editor-inline-content">
          <input
            :value="String((selectedAction.from as { x?: number })?.x ?? '')"
            aria-label="起点 X"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'from', 'x', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="editor-inline-label">起点 Y</div>
        <div class="editor-inline-content">
          <input
            :value="String((selectedAction.from as { y?: number })?.y ?? '')"
            aria-label="起点 Y"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'from', 'y', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="editor-inline-label">终点 X</div>
        <div class="editor-inline-content">
          <input
            :value="String((selectedAction.to as { x?: number })?.x ?? '')"
            aria-label="终点 X"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'to', 'x', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="editor-inline-label">终点 Y</div>
        <div class="editor-inline-content">
          <input
            :value="String((selectedAction.to as { y?: number })?.y ?? '')"
            aria-label="终点 Y"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'to', 'y', ($event.target as HTMLInputElement).value)"
          />
        </div>
      </div>

      <div v-if="selectedAction.mode === ACTION_MODE.txt" class="grid gap-3 md:grid-cols-2">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">起点文字</span>
          <input :value="String(selectedAction.from ?? '')" class="app-input" @input="$emit('update-text-field', 'from', ($event.target as HTMLInputElement).value)" />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">终点文字</span>
          <input :value="String(selectedAction.to ?? '')" class="app-input" @input="$emit('update-text-field', 'to', ($event.target as HTMLInputElement).value)" />
        </label>
      </div>

      <div v-else-if="selectedAction.mode === ACTION_MODE.labelIdx" class="grid gap-3 md:grid-cols-2">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">起点标签</span>
          <AppSelect
            :model-value="typeof selectedAction.from === 'number' ? selectedAction.from : null"
            :options="resolvedSwipeFromLabelOptions"
            :placeholder="labelSelectPlaceholder"
            :disabled="!(labelIndexOptions?.length)"
            test-id="editor-action-swipe-label-from"
            @update:model-value="$emit('update-number-field', 'from', String($event ?? 0))"
          />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">终点标签</span>
          <AppSelect
            :model-value="typeof selectedAction.to === 'number' ? selectedAction.to : null"
            :options="resolvedSwipeToLabelOptions"
            :placeholder="labelSelectPlaceholder"
            :disabled="!(labelIndexOptions?.length)"
            test-id="editor-action-swipe-label-to"
            @update:model-value="$emit('update-number-field', 'to', String($event ?? 0))"
          />
        </label>
        <p v-if="labelSelectHint" class="md:col-span-2 text-xs leading-5 text-amber-700">{{ labelSelectHint }}</p>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import EditorVariableMetaCard from '@/views/script-editor/EditorVariableMetaCard.vue';
import type { Action } from '@/types/bindings/Action';
import { ACTION_MODE, ACTION_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';
import type { EditorInputEntry, EditorInputType, EditorVariableOption } from '@/views/script-editor/editorVariables';

defineOptions({ name: 'EditorStepActionPanel' });

const props = defineProps<{
  selectedAction: Action;
  actionExecMax: number;
  variableDatalistId: string;
  writableCatalogVariableOptions?: Array<{ label: string; value: string; description: string; disabled?: boolean }>;
  resultCatalogVariableOptions?: SelectOption[];
  labelIndexOptions?: LabelSelectOption[];
  labelSelectPlaceholder?: string;
  labelSelectHint?: string | null;
  selectedCaptureOutputTarget?: EditorVariableOption | null;
  selectedCaptureOutputInputEntry?: EditorInputEntry | null;
  selectedActionInputTarget?: EditorVariableOption | null;
  clickModeOptions: Array<{ label: string; value: string; description: string }>;
  swipeModeOptions: Array<{ label: string; value: string; description: string }>;
  createVariable?: (namespace?: 'input' | 'runtime', inputType?: EditorInputType) => Promise<string>;
  jumpToVariable?: (option: EditorVariableOption) => void;
}>();

const emit = defineEmits<{
  'update-exec-max': [value: string];
  'update-field': [field: string, value: string];
  'update-mode': [mode: string];
  'update-point-field': [field: 'p' | 'from' | 'to', axis: 'x' | 'y', value: string];
  'update-number-field': [field: string, value: string];
  'update-text-field': [field: string, value: string];
  'create-variable': [target: 'captureOutput' | 'actionInput'];
  'jump-to-variable': [option: EditorVariableOption];
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
}>();

type SelectOption = { label: string; value: string; description: string; disabled?: boolean };
type LabelSelectOption = { label: string; value: number; description?: string; disabled?: boolean };

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

const withCurrentLabelOption = (options: LabelSelectOption[], value: number | null | undefined) => {
  if (value === null || value === undefined || Number.isNaN(value)) {
    return options;
  }

  if (options.some((option) => option.value === value)) {
    return options;
  }

  return [
    {
      label: `${value}: 未找到标签`,
      value,
      description: '标签文件中不存在该索引，保存时仍会保留当前 idx。',
    },
    ...options,
  ];
};

const resolvedCaptureOutputOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.capture
    ? withCurrentVariableOption(props.writableCatalogVariableOptions ?? [], props.selectedAction.output_var ?? '')
    : props.writableCatalogVariableOptions ?? [],
);

const selectedActionInput = computed(() => {
  if (props.selectedAction.ac !== ACTION_TYPE.click && props.selectedAction.ac !== ACTION_TYPE.swipe) {
    return '';
  }

  if (props.selectedAction.mode === ACTION_MODE.txt || props.selectedAction.mode === ACTION_MODE.labelIdx) {
    return String((props.selectedAction as { input_var?: string }).input_var ?? '');
  }

  return '';
});

const resolvedActionInputOptions = computed(() =>
  withCurrentVariableOption(props.resultCatalogVariableOptions ?? [], selectedActionInput.value),
);

const resolvedLabelIdxOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.click && props.selectedAction.mode === ACTION_MODE.labelIdx
    ? withCurrentLabelOption(props.labelIndexOptions ?? [], props.selectedAction.idx ?? null)
    : props.labelIndexOptions ?? [],
);

const resolvedSwipeFromLabelOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.swipe && props.selectedAction.mode === ACTION_MODE.labelIdx
    ? withCurrentLabelOption(props.labelIndexOptions ?? [], typeof props.selectedAction.from === 'number' ? props.selectedAction.from : null)
    : props.labelIndexOptions ?? [],
);

const resolvedSwipeToLabelOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.swipe && props.selectedAction.mode === ACTION_MODE.labelIdx
    ? withCurrentLabelOption(props.labelIndexOptions ?? [], typeof props.selectedAction.to === 'number' ? props.selectedAction.to : null)
    : props.labelIndexOptions ?? [],
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
