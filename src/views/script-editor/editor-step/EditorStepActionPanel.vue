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

    <template v-else-if="selectedAction.ac === ACTION_TYPE.posAdd || selectedAction.ac === ACTION_TYPE.posMinus">
      <div class="space-y-3">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标策略</span>
          <EditorSelectField
            :model-value="selectedAction.target || null"
            :options="resolvedPolicyTargetOptions"
            :show-description="true"
            placeholder="选择要调整当前位置的策略"
            test-id="editor-action-policy-position-target"
            @update:model-value="$emit('update-field', 'target', String($event || ''))"
          />
        </label>
        <div class="flex flex-wrap gap-2">
          <button
            v-if="createPolicy"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            @click="$emit('create-policy-target')"
          >
            <AppIcon name="plus" :size="14" />
            新建策略
          </button>
          <button
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            :disabled="!selectedAction.target || !jumpToPolicy"
            @click="selectedAction.target ? $emit('jump-policy-target', selectedAction.target) : undefined"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位策略
          </button>
        </div>
        <p class="text-xs leading-5 text-[var(--app-text-faint)]">
          只调整本次运行中的点击索引，不写回策略配置；策略内文字/标签点击会用该索引选择第 N 个匹配目标。
        </p>
      </div>
    </template>

    <template v-else-if="selectedAction.ac === ACTION_TYPE.dropSetNext">
      <div class="space-y-3">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标任务</span>
          <EditorSelectField
            :model-value="selectedAction.task || null"
            :options="resolvedTaskTargetOptions"
            :show-description="true"
            placeholder="选择要切换 UI 变量的任务"
            test-id="editor-action-drop-set-task"
            @update:model-value="selectDropSetTask(String($event || ''))"
          />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">UI 变量</span>
          <EditorSelectField
            :model-value="selectedAction.variable_id || null"
            :options="resolvedDropSetVariableOptions"
            :show-description="true"
            placeholder="选择 Select / Radio 绑定变量"
            test-id="editor-action-drop-set-variable"
            @update:model-value="$emit('update-field', 'variable_id', String($event || ''))"
          />
        </label>
        <div class="flex flex-wrap gap-2">
          <button
            v-if="createTask"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            @click="$emit('create-drop-set-task')"
          >
            <AppIcon name="plus" :size="14" />
            新建任务
          </button>
          <button
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            :disabled="!selectedAction.task || !jumpToTask"
            @click="selectedAction.task ? $emit('jump-drop-set-task', selectedAction.task) : undefined"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位任务
          </button>
        </div>
        <p class="text-xs leading-5 text-[var(--app-text-faint)]">
          执行时把该变量切到配置选项里的下一个值，并写回当前设备/时间模板作用域。
        </p>
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

      <div v-if="selectedAction.mode === ACTION_MODE.txt" class="grid gap-3 md:grid-cols-2">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标文字</span>
          <input :value="String(selectedAction.txt ?? '')" class="app-input" @input="$emit('update-text-field', 'txt', ($event.target as HTMLInputElement).value)" />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">绑定变量</span>
          <EditorSelectField
            :model-value="selectedAction.txt_expr || null"
            :options="resolvedClickTextVariableOptions"
            :show-description="true"
            placeholder="绑定文字变量"
            test-id="editor-action-click-text-var"
            @update:model-value="$emit('update-text-field', 'txt_expr', String($event || ''))"
          />
        </label>
        <button
          v-if="createVariable"
          class="app-button app-button-ghost app-toolbar-button md:col-start-2"
          type="button"
          @click="$emit('create-variable', 'clickText')"
        >
          <AppIcon name="plus" :size="14" />
          新建文字变量
        </button>
      </div>

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
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">固定起点文字</span>
          <input :value="String(selectedAction.from ?? '')" class="app-input" @input="$emit('update-text-field', 'from', ($event.target as HTMLInputElement).value)" />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">固定终点文字</span>
          <input :value="String(selectedAction.to ?? '')" class="app-input" @input="$emit('update-text-field', 'to', ($event.target as HTMLInputElement).value)" />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">起点文字变量</span>
          <EditorSelectField
            :model-value="selectedAction.from_expr || null"
            :options="resolvedSwipeFromTextVariableOptions"
            :show-description="true"
            placeholder="绑定文字变量"
            test-id="editor-action-swipe-from-text-var"
            @update:model-value="$emit('update-text-field', 'from_expr', String($event || ''))"
          />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">终点文字变量</span>
          <EditorSelectField
            :model-value="selectedAction.to_expr || null"
            :options="resolvedSwipeToTextVariableOptions"
            :show-description="true"
            placeholder="绑定文字变量"
            test-id="editor-action-swipe-to-text-var"
            @update:model-value="$emit('update-text-field', 'to_expr', String($event || ''))"
          />
        </label>
        <div v-if="createVariable" class="flex flex-wrap gap-2 md:col-span-2">
          <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('create-variable', 'swipeFromText')">
            <AppIcon name="plus" :size="14" />
            新建起点文字变量
          </button>
          <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('create-variable', 'swipeToText')">
            <AppIcon name="plus" :size="14" />
            新建终点文字变量
          </button>
        </div>
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
import type { EditorReferenceOption, EditorTaskUiVariableOption } from '@/views/script-editor/editorReferences';
import type { EditorInputEntry, EditorInputType, EditorVariableOption } from '@/views/script-editor/editorVariables';

