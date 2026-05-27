<template>
  <EditorOverviewPanel>
    <div v-if="task?.rowType === 'task'">
      <EditorOverviewSection title="基本属性" heading-tag="h1">
        <EditorOverviewField label="任务名称" width="compact">
          <input
              :value="taskName"
              class="app-input"
              type="text"
              data-testid="editor-task-name"
              @input="$emit('update:task-name', ($event.target as HTMLInputElement).value)"
          />
        </EditorOverviewField>

        <EditorOverviewField label="所属分组" width="compact">
          <EditorSelectField
              :model-value="sectionId"
              :options="titleOptions"
              placeholder="未分组"
              test-id="editor-task-section"
              @update:model-value="$emit('update:section-id', ($event as string | null) ?? null)"
          />
        </EditorOverviewField>
      </EditorOverviewSection>

      <EditorOverviewSection title="调度与限制">
        <EditorOverviewField label="进入方式">
          <EditorSelectField
              :model-value="taskTriggerMode"
              :options="taskTriggerModeOptions"
              placeholder="选择进入方式"
              test-id="editor-task-trigger-mode"
              @update:model-value="$emit('update:task-trigger-mode', $event as 'rootOnly' | 'linkOnly' | 'rootAndLink')"
          />
        </EditorOverviewField>

        <EditorOverviewField label="运行周期" width="compact">
          <div class="flex flex-col gap-2">
            <EditorSelectField
                :model-value="defaultTaskCycleValue"
                :options="taskCycleOptions"
                placeholder="选择运行周期"
                test-id="editor-task-cycle"
                @update:model-value="$emit('update:default-task-cycle-value', String($event || 'everyRun'))"
            />
            <input
                v-if="defaultTaskCycleMode === 'weekDay' || defaultTaskCycleMode === 'monthDay'"
                :value="defaultTaskCycleDay"
                class="app-input max-w-[120px]"
                type="number"
                :min="defaultTaskCycleMode === 'weekDay' ? 1 : 1"
                :max="defaultTaskCycleMode === 'weekDay' ? 7 : 31"
                data-testid="editor-task-cycle-day"
                @input="$emit('update:default-task-cycle-day', Number(($event.target as HTMLInputElement).value || 1))"
            />
          </div>
        </EditorOverviewField>

        <EditorOverviewField label="最大次数" width="compact">
          <input
              :value="taskExecMax"
              class="app-input"
              type="number"
              min="0"
              data-testid="editor-task-exec-max"
              @input="$emit('update:task-exec-max', Number(($event.target as HTMLInputElement).value || 0))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="记录选项" width="compact">
          <label class="overview-toggle">
            <input
                :checked="recordSchedule"
                type="checkbox"
                class="h-4 w-4 accent-(--app-accent)"
                data-testid="editor-task-record-schedule"
                @change="$emit('update:record-schedule', ($event.target as HTMLInputElement).checked)"
            />
            <span>调度后记录结果</span>
          </label>
        </EditorOverviewField>
      </EditorOverviewSection>

      <EditorOverviewSection title="UI 显示行为">
        <EditorOverviewField label="任务提醒">
          <EditorSelectField
              :model-value="taskTone"
              :options="taskToneOptions"
              placeholder="选择提醒等级"
              test-id="editor-task-tone"
              @update:model-value="$emit('update:task-tone', $event as 'normal' | 'warning' | 'danger')"
          />
        </EditorOverviewField>

        <EditorOverviewField label="缩进量" width="compact">
          <input
              :value="indentLevel"
              class="app-input"
              type="number"
              min="0"
              max="8"
              data-testid="editor-task-indent-level"
              @input="$emit('update:indent-level', Number(($event.target as HTMLInputElement).value || 0))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="显示选项" width="compact">
          <div class="overview-toggle-list">
            <label class="overview-toggle">
              <input
                  :checked="showEnabledToggle"
                  type="checkbox"
                  class="h-4 w-4 accent-(--app-accent)"
                  data-testid="editor-task-show-enabled-toggle"
                  @change="$emit('update:show-enabled-toggle', ($event.target as HTMLInputElement).checked)"
              />
              <span>UI中显示启用开关</span>
            </label>
            <label class="overview-toggle">
              <input
                  :checked="defaultEnabled"
                  type="checkbox"
                  class="h-4 w-4 accent-(--app-accent)"
                  data-testid="editor-task-default-enabled"
                  @change="$emit('update:default-enabled', ($event.target as HTMLInputElement).checked)"
              />
              <span>默认启用任务</span>
            </label>
          </div>
        </EditorOverviewField>
      </EditorOverviewSection>
    </div>

    <div v-else>
      <EditorOverviewSection title="基本属性" heading-tag="h1">
        <EditorOverviewField label="任务名称" width="compact">
          <input
              :value="taskName"
              class="app-input"
              type="text"
              data-testid="editor-task-name"
              @input="$emit('update:task-name', ($event.target as HTMLInputElement).value)"
          />
        </EditorOverviewField>
      </EditorOverviewSection>
    </div>
  </EditorOverviewPanel>
</template>

<script setup lang="ts">
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { TaskTone } from '@/types/bindings/TaskTone';
import type { TaskTriggerMode } from '@/types/bindings/TaskTriggerMode';
import EditorOverviewField from '@/views/script-editor/EditorOverviewField.vue';
import EditorOverviewPanel from '@/views/script-editor/EditorOverviewPanel.vue';
import EditorOverviewSection from '@/views/script-editor/EditorOverviewSection.vue';
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
@reference "../../style.css";

.overview-toggle-list {
  @apply flex flex-col gap-3 max-w-[16rem];
}

.overview-toggle {
  @apply flex items-center gap-3 rounded-[12px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3 text-sm text-(--app-text-soft) transition-colors hover:bg-white/40;
  cursor: pointer;
}
</style>
