<template>
  <div class="editor-mode-switch">
    <div class="editor-panel-tabs min-w-0 flex-1 overflow-x-auto">
      <button
        v-for="option in options"
        :key="option.id"
        class="editor-panel-tab"
        :class="{ 'editor-panel-tab-active': modelValue === option.id }"
        type="button"
        :title="option.label"
        :data-testid="`editor-mode-${option.id}`"
        @click="$emit('update:modelValue', option.id)"
      >
        {{ collapsed ? option.label.slice(0, 1) : option.label }}
      </button>
    </div>
    <button
      class="app-icon-button shrink-0"
      type="button"
      :title="collapsed ? '展开左侧区域' : '收缩左侧区域'"
      :aria-label="collapsed ? '展开左侧区域' : '收缩左侧区域'"
      @click="$emit('toggle-collapsed')"
    >
      <AppIcon :name="collapsed ? 'panel-left-open' : 'panel-left-close'" :size="15" />
    </button>
  </div>
</template>

<script setup lang="ts">
import AppIcon from '@/components/shared/AppIcon.vue';
import type { EditorModeId } from '@/views/script-editor/editor-policy/editorPolicy';

defineProps<{
  modelValue: EditorModeId;
  options: Array<{ id: EditorModeId; label: string }>;
  collapsed?: boolean;
}>();

defineEmits<{
  'update:modelValue': [value: EditorModeId];
  'toggle-collapsed': [];
}>();
</script>

<style scoped>
.editor-mode-switch {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 0.5rem;
}
</style>
