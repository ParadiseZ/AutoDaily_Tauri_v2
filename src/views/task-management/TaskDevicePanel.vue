<template>
  <SurfacePanel class="flex h-full min-h-0 flex-col gap-5 overflow-hidden">
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div class="space-y-2">
        <div class="flex flex-wrap gap-2 text-sm text-(--app-text-soft)">
          <span>{{ formatPlatformLabel(device.data.platform) }}</span>
          <span>·</span>
          <span>{{ formatConnectLabel(device.data) }}</span>
          <span>·</span>
          <span>{{ taskConnectionBadge.label }}</span>
          <span>·</span>
          <span>{{ formatCaptureMethod(device.data.capMethod) }}</span>
          <span v-if="runtimeView.status.currentScript">· 正在执行 {{ runtimeView.status.currentScript }}</span>
        </div>
        <p v-if="runtimeView.pendingMessage" class="text-sm font-medium text-(--app-accent)">{{ runtimeView.pendingMessage }}</p>
        <p v-else-if="runtimeView.connectionStatus.message" class="text-sm text-(--app-text-faint)">{{ runtimeView.connectionStatus.message }}</p>
      </div>

      <div v-if="showRuntimeActionButton" class="flex flex-wrap gap-2">
        <button v-if="!runtimeView.controls.showStopButton" class="app-button app-button-primary shadow-md shadow-blue-500/20" type="button" :disabled="deviceBusy" @click="$emit('start', device.id)">
          <AppIcon name="play" :size="16" class="fill-current" />
          运行
        </button>
        <button v-else class="app-button app-button-warning" type="button" :disabled="deviceBusy" @click="$emit('stop', device.id)">
          <AppIcon name="square" :size="14" class="fill-current" />
          停止
        </button>
      </div>
    </div>

    <div class="grid gap-3 xl:grid-cols-3">
      <div class="runtime-result-block">
        <div class="flex items-center justify-between gap-2">
          <p class="text-xs font-semibold text-(--app-text-faint)">当前进度</p>
          <StatusBadge :label="runtimeView.progress.label" :tone="runtimeView.progress.tone" />
        </div>
        <p class="mt-2 line-clamp-2 text-sm text-(--app-text-strong)">
          {{ runtimeView.progress.message || '暂无进度事件' }}
        </p>
      </div>

      <div class="runtime-result-block">
        <div class="flex items-center justify-between gap-2">
          <p class="text-xs font-semibold text-(--app-text-faint)">最后结果</p>
          <StatusBadge :label="runtimeScheduleLabel" :tone="runtimeScheduleTone" />
        </div>
        <p class="mt-2 line-clamp-2 text-sm text-(--app-text-strong)">
          {{ runtimeResult.latestSchedule?.message || runtimeResult.latestSchedule?.status || '暂无调度结果' }}
        </p>
      </div>

      <div class="runtime-result-block" :class="runtimeResult.latestTimeout ? 'runtime-result-block-warning' : ''">
        <div class="flex items-center justify-between gap-2">
          <p class="text-xs font-semibold text-(--app-text-faint)">Timeout</p>
          <StatusBadge :label="runtimeTimeoutLabel" :tone="runtimeTimeoutTone" />
        </div>
        <p class="mt-2 line-clamp-2 text-sm text-(--app-text-strong)">
          {{ runtimeTimeoutSummary }}
        </p>
      </div>
    </div>

    <div class="grid min-h-0 flex-1 gap-4 xl:grid-cols-[1.35fr_0.95fr]">
      <SurfacePanel tone="muted" padding="sm" class="flex min-h-0 flex-col gap-4 overflow-hidden">
        <div class="editor-panel-tabs min-w-max">
          <button
            v-for="tab in modeTabs"
            :key="tab.id"
            type="button"
            class="editor-panel-tab"
            :class="{ 'editor-panel-tab-active': activeMode === tab.id }"
            @click="activeMode = tab.id"
          >
            {{ tab.label }}
          </button>
        </div>

        <template v-if="activeMode === 'queue'">
          <div class="flex min-h-0 flex-1 flex-col gap-4 overflow-hidden">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-sm font-semibold text-(--app-text-strong)">待运行队列</p>
              </div>
              <StatusBadge :label="`${queuedAssignments.length} 条`" tone="neutral" />
            </div>

            <div class="grid gap-3 lg:grid-cols-[1fr_220px_auto]">
              <AppSelect v-model="selectedScriptId" :options="scriptOptions" placeholder="选择要追加的脚本" />
              <AppSelect v-model="selectedTemplateId" :options="templateOptions" placeholder="选择时间模板" />
              <button class="app-button app-button-ghost group" type="button" @click="handleAddScript">
                <AppIcon name="list-plus" :size="16" class="text-(--app-text-faint) group-hover:text-(--app-accent) transition-colors" />
                追加
              </button>
            </div>
            <p v-if="!scriptOptions.length" class="text-xs text-(--app-text-faint)">
              没有匹配脚本：{{ formatPlatformLabel(device.data.platform) }}
            </p>
            <p v-else-if="selectedScriptId && !selectedTemplateId" class="text-xs text-(--app-text-faint)">
              追加到总队列前，必须选择一个真实时间模板。
            </p>

            <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
              <div v-if="loadingAssignments" class="py-10 text-sm text-(--app-text-soft)">正在读取队列...</div>
              <div v-else-if="queuedAssignments.length === 0" class="rounded-[20px] border border-dashed border-(--app-border) p-6 text-sm text-(--app-text-soft)">
                当前没有待运行队列项。
              </div>
              <div v-else class="space-y-2">
                <div
                  v-for="assignment in queuedAssignments"
                  :key="assignment.id"
                  class="rounded-[18px] border bg-white/20 px-4 py-3"
                  :class="assignmentWarning(assignment) ? 'border-amber-300/70 bg-amber-50/70 dark:border-amber-500/30 dark:bg-amber-500/10' : 'border-(--app-border)'"
                >
                  <div class="flex items-center gap-3">
                    <div class="flex h-8 w-8 items-center justify-center rounded-full bg-(--app-accent-soft) text-xs font-semibold text-(--app-accent)">
                      {{ assignment.index + 1 }}
                    </div>
                    <div class="min-w-0 flex-1">
                      <div class="flex items-center gap-2">
                        <p class="truncate text-sm font-medium text-(--app-text-strong)">{{ getScriptName(assignment.scriptId) }}</p>
                        <span
                          v-if="assignmentWarning(assignment)"
                          class="inline-flex shrink-0 items-center rounded-full bg-amber-100 px-2 py-0.5 text-[11px] font-medium text-amber-700 dark:bg-amber-500/15 dark:text-amber-300"
                        >
                          最近超时
                        </span>
                      </div>
                      <p class="text-xs text-(--app-text-faint)">{{ getTemplateName(assignment.timeTemplateId) }}</p>
                    </div>
                    <button
                      class="app-button app-button-ghost h-8 px-3 text-sm group"
                      type="button"
                      :disabled="!assignment.timeTemplateId"
                      :title="assignment.timeTemplateId ? '打开模板变量设置' : '请先为脚本选择时间模板'"
                      @click="$emit('openAssignmentSettings', assignment)"
                    >
                      <AppIcon name="edit-3" :size="14" class="opacity-70 transition-opacity group-hover:opacity-100" />
                    </button>
                    <button class="app-button app-button-danger h-8 px-3 text-sm group" type="button" @click="$emit('removeAssignment', device.id, assignment)">
                      <AppIcon name="trash-2" :size="14" class="opacity-60 transition-opacity group-hover:opacity-100" />
                    </button>
                  </div>
                  <p v-if="assignmentWarning(assignment)" class="mt-2 text-xs text-amber-700 dark:text-amber-300">
                    {{ assignmentWarning(assignment) }}
                  </p>
                </div>
              </div>
            </div>

            <div class="flex max-h-60 min-h-0 flex-col space-y-3 rounded-[18px] border border-dashed border-(--app-border) px-4 py-4">
              <div class="flex items-center justify-between gap-3">
                <div>
                  <p class="text-sm font-semibold text-(--app-text-strong)">本日已到期</p>
                </div>
                <span class="rounded-full border border-(--app-border) px-3 py-1 text-xs text-(--app-text-faint)">
                  {{ todayDueAssignments.length }} 条
                </span>
              </div>

              <div v-if="!todayDueAssignments.length" class="text-sm text-(--app-text-soft)">
                当前没有已到期的队列项。
              </div>

              <div v-else class="min-h-0 space-y-2 overflow-y-auto pr-1 custom-scrollbar">
                <div
                  v-for="assignment in todayDueAssignments"
                  :key="assignment.id"
                  class="rounded-[16px] border border-amber-300/70 bg-amber-50/70 px-4 py-3 dark:border-amber-500/30 dark:bg-amber-500/10"
                >
                  <div class="flex items-center justify-between gap-3">
                    <div class="flex min-w-0 flex-1 items-center gap-3">
                      <div class="flex h-8 w-8 items-center justify-center rounded-full bg-amber-500/12 text-xs font-semibold text-amber-700 dark:text-amber-300">
                        {{ assignment.index + 1 }}
                      </div>
                      <div class="min-w-0">
                        <p class="truncate text-sm font-medium text-(--app-text-strong)">{{ getScriptName(assignment.scriptId) }}</p>
                        <p class="mt-1 text-xs text-(--app-text-faint)">{{ getTemplateName(assignment.timeTemplateId) }}</p>
                      </div>
                    </div>
                    <div class="flex shrink-0 items-center gap-2">
                      <span class="rounded-full bg-amber-500/12 px-3 py-1 text-xs text-amber-700 dark:text-amber-300">
                        已到期
                      </span>
                      <button
                        class="app-button app-button-ghost h-8 px-3 text-sm group"
                        type="button"
                        :disabled="!assignment.timeTemplateId"
                        :title="assignment.timeTemplateId ? '打开模板变量设置' : '请先为脚本选择时间模板'"
                        @click="$emit('openAssignmentSettings', assignment)"
                      >
                        <AppIcon name="edit-3" :size="14" class="opacity-70 transition-opacity group-hover:opacity-100" />
                      </button>
                      <button class="app-button app-button-danger h-8 px-3 text-sm group" type="button" @click="$emit('removeAssignment', device.id, assignment)">
                        <AppIcon name="trash-2" :size="14" class="opacity-60 transition-opacity group-hover:opacity-100" />
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </template>

        <template v-else>
          <div class="grid min-h-0 flex-1 gap-4 lg:grid-cols-[220px_minmax(0,1fr)]">
            <div class="min-h-0 space-y-2 overflow-y-auto pr-1 custom-scrollbar">
              <div>
                <p class="text-sm font-semibold text-(--app-text-strong)">临时运行</p>
              </div>

              <div v-if="!temporaryScriptItems.length" class="rounded-[18px] border border-dashed border-(--app-border) px-4 py-6 text-sm text-(--app-text-soft)">
                当前没有可用脚本。
              </div>

              <button
                v-for="script in temporaryScriptItems"
                :key="script.id"
                type="button"
                class="temporary-script-item"
                :class="{ 'temporary-script-item-active': selectedTemporaryScriptId === script.id }"
                @click="selectedTemporaryScriptId = script.id"
              >
                {{ script.name }}
              </button>
            </div>

            <div class="flex min-h-0 flex-col space-y-4 rounded-[18px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4 overflow-hidden">
              <div class="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <p class="text-sm font-semibold text-(--app-text-strong)">
                    {{ selectedTemporaryScriptName || '选择左侧脚本' }}
                  </p>
                </div>

                <div class="flex flex-wrap gap-2">
                  <button class="app-button app-button-ghost group" type="button" :disabled="deviceBusy || !selectedTemporaryScriptId" @click="handleRunTemporaryScript">
                    <AppIcon
                      name="file-play"
                      :size="16"
                      class="text-(--app-text-faint) group-hover:text-(--app-accent) transition-colors"
                    />
                    运行脚本
                  </button>
                  <button class="app-button app-button-ghost group" type="button" :disabled="deviceBusy || !selectedTemporaryTaskId" @click="handleRunTemporaryTask">
                    <AppIcon
                      name="list-checks"
                      :size="16"
                      class="text-(--app-text-faint) group-hover:text-(--app-accent) transition-colors"
                    />
                    运行任务
                  </button>
                </div>
              </div>

              <p v-if="temporaryWarningMessage" class="text-xs text-amber-700 dark:text-amber-300">
                最近超时：{{ temporaryWarningMessage }}
              </p>

              <div v-if="!selectedTemporaryScriptId" class="rounded-[18px] border border-dashed border-(--app-border) px-4 py-6 text-sm text-(--app-text-soft)">
                请选择脚本。
              </div>
              <div v-else-if="props.scriptTaskLoading[selectedTemporaryScriptId]" class="py-10 text-sm text-(--app-text-soft)">
                正在加载任务列表...
              </div>
              <div v-else-if="!temporaryRows.length" class="rounded-[18px] border border-dashed border-(--app-border) px-4 py-6 text-sm text-(--app-text-soft)">
                当前脚本没有任务。
              </div>
              <div v-else class="min-h-0 flex-1 space-y-2 overflow-y-auto pr-1 custom-scrollbar">
                <template v-for="row in temporaryRows" :key="row.id">
                  <div v-if="row.rowType === 'title'" class="rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3 text-sm font-semibold text-(--app-text-strong)">
                    {{ row.name }}
                  </div>
                  <div
                    v-else
                    class="temporary-task-row w-full text-left"
                    :class="{ 'temporary-task-row-active': selectedTemporaryTaskId === row.id }"
                    role="button"
                    tabindex="0"
                    @click="selectedTemporaryTaskId = row.id"
                    @keydown.enter.prevent="selectedTemporaryTaskId = row.id"
                    @keydown.space.prevent="selectedTemporaryTaskId = row.id"
                  >
                    <div class="flex items-center justify-between gap-3">
                      <div class="min-w-0">
                        <p class="truncate text-sm font-medium text-(--app-text-strong)">{{ row.name }}</p>
                        <p class="mt-1 text-xs text-(--app-text-faint)">Task</p>
                      </div>
                      <button class="app-button app-button-ghost h-8 px-3 text-sm" type="button" :disabled="deviceBusy" @click.stop="runSpecificTemporaryTask(row.id)">
                        运行
                      </button>
                    </div>
                  </div>
                </template>
              </div>
            </div>
          </div>
        </template>
      </SurfacePanel>

      <SurfacePanel tone="muted" padding="sm" class="flex min-h-0 flex-col gap-4 overflow-hidden">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-semibold text-(--app-text-strong)">最近运行记录</p>
          </div>
          <button class="app-button app-button-ghost h-10 px-4" type="button" @click="$emit('clearSchedules', device.id)">
            清空
          </button>
        </div>

        <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
          <div v-if="loadingSchedules" class="py-10 text-sm text-(--app-text-soft)">正在读取记录...</div>
          <div v-else-if="assignmentScheduleSections.length === 0" class="rounded-[20px] border border-dashed border-(--app-border) p-6 text-sm text-(--app-text-soft)">
            暂无运行记录。
          </div>
          <div v-else class="space-y-4">
          <section v-for="section in assignmentScheduleSections" :key="section.day" class="space-y-2">
            <div class="text-xs font-semibold uppercase tracking-[0.16em] text-(--app-text-faint)">
              {{ section.label }}
            </div>
            <div
              v-for="record in section.items"
              :key="record.id"
              class="rounded-[18px] border border-(--app-border) bg-white/20 px-4 py-3"
            >
              <button
                class="flex w-full items-start justify-between gap-3 text-left"
                type="button"
                @click="toggleAssignmentSchedule(record.id)"
              >
                <div class="min-w-0">
                  <p class="truncate text-sm font-medium text-(--app-text-strong)">{{ getScriptName(record.scriptId ?? '') }}</p>
                  <p class="mt-1 text-xs text-(--app-text-faint)">
                    {{ formatAssignmentScheduleMeta(record) }}
                  </p>
                </div>
                <div class="flex shrink-0 items-center gap-2">
                  <StatusBadge :label="formatAssignmentScheduleStatus(record.status)" :tone="assignmentScheduleTone(record.status)" />
                  <span class="text-xs text-(--app-text-faint)">{{ expandedAssignmentSchedules[record.id] ? '收起' : '展开' }}</span>
                </div>
              </button>

              <div v-if="expandedAssignmentSchedules[record.id]" class="mt-3 space-y-2 border-t border-(--app-border) pt-3">
                <div v-if="childSchedulesForAssignmentSchedule(record).length === 0" class="text-xs text-(--app-text-soft)">
                  暂无子进程调度记录。
                </div>
                <div
                  v-for="schedule in childSchedulesForAssignmentSchedule(record)"
                  :key="schedule.id"
                  class="rounded-[14px] bg-black/5 px-3 py-2 dark:bg-white/5"
                >
                  <div class="flex items-center justify-between gap-3">
                    <p class="truncate text-xs font-medium text-(--app-text-strong)">{{ getTaskName(schedule.scriptId, schedule.taskId) }}</p>
                    <StatusBadge :label="historyScheduleStatusLabels[schedule.status] ?? schedule.status" :tone="historyScheduleTone(schedule.status)" />
                  </div>
                  <p class="mt-1 text-xs text-(--app-text-faint)">
                    {{ formatDateTime(schedule.startedAt) }} · {{ schedule.message || schedule.taskCycle }}
                  </p>
                </div>
              </div>
            </div>
          </section>
          </div>
        </div>
      </SurfacePanel>
    </div>
  </SurfacePanel>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import StatusBadge from '@/components/shared/StatusBadge.vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import type {
  AssignmentRecord,
  AssignmentSchedule,
  DeviceRuntimeView,
  DeviceScriptSchedule,
  RuntimeProgressEvent,
  RuntimeResultProjection,
  RuntimeTimeoutEvent,
  ScriptTableRecord,
} from '@/types/app/domain';
import type { AssignmentScheduleStatus } from '@/types/bindings/AssignmentScheduleStatus';
import type { RunTarget } from '@/types/bindings/RunTarget';
import type { AssignmentTriggerSource } from '@/types/bindings/AssignmentTriggerSource';
import type { DeviceTable } from '@/types/bindings/DeviceTable';
import type { RunStatus } from '@/types/bindings/RunStatus';
import type { RuntimeScheduleStatus } from '@/types/bindings/RuntimeScheduleStatus';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { TimeTemplate } from '@/types/bindings/TimeTemplate';
import type { TimeoutAction } from '@/types/bindings/TimeoutAction';
import { filterUserVisibleTaskRows } from '@/utils/scriptTaskVisibility';
import { createFullScriptRunTarget, createTaskRunTarget } from '@/utils/runTarget';
import {
  formatCaptureMethod,
  formatConnectLabel,
  formatDateTime,
  formatPlatformLabel,
  formatTemplateWindow,
} from '@/utils/presenters';

