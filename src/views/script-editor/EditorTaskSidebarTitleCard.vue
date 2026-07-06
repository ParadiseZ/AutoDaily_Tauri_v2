<template>
  <article
    class="app-list-item space-y-3 transition-colors"
    :class="[{
      'app-list-item-active': selected,
      'editor-task-card-dragging': dragging,
      'editor-task-drop-target': dropTarget,
      'editor-task-card-hidden': title.isHidden,
    },{
      'editor-task-card-expanded': expanded
    }]"
    :data-testid="`editor-task-item-${title.id}`"
    @mouseenter="$emit('mouseenter', title.id)"
    @mouseup="$emit('mouseup', title.id)"
    @contextmenu="$emit('contextmenu', $event, title.id)"
  >
    <div class="grid grid-cols-[34px_minmax(0,1fr)_28px] items-start gap-2">
      <button
        class="app-drag-handle"
        :class="{ 'app-drag-handle-active': dragging }"
        :data-testid="`editor-task-drag-${title.id}`"
        type="button"
        aria-label="拖动排序"
        @mousedown.prevent="$emit('drag-start', title.id)"
        @click.stop
      >
        <GripVertical class="h-4 w-4" />
      </button>

      <button class="min-w-0 text-left" type="button" @click="$emit('select', title.id)">
        <div class="flex items-center gap-2">
          <span class="rounded-full border border-(--app-border) px-2 py-1 text-[11px]">
            {{ title.index + 1 }}
          </span>
          <p class="truncate text-sm font-semibold" :class="expanded? '' : title.isHidden ? 'text-(--app-text-faint)' : 'text-(--app-text-strong)'">
            {{ title.name }}
          </p>
          <span class="rounded-full bg-(--app-panel-muted) px-2 py-1 text-[11px] text-(--app-text-faint)">
            {{ taskCount }}
          </span>
        </div>
        <p class="mt-2 line-clamp-2 text-xs">{{ title.description?.trim() || '分组标题' }}</p>
      </button>

      <button
        class="editor-task-collapse-trigger"
        :data-testid="`editor-task-group-toggle-${title.id}`"
        type="button"
        :title="expanded ? '收起分组' : '展开分组'"
        :aria-label="expanded ? '收起分组' : '展开分组'"
        @click.stop="$emit('toggle-collapsed', title.id)"
      >
        <ChevronDown class="h-6 w-6 editor-task-collapse-icon" :class="{ 'editor-task-collapse-icon-collapsed': !expanded }" />
      </button>
    </div>

  </article>
</template>

<script setup lang="ts">
import { ChevronDown, GripVertical } from 'lucide-vue-next';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';

defineOptions({ name: 'EditorTaskSidebarTitleCard' });

defineProps<{
  title: ScriptTaskTable;
  selected: boolean;
  dragging: boolean;
  dropTarget: boolean;
  expanded: boolean;
  taskCount: number;
}>();

defineEmits<{
  select: [taskId: string];
  'drag-start': [taskId: string];
  'toggle-collapsed': [taskId: string];
  mouseenter: [taskId: string];
  mouseup: [taskId: string];
  contextmenu: [event: MouseEvent, taskId: string];
}>();
</script>

<style scoped>
.editor-task-collapse-icon {
  transition: transform 0.22s ease, color 0.16s ease;
}

.editor-task-collapse-icon-collapsed {
  transform: rotate(-90deg);
}
</style>
