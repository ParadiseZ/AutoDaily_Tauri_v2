<template>
  <div v-if="selectedUiField" class="space-y-2">
    <EditorOverviewSection
        title="字段详情"
        heading-tag="h1"
        width="wide"
    >
      <EditorOverviewField label="字段名" width="compact">
        <input
            :value="selectedUiField.label"
            class="app-input"
            :data-testid="selectedUiFieldIndex === 0 ? 'editor-ui-field-label-0' : undefined"
            @input="$emit('update-ui-field', selectedUiField.id, 'label', ($event.target as HTMLInputElement).value)"
        />
      </EditorOverviewField>

      <EditorOverviewField v-if="selectedUiField.control === 'checkbox'" label="显示样式" width="compact">
        <EditorSelectField
            :model-value="selectedUiField.checkboxStyle"
            :options="checkboxStyleOptions"
            placeholder="选择样式"
            @update:model-value="$emit('update-ui-field', selectedUiField.id, 'checkboxStyle', String($event))"
        />
      </EditorOverviewField>

      <EditorOverviewField v-if="selectedUiField.control === 'text'" label="可编辑" width="compact">
          <span class="editor-detail-toggle">
            <input
                :checked="selectedUiField.editable"
                type="checkbox"
                class="h-4 w-4"
                style="accent-color: var(--app-accent)"
                @change="$emit('update-ui-field', selectedUiField.id, 'editable', ($event.target as HTMLInputElement).checked)"
            />
            <span>允许用户编辑</span>
          </span>
      </EditorOverviewField>
    </EditorOverviewSection>

    <EditorOverviewSection title="变量绑定">
      <EditorVariableBindingField
        label="绑定变量"
        :model-value="selectedUiField.variableId || null"
        :options="bindOptions"
        placeholder="未绑定"
        :test-id="selectedUiFieldIndex >= 0 ? `editor-ui-field-bind-${selectedUiFieldIndex}` : undefined"
        :show-locate="Boolean(selectedBoundUiVariable && jumpToVariable)"
        :locate-disabled="!selectedBoundUiVariable || !jumpToVariable"
        @update:model-value="selectUiBinding(selectedUiField.id, String($event ?? ''))"
        @locate="selectedBoundUiVariable ? jumpToVariable?.(selectedBoundUiVariable) : undefined"
      />
    </EditorOverviewSection>

    <EditorOverviewSection title="可选值设置">

      <EditorOverviewField v-if="selectedUiField.control === 'text' || selectedUiField.control === 'number'" label="占位提示" width="compact">
        <input
            :value="selectedUiField.placeholder"
            class="app-input"
            @input="$emit('update-ui-field', selectedUiField.id, 'placeholder', ($event.target as HTMLInputElement).value)"
        />
      </EditorOverviewField>

      <EditorOverviewField v-if="selectedUiField.control === 'radio' || selectedUiField.control === 'select'" label="选项(每行一个)">
          <textarea
              :value="selectedUiField.optionsText"
              class="app-textarea min-h-[100px] max-w-[38rem]"
              placeholder="请输入选项内容"
              @input="$emit('update-ui-field', selectedUiField.id, 'optionsText', ($event.target as HTMLTextAreaElement).value)"
          />
      </EditorOverviewField>

      <div
          v-if="selectedUiField.control === 'slider' && sliderValueType"
          class="editor-detail-note"
      >
        <div class="mb-3 flex items-center justify-between gap-3">
          <p class="text-[11px] uppercase tracking-[0.12em] text-(--app-text-faint)">滑块范围</p>
          <span class="text-xs text-(--app-text-soft)">{{ sliderValueType === 'float' ? '浮点变量' : '整数变量' }}</span>
        </div>

        <div class="editor-detail-summary-grid">
          <EditorOverviewField label="最小值" width="compact">
            <input
                :value="sliderDraft.min"
                class="app-input"
                type="number"
                :step="sliderInputStep"
                @input="updateSliderDraft('min', ($event.target as HTMLInputElement).value)"
                @blur="commitSliderDraft('min')"
            />
          </EditorOverviewField>
          <EditorOverviewField label="最大值" width="compact">
            <input
                :value="sliderDraft.max"
                class="app-input"
                type="number"
                :step="sliderInputStep"
                @input="updateSliderDraft('max', ($event.target as HTMLInputElement).value)"
                @blur="commitSliderDraft('max')"
            />
          </EditorOverviewField>
          <EditorOverviewField label="步长" width="compact">
            <input
                :value="sliderDraft.step"
                class="app-input"
                type="number"
                :step="sliderInputStep"
                @input="updateSliderDraft('step', ($event.target as HTMLInputElement).value)"
                @blur="commitSliderDraft('step')"
            />
          </EditorOverviewField>
        </div>
      </div>
      <div
          v-else-if="selectedUiField.control === 'slider'"
          class="editor-detail-note border-dashed"
      >
        请绑定变量。滑块只支持绑定整数或浮点变量，绑定后再设置最小值、最大值和步长。
      </div>

      <EditorOverviewField label="说明" width="compact">
        <input
            :value="selectedUiField.description"
            class="app-input"
            @input="$emit('update-ui-field', selectedUiField.id, 'description', ($event.target as HTMLInputElement).value)"
        />
      </EditorOverviewField>
    </EditorOverviewSection>
  </div>
  <EmptyState
      v-else
      title="选择一个字段元素后编辑"
  />