const props = defineProps<{
  device: DeviceTable;
  runtimeView: DeviceRuntimeView;
  scripts: ScriptTableRecord[];
  timeTemplates: TimeTemplate[];
  assignments: AssignmentRecord[];
  assignmentSchedules: AssignmentSchedule[];
  schedules: DeviceScriptSchedule[];
  scriptTasksByScriptId: Record<string, ScriptTaskTable[]>;
  scriptTaskLoading: Record<string, boolean>;
  progressEvent: RuntimeProgressEvent | null;
  timeoutEvent: RuntimeTimeoutEvent | null;
  runtimeResult: RuntimeResultProjection;
  loadingAssignments: boolean;
  loadingSchedules: boolean;
  devicePendingAction: 'spawning' | 'starting' | 'pausing' | 'stopping' | 'shuttingDown' | 'restarting' | 'syncing' | null;
  deviceBusy: boolean;
}>();

const emit = defineEmits<{
  start: [deviceId: string];
  stop: [deviceId: string];
  addScript: [deviceId: string, scriptId: string, timeTemplateId: string | null];
  ensureScriptTasks: [scriptId: string];
  runTarget: [deviceId: string, target: RunTarget];
  openAssignmentSettings: [assignment: AssignmentRecord];
  removeAssignment: [deviceId: string, assignment: AssignmentRecord];
  clearSchedules: [deviceId: string];
}>();