defineOptions({ name: 'EditorStepActionPanel' });

const props = defineProps<{
  selectedAction: Action;
  actionExecMax: number;
  variableDatalistId: string;
  writableCatalogVariableOptions?: Array<{ label: string; value: string; description: string; disabled?: boolean }>;
  resultCatalogVariableOptions?: SelectOption[];
  textVariableOptions?: SelectOption[];
  labelIndexOptions?: LabelSelectOption[];
  labelSelectPlaceholder?: string;
  labelSelectHint?: string | null;
  selectedCaptureOutputTarget?: EditorVariableOption | null;
  selectedCaptureOutputInputEntry?: EditorInputEntry | null;
  selectedActionInputTarget?: EditorVariableOption | null;
  policyReferenceOptions?: EditorReferenceOption[];
  taskReferenceOptions?: EditorReferenceOption[];
  taskUiVariableOptions?: EditorTaskUiVariableOption[];
  clickModeOptions: Array<{ label: string; value: string; description: string }>;
  swipeModeOptions: Array<{ label: string; value: string; description: string }>;
  createVariable?: (namespace?: 'input' | 'runtime', inputType?: EditorInputType, options?: { preferredKey?: string; name?: string; select?: boolean; silent?: boolean }) => Promise<string>;
  jumpToVariable?: (option: EditorVariableOption) => void;
  createPolicy?: () => Promise<string>;
  jumpToPolicy?: (id: string) => void;
  createTask?: () => Promise<string>;
  jumpToTask?: (id: string) => void;
}>();

const emit = defineEmits<{
  'update-exec-max': [value: string];
  'update-field': [field: string, value: string];
  'update-mode': [mode: string];
  'update-point-field': [field: 'p' | 'from' | 'to', axis: 'x' | 'y', value: string];
  'update-number-field': [field: string, value: string];
  'update-text-field': [field: string, value: string];
  'create-variable': [target: 'captureOutput' | 'actionInput' | 'clickText' | 'swipeFromText' | 'swipeToText'];
  'jump-to-variable': [option: EditorVariableOption];
  'create-policy-target': [];
  'jump-policy-target': [id: string];
  'create-drop-set-task': [];
  'jump-drop-set-task': [id: string];
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
}>();

type SelectOption = { label: string; value: string; description?: string; disabled?: boolean };
type LabelSelectOption = { label: string; value: number; description?: string; disabled?: boolean };

