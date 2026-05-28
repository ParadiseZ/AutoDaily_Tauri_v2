<template>
  <header class="editor-window-titlebar">
    <div v-if="$slots.prefix" class="editor-window-titlebar__prefix">
      <slot name="prefix" />
    </div>

    <div class="editor-window-titlebar__drag" data-tauri-drag-region @mousedown.left.prevent="handleStartDragging">
      <div class="space-y-1">
        <div class="flex flex-wrap items-center gap-2">
          <h6 class="editor-window-titlebar__title">
            {{ title }}
          </h6>
          <span
            v-if="statusLabel"
            class="rounded-full px-3 py-1 text-xs font-medium"
            :class="statusToneClass"
          >
            {{ statusLabel }}
          </span>
        </div>
      </div>
    </div>

    <div v-if="$slots['title-actions']" class="editor-window-titlebar__title-actions">
      <slot name="title-actions" />
    </div>

    <div v-if="$slots.actions" class="editor-window-titlebar__actions">
      <slot name="actions" />
    </div>

    <div v-if="windowHandle" class="editor-window-titlebar__window-controls">
      <button class="editor-window-titlebar__window-button" type="button" title="最小化" aria-label="最小化" @click="handleMinimize">
        <AppIcon name="minus" :size="14" />
      </button>
      <button
        class="editor-window-titlebar__window-button"
        type="button"
        :title="isMaximized ? '还原' : '最大化'"
        :aria-label="isMaximized ? '还原' : '最大化'"
        @click="handleToggleMaximize"
      >
        <AppIcon :name="isMaximized ? 'copy' : 'square'" :size="13" />
      </button>
      <button class="editor-window-titlebar__window-button editor-window-titlebar__window-button--close" type="button" title="关闭" aria-label="关闭" @click="handleCloseWindow">
        <AppIcon name="x" :size="14" />
      </button>
    </div>
  </header>
</template>

<script setup lang="ts">
import { watch, ref, onMounted, onBeforeUnmount, computed } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import AppIcon from '@/components/shared/AppIcon.vue';

const props = withDefaults(
  defineProps<{
    title: string;
    meta?: string | null;
    statusLabel?: string | null;
    statusTone?: 'info' | 'success' | 'warning' | 'danger';
  }>(),
  {
    meta: null,
    statusLabel: null,
    statusTone: 'info',
  },
);

const isTauriRuntime =
  typeof window !== 'undefined' &&
  Boolean((window as typeof window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);

const windowHandle = isTauriRuntime ? getCurrentWindow() : null;
const isMaximized = ref(false);
const originalDecorations = ref<boolean | null>(null);
let detachResizeListener: (() => void) | null = null;

const statusToneClass = computed(() => {
  if (props.statusTone === 'danger') return 'bg-red-500/12 text-red-700';
  if (props.statusTone === 'warning') return 'bg-amber-500/12 text-amber-700';
  if (props.statusTone === 'success') return 'bg-emerald-500/12 text-emerald-700';
  return 'bg-sky-500/12 text-sky-700';
});

async function syncMaximizedState() {
  if (!windowHandle) {
    isMaximized.value = false;
    return;
  }
  isMaximized.value = await windowHandle.isMaximized();
}

async function handleMinimize() {
  if (!windowHandle) {
    return;
  }
  await windowHandle.minimize();
}

async function handleStartDragging() {
  if (!windowHandle) {
    return;
  }
  await windowHandle.startDragging();
}

async function handleToggleMaximize() {
  if (!windowHandle) {
    return;
  }
  await windowHandle.toggleMaximize();
  await syncMaximizedState();
}

async function handleCloseWindow() {
  if (!windowHandle) {
    return;
  }
  await windowHandle.close();
}

onMounted(async () => {
  if (!windowHandle) {
    return;
  }

  originalDecorations.value = await windowHandle.isDecorated();
  await windowHandle.setDecorations(false);
  await windowHandle.setTitle(props.title || '脚本编辑器');
  await syncMaximizedState();
  detachResizeListener = await windowHandle.onResized(() => {
    void syncMaximizedState();
  });
});

watch(
  () => props.title,
  (title) => {
    if (!windowHandle) {
      return;
    }
    void windowHandle.setTitle(title || '脚本编辑器');
  },
);

onBeforeUnmount(() => {
  detachResizeListener?.();
  detachResizeListener = null;
  if (windowHandle && originalDecorations.value !== null) {
    void windowHandle.setDecorations(originalDecorations.value);
  }
});
</script>

<style scoped>
.editor-window-titlebar {
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto auto auto;
  align-items: center;
  gap: 0.75rem;
  min-height: 64px;
  border-bottom: 1px solid color-mix(in srgb, var(--app-border) 92%, transparent);
  background:
    radial-gradient(circle at 10% 14%, rgba(70, 110, 255, 0.14), transparent 24%),
    radial-gradient(circle at 88% 16%, rgba(87, 196, 255, 0.14), transparent 22%),
    var(--app-toolbar-bg);
  box-shadow: var(--app-shadow-soft);
  backdrop-filter: blur(16px);
  padding: 0.65rem 0.75rem 0.65rem 1rem;
}

.editor-window-titlebar__prefix,
.editor-window-titlebar__title-actions,
.editor-window-titlebar__actions,
.editor-window-titlebar__window-controls {
  display: inline-flex;
  align-items: center;
  gap: 0.45rem;
}

.editor-window-titlebar__drag {
  min-width: 0;
}

.editor-window-titlebar__title {
  margin: 0;
  color: var(--app-text-strong);
  font-size: 1rem;
  font-weight: 600;
  letter-spacing: -0.04em;
  line-height: 1;
}

.editor-window-titlebar__window-controls {
  gap: 0;
  align-self: stretch;
  margin-left: auto;
}

.editor-window-titlebar__window-button {
  width: 2.75rem;
  height: 100%;
  min-height: 2.4rem;
  border: none;
  background: transparent;
  color: var(--app-text-soft);
  transition: background 0.16s ease, color 0.16s ease;
}

.editor-window-titlebar__window-button:hover {
  background: rgba(70, 110, 255, 0.12);
  color: var(--app-text-strong);
  cursor: pointer;
}

.editor-window-titlebar__window-button--close:hover {
  background: rgba(239, 68, 68, 0.18);
  color: rgb(185, 28, 28);
}

@media (max-width: 1279px) {
  .editor-window-titlebar {
    grid-template-columns: auto minmax(0, 1fr) auto;
    align-items: start;
  }

  .editor-window-titlebar__title-actions,
  .editor-window-titlebar__actions {
    grid-column: 1 / -1;
    flex-wrap: wrap;
  }

  .editor-window-titlebar__window-controls {
    grid-column: 3;
    grid-row: 1;
  }
}
</style>
