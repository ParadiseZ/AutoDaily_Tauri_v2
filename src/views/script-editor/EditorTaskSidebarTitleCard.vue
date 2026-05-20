<template>
  <article
    class="app-list-item space-y-3 transition-colors"
    :class="{
      'app-list-item-active': selected,
      'editor-task-card-dragging': dragging,
      'editor-task-drop-target': dropTarget,
    }"
    :data-testid="`editor-task-item-${title.id}`"
    @mouseenter="$emit('mouseenter', title.id)"
    @mouseup="$emit('mouseup', title.id)"
    @contextmenu="$emit('contextmenu', $event, title.id)"
  >
    <div class="grid grid-cols-[34px_28px_minmax(0,1fr)_auto] items-start gap-2">
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

      <button
        class="editor-task-collapse-trigger"
        :data-testid="`editor-task-group-toggle-${title.id}`"
        type="button"
        :title="expanded ? '收起分组' : '展开分组'"
        :aria-label="expanded ? '收起分组' : '展开分组'"
        @click.stop="$emit('toggle-collapsed', title.id)"
      >
        <ChevronDown v-if="expanded" class="h-4 w-4" />
        <ChevronRight v-else class="h-4 w-4" />
      </button>

      <button class="min-w-0 text-left" type="button" @click="$emit('select', title.id)">
        <div class="flex items-center gap-2">
          <span class="rounded-full border border-(--app-border) px-2 py-1 text-[11px] text-(--app-text-faint)">
            {{ title.index + 1 }}
          </span>
          <p class="truncate text-sm font-semibold text-(--app-text-strong)">
            {{ title.name }}
          </p>
          <span class="rounded-full bg-(--app-panel-muted) px-2 py-1 text-[11px] text-(--app-text-faint)">
            {{ taskCount }}
          </span>
        </div>
        <p class="mt-2 text-xs text-(--app-text-faint)">标题行 · 分组标题</p>
      </button>

      <div class="flex flex-col items-end gap-2">
        <button
          class="app-icon-button app-icon-button-sec"
          type="button"
          :title="title.isHidden ? '点击显示' : '点击隐藏'"
          @click.stop="$emit('toggle-hidden', title.id)"
        >
          <EyeOff v-if="title.isHidden" class="h-4 w-4" />
          <Eye v-else class="h-4 w-4" />
        </button>
      </div>
    </div>

    <div class="flex flex-wrap gap-2">
      <button class="app-icon-button app-icon-button-sec" type="button" aria-label="复制" title="复制" @click.stop="$emit('duplicate', title.id)">
        <Copy class="h-4 w-4" />
      </button>
      <button
        class="app-icon-button app-crash-icon app-icon-button-sec"
        type="button"
        :disabled="disableRemove"
        aria-label="删除"
        title="删除"
        @click.stop="$emit('remove', title.id)"
      >
        <Trash2 class="h-4 w-4" />
      </button>
    </div>
  </article>
</template>

<script setup lang="ts">
import { ChevronDown, ChevronRight, Copy, Eye, EyeOff, GripVertical, Trash2 } from 'lucide-vue-next';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';

defineOptions({ name: 'EditorTaskSidebarTitleCard' });

defineProps<{
  title: ScriptTaskTable;
  selected: boolean;
  dragging: boolean;
  dropTarget: boolean;
  expanded: boolean;
  taskCount: number;
  disableRemove: boolean;
}>();

defineEmits<{
  select: [taskId: string];
  duplicate: [taskId: string];
  'toggle-hidden': [taskId: string];
  remove: [taskId: string];
  'drag-start': [taskId: string];
  'toggle-collapsed': [taskId: string];
  mouseenter: [taskId: string];
  mouseup: [taskId: string];
  contextmenu: [event: MouseEvent, taskId: string];
}>();
</script>
