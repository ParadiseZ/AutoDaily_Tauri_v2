<template>
  <SurfacePanel class="space-y-5">
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div class="space-y-2">
        <div class="flex flex-wrap items-center gap-2">
          <h2 class="text-lg font-semibold text-(--app-text-strong)">{{ device.data.deviceName }}</h2>
          <StatusBadge :label="formatStatusLabel(status)" :tone="formatStatusTone(status.kind)" />
        </div>
        <div class="flex flex-wrap gap-2 text-sm text-(--app-text-soft)">
          <span>{{ formatPlatformLabel(device.data.platform) }}</span>
          <span>·</span>
          <span>{{ formatConnectLabel(device.data.adbConnect) }}</span>
          <span>·</span>
          <span>{{ formatCaptureMethod(device.data.capMethod) }}</span>
          <span v-if="status.currentScript">· 正在执行 {{ status.currentScript }}</span>
        </div>
        <p v-if="status.message" class="text-sm text-(--app-text-faint)">{{ status.message }}</p>
      </div>

      <div class="flex flex-wrap gap-2">
        <button class="app-button app-button-primary shadow-md shadow-blue-500/20" type="button" @click="$emit('start', device.id)">
          <AppIcon name="play" :size="16" class="fill-current" />
          运行队列
        </button>
        <button class="app-button app-button-ghost group" type="button" @click="$emit('pause', device.id)">
          <AppIcon name="pause" :size="16" class="text-(--app-text-faint) group-hover:text-(--app-text-strong) transition-colors" />
          暂停
        </button>
        <button class="app-button app-button-warning" type="button" @click="$emit('stop', device.id)">
          <AppIcon name="square" :size="14" class="fill-current" />
          停止
        </button>
      </div>
    </div>

    <div class="grid gap-3 xl:grid-cols-3">
      <div class="runtime-result-block">
        <div class="flex items-center justify-between gap-2">
          <p class="text-xs font-semibold text-(--app-text-faint)">当前进度</p>
          <StatusBadge :label="runtimeProgressLabel" :tone="runtimeProgressTone" />
        </div>
        <p class="mt-2 line-clamp-2 text-sm text-(--app-text-strong)">
          {{ runtimeResult.latestProgress?.message || status.message || '暂无进度事件' }}
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

    <div class="grid gap-4 xl:grid-cols-[1.35fr_0.95fr]">
      <SurfacePanel tone="muted" padding="sm" class="space-y-4">
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
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <div>
                <p class="text-sm font-semibold text-(--app-text-strong)">总队列</p>
                <p class="text-xs text-(--app-text-faint)">追加到这里的是正式设备队列，在线设备会立即同步。</p>
              </div>
              <StatusBadge :label="`${assignments.length} 条`" tone="neutral" />
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
              当前没有与设备平台匹配的脚本。设备平台为 {{ formatPlatformLabel(device.data.platform) }}。
            </p>

            <div class="space-y-3 rounded-[18px] border border-dashed border-(--app-border) px-4 py-4">
              <div class="flex items-center justify-between gap-3">
                <div>
                  <p class="text-sm font-semibold text-(--app-text-strong)">待运行队列</p>
                  <p class="text-xs text-(--app-text-faint)">今天还未调度，且当前时间尚未超过模板窗口结束时间的队列项。</p>
                </div>
                <span class="rounded-full border border-(--app-border) px-3 py-1 text-xs text-(--app-text-faint)">
                  {{ pendingAssignments.length }} 条
                </span>
              </div>

              <div v-if="!pendingAssignments.length" class="text-sm text-(--app-text-soft)">
                当前没有待运行队列项。
              </div>

              <div v-else class="space-y-2">
                <div
                  v-for="item in pendingAssignments"
                  :key="item.assignment.id"
                  class="rounded-[16px] border border-(--app-border) bg-white/70 px-4 py-3 dark:bg-white/5"
                >
                  <div class="flex items-center justify-between gap-3">
                    <div class="min-w-0">
                      <p class="truncate text-sm font-medium text-(--app-text-strong)">{{ getScriptName(item.assignment.scriptId) }}</p>
                      <p class="mt-1 text-xs text-(--app-text-faint)">{{ getTemplateName(item.assignment.timeTemplateId) }}</p>
                    </div>
                    <span class="rounded-full px-3 py-1 text-xs" :class="item.waiting ? 'bg-sky-500/12 text-sky-700' : 'bg-emerald-500/12 text-emerald-700'">
                      {{ item.waiting ? '未到时段' : '待执行' }}
                    </span>
                  </div>
                </div>
              </div>
            </div>

            <div v-if="loadingAssignments" class="py-10 text-sm text-(--app-text-soft)">正在读取队列...</div>
            <div v-else-if="assignments.length === 0" class="rounded-[20px] border border-dashed border-(--app-border) p-6 text-sm text-(--app-text-soft)">
              当前设备为空闲状态。为它追加脚本后，就可以直接从这里启动或暂停执行。
            </div>
            <div v-else class="space-y-2">
              <div
                v-for="assignment in assignments"
                :key="assignment.id"
                class="rounded-[18px] border bg-white/20 px-4 py-3 dark:bg-white/5"
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
        </template>

        <template v-else>
          <div class="grid min-h-[420px] gap-4 lg:grid-cols-[220px_minmax(0,1fr)]">
            <div class="space-y-2">
              <div>
                <p class="text-sm font-semibold text-(--app-text-strong)">临时运行</p>
                <p class="text-xs text-(--app-text-faint)">这里只做临时调试，不读时间模板，不写正式调度记录。</p>
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

            <div class="space-y-4 rounded-[18px] border border-(--app-border) bg-white/45 px-4 py-4 dark:bg-white/5">
              <div class="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <p class="text-sm font-semibold text-(--app-text-strong)">
                    {{ selectedTemporaryScriptName || '选择左侧脚本' }}
                  </p>
                  <p class="text-xs text-(--app-text-faint)">右侧直接列出任务，不显示时间模板和任务周期。</p>
                </div>

                <div class="flex flex-wrap gap-2">
                  <button class="app-button app-button-ghost group" type="button" :disabled="!selectedTemporaryScriptId" @click="handleRunTemporaryScript">
                    <AppIcon
                      name="file-play"
                      :size="16"
                      class="text-(--app-text-faint) group-hover:text-(--app-accent) transition-colors"
                    />
                    运行脚本
                  </button>
                  <button class="app-button app-button-ghost group" type="button" :disabled="!selectedTemporaryTaskId" @click="handleRunTemporaryTask">
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
                左侧选择一个脚本后，这里显示它的全部任务。
              </div>
              <div v-else-if="props.scriptTaskLoading[selectedTemporaryScriptId]" class="py-10 text-sm text-(--app-text-soft)">
                正在加载任务列表...
              </div>
              <div v-else-if="!temporaryRows.length" class="rounded-[18px] border border-dashed border-(--app-border) px-4 py-6 text-sm text-(--app-text-soft)">
                当前脚本没有任务。
              </div>
              <div v-else class="space-y-2">
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
                      <button class="app-button app-button-ghost h-8 px-3 text-sm" type="button" @click.stop="runSpecificTemporaryTask(row.id)">
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

      <SurfacePanel tone="muted" padding="sm" class="space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-semibold text-(--app-text-strong)">最近运行记录</p>
            <p class="text-xs text-(--app-text-faint)">帮助快速判断设备是否稳定执行。</p>
          </div>
          <button class="app-button app-button-ghost h-10 px-4" type="button" @click="$emit('clearSchedules', device.id)">
            清空
          </button>
        </div>

        <div v-if="loadingSchedules" class="py-10 text-sm text-(--app-text-soft)">正在读取记录...</div>
        <div v-else-if="schedules.length === 0" class="rounded-[20px] border border-dashed border-(--app-border) p-6 text-sm text-(--app-text-soft)">
          还没有运行历史。首次执行完成后，这里会显示最近的调度结果。
        </div>
        <div v-else class="space-y-2">
          <div
            v-for="schedule in schedules.slice(0, 6)"
            :key="schedule.id"
            class="rounded-[18px] border border-(--app-border) bg-white/20 px-4 py-3 dark:bg-white/5"
          >
            <div class="flex items-center justify-between gap-3">
              <p class="truncate text-sm font-medium text-(--app-text-strong)">{{ getScriptName(schedule.scriptId) }}</p>
              <StatusBadge :label="schedule.status" :tone="schedule.status === 'Success' ? 'success' : schedule.status === 'Skipped' ? 'warning' : 'danger'" />
            </div>
            <p class="mt-1 text-xs text-(--app-text-faint)">
              {{ formatDateTime(schedule.startedAt) }} · {{ schedule.message || schedule.taskCycle }}
            </p>
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
  DeviceRuntimeStatus,
  RunTarget,
  RuntimeProgressEvent,
  RuntimeResultProjection,
  RuntimeTimeoutEvent,
  ScriptTableRecord,
} from '@/types/app/domain';
import type { DeviceTable } from '@/types/bindings/DeviceTable';
import type { DeviceScriptSchedule } from '@/types/bindings/DeviceScriptSchedule';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { TimeTemplate } from '@/types/bindings/TimeTemplate';
import { filterUserVisibleTaskRows } from '@/utils/scriptTaskVisibility';
import {
  formatCaptureMethod,
  formatConnectLabel,
  formatDateTime,
  formatPlatformLabel,
  formatStatusLabel,
  formatStatusTone,
  formatTemplateWindow,
} from '@/utils/presenters';