const modeTabs = [
  { id: 'queue', label: '队列任务' },
  { id: 'temporary', label: '临时运行' },
] as const;

const activeMode = ref<'queue' | 'temporary'>('queue');
const selectedScriptId = ref('');
const selectedTemplateId = ref('');
const selectedTemporaryScriptId = ref('');
const selectedTemporaryTaskId = ref('');
const expandedAssignmentSchedules = ref<Record<string, boolean>>({});
const nowTick = ref(Date.now());
const timer = window.setInterval(() => {
  nowTick.value = Date.now();
}, 60_000);

const devicePlatform = computed(() => props.device.data.platform ?? 'android');

const scriptOptions = computed(() =>
  props.scripts
    .filter((script) => (script.data.platform ?? 'android') === devicePlatform.value)
    .map((script) => ({
      label: script.data.name,
      value: script.id,
      description: script.data.description || formatPlatformLabel(script.data.platform),
    })),
);

const temporaryScriptItems = computed(() =>
  props.scripts
    .filter((script) => (script.data.platform ?? 'android') === devicePlatform.value)
    .map((script) => ({
      id: script.id,
      name: script.data.name,
    })),
);

const templateOptions = computed(() =>
  props.timeTemplates.map((template) => ({
    label: template.name,
    value: template.id,
    description: formatTemplateWindow(template),
  })),
);

