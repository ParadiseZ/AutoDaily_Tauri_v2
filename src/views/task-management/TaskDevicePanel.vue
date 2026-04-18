<template>
  <SurfacePanel class="space-y-5">
    <div class="flex flex-wrap items-start justify-between gap-4">
      <div class="space-y-2">
        <div class="flex flex-wrap items-center gap-2">
          <h2 class="text-lg font-semibold text-[var(--app-text-strong)]">{{ device.data.deviceName }}</h2>
          <StatusBadge :label="formatStatusLabel(status)" :tone="formatStatusTone(status.kind)" />
        </div>
        <div class="flex flex-wrap gap-2 text-sm text-[var(--app-text-soft)]">
          <span>{{ formatPlatformLabel(device.data.platform) }}</span>
          <span>·</span>
          <span>{{ formatConnectLabel(device.data.adbConnect) }}</span>
          <span>·</span>
          <span>{{ formatCaptureMethod(device.data.capMethod) }}</span>
          <span v-if="status.currentScript">· 正在执行 {{ status.currentScript }}</span>
        </div>
        <p v-if="status.message" class="text-sm text-[var(--app-text-faint)]">{{ status.message }}</p>
      </div>

      <div class="flex flex-wrap gap-2">
        <button class="app-button app-button-primary shadow-md shadow-blue-500/20" type="button" @click="$emit('start', device.id)">
          <AppIcon name="play" :size="16" class="fill-current" />
          运行队列
        </button>
        <button class="app-button app-button-ghost group" type="button" @click="$emit('pause', device.id)">
          <AppIcon name="pause" :size="16" class="text-[var(--app-text-faint)] group-hover:text-[var(--app-text-strong)] transition-colors" />
          暂停
        </button>
        <button class="app-button app-button-warning" type="button" @click="$emit('stop', device.id)">
          <AppIcon name="square" :size="14" class="fill-current" />
          停止
        </button>
      </div>
    </div>

    <div class="grid gap-4 xl:grid-cols-[1.35fr_0.95fr]">
      <SurfacePanel tone="muted" padding="sm" class="space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-semibold text-[var(--app-text-strong)]">脚本队列</p>
            <p class="text-xs text-[var(--app-text-faint)]">队列改动会同步到本地调度表，在线设备会立即收到更新。</p>
          </div>
          <StatusBadge :label="`${assignments.length} 条`" tone="neutral" />
        </div>

        <div class="grid gap-3 lg:grid-cols-[1fr_220px_auto]">
          <AppSelect v-model="selectedScriptId" :options="scriptOptions" placeholder="选择要追加的脚本" />
          <AppSelect v-model="selectedTemplateId" :options="templateOptions" placeholder="选择时间模板" />
          <button class="app-button app-button-ghost group" type="button" @click="handleAddScript">
            <AppIcon name="list-plus" :size="16" class="text-[var(--app-text-faint)] group-hover:text-[var(--app-accent)] transition-colors" />
            追加
          </button>
        </div>
        <p v-if="!scriptOptions.length" class="text-xs text-[var(--app-text-faint)]">
          当前没有与设备平台匹配的脚本。设备平台为 {{ formatPlatformLabel(device.data.platform) }}。
        </p>

        <div v-if="loadingAssignments" class="py-10 text-sm text-[var(--app-text-soft)]">正在读取队列...</div>
        <div v-else-if="assignments.length === 0" class="rounded-[20px] border border-dashed border-[var(--app-border)] p-6 text-sm text-[var(--app-text-soft)]">
          当前设备为空闲状态。为它追加脚本后，就可以直接从这里启动或暂停执行。
        </div>
        <div v-else class="space-y-2">
          <div
            v-for="assignment in assignments"
            :key="assignment.id"
            class="flex items-center gap-3 rounded-[18px] border border-[var(--app-border)] bg-white/20 px-4 py-3 dark:bg-white/5"
          >
            <div class="flex h-8 w-8 items-center justify-center rounded-full bg-[var(--app-accent-soft)] text-xs font-semibold text-[var(--app-accent)]">
              {{ assignment.index + 1 }}
            </div>
            <div class="min-w-0 flex-1">
              <p class="truncate text-sm font-medium text-[var(--app-text-strong)]">{{ getScriptName(assignment.scriptId) }}</p>
              <p class="text-xs text-[var(--app-text-faint)]">{{ getTemplateName(assignment.timeTemplateId) }}</p>
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
        </div>

        <div class="space-y-3 rounded-[18px] border border-dashed border-[var(--app-border)] px-4 py-4">
          <div class="flex items-center justify-between gap-3">
            <div>
              <p class="text-sm font-semibold text-[var(--app-text-strong)]">临时运行</p>
              <p class="text-xs text-[var(--app-text-faint)]">
                不改队列定义，按 `everyRun` 临时装填 `FullScript / Task`，不写正式调度记录，但会保留运行日志。
              </p>
            </div>
          </div>

          <div class="grid gap-3 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)]">
            <AppSelect
              v-model="selectedTemporaryScriptId"
              :options="scriptOptions"
              placeholder="选择要临时运行的脚本"
            />
            <AppSelect
              v-model="selectedTemporaryTaskId"
              :options="temporaryTaskOptions"
              :placeholder="temporaryTaskPlaceholder"
            />
          </div>

          <div class="flex flex-wrap gap-2">
            <button class="app-button app-button-ghost group" type="button" @click="handleRunTemporaryScript">
              <AppIcon
                name="file-play"
                :size="16"
                class="text-[var(--app-text-faint)] group-hover:text-[var(--app-accent)] transition-colors"
              />
              运行脚本
            </button>
            <button class="app-button app-button-ghost group" type="button" @click="handleRunTemporaryTask">
              <AppIcon
                name="list-checks"
                :size="16"
                class="text-[var(--app-text-faint)] group-hover:text-[var(--app-accent)] transition-colors"
              />
              运行任务
            </button>
          </div>
        </div>
      </SurfacePanel>

      <SurfacePanel tone="muted" padding="sm" class="space-y-4">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-semibold text-[var(--app-text-strong)]">最近运行记录</p>
            <p class="text-xs text-[var(--app-text-faint)]">帮助快速判断设备是否稳定执行。</p>
          </div>
          <button class="app-button app-button-ghost h-10 px-4" type="button" @click="$emit('clearSchedules', device.id)">
            清空
          </button>
        </div>

        <div v-if="loadingSchedules" class="py-10 text-sm text-[var(--app-text-soft)]">正在读取记录...</div>
        <div v-else-if="schedules.length === 0" class="rounded-[20px] border border-dashed border-[var(--app-border)] p-6 text-sm text-[var(--app-text-soft)]">
          还没有运行历史。首次执行完成后，这里会显示最近的调度结果。
        </div>
        <div v-else class="space-y-2">
          <div
            v-for="schedule in schedules.slice(0, 6)"
            :key="schedule.id"
            class="rounded-[18px] border border-[var(--app-border)] bg-white/20 px-4 py-3 dark:bg-white/5"
          >
            <div class="flex items-center justify-between gap-3">
              <p class="truncate text-sm font-medium text-[var(--app-text-strong)]">{{ getScriptName(schedule.scriptId) }}</p>
              <StatusBadge :label="schedule.status" :tone="schedule.status === 'Success' ? 'success' : schedule.status === 'Skipped' ? 'warning' : 'danger'" />
            </div>
            <p class="mt-1 text-xs text-[var(--app-text-faint)]">
              {{ formatDateTime(schedule.startedAt) }} · {{ schedule.message || schedule.taskCycle }}
            </p>
          </div>
        </div>
      </SurfacePanel>
    </div>
  </SurfacePanel>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import StatusBadge from '@/components/shared/StatusBadge.vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import type {
  AssignmentRecord,
  DeviceRuntimeStatus,
  RunTarget,
  RuntimeProgressEvent,
  ScriptTableRecord,
} from '@/types/app/domain';
import type { DeviceTable } from '@/types/bindings/DeviceTable';
import type { DeviceScriptSchedule } from '@/types/bindings/DeviceScriptSchedule';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { TimeTemplate } from '@/types/bindings/TimeTemplate';
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

