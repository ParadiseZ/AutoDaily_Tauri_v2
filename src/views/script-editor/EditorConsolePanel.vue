<template>
  <SurfacePanel padding="sm" class="shrink-0 overflow-hidden">
    <div
      class="editor-console-resize"
      title="拖动调整控制台高度"
      @mousedown.prevent="startResize"
    />
    <div class="flex items-center justify-between border-b border-[var(--app-border)] px-4 py-3">
      <div class="space-y-1">
        <p class="text-xs uppercase tracking-[0.16em] text-[var(--app-text-faint)]">Console</p>
        <h2 class="text-sm font-semibold text-[var(--app-text-strong)]">调试输出</h2>
      </div>
      <div class="flex items-center gap-2">
        <span class="text-xs text-[var(--app-text-faint)]">{{ lines.length }} / {{ maxLines }}</span>
        <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('clear')">
          清空
        </button>
      </div>
    </div>

    <div ref="scrollPanel" class="overflow-y-auto bg-slate-950 px-4 py-3 font-mono text-xs leading-6 text-slate-100" :style="{ height: `${height}px` }">
      <div v-if="lines.length" class="space-y-1">
        <div v-for="(line, index) in lines" :key="`${index}-${line}`" class="whitespace-pre-wrap break-all">
          {{ line }}
        </div>
      </div>
      <p v-else class="text-slate-500">暂无输出</p>
    </div>
  </SurfacePanel>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from 'vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';

const props = withDefaults(
  defineProps<{
    lines: string[];
    maxLines?: number;
    initialHeight?: number;
    minHeight?: number;
    maxHeight?: number;
  }>(),
  {
    maxLines: 300,
    initialHeight: 120,
    minHeight: 96,
    maxHeight: 360,
  },
);

defineEmits<{
  clear: [];
}>();

const height = ref(props.initialHeight);
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

const clampHeight = (value: number) => Math.max(props.minHeight, Math.min(props.maxHeight, value));

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

const startResize = (event: MouseEvent) => {
  resizing.value = true;
  resizeStartY.value = event.clientY;
  resizeStartHeight.value = height.value;
};

watch(
  () => props.lines.length,
  () => {
    void syncBottom();
  },
  { immediate: true },
);

window.addEventListener('mousemove', handleMouseMove);
window.addEventListener('mouseup', stopResize);

onBeforeUnmount(() => {
  window.removeEventListener('mousemove', handleMouseMove);
  window.removeEventListener('mouseup', stopResize);
});
</script>

<style scoped>
.editor-console-resize {
  height: 10px;
  cursor: ns-resize;
  background:
    linear-gradient(180deg, transparent 0, transparent 3px, color-mix(in srgb, var(--app-border) 70%, transparent) 3px, color-mix(in srgb, var(--app-border) 70%, transparent) 4px, transparent 4px);
}
</style>