const templateMap = computed(() =>
  Object.fromEntries(props.timeTemplates.map((template) => [template.id, template])),
);

const selectedTemporaryScriptName = computed(() =>
  temporaryScriptItems.value.find((script) => script.id === selectedTemporaryScriptId.value)?.name ?? '',
);

const taskConnectionBadge = computed(() => {
  if (!props.device.data.enable) {
    return { label: '未启用', tone: 'neutral' as const };
  }

  if (props.runtimeView.connectionStatus.kind === 'connected') {
    return { label: '已连接', tone: 'success' as const };
  }

  return { label: '未连接', tone: 'danger' as const };
});

const showRuntimeActionButton = computed(() => Boolean(props.device.data.enable));

const temporaryRows = computed(() =>
  filterUserVisibleTaskRows(props.scriptTasksByScriptId[selectedTemporaryScriptId.value] ?? [])
    .sort((left, right) => left.index - right.index),
);

const normalizeWarningMessage = (message?: string | null) => message?.trim() || null;

const timeoutActionLabels: Record<TimeoutAction, string> = {
  skipCurrentTask: '跳过任务',
  runRecoveryTask: '恢复任务',
  stopExecution: '停止执行',
};

const runtimeScheduleStatusLabels: Record<RuntimeScheduleStatus, string> = {
  queued: '已排队',
  running: '运行中',
  success: '成功',
  failed: '失败',
  skipped: '已跳过',
  cleared: '已清空',
};

