<template>
  <SurfacePanel padding="sm" class="editor-mode-rail-panel flex h-full min-h-0 flex-col items-center gap-2 overflow-visible">
    <button
      v-for="option in modeRailOptions"
      :key="option.id"
      class="editor-mode-icon"
      :class="{ 'editor-mode-icon-active': modelValue === option.id }"
      type="button"
      :title="option.label"
      :aria-label="option.label"
      :data-testid="`editor-mode-${option.id}`"
      @click="$emit('update:modelValue', option.id)"
    >
      <component :is="option.icon" class="h-4 w-4" />
      <span class="editor-mode-tooltip">{{ option.label }}</span>
    </button>
  </SurfacePanel>
</template>

<script setup lang="ts">
import { Boxes, FolderTree, Shield, ListTodo } from 'lucide-vue-next';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { EditorModeId } from '@/views/script-editor/editor-policy/editorPolicy';

defineProps<{
  modelValue: EditorModeId;
}>();

defineEmits<{
  'update:modelValue': [value: EditorModeId];
}>();

const modeRailOptions: Array<{ id: EditorModeId; label: string; icon: typeof ListTodo }> = [
  { id: 'task', label: '任务', icon: ListTodo },
  { id: 'policy', label: '策略', icon: Shield },
  { id: 'policyGroup', label: '策略组', icon: FolderTree },
  { id: 'policySet', label: '策略集', icon: Boxes },
];
</script>

<style scoped>
.editor-mode-icon {
  position: relative;
  display: grid;
  place-items: center;
  width: 2.8rem;
  height: 2.8rem;
  border-radius: 16px;
  border: 1px solid transparent;
  color: var(--app-text-soft);
  transition: border-color 0.16s ease, background 0.16s ease, color 0.16s ease;
}

.editor-mode-icon:hover {
  background: rgba(255, 255, 255, 0.72);
}

.editor-mode-icon-active {
  border-color: rgba(70, 110, 255, 0.28);
  background: linear-gradient(180deg, rgba(225, 234, 255, 0.92), rgba(213, 227, 255, 0.78));
  color: var(--app-text-strong);
  box-shadow:
    inset 0 0 0 1px rgba(70, 110, 255, 0.12),
    0 10px 20px rgba(70, 110, 255, 0.12);
}

.editor-mode-tooltip {
  pointer-events: none;
  position: absolute;
  z-index: 60;
  left: calc(100% + 0.32rem);
  top: 50%;
  transform: translateY(-50%) scale(0.96);
  border-radius: 999px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.96);
  padding: 0.28rem 0.65rem;
  white-space: nowrap;
  font-size: 0.72rem;
  font-weight: 700;
  color: var(--app-text-strong);
  opacity: 0;
  box-shadow: var(--app-shadow-soft);
  transition: opacity 0.16s ease, transform 0.16s ease;
}

.editor-mode-rail-panel {
  position: relative;
  z-index: 40;
}

.editor-mode-icon:hover .editor-mode-tooltip,
.editor-mode-icon:focus-visible .editor-mode-tooltip {
  opacity: 1;
  transform: translateY(-50%) scale(1);
}
</style>
