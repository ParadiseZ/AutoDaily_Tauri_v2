<template>
  <div class="space-y-3">
    <EditorOverviewField label="统计目标" width="compact">
      <EditorSelectField
        :model-value="modelValue.type"
        :options="visionCountTargetTypeOptions"
        placeholder="选择统计目标"
        :test-id="testId ? `${testId}-type` : undefined"
        @update:model-value="updateTargetType(String($event || 'all'))"
      />
    </EditorOverviewField>

    <EditorOverviewField v-if="modelValue.type === 'detLabel'" label="YOLO 标签" width="compact">
      <div class="space-y-2">
        <EditorSelectField
          :model-value="modelValue.idx"
          :options="resolvedLabelIndexOptions"
          :placeholder="labelSelectPlaceholder"
          :disabled="!labelIndexOptions.length"
          searchable
          search-placeholder="搜索标签"
          :test-id="testId ? `${testId}-det-label-idx` : undefined"
          @update:model-value="updateDetLabel(Number($event ?? 0) || 0)"
        />
        <p v-if="labelSelectHint" class="text-xs leading-5 text-amber-700">{{ labelSelectHint }}</p>
      </div>
    </EditorOverviewField>

    <template v-else-if="modelValue.type === 'ocrText'">
      <EditorOverviewField label="OCR 文字" width="compact">
        <input
          :value="modelValue.text"
          class="app-input"
          :class="{ 'app-input-invalid': !modelValue.text.trim() }"
          placeholder="请输入要统计的文字"
          :data-testid="testId ? `${testId}-ocr-text` : undefined"
          @input="updateOcrText(($event.target as HTMLInputElement).value)"
        />
      </EditorOverviewField>

      <EditorOverviewField label="文字匹配" width="compact">
        <EditorSelectField
          :model-value="modelValue.mode"
          :options="ocrTextMatchModeOptions"
          placeholder="选择匹配方式"
          :test-id="testId ? `${testId}-ocr-mode` : undefined"
          @update:model-value="updateOcrMode(String($event || 'exact'))"
        />
      </EditorOverviewField>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { OcrTextMatchMode } from '@/types/bindings/OcrTextMatchMode';
import type { VisionCountTarget } from '@/types/bindings/VisionCountTarget';
import EditorOverviewField from '@/views/script-editor/EditorOverviewField.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import { ocrTextMatchModeOptions, visionCountTargetTypeOptions } from '@/views/script-editor/editorCondition';

const props = withDefaults(
  defineProps<{
    modelValue: VisionCountTarget;
    labelIndexOptions?: Array<{ label: string; value: number; description?: string; disabled?: boolean }>;
    labelSelectPlaceholder?: string;
    labelSelectHint?: string | null;
    testId?: string;
  }>(),
  {
    labelIndexOptions: () => [],
    labelSelectPlaceholder: '请先设置图像检测模型标签文件',
    labelSelectHint: null,
    testId: undefined,
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: VisionCountTarget];
}>();

const resolvedLabelIndexOptions = computed(() => {
  if (props.modelValue.type !== 'detLabel') return props.labelIndexOptions;
  const currentIdx = props.modelValue.idx;
  if (props.labelIndexOptions.some((option) => option.value === currentIdx)) return props.labelIndexOptions;
  return [
    {
      label: `${currentIdx}: 未找到标签`,
      value: currentIdx,
      description: '标签文件中不存在该索引。',
    },
    ...props.labelIndexOptions,
  ];
});

const updateTargetType = (type: string) => {
  if (type === 'detLabel') {
    emit('update:modelValue', { type, idx: props.labelIndexOptions[0]?.value ?? 0 });
    return;
  }
  if (type === 'ocrText') {
    emit('update:modelValue', { type, text: '', mode: 'exact' });
    return;
  }
  emit('update:modelValue', { type: 'all' });
};

const updateDetLabel = (idx: number) => emit('update:modelValue', { type: 'detLabel', idx });

const updateOcrText = (text: string) => {
  if (props.modelValue.type !== 'ocrText') return;
  emit('update:modelValue', { ...props.modelValue, text });
};

const updateOcrMode = (mode: string) => {
  if (props.modelValue.type !== 'ocrText') return;
  emit('update:modelValue', { ...props.modelValue, mode: mode as OcrTextMatchMode });
};
</script>