const historyScheduleStatusLabels: Record<RunStatus, string> = {
  success: '成功',
  failed: '失败',
  skipped: '已跳过',
};

const assignmentScheduleStatusLabels: Record<AssignmentScheduleStatus, string> = {
  planned: '已计划',
  dispatched: '已派发',
  running: '运行中',
  success: '成功',
  failed: '失败',
  skipped: '已跳过',
  cancelled: '已取消',
  stopped: '已停止',
};

const triggerSourceLabels: Record<AssignmentTriggerSource, string> = {
  planner: '自动调度',
  user: '临时运行',
  debug: '调试',
};

const historyScheduleTone = (status: RunStatus) => {
  if (status === 'success') return 'success';
  if (status === 'skipped') return 'warning';
  if (status === 'failed') return 'danger';
  return 'neutral';
};

const runtimeScheduleToneFromStatus = (status: RuntimeScheduleStatus) => {
  if (status === 'success') return 'success';
  if (status === 'skipped' || status === 'queued' || status === 'running') return 'warning';
  if (status === 'failed') return 'danger';
  return 'neutral';
};

const assignmentScheduleTone = (status: AssignmentScheduleStatus) => {
  if (status === 'success') return 'success';
  if (status === 'planned' || status === 'dispatched' || status === 'running' || status === 'skipped') return 'warning';
  if (status === 'failed' || status === 'cancelled' || status === 'stopped') return 'danger';
  return 'neutral';
};

