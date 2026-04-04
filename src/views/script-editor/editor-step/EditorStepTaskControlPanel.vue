<template>
  <div class="grid gap-3 xl:grid-cols-2">
    <div class="space-y-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
      <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标</p>
      <div class="grid gap-3 xl:grid-cols-[78px_minmax(0,1fr)_78px_minmax(0,1fr)] xl:items-center">
        <div class="editor-inline-label">动作类型</div>
        <div>
          <EditorSelectField
            :model-value="selectedTaskControl.type"
            :options="taskControlTypeOptions"
            placeholder="动作类型"
            @update:model-value="$emit('update-type', String($event || TASK_CONTROL_TYPE.setState))"
          />
        </div>

        <div class="editor-inline-label">目标类型</div>
        <div>
          <EditorSelectField
            :model-value="selectedTaskControl.target.type"
            :options="stateTargetTypeOptions"
            placeholder="目标类型"
            @update:model-value="$emit('update-target-type', String($event || STATE_TARGET_TYPE.task))"
          />
        </div>

        <div class="editor-inline-label">目标 ID</div>
        <div class="xl:col-span-3">
          <input
            :value="selectedTaskControl.target.id"
            aria-label="目标 ID"
            class="app-input"
            @input="$emit('update-target-id', ($event.target as HTMLInputElement).value)"
          />
        </div>
      </div>
    </div>

    <div class="space-y-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
      <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">状态</p>
      <div class="grid gap-3 xl:grid-cols-[78px_minmax(0,1fr)] xl:items-center">
        <div class="editor-inline-label">状态类型</div>
        <div>
          <EditorSelectField
            :model-value="selectedTaskControl.status.type"
            :options="stateStatusTypeOptions"
            placeholder="状态类型"
            @update:model-value="$emit('update-status-type', String($event || STATE_STATUS_TYPE.done))"
          />
        </div>
      </div>
      <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] bg-white/45 px-4 py-3">
        <input
          :checked="Boolean(selectedTaskControl.status.value)"
          type="checkbox"
          class="h-4 w-4"
          style="accent-color: var(--app-accent)"
          @change="$emit('update-status-value', ($event.target as HTMLInputElement).checked)"
        />
        <span class="text-sm text-[var(--app-text-soft)]">状态值为真</span>
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import type { TaskControl } from '@/types/bindings/TaskControl';
import { STATE_STATUS_TYPE, STATE_TARGET_TYPE, TASK_CONTROL_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';

defineOptions({ name: 'EditorStepTaskControlPanel' });

defineProps<{
  selectedTaskControl: TaskControl;
  taskControlTypeOptions: Array<{ label: string; value: string; description: string }>;
  stateTargetTypeOptions: Array<{ label: string; value: string; description: string }>;
  stateStatusTypeOptions: Array<{ label: string; value: string; description: string }>;
}>();

defineEmits<{
  'update-type': [value: string];
  'update-target-type': [value: string];
  'update-target-id': [value: string];
  'update-status-type': [value: string];
  'update-status-value': [value: boolean];
}>();
</script>

<style scoped>
.editor-inline-label {
  color: var(--app-text-faint);
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}
</style>