const selectedScriptId = ref('');
const selectedTemplateId = ref('');
const selectedTemporaryScriptId = ref('');
const selectedTemporaryTaskId = ref('');

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

const getScriptName = (scriptId: string) => {
  return props.scripts.find((script) => script.id === scriptId)?.data.name || '未知脚本';
};

const getTemplateName = (templateId: string | null) => {
  return formatTemplateWindow(templateId ? templateMap.value[templateId] : null);
};

const temporaryTaskOptions = computed(() =>
  (props.scriptTasksByScriptId[selectedTemporaryScriptId.value] ?? [])
    .filter((task) => task.rowType === 'task' && !task.isDeleted)
    .map((task) => ({
      label: task.name,
      value: task.id,
      description: `触发方式：${task.triggerMode} · 执行次数上限：${task.execMax || '无限'}`,
    })),
);

const temporaryTaskPlaceholder = computed(() => {
  if (!selectedTemporaryScriptId.value) {
    return '先选择脚本';
  }
  if (props.scriptTaskLoading[selectedTemporaryScriptId.value]) {
    return '正在加载任务列表...';
  }
  return '选择要临时运行的任务';
});

watch(
  selectedTemporaryScriptId,
  (scriptId) => {
    selectedTemporaryTaskId.value = '';
    if (!scriptId) {
      return;
    }
    emit('ensureScriptTasks', scriptId);
  },
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

const handleAddScript = () => {
  if (!selectedScriptId.value) {
    return;
  }

  emit('addScript', props.device.id, selectedScriptId.value, selectedTemplateId.value || null);
  selectedScriptId.value = '';
  selectedTemplateId.value = '';
};
</script>