const formatAssignmentScheduleStatus = (status: AssignmentScheduleStatus) => assignmentScheduleStatusLabels[status];

const runtimeScheduleLabel = computed(() =>
  (props.runtimeResult.latestSchedule
    ? runtimeScheduleStatusLabels[props.runtimeResult.latestSchedule.status]
    : null) ?? '暂无',
);

const runtimeScheduleTone = computed(() => {
  const status = props.runtimeResult.latestSchedule?.status;
  return status ? runtimeScheduleToneFromStatus(status) : 'neutral';
});

const runtimeTimeoutLabel = computed(() => {
  if (!props.runtimeResult.latestTimeout) {
    return '未发生';
  }

  const action = props.runtimeResult.latestTimeout.timeoutAction;
  return action ? timeoutActionLabels[action] ?? action : '已触发';
});

const runtimeTimeoutTone = computed(() => {
  if (!props.runtimeResult.latestTimeout) return 'success';
  if (props.runtimeResult.timeoutActionResult === 'failed') return 'danger';
  if (props.runtimeResult.timeoutActionResult === 'pending') return 'warning';
  return 'info';
});

const runtimeTimeoutResultLabel = computed(() => {
  switch (props.runtimeResult.timeoutActionResult) {
    case 'skipped':
      return '已跳过当前任务';
    case 'recovered':
      return '已进入恢复任务';
    case 'stopped':
      return '已停止执行';
    case 'failed':
      return '动作后失败';
    case 'pending':
      return '等待后续结果';
    default:
      return '没有 timeout';
  }
});

const runtimeTimeoutSummary = computed(() => {
  const timeout = props.runtimeResult.latestTimeout;
  if (!timeout) {
    return '当前设备还没有 timeout 事件。';
  }

  return `${runtimeTimeoutResultLabel.value} · ${timeout.detail || timeout.message}`;
});

const buildTimeoutWarningMessage = (timeoutEvent: RuntimeTimeoutEvent | null) => {
  if (!timeoutEvent) {
    return null;
  }

  const reason = normalizeWarningMessage(timeoutEvent.message);
  if (!reason) {
    return null;
  }

  return `${formatDateTime(timeoutEvent.at)} · ${reason}`;
};

const assignmentWarning = (assignment: AssignmentRecord) => {
  const timeoutEvent = props.timeoutEvent;
  if (!timeoutEvent) {
    return null;
  }

  const matchesAssignment = timeoutEvent.assignmentId === assignment.id;
  const matchesScript = !timeoutEvent.assignmentId && timeoutEvent.scriptId === assignment.scriptId;
  if (!matchesAssignment && !matchesScript) {
    return null;
  }

  return buildTimeoutWarningMessage(timeoutEvent);
};

