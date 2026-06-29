<template>
  <div class="space-y-2">
    <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">{{ label }}</span>
    <EditorSelectField
      :model-value="modelValue"
      :options="options"
      :show-description="showDescription"
      :placeholder="placeholder"
      :test-id="testId"
      @update:model-value="$emit('update:modelValue', String($event || ''))"
    />
    <div v-if="showCreate || showLocate" class="flex flex-wrap gap-2">
      <button
        v-if="showCreate"
        class="app-button app-button-ghost app-toolbar-button"
        type="button"
        :data-testid="createTestId"
        @click="$emit('create')"
      >
        <AppIcon name="plus" :size="14" />
        {{ createLabel }}
      </button>
      <button
        v-if="showLocate"
        class="app-button app-button-ghost app-toolbar-button"
        type="button"
        :data-testid="locateTestId"
        :disabled="locateDisabled"
        @click="$emit('locate')"
      >
        <AppIcon name="locate-fixed" :size="14" />
        {{ locateLabel }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import AppIcon from '@/components/shared/AppIcon.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';

type SelectOption = { label: string; value: string; description?: string; disabled?: boolean };

withDefaults(
  defineProps<{
    label: string;
    modelValue: string | null;
    options: SelectOption[];
    placeholder: string;
    testId?: string;
    createLabel?: string;
    locateLabel?: string;
    createTestId?: string;
    locateTestId?: string;
    showCreate?: boolean;
    showLocate?: boolean;
    locateDisabled?: boolean;
    showDescription?: boolean;
  }>(),
  {
    testId: undefined,
    createLabel: '新建变量',
    locateLabel: '定位变量',
    createTestId: undefined,
    locateTestId: undefined,
    showCreate: false,
    showLocate: false,
    locateDisabled: false,
    showDescription: true,
  },
);

defineEmits<{
  'update:modelValue': [value: string];
  create: [];
  locate: [];
}>();

defineOptions({ name: 'EditorVariableBindingField' });
</script>