const props = defineProps<{
  device: DeviceTable;
  status: DeviceRuntimeStatus;
  scripts: ScriptTableRecord[];
  timeTemplates: TimeTemplate[];
  assignments: AssignmentRecord[];
  schedules: DeviceScriptSchedule[];
  scriptTasksByScriptId: Record<string, ScriptTaskTable[]>;
  scriptTaskLoading: Record<string, boolean>;
  progressEvent: RuntimeProgressEvent | null;
  timeoutEvent: RuntimeTimeoutEvent | null;
  runtimeResult: RuntimeResultProjection;
  loadingAssignments: boolean;
  loadingSchedules: boolean;
}>();

const emit = defineEmits<{
  start: [deviceId: string];
  pause: [deviceId: string];
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

const templateOptions = computed(() => [
  { label: '每次', value: '' },
  ...props.timeTemplates.map((template) => ({
    label: template.name,
    value: template.id,
    description: formatTemplateWindow(template),
  })),
]);

const templateMap = computed(() =>
  Object.fromEntries(props.timeTemplates.map((template) => [template.id, template])),
);

const selectedTemporaryScriptName = computed(() =>
  temporaryScriptItems.value.find((script) => script.id === selectedTemporaryScriptId.value)?.name ?? '',
);

const temporaryRows = computed(() =>
  filterUserVisibleTaskRows(props.scriptTasksByScriptId[selectedTemporaryScriptId.value] ?? [])
    .sort((left, right) => left.index - right.index),
);

const normalizeWarningMessage = (message?: string | null) => message?.trim() || null;

const timeoutActionLabels: Record<string, string> = {
  SkipCurrentTask: '跳过任务',
  RunRecoveryTask: '恢复任务',
  StopExecution: '停止执行',
  skipCurrentTask: '跳过任务',
  runRecoveryTask: '恢复任务',
  stopExecution: '停止执行',
};

const scheduleStatusLabels: Record<string, string> = {
  Queued: '已排队',
  Running: '运行中',
  Success: '成功',
  Failed: '失败',
  Skipped: '已跳过',
  Cleared: '已清空',
};

const progressPhaseLabels: Record<string, string> = {
  Idle: '空闲',
  Loading: '加载中',
  Planning: '规划中',
  Executing: '执行中',
  Paused: '已暂停',
  Completed: '已完成',
  Failed: '失败',
};

const runtimeProgressLabel = computed(() =>
  progressPhaseLabels[props.runtimeResult.latestProgress?.phase ?? ''] ?? props.runtimeResult.latestProgress?.phase ?? '暂无',
);

const runtimeProgressTone = computed(() => {
  const phase = props.runtimeResult.latestProgress?.phase;
  if (phase === 'Failed') return 'danger';
  if (phase === 'Completed') return 'success';
  if (phase === 'Paused') return 'warning';
  if (phase === 'Executing' || phase === 'Loading' || phase === 'Planning') return 'info';
  return 'neutral';
});

const runtimeScheduleLabel = computed(() =>
  scheduleStatusLabels[props.runtimeResult.latestSchedule?.status ?? ''] ?? props.runtimeResult.latestSchedule?.status ?? '暂无',
);

const runtimeScheduleTone = computed(() => {
  const status = props.runtimeResult.latestSchedule?.status;
  if (status === 'Success') return 'success';
  if (status === 'Skipped' || status === 'Queued' || status === 'Running') return 'warning';
  if (status === 'Failed') return 'danger';
  return 'neutral';
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

const getTemplateName = (templateId: string | null) => {
  return formatTemplateWindow(templateId ? templateMap.value[templateId] : null);
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

const ranAssignmentIdsToday = computed(() =>
  new Set(
    props.schedules
      .filter((schedule) => schedule.assignmentId && scheduleDayKey(schedule.startedAt) === currentDayKey.value)
      .map((schedule) => String(schedule.assignmentId)),
  ),
);

const pendingAssignments = computed(() =>
  props.assignments
    .map((assignment) => {
      if (!assignment.timeTemplateId || ranAssignmentIdsToday.value.has(assignment.id)) {
        return null;
      }

      const template = templateMap.value[assignment.timeTemplateId];
      if (!template) {
        return null;
      }

      const startMinute = parseTimeToMinutes(template.startTime);
      const endMinute = parseTimeToMinutes(template.endTime);
      if (endMinute !== null && currentMinuteOfDay.value > endMinute) {
        return null;
      }

      return {
        assignment,
        waiting: startMinute !== null && currentMinuteOfDay.value < startMinute,
      };
    })
    .filter((item): item is { assignment: AssignmentRecord; waiting: boolean } => Boolean(item)),
);

const handleRunTemporaryScript = () => {
  if (!selectedTemporaryScriptId.value) {
    return;
  }

  emit('runTarget', props.device.id, {
    type: 'fullScript',
    scriptId: selectedTemporaryScriptId.value,
  });
};

const handleRunTemporaryTask = () => {
  if (!selectedTemporaryScriptId.value || !selectedTemporaryTaskId.value) {
    return;
  }

  emit('runTarget', props.device.id, {
    type: 'task',
    scriptId: selectedTemporaryScriptId.value,
    taskId: selectedTemporaryTaskId.value,
  });
};

const runSpecificTemporaryTask = (taskId: string) => {
  selectedTemporaryTaskId.value = taskId;
  handleRunTemporaryTask();
};

const handleAddScript = () => {
  if (!selectedScriptId.value) {
    return;
  }

  emit('addScript', props.device.id, selectedScriptId.value, selectedTemplateId.value || null);
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
.editor-panel-tabs {
  display: inline-flex;
  gap: 0.4rem;
  border-radius: 16px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.7);
  padding: 0.3rem;
}

.editor-panel-tab {
  border-radius: 12px;
  border: 1px solid transparent;
  background: transparent;
  padding: 0.55rem 0.9rem;
  font-size: 0.84rem;
  font-weight: 600;
  color: var(--app-text-soft);
  transition: border-color 0.16s ease, background 0.16s ease, color 0.16s ease;
}

.editor-panel-tab:hover {
  color: var(--app-text-strong);
}

.editor-panel-tab-active {
  border-color: color-mix(in srgb, var(--app-accent) 28%, var(--app-border));
  background: color-mix(in srgb, var(--app-accent-soft) 58%, white);
  color: var(--app-text-strong);
}

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