const temporaryWarningMessage = computed(() => {
  const timeoutEvent = props.timeoutEvent;
  if (!timeoutEvent || !selectedTemporaryScriptId.value) {
    return null;
  }

  if (timeoutEvent.scriptId !== selectedTemporaryScriptId.value) {
    return null;
  }

  if (selectedTemporaryTaskId.value && timeoutEvent.taskId && timeoutEvent.taskId !== selectedTemporaryTaskId.value) {
    return null;
  }

  return buildTimeoutWarningMessage(timeoutEvent);
});

const getScriptName = (scriptId: string) => {
  return props.scripts.find((script) => script.id === scriptId)?.data.name || '未知脚本';
};

const getTaskName = (scriptId: string, taskId: string) => {
  return props.scriptTasksByScriptId[scriptId]?.find((task) => task.id === taskId)?.name || '未知任务';
};

const getTemplateName = (templateId: string | null) => {
  return formatTemplateWindow(templateId ? templateMap.value[templateId] : null);
};

const assignmentScheduleTime = (record: AssignmentSchedule) =>
  record.startedAt || record.createdAt || record.windowStartAt || '';

const formatDayLabel = (day: string) => {
  if (day === currentDayKey.value) {
    return '今天';
  }
  return day;
};

const assignmentScheduleSections = computed(() => {
  const groups = new Map<string, AssignmentSchedule[]>();
  for (const record of props.assignmentSchedules) {
    const time = assignmentScheduleTime(record);
    const day = time ? scheduleDayKey(time) : '未记录日期';
    groups.set(day, [...(groups.get(day) ?? []), record]);
  }

  return [...groups.entries()]
    .sort(([left], [right]) => right.localeCompare(left))
    .map(([day, items]) => ({
      day,
      label: formatDayLabel(day),
      items: [...items].sort((left, right) => {
        const leftTime = Date.parse(assignmentScheduleTime(left)) || 0;
        const rightTime = Date.parse(assignmentScheduleTime(right)) || 0;
        return rightTime - leftTime || left.orderIndex - right.orderIndex;
      }),
    }));
});

const formatAssignmentScheduleMeta = (record: AssignmentSchedule) => {
  const source = triggerSourceLabels[record.triggerSource] ?? record.triggerSource;
  const time = assignmentScheduleTime(record);
  const timeText = time ? formatDateTime(time) : '未记录时间';
  const detail = record.message || (record.windowStartAt ? `窗口 ${formatDateTime(record.windowStartAt)}` : '');
  return [source, timeText, detail].filter(Boolean).join(' · ');
};

const toggleAssignmentSchedule = (recordId: string) => {
  expandedAssignmentSchedules.value = {
    ...expandedAssignmentSchedules.value,
    [recordId]: !expandedAssignmentSchedules.value[recordId],
  };
};

const parseTimeToMinutes = (value: string | null | undefined) => {
  if (!value || !/^\d{2}:\d{2}$/.test(value)) {
    return null;
  }
  const [hours, minutes] = value.split(':').map((item) => Number(item));
  if (!Number.isFinite(hours) || !Number.isFinite(minutes)) {
    return null;
  }
  return hours * 60 + minutes;
};

const isOverdueByTemplateWindow = (startMinute: number | null, endMinute: number | null, currentMinute: number) => {
  if (startMinute === null && endMinute === null) {
    return false;
  }

  if (startMinute !== null && endMinute !== null) {
    if (startMinute <= endMinute) {
      return currentMinute > endMinute;
    }
    return currentMinute > endMinute && currentMinute < startMinute;
  }

  if (endMinute !== null) {
    return currentMinute > endMinute;
  }

  return currentMinute >= (startMinute ?? 0);
};

const currentDayKey = computed(() => {
  const current = new Date(nowTick.value);
  return `${current.getFullYear()}-${String(current.getMonth() + 1).padStart(2, '0')}-${String(current.getDate()).padStart(2, '0')}`;
});

const currentMinuteOfDay = computed(() => {
  const current = new Date(nowTick.value);
  return current.getHours() * 60 + current.getMinutes();
});

const scheduleDayKey = (value: string) => {
  const date = new Date(value);
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
};

