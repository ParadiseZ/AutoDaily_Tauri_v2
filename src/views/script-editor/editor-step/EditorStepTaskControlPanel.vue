<template>
  <div class="grid gap-3 xl:grid-cols-2">
    <div class="space-y-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
      <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标</p>
      <div class="grid gap-3">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">动作类型</span>
          <AppSelect
            :model-value="selectedTaskControl.type"
            :options="taskControlTypeOptions"
            placeholder="动作类型"
            @update:model-value="$emit('update-type', String($event || TASK_CONTROL_TYPE.setState))"
          />
        </label>
        <div class="grid gap-3 md:grid-cols-2">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标类型</span>
            <AppSelect
              :model-value="selectedTaskControl.target.type"
              :options="stateTargetTypeOptions"
              placeholder="目标类型"
              @update:model-value="$emit('update-target-type', String($event || STATE_TARGET_TYPE.task))"
            />
          </label>
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标 ID</span>
            <input :value="selectedTaskControl.target.id" class="app-input" @input="$emit('update-target-id', ($event.target as HTMLInputElement).value)" />
          </label>
        </div>
      </div>
    </div>

    <div class="space-y-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
      <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">状态</p>
      <label class="space-y-2">
        <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">状态类型</span>
        <AppSelect
          :model-value="selectedTaskControl.status.type"
          :options="stateStatusTypeOptions"
          placeholder="状态类型"
          @update:model-value="$emit('update-status-type', String($event || STATE_STATUS_TYPE.done))"
        />
      </label>
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
import AppSelect from '@/components/shared/AppSelect.vue';
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
