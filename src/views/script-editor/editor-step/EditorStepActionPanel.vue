<template>
  <div class="space-y-3">
    <template v-if="selectedAction.ac === ACTION_TYPE.capture">
      <label class="space-y-2">
        <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输出变量</span>
        <input :value="selectedAction.output_var || ''" :list="variableDatalistId" class="app-input" @input="$emit('update-field', 'output_var', ($event.target as HTMLInputElement).value)" />
      </label>
    </template>

    <template v-else-if="selectedAction.ac === ACTION_TYPE.launchApp || selectedAction.ac === ACTION_TYPE.stopApp">
      <label class="space-y-2">
        <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">包名</span>
        <input :value="selectedAction.pkg_name || ''" class="app-input" @input="$emit('update-field', 'pkg_name', ($event.target as HTMLInputElement).value)" />
      </label>
    </template>

    <template v-else-if="selectedAction.ac === ACTION_TYPE.click">
      <div class="editor-inline-grid">
        <div class="editor-inline-label">点击方式</div>
        <div class="editor-inline-content">
          <EditorSelectField
            :model-value="String(selectedAction.mode || ACTION_MODE.point)"
            :options="clickModeOptions"
            placeholder="点击方式"
            @update:model-value="$emit('update-mode', String($event || ACTION_MODE.point))"
          />
        </div>
      </div>

      <div v-if="selectedAction.mode === ACTION_MODE.point || selectedAction.mode === ACTION_MODE.percent" class="editor-inline-grid">
        <div class="editor-inline-label">X</div>
        <div class="editor-inline-content">
          <input
            :value="String((selectedAction.p as { x?: number })?.x ?? '')"
            aria-label="X"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'p', 'x', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="editor-inline-label">Y</div>
        <div class="editor-inline-content">
          <input
            :value="String((selectedAction.p as { y?: number })?.y ?? '')"
            aria-label="Y"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'p', 'y', ($event.target as HTMLInputElement).value)"
          />
        </div>
      </div>

      <label v-else-if="selectedAction.mode === ACTION_MODE.txt" class="space-y-2">
        <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标文字</span>
        <input :value="String(selectedAction.txt ?? '')" class="app-input" @input="$emit('update-text-field', 'txt', ($event.target as HTMLInputElement).value)" />
      </label>

      <label v-else class="space-y-2">
        <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">标签索引</span>
        <input :value="String(selectedAction.idx ?? 0)" class="app-input" type="number" @input="$emit('update-number-field', 'idx', ($event.target as HTMLInputElement).value)" />
      </label>
    </template>

    <template v-else-if="selectedAction.ac === ACTION_TYPE.swipe">
      <div class="editor-inline-grid">
        <div class="editor-inline-label">滑动方式</div>
        <div class="editor-inline-content">
          <EditorSelectField
            :model-value="String(selectedAction.mode || ACTION_MODE.point)"
            :options="swipeModeOptions"
            placeholder="滑动方式"
            @update:model-value="$emit('update-mode', String($event || ACTION_MODE.point))"
          />
        </div>
        <div class="editor-inline-label">持续时间</div>
        <div class="editor-inline-content">
          <input :value="String(selectedAction.duration ?? 300)" class="app-input" type="number" @input="$emit('update-number-field', 'duration', ($event.target as HTMLInputElement).value)" />
        </div>
      </div>

      <div v-if="selectedAction.mode === ACTION_MODE.point || selectedAction.mode === ACTION_MODE.percent" class="editor-inline-grid">
        <div class="editor-inline-label">起点 X</div>
        <div class="editor-inline-content">
          <input
            :value="String((selectedAction.from as { x?: number })?.x ?? '')"
            aria-label="起点 X"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'from', 'x', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="editor-inline-label">起点 Y</div>
        <div class="editor-inline-content">
          <input
            :value="String((selectedAction.from as { y?: number })?.y ?? '')"
            aria-label="起点 Y"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'from', 'y', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="editor-inline-label">终点 X</div>
        <div class="editor-inline-content">
          <input
            :value="String((selectedAction.to as { x?: number })?.x ?? '')"
            aria-label="终点 X"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'to', 'x', ($event.target as HTMLInputElement).value)"
          />
        </div>
        <div class="editor-inline-label">终点 Y</div>
        <div class="editor-inline-content">
          <input
            :value="String((selectedAction.to as { y?: number })?.y ?? '')"
            aria-label="终点 Y"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'to', 'y', ($event.target as HTMLInputElement).value)"
          />
        </div>
      </div>

      <div v-else-if="selectedAction.mode === ACTION_MODE.txt" class="grid gap-3 md:grid-cols-2">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">起点文字</span>
          <input :value="String(selectedAction.from ?? '')" class="app-input" @input="$emit('update-text-field', 'from', ($event.target as HTMLInputElement).value)" />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">终点文字</span>
          <input :value="String(selectedAction.to ?? '')" class="app-input" @input="$emit('update-text-field', 'to', ($event.target as HTMLInputElement).value)" />
        </label>
      </div>

      <div v-else class="grid gap-3 md:grid-cols-2">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">起点标签</span>
          <input :value="String(selectedAction.from ?? 0)" class="app-input" type="number" @input="$emit('update-number-field', 'from', ($event.target as HTMLInputElement).value)" />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">终点标签</span>
          <input :value="String(selectedAction.to ?? 1)" class="app-input" type="number" @input="$emit('update-number-field', 'to', ($event.target as HTMLInputElement).value)" />
        </label>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import type { Action } from '@/types/bindings/Action';
import { ACTION_MODE, ACTION_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';

defineOptions({ name: 'EditorStepActionPanel' });

defineProps<{
  selectedAction: Action;
  variableDatalistId: string;
  clickModeOptions: Array<{ label: string; value: string; description: string }>;
  swipeModeOptions: Array<{ label: string; value: string; description: string }>;
}>();

defineEmits<{
  'update-field': [field: string, value: string];
  'update-mode': [mode: string];
  'update-point-field': [field: 'p' | 'from' | 'to', axis: 'x' | 'y', value: string];
  'update-number-field': [field: string, value: string];
  'update-text-field': [field: string, value: string];
}>();
</script>

<style scoped>
.editor-inline-grid {
  display: grid;
  gap: 0.75rem;
}

@media (min-width: 768px) {
  .editor-inline-grid {
    grid-template-columns: 78px minmax(0, 1fr) 78px minmax(0, 1fr);
    align-items: center;
  }
}

.editor-inline-label {
  display: flex;
  align-items: center;
  min-height: 44px;
  color: var(--app-text-faint);
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.editor-inline-content {
  min-height: 44px;
}
</style>
