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

          <div class="grid gap-3">
            <label class="space-y-2">
              <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">字段名</span>
              <input
                :value="selectedUiField.label"
                class="app-input"
                :data-testid="selectedUiFieldIndex === 0 ? 'editor-ui-field-label-0' : undefined"
                @input="$emit('update-ui-field', selectedUiField.id, 'label', ($event.target as HTMLInputElement).value)"
              />
            </label>

            <label v-if="selectedUiField.control === 'checkbox'" class="space-y-2">
              <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">显示样式</span>
              <AppSelect
                :model-value="selectedUiField.checkboxStyle"
                :options="checkboxStyleOptions"
                placeholder="选择样式"
                @update:model-value="$emit('update-ui-field', selectedUiField.id, 'checkboxStyle', String($event))"
              />
            </label>

            <label
              v-if="selectedUiField.control === 'text'"
              class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3"
            >
              <input
                :checked="selectedUiField.editable"
                type="checkbox"
                class="h-4 w-4"
                style="accent-color: var(--app-accent)"
                @change="$emit('update-ui-field', selectedUiField.id, 'editable', ($event.target as HTMLInputElement).checked)"
              />
              <span class="text-sm text-[var(--app-text-soft)]">文本变量允许用户在设置页直接编辑</span>
            </label>
          </div>
        </section>

        <section class="rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
          <div class="mb-3">
            <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">变量绑定</p>
          </div>

          <div class="grid gap-3">
            <label class="space-y-2">
              <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">绑定变量</span>
              <AppSelect
                :model-value="selectedUiField.variableId || null"
                :options="bindOptions"
                placeholder="未绑定"
                :test-id="selectedUiFieldIndex === 0 ? 'editor-ui-field-bind-0' : undefined"
                @update:model-value="selectUiBinding(selectedUiField.id, String($event ?? ''))"
              />
            </label>

            <div
              v-if="selectedBoundUiVariable"
              class="rounded-[14px] border border-[var(--app-border)] bg-white/50 px-4 py-4"
            >
              <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                <div class="space-y-1">
                  <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">作用域</p>
                  <p class="text-sm font-medium text-[var(--app-text-strong)]">{{ selectedBoundUiVariable.namespace }}</p>
                </div>
                <div class="space-y-1">
                  <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">类型</p>
                  <p class="text-sm font-medium text-[var(--app-text-strong)]">{{ selectedBoundUiVariable.valueType }}</p>
                </div>
                <div class="space-y-1 md:col-span-2">
                  <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">变量键</p>
                  <p class="text-sm font-medium text-[var(--app-text-strong)] break-all">{{ selectedBoundUiVariable.key }}</p>
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

          <div class="grid gap-3">
            <label class="space-y-2">
              <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">说明</span>
              <input
                :value="selectedUiField.description"
                class="app-input"
                @input="$emit('update-ui-field', selectedUiField.id, 'description', ($event.target as HTMLInputElement).value)"
              />
            </label>

            <label v-if="selectedUiField.control === 'text' || selectedUiField.control === 'number'" class="space-y-2">
              <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">占位提示</span>
              <input
                :value="selectedUiField.placeholder"
                class="app-input"
                @input="$emit('update-ui-field', selectedUiField.id, 'placeholder', ($event.target as HTMLInputElement).value)"
              />
            </label>

            <label v-if="selectedUiField.control === 'radio' || selectedUiField.control === 'select'" class="space-y-2">
              <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">选项</span>
              <textarea
                :value="selectedUiField.optionsText"
                class="app-textarea min-h-[100px]"
                placeholder="每行一个选项"
                @input="$emit('update-ui-field', selectedUiField.id, 'optionsText', ($event.target as HTMLTextAreaElement).value)"
              />
            </label>
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
import { computed } from 'vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import { getUiControlLabel, type EditorUiField } from '@/views/script-editor/editorSchema';
import type { EditorVariableOption } from '@/views/script-editor/editorVariables';
import { buildUiBindOptions } from '@/views/script-editor/editorUiPreview';

defineOptions({ name: 'EditorUiFieldDetailsPanel' });

const props = defineProps<{
  selectedUiField: EditorUiField | null;
  selectedUiFieldIndex: number;
  variableOptions: EditorVariableOption[];
}>();

const emit = defineEmits<{
  'update-ui-field': [fieldId: string, field: 'label' | 'key' | 'editable' | 'checkboxStyle' | 'variableId' | 'inputKey' | 'description' | 'placeholder' | 'optionsText', value: string | boolean];
  'remove-ui-field': [fieldId: string];
}>();

const checkboxStyleOptions = [
  { label: 'Checkbox', value: 'checkbox', description: '传统勾选框。' },
  { label: 'Switch', value: 'switch', description: '开关样式。' },
];

const bindOptions = computed(() => buildUiBindOptions(props.variableOptions));
const selectedBoundUiVariable = computed(() => {
  if (!props.selectedUiField?.variableId) {
    return null;
  }

  return props.variableOptions.find((item) => item.id === props.selectedUiField?.variableId) ?? null;
});

const selectUiBinding = (fieldId: string, variableId: string) => {
  const matched = props.variableOptions.find((item) => item.id === variableId) ?? null;
  emit('update-ui-field', fieldId, 'variableId', variableId);
  emit('update-ui-field', fieldId, 'inputKey', matched?.key.startsWith('input.') ? matched.key.slice('input.'.length) : '');
};
</script>
