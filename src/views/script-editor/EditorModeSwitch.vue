<template>
  <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] p-2">
    <div class="grid grid-cols-2 gap-2">
      <button
        v-for="option in options"
        :key="option.id"
        class="editor-mode-tab"
        :class="{ 'editor-mode-tab-active': modelValue === option.id }"
        type="button"
        :data-testid="`editor-mode-${option.id}`"
        @click="$emit('update:modelValue', option.id)"
      >
        {{ option.label }}
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { EditorModeId } from '@/views/script-editor/editor-policy/editorPolicy';

defineProps<{
  modelValue: EditorModeId;
  options: Array<{ id: EditorModeId; label: string }>;
}>();

defineEmits<{
  'update:modelValue': [value: EditorModeId];
}>();
</script>

<style scoped>
.editor-mode-tab {
  border-radius: 14px;
  border: 1px solid transparent;
  padding: 0.7rem 0.75rem;
  color: var(--app-text-soft);
  font-size: 0.88rem;
  font-weight: 700;
  transition: border-color 0.16s ease, background 0.16s ease, color 0.16s ease;
}

.editor-mode-tab:hover {
  background: rgba(255, 255, 255, 0.6);
}

.editor-mode-tab-active {
  border-color: rgba(70, 110, 255, 0.2);
  background: color-mix(in srgb, var(--app-state-active-bg) 88%, white);
  color: var(--app-text-strong);
  box-shadow: inset 0 0 0 1px rgba(70, 110, 255, 0.08);
}
</style>
