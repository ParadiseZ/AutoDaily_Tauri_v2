<template>
  <div v-if="task?.rowType === 'task'" class="rounded-[18px] border border-(--app-border) bg-(--app-panel-muted) px-5 py-5">
    <div class="mb-4">
      <p class="text-sm font-semibold text-(--app-text-strong)">任务行设置</p>
      <p class="mt-1 text-xs text-(--app-text-faint)">这里放运行入口、分组、缩进、默认周期和普通用户预览相关配置。</p>
    </div>

    <div class="grid gap-x-3 gap-y-3 lg:grid-cols-[72px_minmax(0,1fr)_72px_minmax(0,1fr)]">
      <div class="overview-label">任务名称</div>
      <div class="overview-content">
        <input
          :value="taskName"
          class="app-input"
          type="text"
          data-testid="editor-task-name"
          @input="$emit('update:task-name', ($event.target as HTMLInputElement).value)"
        />
      </div>

      <div class="overview-label" />
      <div class="overview-content" />

      <div class="overview-label">进入方式</div>
      <div class="overview-content">
        <EditorSelectField
          :model-value="taskTriggerMode"
          :options="taskTriggerModeOptions"
          placeholder="选择进入方式"
          test-id="editor-task-trigger-mode"
          @update:model-value="$emit('update:task-trigger-mode', $event as 'rootOnly' | 'linkOnly' | 'rootAndLink')"
        />
      </div>

      <div class="overview-label">所属分组</div>
      <div class="overview-content">
        <EditorSelectField
          :model-value="sectionId"
          :options="titleOptions"
          placeholder="未分组"
          test-id="editor-task-section"
          @update:model-value="$emit('update:section-id', ($event as string | null) ?? null)"
        />
      </div>

      <div class="overview-label">缩进量</div>
      <div class="overview-content">
        <input
          :value="indentLevel"
          class="app-input"
          type="number"
          min="0"
          max="8"
          data-testid="editor-task-indent-level"
          @input="$emit('update:indent-level', Number(($event.target as HTMLInputElement).value || 0))"
        />
      </div>

      <div class="overview-label">任务提醒</div>
      <div class="overview-content">
        <EditorSelectField
          :model-value="taskTone"
          :options="taskToneOptions"
          placeholder="选择提醒等级"
          test-id="editor-task-tone"
          @update:model-value="$emit('update:task-tone', $event as 'normal' | 'warning' | 'danger')"
        />
      </div>

      <div class="overview-label">默认周期</div>
      <div class="overview-content flex flex-wrap items-center gap-3">
        <EditorSelectField
          :model-value="defaultTaskCycleValue"
          :options="taskCycleOptions"
          placeholder="选择默认周期"
          test-id="editor-task-cycle"
          @update:model-value="$emit('update:default-task-cycle-value', String($event || 'everyRun'))"
        />
        <input
          v-if="defaultTaskCycleMode === 'weekDay' || defaultTaskCycleMode === 'monthDay'"
          :value="defaultTaskCycleDay"
          class="app-input max-w-[92px]"
          type="number"
          :min="defaultTaskCycleMode === 'weekDay' ? 1 : 1"
          :max="defaultTaskCycleMode === 'weekDay' ? 7 : 31"
          data-testid="editor-task-cycle-day"
          @input="$emit('update:default-task-cycle-day', Number(($event.target as HTMLInputElement).value || 1))"
        />
      </div>

      <div class="overview-label">最大次数</div>
      <div class="overview-content">
        <input
          :value="taskExecMax"
          class="app-input"
          type="number"
          min="0"
          data-testid="editor-task-exec-max"
          @input="$emit('update:task-exec-max', Number(($event.target as HTMLInputElement).value || 0))"
        />
      </div>

      <div class="overview-label">记录调度</div>
      <label class="overview-content overview-check">
        <input
          :checked="recordSchedule"
          type="checkbox"
          class="h-4 w-4 accent-(--app-accent)"
          data-testid="editor-task-record-schedule"
          @change="$emit('update:record-schedule', ($event.target as HTMLInputElement).checked)"
        />
        <span>执行或调度后记录结果</span>
      </label>

      <div class="overview-label">显示开关</div>
      <label class="overview-content overview-check">
        <input
          :checked="showEnabledToggle"
          type="checkbox"
          class="h-4 w-4 accent-(--app-accent)"
          data-testid="editor-task-show-enabled-toggle"
          @change="$emit('update:show-enabled-toggle', ($event.target as HTMLInputElement).checked)"
        />
        <span>普通用户预览中显示启用开关</span>
      </label>

      <div class="overview-label">默认启用</div>
      <label class="overview-content overview-check">
        <input
          :checked="defaultEnabled"
          type="checkbox"
          class="h-4 w-4 accent-(--app-accent)"
          data-testid="editor-task-default-enabled"
          @change="$emit('update:default-enabled', ($event.target as HTMLInputElement).checked)"
        />
        <span>该任务默认启用</span>
      </label>
    </div>
  </div>

  <div v-else class="rounded-[18px] border border-dashed border-(--app-border) bg-(--app-panel-muted) px-5 py-5 text-sm text-(--app-text-soft)">
    <div class="grid gap-3 md:grid-cols-[72px_minmax(0,1fr)] md:items-center">
      <div class="overview-label">标题名称</div>
      <div class="overview-content">
        <input
          :value="taskName"
          class="app-input"
          type="text"
          data-testid="editor-task-name"
          @input="$emit('update:task-name', ($event.target as HTMLInputElement).value)"
        />
      </div>
    </div>
    <p class="mt-4">标题行没有运行入口、分组、周期和启用开关配置。右侧任务概览工作区仅在普通任务行时显示这些设置。</p>
  </div>
