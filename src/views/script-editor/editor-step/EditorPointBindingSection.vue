<template>
  <EditorPresetBindingSection
    :label="label"
    :model-value="source"
    :options="sourceOptions"
    :placeholder="mode === 'percent' ? '选择百分比来源' : '选择坐标来源'"
    :test-id="sourceTestId"
    @update:model-value="$emit('update-source', $event)"
  >
    <template #fixed>
      <div class="grid gap-3 md:grid-cols-2">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">X</span>
          <input
            :value="String(point.x ?? '')"
            class="app-input"
            type="number"
            :min="mode === 'percent' ? 0 : undefined"
            :max="mode === 'percent' ? 1 : undefined"
            :step="mode === 'percent' ? 0.01 : 1"
            :aria-label="`${label} X`"
            @input="$emit('update-point', 'x', ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">Y</span>
          <input
            :value="String(point.y ?? '')"
            class="app-input"
            type="number"
            :min="mode === 'percent' ? 0 : undefined"
            :max="mode === 'percent' ? 1 : undefined"
            :step="mode === 'percent' ? 0.01 : 1"
            :aria-label="`${label} Y`"
            @input="$emit('update-point', 'y', ($event.target as HTMLInputElement).value)"
          />
        </label>
      </div>
    </template>

    <template #binding>
      <EditorVariableBindingField
        :label="variableLabel"
        :model-value="variableValue || null"
        :options="variableOptions"
        :placeholder="mode === 'percent' ? '绑定 JSON 百分比点位变量' : '绑定 JSON 坐标点位变量'"
        :test-id="variableTestId"
        :create-label="createLabel"
        :show-create="showCreate"
        :show-locate="showLocate"
        :locate-disabled="!showLocate"
        @update:model-value="$emit('update-variable', $event)"
        @create="$emit('create')"
        @locate="$emit('locate')"
      />
      <p class="text-xs leading-5 text-(--app-text-soft)">
        变量值使用 JSON <code>{ "x": ..., "y": ... }</code>。{{ mode === 'percent' ? 'X / Y 必须在 0 到 1 之间。' : 'X / Y 使用非负绝对坐标。' }}
      </p>
    </template>
  </EditorPresetBindingSection>
</template>

<script setup lang="ts">
import EditorPresetBindingSection from '@/views/script-editor/EditorPresetBindingSection.vue';
import EditorVariableBindingField from '@/views/script-editor/EditorVariableBindingField.vue';

type SelectOption = { label: string; value: string; description?: string; disabled?: boolean };

withDefaults(
  defineProps<{
    label: string;
    mode: 'point' | 'percent';
    source: string;
    point: { x?: number; y?: number };
    variableLabel: string;
    variableValue?: string | null;
    variableOptions: SelectOption[];
    sourceOptions: SelectOption[];
    sourceTestId?: string;
    variableTestId?: string;
    createLabel?: string;
    showCreate?: boolean;
    showLocate?: boolean;
  }>(),
  {
    variableValue: null,
    sourceTestId: undefined,
    variableTestId: undefined,
    createLabel: '新建点位变量',
    showCreate: false,
    showLocate: false,
  },
);

defineEmits<{
  'update-source': [value: string];
  'update-point': [axis: 'x' | 'y', value: string];
  'update-variable': [value: string];
  create: [];
  locate: [];
}>();

defineOptions({ name: 'EditorPointBindingSection' });
</script>
