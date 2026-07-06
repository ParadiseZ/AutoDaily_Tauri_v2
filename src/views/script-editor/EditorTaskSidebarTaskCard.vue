<template>
  <article
    class="app-list-item space-y-3 transition-colors"
    :class="{
      'app-list-item-active': selected,
      'editor-task-drop-target': dropTarget,
      'editor-task-card-dragging': dragging,
      'editor-task-card-hidden': task.isHidden,
    }"
    :data-testid="`editor-task-item-${task.id}`"
    @mouseenter="$emit('mouseenter', task.id)"
    @mouseup="$emit('mouseup', task.id)"
    @contextmenu="$emit('contextmenu', $event, task.id)"
  >
    <div class="grid grid-cols-[34px_minmax(0,1fr)] items-start gap-2" @click="$emit('select', task.id)">
      <button
        class="app-drag-handle"
        :class="{ 'app-drag-handle-active': dragging }"
        :data-testid="`editor-task-drag-${task.id}`"
        type="button"
        aria-label="拖动排序"
        @mousedown.prevent="$emit('drag-start', task.id)"
        @click.stop
      >
        <GripVertical class="h-4 w-4" />
      </button>

      <button
        class="min-w-0 flex-1 text-left"
        type="button"
        :style="{ paddingLeft: `${task.indentLevel * 0.85}rem` }"
        @click="$emit('select', task.id)"
      >
        <div class="flex items-center gap-2">
          <span class="rounded-full border border-(--app-border) px-2 py-1 text-[11px] text-(--app-text-faint)">
            {{ task.index + 1 }}
          </span>
          <p class="truncate text-sm font-semibold" :class="task.isHidden ? 'text-(--app-text-faint)' : 'text-(--app-text-strong)'">
            {{ task.name }}
          </p>
        </div>
        <p class="mt-2 line-clamp-2 text-xs text-(--app-text-faint)">
          {{ task.description?.trim() ||'' }}
        </p>
      </button>
    </div>

  </article>
</template>

<script setup lang="ts">
import { GripVertical } from 'lucide-vue-next';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';

defineOptions({ name: 'EditorTaskSidebarTaskCard' });

defineProps<{
  task: ScriptTaskTable;
  selected: boolean;
  dropTarget: boolean;
  dragging: boolean;
}>();

defineEmits<{
  select: [taskId: string];
  'drag-start': [taskId: string];
  mouseenter: [taskId: string];
  mouseup: [taskId: string];
  contextmenu: [event: MouseEvent, taskId: string];
}>();
</script>