const childSchedulesForAssignmentSchedule = (record: AssignmentSchedule) => {
  const startedAt = record.startedAt ? Date.parse(record.startedAt) : null;
  const completedAt = record.completedAt ? Date.parse(record.completedAt) : null;
  const fallbackDay = scheduleDayKey(assignmentScheduleTime(record));

  return props.schedules
    .filter((schedule) => {
      if (record.assignmentId && schedule.assignmentId === record.assignmentId) {
        return true;
      }
      if (record.scriptId && schedule.scriptId !== record.scriptId) {
        return false;
      }
      const scheduleStartedAt = Date.parse(schedule.startedAt);
      if (!Number.isFinite(scheduleStartedAt)) {
        return false;
      }
      if (startedAt !== null && Number.isFinite(startedAt) && scheduleStartedAt < startedAt) {
        return false;
      }
      if (completedAt !== null && Number.isFinite(completedAt) && scheduleStartedAt > completedAt) {
        return false;
      }
      if (startedAt === null && completedAt === null) {
        return scheduleDayKey(schedule.startedAt) === fallbackDay;
      }
      return true;
    })
    .sort((left, right) => (Date.parse(left.startedAt) || 0) - (Date.parse(right.startedAt) || 0));
};

const ranAssignmentIdsToday = computed(() =>
  new Set(
    props.schedules
      .filter((schedule) => schedule.assignmentId && scheduleDayKey(schedule.startedAt) === currentDayKey.value)
      .map((schedule) => String(schedule.assignmentId)),
  ),
);

const todayDueAssignments = computed(() =>
  props.assignments
    .filter((assignment) => {
      if (!assignment.timeTemplateId || ranAssignmentIdsToday.value.has(assignment.id)) {
        return false;
      }

      const template = templateMap.value[assignment.timeTemplateId];
      if (!template) {
        return false;
      }

      const startMinute = parseTimeToMinutes(template.startTime);
      const endMinute = parseTimeToMinutes(template.endTime);
      return isOverdueByTemplateWindow(startMinute, endMinute, currentMinuteOfDay.value);
    })
    .sort((left, right) => left.index - right.index),
);

const todayDueAssignmentIds = computed(() => new Set(todayDueAssignments.value.map((assignment) => assignment.id)));

const queuedAssignments = computed(() =>
  props.assignments
    .filter((assignment) => !todayDueAssignmentIds.value.has(assignment.id))
    .sort((left, right) => left.index - right.index),
);

const handleRunTemporaryScript = () => {
  if (!selectedTemporaryScriptId.value) {
    return;
  }

  emit('runTarget', props.device.id, createFullScriptRunTarget(selectedTemporaryScriptId.value));
};

const handleRunTemporaryTask = () => {
  if (!selectedTemporaryScriptId.value || !selectedTemporaryTaskId.value) {
    return;
  }

  emit('runTarget', props.device.id, createTaskRunTarget(selectedTemporaryScriptId.value, selectedTemporaryTaskId.value));
};

const runSpecificTemporaryTask = (taskId: string) => {
  selectedTemporaryTaskId.value = taskId;
  handleRunTemporaryTask();
};

const handleAddScript = () => {
  emit('addScript', props.device.id, selectedScriptId.value, selectedTemplateId.value);
  selectedScriptId.value = '';
  selectedTemplateId.value = '';
};

watch(
  selectedTemporaryScriptId,
  (scriptId) => {
    if (!scriptId) {
      selectedTemporaryTaskId.value = '';
      return;
    }
    emit('ensureScriptTasks', scriptId);
  },
);

watch(
  temporaryScriptItems,
  (items) => {
    if (items.some((item) => item.id === selectedTemporaryScriptId.value)) {
      return;
    }
    selectedTemporaryScriptId.value = items[0]?.id ?? '';
  },
  { immediate: true },
);

watch(
  temporaryRows,
  (rows) => {
    if (rows.some((row) => row.id === selectedTemporaryTaskId.value && row.rowType === 'task')) {
      return;
    }
    selectedTemporaryTaskId.value = rows.find((row) => row.rowType === 'task')?.id ?? '';
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  window.clearInterval(timer);
});
</script>

<style scoped>
.temporary-script-item,
.temporary-task-row {
  border-radius: 16px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.7);
  padding: 0.85rem 1rem;
  transition: border-color 0.16s ease, background 0.16s ease, box-shadow 0.16s ease;
}

.temporary-script-item {
  width: 100%;
  text-align: left;
  font-size: 0.92rem;
  font-weight: 600;
  color: var(--app-text-strong);
}

.temporary-script-item-active,
.temporary-task-row-active {
  border-color: color-mix(in srgb, var(--app-accent) 34%, var(--app-border));
  background: color-mix(in srgb, var(--app-accent-soft) 58%, white);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--app-accent) 16%, transparent);
}

.runtime-result-block {
  min-height: 92px;
  border-radius: 18px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.58);
  padding: 0.9rem 1rem;
}

.runtime-result-block-warning {
  border-color: color-mix(in srgb, rgb(245 158 11) 42%, var(--app-border));
  background: color-mix(in srgb, rgb(254 243 199) 58%, white);
}
</style>