</template>

<script setup lang="ts">
import { computed, reactive, watch } from 'vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import EditorOverviewField from '@/views/script-editor/EditorOverviewField.vue';
import EditorOverviewSection from '@/views/script-editor/EditorOverviewSection.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import EditorVariableBindingField from '@/views/script-editor/EditorVariableBindingField.vue';
import { type EditorUiField } from '@/views/script-editor/editorSchema';
import { type EditorVariableOption } from '@/views/script-editor/editorVariables';
import { buildUiBindOptions } from '@/views/script-editor/editorUiPreview';

defineOptions({ name: 'EditorUiFieldDetailsPanel' });

const props = defineProps<{
  selectedUiField: EditorUiField | null;
  selectedUiFieldIndex: number;
  variableOptions: EditorVariableOption[];
  jumpToVariable?: (option: EditorVariableOption) => void;
}>();

const emit = defineEmits<{
  'update-ui-field': [fieldId: string, field: 'label' | 'key' | 'editable' | 'checkboxStyle' | 'variableId' | 'inputKey' | 'description' | 'placeholder' | 'optionsText' | 'min' | 'max' | 'step' | 'numericMode', value: string | boolean];
}>();

const checkboxStyleOptions = [
  { label: 'Checkbox', value: 'checkbox', description: '传统勾选框。' },
  { label: 'Switch', value: 'switch', description: '开关样式。' },
];

const bindOptions = computed(() => {
  const control = props.selectedUiField?.control;
  const isPointControl = control === 'point' || control === 'percentPoint';
  const options = control === 'slider'
    ? props.variableOptions.filter((item) => item.valueType === 'int' || item.valueType === 'float')
    : isPointControl
      ? props.variableOptions.filter(
          (item) => item.namespace === 'input' && (item.valueType === 'json' || item.valueType === 'object'),
        )
      : props.variableOptions;
  return buildUiBindOptions(options, isPointControl).map((option) => ({
    ...option,
    value: option.value ?? '',
  }));
});
const selectedBoundUiVariable = computed(() => {
  if (!props.selectedUiField?.variableId) {
    return null;
  }

  return props.variableOptions.find((item) => item.id === props.selectedUiField?.variableId) ?? null;
});
const sliderValueType = computed(() => {
  if (props.selectedUiField?.control !== 'slider') {
    return null;
  }

  if (selectedBoundUiVariable.value?.valueType === 'float') {
    return 'float' as const;
  }

  if (selectedBoundUiVariable.value?.valueType === 'int') {
    return 'int' as const;
  }

  return null;
});
const sliderInputStep = computed(() => (sliderValueType.value === 'float' ? '0.01' : '1'));

const sliderDraft = reactive({
  min: '',
  max: '',
  step: '',
});

watch(
  () =>
    props.selectedUiField
      ? `${props.selectedUiField.id}:${sliderValueType.value}:${props.selectedUiField.min}:${props.selectedUiField.max}:${props.selectedUiField.step}`
      : '',
  () => {
    sliderDraft.min = props.selectedUiField ? String(props.selectedUiField.min) : '';
    sliderDraft.max = props.selectedUiField ? String(props.selectedUiField.max) : '';
    sliderDraft.step = props.selectedUiField ? String(props.selectedUiField.step) : '';
  },
  { immediate: true },
);

const updateSliderDraft = (field: 'min' | 'max' | 'step', value: string) => {
  sliderDraft[field] = value;
};

const commitSliderDraft = (field: 'min' | 'max' | 'step') => {
  if (!props.selectedUiField) {
    return;
  }
  emit('update-ui-field', props.selectedUiField.id, field, sliderDraft[field]);
};

const selectUiBinding = (fieldId: string, variableId: string) => {
  const matched = props.variableOptions.find((item) => item.id === variableId) ?? null;
  emit('update-ui-field', fieldId, 'variableId', variableId);
  emit('update-ui-field', fieldId, 'inputKey', matched?.key.startsWith('input.') ? matched.key.slice('input.'.length) : '');
};
</script>

<style scoped>
@reference "../../style.css";

.editor-detail-value {
  @apply flex min-h-[44px] items-center text-[0.92rem] font-semibold text-(--app-text-strong);
}

.editor-detail-toggle {
  @apply flex min-h-[44px] items-center gap-3 rounded-[16px] border border-(--app-border) px-4 py-3 text-sm text-(--app-text-soft);
}

.editor-detail-summary-grid {
  @apply flex flex-col gap-4;
}

.editor-detail-note {
  @apply max-w-[38rem] rounded-[14px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4 text-sm leading-6 text-(--app-text-soft);
}

.app-textarea {
  @apply h-50;
}
</style>
