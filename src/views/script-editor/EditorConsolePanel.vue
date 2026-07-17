<template>
  <SurfacePanel padding="none" class="shrink-0 overflow-hidden flex flex-col">
    <div
      class="editor-console-resize"
      title="拖动调整控制台高度"
      @mousedown.prevent="startResize"
    />
    <div class="flex items-center justify-between border-b border-(--app-border) px-1 py-1">
      <div class="space-y-1">
        <!-- <p class="text-xs font-medium text-(--app-text-soft)">底部控制台</p> -->
      </div>
      <div class="flex items-center gap-2">
        <span class="text-xs text-(--app-text-faint)">{{ entries.length }} / {{ maxLines }}</span>
        <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('clear')">
          清空
        </button>
        <button class="app-icon-button group" type="button" title="关闭控制台" aria-label="关闭控制台" @click="$emit('close')">
          <AppIcon name="x" :size="14" class="text-(--app-text-soft) group-hover:text-(--app-text-strong) transition-colors" />
        </button>
      </div>
    </div>

    <div
      ref="scrollPanel"
      class="editor-console-body overflow-y-auto bg-slate-950 px-2 py-3 font-mono text-xs leading-6 text-slate-100"
      :style="{ height: `${height}px` }"
      draggable="false"
      @mousedown.stop
      @dragstart.prevent
    >
      <div v-if="entries.length" class="space-y-1">
        <div
          v-for="(entry, index) in entries"
          :key="`${index}-${entry.time}-${entry.message}`"
          class="editor-console-line whitespace-pre-wrap break-all"
          :class="`editor-console-line--${entry.level}`"
          draggable="false"
          @dragstart.prevent
        >
          <span class="editor-console-line__time">[{{ entry.time }}]</span>
          <span class="editor-console-line__message">{{ entry.message }}</span>
        </div>
      </div>
      <p v-else class="text-slate-500">暂无输出</p>
    </div>
  </SurfacePanel>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';

interface EditorConsoleEntry {
  time: string;
  message: string;
  level: 'info' | 'warning' | 'error' | 'debug';
}

const props = withDefaults(
  defineProps<{
    entries: EditorConsoleEntry[];
    maxLines?: number;
    initialHeight?: number;
    minHeight?: number;
  }>(),
  {
    maxLines: 300,
    initialHeight: 120,
    minHeight: 96,
  },
);

defineEmits<{
  clear: [];
  close: [];
}>();

const maxHeight = () => window.innerHeight * 0.9;
const clampHeight = (value: number) => Math.min(maxHeight(), Math.max(props.minHeight, value));

const height = ref(clampHeight(props.initialHeight));
const scrollPanel = ref<HTMLElement | null>(null);
const resizing = ref(false);
const resizeStartY = ref(0);
const resizeStartHeight = ref(props.initialHeight);

const syncBottom = async () => {
  await nextTick();
  if (!scrollPanel.value) {
    return;
  }
  scrollPanel.value.scrollTop = scrollPanel.value.scrollHeight;
};

const handleMouseMove = (event: MouseEvent) => {
  if (!resizing.value) {
    return;
  }
  const delta = resizeStartY.value - event.clientY;
  height.value = clampHeight(resizeStartHeight.value + delta);
};

const stopResize = () => {
  resizing.value = false;
};

const handleWindowResize = () => {
  height.value = clampHeight(height.value);
};

const startResize = (event: MouseEvent) => {
  resizing.value = true;
  resizeStartY.value = event.clientY;
  resizeStartHeight.value = height.value;
};

watch(
  () => props.entries.length,
  () => {
    void syncBottom();
  },
  { immediate: true },
);

window.addEventListener('mousemove', handleMouseMove);
window.addEventListener('mouseup', stopResize);
window.addEventListener('resize', handleWindowResize);

onBeforeUnmount(() => {
  window.removeEventListener('mousemove', handleMouseMove);
  window.removeEventListener('mouseup', stopResize);
  window.removeEventListener('resize', handleWindowResize);
});
</script>

<style scoped>
.editor-console-resize {
  height: 4px;
  cursor: ns-resize;
  background:
    linear-gradient(180deg, transparent 0, transparent 3px, color-mix(in srgb, var(--app-border) 70%, transparent) 3px, color-mix(in srgb, var(--app-border) 70%, transparent) 4px, transparent 4px);
}
.editor-console-resize:hover{
  background-color: rgba(70, 110, 255, 0.96);
}

.editor-console-body,
.editor-console-line,
.editor-console-line__message,
.editor-console-line__time {
  user-select: text;
  -webkit-user-select: text;
}

.editor-console-body ::selection,
.editor-console-line ::selection,
.editor-console-line__message ::selection,
.editor-console-line__time ::selection {
  background: rgba(96, 165, 250, 0.42);
  color: rgb(255, 255, 255);
}

.editor-console-body ::-moz-selection,
.editor-console-line ::-moz-selection,
.editor-console-line__message ::-moz-selection,
.editor-console-line__time ::-moz-selection {
  background: rgba(96, 165, 250, 0.42);
  color: rgb(255, 255, 255);
}

.editor-console-body {
  cursor: text;
}

.editor-console-line {
  display: flex;
  align-items: flex-start;
  gap: 0.5rem;
}

.editor-console-line__time {
  color: rgba(148, 163, 184, 0.92);
  flex: 0 0 auto;
}

.editor-console-line__message {
  min-width: 0;
  flex: 1 1 auto;
}

.editor-console-line--info .editor-console-line__message {
  color: rgb(74, 222, 128);
}

.editor-console-line--warning .editor-console-line__message {
  color: rgb(250, 204, 21);
}

.editor-console-line--error .editor-console-line__message {
  color: rgb(248, 113, 113);
}

.editor-console-line--debug .editor-console-line__message {
  color: rgb(191, 219, 254);
}
</style>