const withCurrentVariableOption = (options: SelectOption[], value: string) => {
  const trimmedValue = value.trim();
  if (!trimmedValue || options.some((option) => option.value === trimmedValue)) {
    return options;
  }

  return [
    {
      label: `当前绑定不存在：${trimmedValue}`,
      value: trimmedValue,
      description: '变量目录里找不到该绑定，保存时仍会保留当前值。',
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

const resolvedPolicyTargetOptions = computed(() => {
  const options = props.policyReferenceOptions ?? [];
  if (props.selectedAction.ac !== ACTION_TYPE.posAdd && props.selectedAction.ac !== ACTION_TYPE.posMinus) {
    return options;
  }
  const target = props.selectedAction.target?.trim() ?? '';
  if (!target || options.some((option) => option.value === target)) {
    return options;
  }
  return [
    {
      label: target,
      value: target,
      description: '未解析策略',
    },
    ...options,
  ];
});

const resolvedTaskTargetOptions = computed(() => {
  const options = props.taskReferenceOptions ?? [];
  if (props.selectedAction.ac !== ACTION_TYPE.dropSetNext) {
    return options;
  }
  const taskId = props.selectedAction.task?.trim() ?? '';
  if (!taskId || options.some((option) => option.value === taskId)) {
    return options;
  }
  return [
    {
      label: taskId,
      value: taskId,
      description: '未解析任务',
    },
    ...options,
  ];
});

const dropSetVariableOptions = computed(() => {
  if (props.selectedAction.ac !== ACTION_TYPE.dropSetNext) {
    return [];
  }
  const taskId = props.selectedAction.task?.trim() ?? '';
  return (props.taskUiVariableOptions ?? [])
    .filter((option) => !taskId || option.taskId === taskId)
    .map((option) => ({
      label: option.label,
      value: option.variableId,
      description: option.description ?? `${option.taskLabel} · ${option.options.length} 个选项`,
    }));
});

const resolvedDropSetVariableOptions = computed(() => {
  const options = dropSetVariableOptions.value;
  if (props.selectedAction.ac !== ACTION_TYPE.dropSetNext) {
    return options;
  }
  const variableId = props.selectedAction.variable_id?.trim() ?? '';
  if (!variableId || options.some((option) => option.value === variableId)) {
    return options;
  }
  return [
    {
      label: variableId,
      value: variableId,
      description: '未解析 UI 变量',
    },
    ...options,
  ];
});

const selectDropSetTask = (taskId: string) => {
  emit('update-field', 'task', taskId);
  if (props.selectedAction.ac !== ACTION_TYPE.dropSetNext) {
    return;
  }
  const currentVariableId = props.selectedAction.variable_id?.trim() ?? '';
  const nextOptions = (props.taskUiVariableOptions ?? []).filter((option) => option.taskId === taskId);
  if (!nextOptions.some((option) => option.variableId === currentVariableId)) {
    emit('update-field', 'variable_id', nextOptions[0]?.variableId ?? '');
  }
};

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
const resolvedClickTextVariableOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.click && props.selectedAction.mode === ACTION_MODE.txt
    ? withCurrentVariableOption(props.textVariableOptions ?? [], props.selectedAction.txt_expr ?? '')
    : props.textVariableOptions ?? [],
);
const resolvedSwipeFromTextVariableOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.swipe && props.selectedAction.mode === ACTION_MODE.txt
    ? withCurrentVariableOption(props.textVariableOptions ?? [], props.selectedAction.from_expr ?? '')
    : props.textVariableOptions ?? [],
);
const resolvedSwipeToTextVariableOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.swipe && props.selectedAction.mode === ACTION_MODE.txt
    ? withCurrentVariableOption(props.textVariableOptions ?? [], props.selectedAction.to_expr ?? '')
    : props.textVariableOptions ?? [],
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
