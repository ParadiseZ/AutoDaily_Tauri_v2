<template>
  <div class="min-h-0 overflow-y-auto pr-1 custom-scrollbar">
    <div v-if="selectedUiField" class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
      <div class="flex items-start justify-between gap-3">
        <div>
          <p class="text-sm font-semibold text-[var(--app-text-strong)]">字段详情</p>
          <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ getUiControlLabel(selectedUiField.control) }}</p>
        </div>
        <button class="app-button app-button-danger app-toolbar-button" type="button" @click="$emit('remove-ui-field', selectedUiField.id)">
          删除字段
        </button>
      </div>

      <div class="mt-4 space-y-4">
        <section class="rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
          <div class="mb-3">
            <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">字段本体</p>
          </div>

          <div class="detail-grid">
            <label class="detail-item">
              <span class="detail-label">字段名</span>
              <input
                :value="selectedUiField.label"
                class="app-input"
                :data-testid="selectedUiFieldIndex === 0 ? 'editor-ui-field-label-0' : undefined"
                @input="$emit('update-ui-field', selectedUiField.id, 'label', ($event.target as HTMLInputElement).value)"
              />
            </label>

            <div v-if="selectedUiField.control === 'checkbox'" class="detail-item">
              <div class="detail-label">显示样式</div>
              <div>
                <EditorSelectField
                  :model-value="selectedUiField.checkboxStyle"
                  :options="checkboxStyleOptions"
                  placeholder="选择样式"
                  @update:model-value="$emit('update-ui-field', selectedUiField.id, 'checkboxStyle', String($event))"
                />
              </div>
            </div>

            <label
              v-if="selectedUiField.control === 'text'"
              class="detail-item detail-span-2"
            >
              <span class="detail-label">可编辑</span>
              <span class="flex min-h-[44px] items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3 text-sm text-[var(--app-text-soft)]">
                <input
                  :checked="selectedUiField.editable"
                  type="checkbox"
                  class="h-4 w-4"
                  style="accent-color: var(--app-accent)"
                  @change="$emit('update-ui-field', selectedUiField.id, 'editable', ($event.target as HTMLInputElement).checked)"
                />
                <span>文本变量允许用户在设置页直接编辑</span>
              </span>
            </label>

          </div>
        </section>

        <section class="rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
          <div class="mb-3">
            <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">变量绑定</p>
          </div>

          <div class="detail-grid">
            <div class="detail-item detail-span-2">
              <div class="detail-label">绑定变量</div>
              <div>
                <EditorSelectField
                  :model-value="selectedUiField.variableId || null"
                  :options="bindOptions"
                  placeholder="未绑定"
                  :test-id="selectedUiFieldIndex === 0 ? 'editor-ui-field-bind-0' : undefined"
                  @update:model-value="selectUiBinding(selectedUiField.id, String($event ?? ''))"
                />
              </div>
            </div>

            <div
              v-if="selectedBoundUiVariable"
              class="detail-span-2 rounded-[14px] border border-[var(--app-border)] bg-white/50 px-4 py-4"
            >
              <div class="detail-grid">
                <div class="detail-item">
                  <div class="detail-label">作用域</div>
                  <div class="detail-value">{{ selectedBoundVariableNamespaceLabel }}</div>
                </div>
                <div class="detail-item">
                  <div class="detail-label">类型</div>
                  <div class="detail-value">{{ selectedBoundVariableTypeLabel }}</div>
                </div>
                <div class="detail-item detail-span-2">
                  <div class="detail-label">变量键</div>
                  <div class="detail-value break-all">{{ selectedBoundVariableKeyLabel }}</div>
                </div>
              </div>
              <p
                v-if="selectedBoundUiVariable.description"
                class="mt-3 text-sm leading-6 text-[var(--app-text-soft)]"
              >
                {{ selectedBoundUiVariable.description }}
              </p>
            </div>
          </div>
        </section>

        <section class="rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
          <div class="mb-3">
            <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">展示内容</p>
          </div>

          <div class="detail-grid">
            <label class="detail-item">
              <span class="detail-label">说明</span>
              <input
                :value="selectedUiField.description"
                class="app-input"
                @input="$emit('update-ui-field', selectedUiField.id, 'description', ($event.target as HTMLInputElement).value)"
              />
            </label>

            <label v-if="selectedUiField.control === 'text' || selectedUiField.control === 'number'" class="detail-item">
              <span class="detail-label">占位提示</span>
              <input
                :value="selectedUiField.placeholder"
                class="app-input"
                @input="$emit('update-ui-field', selectedUiField.id, 'placeholder', ($event.target as HTMLInputElement).value)"
              />
            </label>

            <label v-if="selectedUiField.control === 'radio' || selectedUiField.control === 'select'" class="detail-item detail-item-top detail-span-2">
              <span class="detail-label">选项</span>
              <textarea
                :value="selectedUiField.optionsText"
                class="app-textarea min-h-[100px]"
                placeholder="每行一个选项"
                @input="$emit('update-ui-field', selectedUiField.id, 'optionsText', ($event.target as HTMLTextAreaElement).value)"
              />
            </label>

            <div
              v-if="selectedUiField.control === 'slider' && sliderValueType"
              class="detail-span-2 rounded-[14px] border border-[var(--app-border)] bg-white/45 px-4 py-4"
            >
              <div class="mb-3 flex items-center justify-between gap-3">
                <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">滑块范围</p>
                <span class="text-xs text-[var(--app-text-soft)]">{{ sliderValueType === 'float' ? '浮点变量' : '整数变量' }}</span>
              </div>

              <div class="detail-grid">
                <label class="detail-item">
                  <span class="detail-label">最小值</span>
                  <input
                    :value="sliderDraft.min"
                    class="app-input"
                    type="number"
                    :step="sliderInputStep"
                    @input="updateSliderDraft('min', ($event.target as HTMLInputElement).value)"
                    @blur="commitSliderDraft('min')"
                  />
                </label>
                <label class="detail-item">
                  <span class="detail-label">最大值</span>
                  <input
                    :value="sliderDraft.max"
                    class="app-input"
                    type="number"
                    :step="sliderInputStep"
                    @input="updateSliderDraft('max', ($event.target as HTMLInputElement).value)"
                    @blur="commitSliderDraft('max')"
                  />
                </label>
                <label class="detail-item">
                  <span class="detail-label">步长</span>
                  <input
                    :value="sliderDraft.step"
                    class="app-input"
                    type="number"
                    :step="sliderInputStep"
                    @input="updateSliderDraft('step', ($event.target as HTMLInputElement).value)"
                    @blur="commitSliderDraft('step')"
                  />
                </label>
              </div>
            </div>

            <div
              v-else-if="selectedUiField.control === 'slider'"
              class="detail-span-2 rounded-[14px] border border-dashed border-[var(--app-border)] bg-white/35 px-4 py-4 text-sm leading-6 text-[var(--app-text-soft)]"
            >
              请绑定变量。滑块只支持绑定整数或浮点变量，绑定后再设置最小值、最大值和步长。
            </div>
          </div>
        </section>
      </div>
    </div>

    <EmptyState
      v-else
      title="选择一个字段"
      description="点击中间字段列表或上方预览项，下面会切换到当前字段的可编辑内容。"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, watch } from 'vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import { getUiControlLabel, type EditorUiField } from '@/views/script-editor/editorSchema';
