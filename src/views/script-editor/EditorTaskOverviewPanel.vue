<template>
    <div class="overview-scroll custom-scrollbar">
      <div v-if="task?.rowType === 'task'">
        <!-- Card 1: 基本属性 -->
        <section class="overview-card">
          <div class="overview-card-header">
            <h1 class="overview-card-title">基本属性</h1>
          </div>
          <div class="overview-stack">
            <!-- 任务名称 -->
            <div class="overview-field-2">
              <span class="overview-label">任务名称</span>
              <input
                  :value="taskName"
                  class="app-input"
                  type="text"
                  data-testid="editor-task-name"
                  @input="$emit('update:task-name', ($event.target as HTMLInputElement).value)"
              />
            </div>
            <!-- 所属分组 -->
            <div class="overview-field-2">
              <span class="overview-label">所属分组</span>
              <EditorSelectField
                  :model-value="sectionId"
                  :options="titleOptions"
                  placeholder="未分组"
                  test-id="editor-task-section"
                  @update:model-value="$emit('update:section-id', ($event as string | null) ?? null)"
              />
            </div>
          </div>
        </section>

        <!-- Card 2: 调度与限制 -->
        <section class="overview-card">
          <div class="overview-card-header">
            <h3 class="overview-card-title">调度与限制</h3>
          </div>
          <div class="overview-stack">
            <!-- 进入方式 -->
            <div class="overview-field-1">
              <span class="overview-label">进入方式</span>
              <EditorSelectField
                  :model-value="taskTriggerMode"
                  :options="taskTriggerModeOptions"
                  placeholder="选择进入方式"
                  test-id="editor-task-trigger-mode"
                  @update:model-value="$emit('update:task-trigger-mode', $event as 'rootOnly' | 'linkOnly' | 'rootAndLink')"
              />
            </div>
            <!-- 运行周期 -->
            <div class="overview-field-2">
              <span class="overview-label">运行周期</span>
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
            </div>
            <!-- 最大次数 -->
            <div class="overview-field-2">
              <span class="overview-label">最大次数</span>
              <input
                  :value="taskExecMax"
                  class="app-input"
                  type="number"
                  min="0"
                  data-testid="editor-task-exec-max"
                  @input="$emit('update:task-exec-max', Number(($event.target as HTMLInputElement).value || 0))"
              />
            </div>
            <!-- 记录调度 -->
            <div class="overview-field-2">
              <span class="overview-label">记录选项</span>
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
            </div>
          </div>
        </section>


        <!-- Card 3: UI 显示行为 -->
        <section class="overview-card">
          <div class="overview-card-header">
            <h3 class="overview-card-title">UI 显示行为</h3>
          </div>
          <div class="overview-stack">
            <!-- 任务提醒 -->
            <div class="overview-field-1">
              <span class="overview-label">任务提醒</span>
              <EditorSelectField
                  :model-value="taskTone"
                  :options="taskToneOptions"
                  placeholder="选择提醒等级"
                  test-id="editor-task-tone"
                  @update:model-value="$emit('update:task-tone', $event as 'normal' | 'warning' | 'danger')"
              />
            </div>
            <!-- 缩进量 -->
            <div class="overview-field-2">
              <span class="overview-label">缩进量</span>
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
            <div class="overview-field">
              <span class="overview-label">显示选项</span>
              <div class="overview-toggle-list">
                <!-- 显示开关 -->
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
                <!-- 是否启用 -->
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
            </div>
          </div>
        </section>
      </div>

      <div v-else>
        <!-- Card 1: 基本属性 -->
        <section class="overview-card">
          <div class="overview-card-header">
            <h1 class="overview-card-title">基本属性</h1>
          </div>
          <div class="overview-stack">
            <!-- 任务名称 -->
            <div class="overview-field-2">
              <span class="overview-label">任务名称</span>
              <input
                  :value="taskName"
                  class="app-input"
                  type="text"
                  data-testid="editor-task-name"
                  @input="$emit('update:task-name', ($event.target as HTMLInputElement).value)"
              />
            </div>
          </div>
        </section>
      </div>
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
@reference "../../style.css";

.overview-scroll {
  @apply min-h-0 flex-1 space-y-2 overflow-y-auto pr-1;
}

.overview-card {
  @apply rounded-[16px] border border-(--app-border) bg-(--app-panel) px-5 py-5 shadow-[0_4px_12px_rgba(15,23,42,0.03)];
}

.overview-card-header {
  @apply mb-4 border-b border-(--app-border) pb-3;
}

.overview-card-title {
  @apply text-sm font-semibold text-(--app-text-strong);
}

.overview-stack {
  @apply flex w-full max-w-[38rem] flex-col gap-4;
}

.overview-field-1 {
  @apply flex w-full flex-col gap-1.5;
}
.overview-field-2 {
  @apply flex max-w-[16rem] flex-col gap-1.5;
}

.overview-label {
  @apply text-xs font-semibold tracking-wide text-(--app-text-soft);
}

.overview-toggle-list {
  @apply flex flex-col gap-3 max-w-[16rem];
}

.overview-toggle {
  @apply flex items-center gap-3 rounded-[12px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3 text-sm text-(--app-text-soft) transition-colors hover:bg-white/40;
  cursor: pointer;
}
</style>
