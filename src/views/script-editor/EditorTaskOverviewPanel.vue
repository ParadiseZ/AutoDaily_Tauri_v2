  <template>
    <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar space-y-2">
      <div v-if="task?.rowType === 'task'">
        <!-- Card 1: 基本属性 -->
        <section class="rounded-[16px] border border-(--app-border) bg-(--app-panel) shadow-[0_4px_12px_rgba(15,23,42,0.03)] px-5 py-5">
          <div class="border-b border-(--app-border) pb-3 mb-4">
            <h1 class="text-sm font-semibold text-(--app-text-strong)">基本属性</h1>
          </div>
          <div class="grid gap-4 sm:grid-cols-2">
            <!-- 任务名称 -->
            <div class="flex flex-col gap-1.5">
              <span class="text-xs font-semibold text-(--app-text-soft) tracking-wide">任务名称</span>
              <input
                  :value="taskName"
                  class="app-input"
                  type="text"
                  data-testid="editor-task-name"
                  @input="$emit('update:task-name', ($event.target as HTMLInputElement).value)"
              />
            </div>
            <!-- 所属分组 -->
            <div class="flex flex-col gap-1.5">
              <span class="text-xs font-semibold text-(--app-text-soft) tracking-wide">所属分组</span>
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
        <section class="rounded-[16px] border border-(--app-border) bg-(--app-panel) shadow-[0_4px_12px_rgba(15,23,42,0.03)] px-5 py-5">
          <div class="border-b border-(--app-border) pb-3 mb-4">
            <h3 class="text-sm font-semibold text-(--app-text-strong)">调度与限制</h3>
          </div>
          <div class="grid gap-4 sm:grid-cols-2">
            <!-- 进入方式 -->
            <div class="flex flex-col gap-1.5">
              <span class="text-xs font-semibold text-(--app-text-soft) tracking-wide">进入方式</span>
              <EditorSelectField
                  :model-value="taskTriggerMode"
                  :options="taskTriggerModeOptions"
                  placeholder="选择进入方式"
                  test-id="editor-task-trigger-mode"
                  @update:model-value="$emit('update:task-trigger-mode', $event as 'rootOnly' | 'linkOnly' | 'rootAndLink')"
              />
            </div>
            <!-- 运行周期 -->
            <div class="flex flex-col gap-1.5">
              <span class="text-xs font-semibold text-(--app-text-soft) tracking-wide">运行周期</span>
              <div class="flex items-center gap-2">
                <EditorSelectField
                    :model-value="defaultTaskCycleValue"
                    :options="taskCycleOptions"
                    placeholder="选择运行周期"
                    test-id="editor-task-cycle"
                    class="flex-1"
                    @update:model-value="$emit('update:default-task-cycle-value', String($event || 'everyRun'))"
                />
                <input
                    v-if="defaultTaskCycleMode === 'weekDay' || defaultTaskCycleMode === 'monthDay'"
                    :value="defaultTaskCycleDay"
                    class="app-input max-w-[80px] shrink-0"
                    type="number"
                    :min="defaultTaskCycleMode === 'weekDay' ? 1 : 1"
                    :max="defaultTaskCycleMode === 'weekDay' ? 7 : 31"
                    data-testid="editor-task-cycle-day"
                    @input="$emit('update:default-task-cycle-day', Number(($event.target as HTMLInputElement).value || 1))"
                />
              </div>
            </div>
            <!-- 最大次数 -->
            <div class="flex flex-col gap-1.5">
              <span class="text-xs font-semibold text-(--app-text-soft) tracking-wide">最大次数</span>
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
            <div class="flex flex-col gap-1.5">
              <span class="text-xs font-semibold text-(--app-text-soft) tracking-wide">记录选项</span>
              <label class="flex items-center gap-3 rounded-[12px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-2.5 text-sm text-(--app-text-soft) hover:bg-white/40 cursor-pointer transition-colors">
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
        <section class="rounded-[16px] border border-(--app-border) bg-(--app-panel) shadow-[0_4px_12px_rgba(15,23,42,0.03)] px-5 py-5">
          <div class="border-b border-(--app-border) pb-3 mb-4">
            <h3 class="text-sm font-semibold text-(--app-text-strong)">UI 显示行为</h3>
          </div>
          <div class="grid gap-3 sm:grid-cols-2">
            <!-- 任务提醒 -->
            <div class="flex flex-col gap-1.5">
              <span class="text-xs font-semibold text-(--app-text-soft) tracking-wide">任务提醒</span>
              <EditorSelectField
                  :model-value="taskTone"
                  :options="taskToneOptions"
                  placeholder="选择提醒等级"
                  test-id="editor-task-tone"
                  @update:model-value="$emit('update:task-tone', $event as 'normal' | 'warning' | 'danger')"
              />
            </div>
            <!-- 缩进量 -->
            <div class="flex flex-col gap-1.5">
              <span class="text-xs font-semibold text-(--app-text-soft) tracking-wide">缩进量</span>
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
            <!-- 显示开关 -->
            <label class="flex items-center gap-3 rounded-[12px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3 text-sm text-(--app-text-soft) hover:bg-white/40 cursor-pointer transition-colors">
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
            <label class="flex items-center gap-3 rounded-[12px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3 text-sm text-(--app-text-soft) hover:bg-white/40 cursor-pointer transition-colors">
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
        </section>
      </div>

      <div v-else>
        <!-- Card 1: 基本属性 -->
        <section class="rounded-[16px] border border-(--app-border) bg-(--app-panel) shadow-[0_4px_12px_rgba(15,23,42,0.03)] px-5 py-5">
          <div class="border-b border-(--app-border) pb-3 mb-4">
            <h1 class="text-sm font-semibold text-(--app-text-strong)">基本属性</h1>
          </div>
          <div class="grid gap-4 sm:grid-cols-2">
            <!-- 任务名称 -->
            <div class="flex flex-col gap-1.5">
              <span class="text-xs font-semibold text-(--app-text-soft) tracking-wide">任务名称</span>
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
  background: var(--app-panel-muted);
  padding: 0.75rem 0.9rem;
  color: var(--app-text-soft);
  font-size: 0.92rem;
}
</style>