import { getVariableDisplayKey, getVariableValueTypeLabel, type EditorVariableOption } from '@/views/script-editor/editorVariables';
import { buildUiBindOptions } from '@/views/script-editor/editorUiPreview';

defineOptions({ name: 'EditorUiFieldDetailsPanel' });

const props = defineProps<{
  selectedUiField: EditorUiField | null;
  selectedUiFieldIndex: number;
  variableOptions: EditorVariableOption[];
}>();

const emit = defineEmits<{
  'update-ui-field': [fieldId: string, field: 'label' | 'key' | 'editable' | 'checkboxStyle' | 'variableId' | 'inputKey' | 'description' | 'placeholder' | 'optionsText' | 'min' | 'max' | 'step' | 'numericMode', value: string | boolean];
  'remove-ui-field': [fieldId: string];
}>();

const checkboxStyleOptions = [
  { label: 'Checkbox', value: 'checkbox', description: '传统勾选框。' },
  { label: 'Switch', value: 'switch', description: '开关样式。' },
];

const bindOptions = computed(() => {
  const options = props.selectedUiField?.control === 'slider'
    ? props.variableOptions.filter((item) => item.valueType === 'int' || item.valueType === 'float')
    : props.variableOptions;
  return buildUiBindOptions(options);
});
const selectedBoundUiVariable = computed(() => {
  if (!props.selectedUiField?.variableId) {
    return null;
  }

  return props.variableOptions.find((item) => item.id === props.selectedUiField?.variableId) ?? null;
});
const selectedBoundVariableNamespaceLabel = computed(() => {
  if (!selectedBoundUiVariable.value) {
    return '';
  }
  if (selectedBoundUiVariable.value.namespace === 'runtime') return 'Runtime';
  if (selectedBoundUiVariable.value.namespace === 'system') return 'System';
  return 'Input';
});
const selectedBoundVariableTypeLabel = computed(() =>
  selectedBoundUiVariable.value ? getVariableValueTypeLabel(selectedBoundUiVariable.value.valueType) : '',
);
const selectedBoundVariableKeyLabel = computed(() =>
  selectedBoundUiVariable.value ? getVariableDisplayKey(selectedBoundUiVariable.value.key, selectedBoundUiVariable.value.namespace) : '',
);
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
.detail-grid {
  display: grid;
  gap: 0.9rem 1rem;
}

.detail-item {
  display: grid;
  gap: 0.75rem;
}

.detail-label {
  display: flex;
  align-items: center;
  min-height: 44px;
  color: var(--app-text-faint);
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.detail-value {
  min-height: 44px;
  display: flex;
  align-items: center;
  color: var(--app-text-strong);
  font-size: 0.92rem;
  font-weight: 600;
}

@media (min-width: 768px) {
  .detail-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .detail-item {
    grid-template-columns: 78px minmax(0, 1fr);
    align-items: center;
  }

  .detail-item-top {
    align-items: start;
  }

  .detail-span-2 {
    grid-column: 1 / -1;
  }
}
</style>
