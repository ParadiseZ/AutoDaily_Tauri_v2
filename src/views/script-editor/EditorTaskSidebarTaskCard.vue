<template>
  <article
    class="app-list-item space-y-3 transition-colors"
    :class="{
      'app-list-item-active': selected,
      'editor-task-drop-target': dropTarget,
      'editor-task-card-dragging': dragging,
    }"
    :data-testid="`editor-task-item-${task.id}`"
    @mouseenter="$emit('mouseenter', task.id)"
    @mouseup="$emit('mouseup', task.id)"
    @contextmenu="$emit('contextmenu', $event, task.id)"
  >
    <div class="grid grid-cols-[34px_minmax(0,1fr)_auto] items-start gap-2" @click="$emit('select', task.id)">
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
          <p class="truncate text-sm font-semibold text-(--app-text-strong)">
            {{ task.name }}
          </p>
        </div>
        <p class="mt-2 text-xs text-(--app-text-faint)">
          {{ formatTriggerModeLabel(task.triggerMode) }} · {{ task.data.steps.length }} 个步骤
        </p>
      </button>

      <div class="flex flex-col items-end gap-2">
        <button
          class="app-icon-button app-icon-button-sec"
          type="button"
          :title="task.isHidden ? '点击显示' : '点击隐藏'"
          @click.stop="$emit('toggle-hidden', task.id)"
        >
          <EyeOff v-if="task.isHidden" class="h-4 w-4" />
          <Eye v-else class="h-4 w-4" />
        </button>
      </div>
    </div>

    <div class="flex flex-wrap gap-2">
      <button class="app-icon-button app-icon-button-sec" type="button" aria-label="复制" title="复制" @click.stop="$emit('duplicate', task.id)">
        <Copy class="h-4 w-4" />
      </button>
      <button
        class="app-icon-button app-crash-icon app-icon-button-sec"
        type="button"
        :disabled="disableRemove"
        aria-label="删除"
        title="删除"
        @click.stop="$emit('remove', task.id)"
      >
        <Trash2 class="h-4 w-4" />
      </button>
    </div>
  </article>
</template>

<script setup lang="ts">
import { Copy, Eye, EyeOff, GripVertical, Trash2 } from 'lucide-vue-next';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import { formatTaskTriggerModeLabel } from '@/utils/presenters';

defineOptions({ name: 'EditorTaskSidebarTaskCard' });

defineProps<{
  task: ScriptTaskTable;
  selected: boolean;
  dropTarget: boolean;
  dragging: boolean;
  disableRemove: boolean;
}>();

defineEmits<{
  select: [taskId: string];
  duplicate: [taskId: string];
  'toggle-hidden': [taskId: string];
  remove: [taskId: string];
  'drag-start': [taskId: string];
  mouseenter: [taskId: string];
  mouseup: [taskId: string];
  contextmenu: [event: MouseEvent, taskId: string];
}>();

const formatTriggerModeLabel = (value: ScriptTaskTable['triggerMode']) => formatTaskTriggerModeLabel(value);
</script>
