<template>
  <div class="space-y-3">
    <label class="space-y-2">
      <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">{{ label }}</span>
      <EditorSelectField
        :model-value="modelValue"
        :options="options"
        :placeholder="placeholder"
        :test-id="testId"
        @update:model-value="$emit('update:modelValue', String($event || fixedValue))"
      />
    </label>

    <slot v-if="modelValue === fixedValue" name="fixed" />
    <slot v-else name="binding" />
  </div>
</template>

<script setup lang="ts">
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';

type SelectOption = { label: string; value: string; description?: string; disabled?: boolean };

withDefaults(
  defineProps<{
    label: string;
    modelValue: string;
    options: SelectOption[];
    placeholder?: string;
    testId?: string;
    fixedValue?: string;
  }>(),
  {
    placeholder: '选择取值方式',
    testId: undefined,
    fixedValue: 'fixed',
  },
);

defineEmits<{
  'update:modelValue': [value: string];
}>();

defineOptions({ name: 'EditorPresetBindingSection' });
</script>
