<template>
  <div class="editor-code-field" :data-testid="testId">
    <Codemirror
      :model-value="modelValue"
      :extensions="extensions"
      :placeholder="placeholder"
      :style="{ minHeight: `${minHeight}px` }"
      @update:model-value="$emit('update:modelValue', $event)"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Codemirror } from 'vue-codemirror';
import { basicSetup } from 'codemirror';

const props = withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    minHeight?: number;
    testId?: string;
  }>(),
  {
    placeholder: '',
    minHeight: 220,
    testId: undefined,
  },
);

defineEmits<{
  'update:modelValue': [value: string];
}>();

const extensions = computed(() => [basicSetup]);
</script>

<style scoped>
.editor-code-field {
  border: 1px solid var(--app-border);
  border-radius: 16px;
  overflow: hidden;
  background: color-mix(in srgb, var(--app-panel-muted) 82%, white 18%);
}

.editor-code-field :deep(.cm-editor) {
  font-family: "Cascadia Code", "Fira Code", "JetBrains Mono", Consolas, monospace;
  font-size: 0.9rem;
}

.editor-code-field :deep(.cm-focused) {
  outline: none;
}

.editor-code-field :deep(.cm-scroller) {
  min-height: inherit;
}
</style>