</template>

<script setup lang="ts">
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { TaskTone } from '@/types/bindings/TaskTone';
import type { TaskTriggerMode } from '@/types/bindings/TaskTriggerMode';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import { taskCycleOptions, taskToneOptions, taskTriggerModeOptions } from '@/views/script-editor/editorTaskMeta';

defineOptions({ name: 'EditorTaskOverviewPanel' });

defineProps<{
  task: ScriptTaskTable | null;
  taskName: string;
  taskTriggerMode: TaskTriggerMode;
  recordSchedule: boolean;
  sectionId: string | null;
  indentLevel: number;
  defaultTaskCycleValue: string;
  defaultTaskCycleMode: 'named' | 'weekDay' | 'monthDay';
  defaultTaskCycleDay: number;
  taskExecMax: number;
  showEnabledToggle: boolean;
  defaultEnabled: boolean;
  taskTone: TaskTone;
  titleOptions: Array<{ label: string; value: string | null; description?: string; disabled?: boolean }>;
}>();

defineEmits<{
  'update:task-name': [value: string];
  'update:task-trigger-mode': [value: TaskTriggerMode];
  'update:record-schedule': [value: boolean];
  'update:section-id': [value: string | null];
  'update:indent-level': [value: number];
  'update:default-task-cycle-value': [value: string];
  'update:default-task-cycle-day': [value: number];
  'update:task-exec-max': [value: number];
  'update:show-enabled-toggle': [value: boolean];
  'update:default-enabled': [value: boolean];
  'update:task-tone': [value: TaskTone];
}>();
</script>

<style scoped>
.overview-label {
  display: flex;
  align-items: center;
  min-height: 44px;
  color: var(--app-text-faint);
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.overview-content {
  min-height: 44px;
}

.overview-check {
  display: flex;
  align-items: center;
  gap: 0.7rem;
  border-radius: 14px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.55);
  padding: 0.75rem 0.9rem;
  color: var(--app-text-soft);
  font-size: 0.92rem;
}
</style>
