<template>
  <div
    v-if="useRadioGroup"
    class="editor-select-group"
    :class="{ 'editor-select-group-disabled': disabled }"
    :data-testid="testId"
    role="radiogroup"
  >
    <button
      v-for="option in options"
      :key="String(option.value)"
      class="editor-select-chip"
      :class="{
        'editor-select-chip-active': isSelected(option.value),
        'editor-select-chip-disabled': disabled || option.disabled,
      }"
      type="button"
      role="radio"
      :aria-checked="isSelected(option.value)"
      :data-testid="testId ? `${testId}-option-${String(option.value)}` : undefined"
      :disabled="disabled || option.disabled"
      @click="selectOption(option.value)"
    >
      <span class="editor-select-chip-main">
        <span class="editor-select-chip-radio" :class="{ 'editor-select-chip-radio-active': isSelected(option.value) }" />
        <span class="editor-select-chip-copy">
          <span class="editor-select-chip-label">{{ option.label }}</span>
          <span v-if="showDescription && option.description" class="editor-select-chip-description">{{ option.description }}</span>
        </span>
      </span>
    </button>
  </div>

  <AppSelect
    v-else
    :model-value="modelValue"
    :options="options"
    :placeholder="placeholder"
    :disabled="disabled"
    :align="align"
    :show-description="showDescription"
    :test-id="testId"
    @update:model-value="$emit('update:modelValue', $event)"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';
import AppSelect from '@/components/shared/AppSelect.vue';

type SelectValue = string | number | null;

interface SelectOption {
  label: string;
  value: SelectValue;
  description?: string;
  disabled?: boolean;
}

const props = withDefaults(
  defineProps<{
    modelValue: SelectValue;
    options: SelectOption[];
    placeholder?: string;
    disabled?: boolean;
    align?: 'left' | 'right';
    showDescription?: boolean;
    testId?: string;
  }>(),
  {
    placeholder: '请选择',
    disabled: false,
    align: 'left',
    showDescription: false,
    testId: undefined,
  },
);

const emit = defineEmits<{
  'update:modelValue': [value: SelectValue];
}>();

const useRadioGroup = computed(() => props.options.length > 0 && props.options.length <= 3);

const isSelected = (value: SelectValue) => String(value) === String(props.modelValue);

const selectOption = (value: SelectValue) => {
  if (props.disabled) {
    return;
  }
  emit('update:modelValue', value);
};
</script>

<style scoped>
.editor-select-group {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.editor-select-group-disabled {
  opacity: 0.68;
}

.editor-select-chip {
  display: inline-flex;
  min-width: 88px;
  flex: 1 1 0;
  align-items: center;
  border-radius: 14px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.72);
  padding: 0.65rem 0.8rem;
  text-align: left;
  transition: border-color 0.16s ease, background 0.16s ease, box-shadow 0.16s ease;
}

.editor-select-chip:hover {
  border-color: color-mix(in srgb, var(--app-accent) 26%, var(--app-border));
}

.editor-select-chip-active {
  border-color: var(--app-state-active-border);
  background: var(--app-state-active-bg);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--app-accent) 18%, transparent);
}

.editor-select-chip-disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.editor-select-chip-label {
  color: var(--app-text-strong);
  font-size: 0.92rem;
  font-weight: 600;
}

.editor-select-chip-main {
  display: flex;
  width: 100%;
  align-items: flex-start;
  gap: 0.65rem;
}

.editor-select-chip-copy {
  display: flex;
  min-width: 0;
  flex: 1 1 auto;
  flex-direction: column;
  align-items: flex-start;
  gap: 0.18rem;
}

.editor-select-chip-radio {
  width: 1rem;
  height: 1rem;
  flex: 0 0 1rem;
  margin-top: 0.1rem;
  border-radius: 999px;
  border: 1px solid var(--app-border-strong);
  background: white;
  box-shadow: inset 0 0 0 2px white;
}

.editor-select-chip-radio-active {
  border-color: var(--app-accent);
  background: var(--app-accent);
}

.editor-select-chip-description {
  color: var(--app-text-faint);
  font-size: 0.72rem;
  line-height: 1.35;
}
</style>
