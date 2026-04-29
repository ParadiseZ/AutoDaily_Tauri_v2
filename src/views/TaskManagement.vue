<template>
  <div class="flex h-full min-h-0 flex-col gap-4">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div v-if="deviceIds.length" class="task-device-tabs">
        <button
          v-for="device in orderedDevices"
          :key="device.id"
          type="button"
          class="task-device-tab"
          :class="{ 'task-device-tab-active': device.id === activeDevice?.id }"
          @click="deviceStore.selectedDeviceId = device.id"
        >
          <span class="truncate font-semibold">{{ device.data.deviceName }}</span>
          <span class="task-device-tab-meta">
            {{ deviceStore.getDeviceStatus(device.id).kind === 'running' ? '运行中' : deviceStore.isDeviceOnline(device.id) ? '在线' : '离线' }}
          </span>
          <span class="task-device-tab-count">{{ taskStore.assignmentsByDevice[device.id]?.length || 0 }}</span>
        </button>
      </div>

      <div v-if="deviceIds.length" class="ml-auto flex flex-wrap items-center gap-2">
        <button class="app-button app-button-ghost group text-(--app-text-strong)" type="button" @click="deviceStore.pauseDevices(deviceIds)">
          <AppIcon name="pause" :size="16" class="text-(--app-text-faint) group-hover:text-(--app-text-strong) transition-colors" />
          全部暂停
        </button>
        <button class="app-button app-button-warning shadow-md shadow-amber-500/10" type="button" @click="deviceStore.stopDevices(deviceIds)">
          <AppIcon name="square" :size="14" class="fill-current" />
          全部停止
        </button>
        <button class="app-button app-button-primary shadow-lg shadow-(--app-vibrant-blue)/30 hover:shadow-(--app-vibrant-blue)/50 transition-shadow" type="button" @click="handleStartAllDevices">
          <AppIcon name="play" :size="16" class="fill-current" />
          全部启动
        </button>
      </div>
    </div>

    <EmptyState
      v-if="!deviceIds.length"
      title="还没有可调度设备"
      icon="monitor-smartphone"
    />

    <div v-else class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
      <TaskDevicePanel
        v-if="activeDevice"
        :device="activeDevice"
        :status="deviceStore.getDeviceStatus(activeDevice.id)"
        :scripts="scriptStore.sortedScripts"
        :time-templates="taskStore.timeTemplates"
        :assignments="taskStore.assignmentsByDevice[activeDevice.id] ?? []"
        :schedules="taskStore.schedulesByDevice[activeDevice.id] ?? []"
        :script-tasks-by-script-id="scriptStore.tasksByScriptId"
        :script-task-loading="scriptStore.taskLoading"
        :progress-event="runtimeStore.getLatestProgress(activeDevice.id)"
        :timeout-event="runtimeStore.getLatestTimeout(activeDevice.id)"
        :runtime-result="runtimeStore.getRuntimeResult(activeDevice.id)"
        :loading-assignments="Boolean(taskStore.loadingAssignments[activeDevice.id])"
        :loading-schedules="Boolean(taskStore.loadingSchedules[activeDevice.id])"
        @add-script="handleAddScript"
        @ensure-script-tasks="handleEnsureScriptTasks"
        @open-assignment-settings="handleOpenAssignmentSettings"
        @remove-assignment="handleRemoveAssignment"
        @run-target="handleRunTarget"
        @clear-schedules="handleClearSchedules"
        @start="handleStartDevice"
        @pause="deviceStore.pauseDevice"
        @stop="deviceStore.stopDevice"
      />
    </div>

    <AppDialog
      :open="assignmentSettingsOpen"
      title="脚本模板设置"
      width-class="max-w-4xl"
      @close="assignmentSettingsOpen = false"
    >
      <ScriptTemplateValuePanel
        v-if="assignmentSettingsScope && assignmentSettingsScript"
        :script="assignmentSettingsScript"
        :tasks="assignmentSettingsTasks"
        :scope="assignmentSettingsScope"
      />
    </AppDialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { confirm } from '@tauri-apps/plugin-dialog';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppDialog from '@/components/shared/AppDialog.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import { runtimeService } from '@/services/runtimeService';
import TaskDevicePanel from '@/views/task-management/TaskDevicePanel.vue';
import ScriptTemplateValuePanel from '@/views/script-template-values/ScriptTemplateValuePanel.vue';
import { useDeviceStore } from '@/store/device';
import { useRuntimeStore } from '@/store/runtime';
import { useScriptStore } from '@/store/script';
import { useTaskStore } from '@/store/task';
import { formatTemplateWindow } from '@/utils/presenters';
import { showToast } from '@/utils/toast';
import { validateDeviceQueueRecoveryForDevice, validateDeviceRuntimePlatform, validateRunTargetRecoveryForDevice } from '@/utils/runtimePolicy';
import type { AssignmentRecord, RunTarget } from '@/types/app/domain';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';

const deviceStore = useDeviceStore();
const runtimeStore = useRuntimeStore();
const scriptStore = useScriptStore();
const taskStore = useTaskStore();

const deviceIds = computed(() => deviceStore.devices.map((device) => device.id));
const orderedDevices = computed(() =>
  [...deviceStore.devices].sort((left, right) => Number(right.data.enable) - Number(left.data.enable)),
);
const activeDevice = computed(() =>
  orderedDevices.value.find((device) => device.id === deviceStore.selectedDeviceId) ?? orderedDevices.value[0] ?? null,
);
const assignmentSettingsOpen = ref(false);
const assignmentSettingsScriptId = ref<string | null>(null);
const assignmentSettingsTasks = ref<ScriptTaskTable[]>([]);
const assignmentSettingsScope = ref<{
  deviceId: string;
  deviceName: string;
  timeTemplateId: string;
  templateLabel: string;
  accountId?: string | null;
} | null>(null);

const assignmentSettingsScript = computed(() =>
  assignmentSettingsScriptId.value
    ? scriptStore.sortedScripts.find((item) => item.id === assignmentSettingsScriptId.value) ?? null
    : null,
);

const loadPageData = async () => {
  await Promise.all([deviceStore.refreshAll(), scriptStore.loadScripts()]);
  await taskStore.hydrateForDevices(deviceIds.value);
  if (!deviceStore.selectedDeviceId && orderedDevices.value.length) {
    deviceStore.selectedDeviceId = orderedDevices.value[0].id;
  }
};

const handleAddScript = async (deviceId: string, scriptId: string, timeTemplateId: string | null) => {
  try {
    await taskStore.createAssignment(deviceId, scriptId, timeTemplateId);
    showToast('脚本已加入设备队列', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '脚本追加失败', 'error');
  }
};

const handleEnsureScriptTasks = async (scriptId: string) => {
  if (!scriptId || scriptStore.tasksByScriptId[scriptId] || scriptStore.taskLoading[scriptId]) {
    return;
  }

  try {
    await scriptStore.loadScriptTasks(scriptId);
  } catch (error) {
    showToast(error instanceof Error ? error.message : '读取脚本任务失败', 'error');
  }
};

const handleRemoveAssignment = async (deviceId: string, assignment: AssignmentRecord) => {
  try {
    await taskStore.removeAssignment(deviceId, assignment);
    showToast('脚本已从队列移除', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '移除失败', 'error');
  }
};

const handleOpenAssignmentSettings = async (assignment: AssignmentRecord) => {
  if (!assignment.timeTemplateId) {
    showToast('请先为这条脚本分配选择时间模板。', 'warning');
    return;
  }

  const device = deviceStore.devices.find((item) => item.id === assignment.deviceId);
  const script = scriptStore.sortedScripts.find((item) => item.id === assignment.scriptId);
  const template = taskStore.timeTemplates.find((item) => item.id === assignment.timeTemplateId);

  if (!device || !script || !template) {
    showToast('当前分配缺少设备、脚本或时间模板信息。', 'error');
    return;
  }

  const tasks =
    scriptStore.tasksByScriptId[assignment.scriptId] ?? (await scriptStore.loadScriptTasks(assignment.scriptId).catch(() => []));

  assignmentSettingsScriptId.value = script.id;
  assignmentSettingsTasks.value = tasks;
  assignmentSettingsScope.value = {
    deviceId: assignment.deviceId,
    deviceName: device.data.deviceName,
    timeTemplateId: assignment.timeTemplateId,
    templateLabel: formatTemplateWindow(template),
    accountId: null,
  };
  assignmentSettingsOpen.value = true;
};

const handleClearSchedules = async (deviceId: string) => {
  try {
    await taskStore.clearSchedules(deviceId);
    runtimeStore.clearTimeoutState(deviceId);
    showToast('运行记录已清空', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '清空失败', 'error');
  }
};

const validateDeviceQueueStart = (deviceId: string) => {
  const device = deviceStore.devices.find((item) => item.id === deviceId);
  if (!device) {
    return '目标设备不存在。';
  }

  const platformError = validateDeviceRuntimePlatform(device);
  if (platformError) {
    return platformError;
  }

  return validateDeviceQueueRecoveryForDevice(
    device,
    taskStore.assignmentsByDevice[deviceId] ?? [],
    scriptStore.sortedScripts,
  );
};

const validateTemporaryRun = async (deviceId: string, target: RunTarget) => {
  const device = deviceStore.devices.find((item) => item.id === deviceId);
  if (!device) {
    return '目标设备不存在。';
  }

  const platformError = validateDeviceRuntimePlatform(device);
  if (platformError) {
    return platformError;
  }

  if (target.type !== 'fullScript' && target.type !== 'task') {
    return '任务管理页当前只支持临时运行整脚本或单任务。';
  }

  const script = scriptStore.sortedScripts.find((item) => item.id === target.scriptId);
  if (!script) {
    return '目标脚本不存在。';
  }

  const tasks =
    scriptStore.tasksByScriptId[target.scriptId] ?? (await scriptStore.loadScriptTasks(target.scriptId).catch(() => []));

  if (
    target.type === 'task' &&
    !tasks.some((task) => task.id === target.taskId && task.rowType === 'task' && !task.isDeleted)
  ) {
    return '目标任务不存在，或不是可执行 Task。';
  }

  return validateRunTargetRecoveryForDevice(device, script, tasks);
};

const prepareCurrentRunForTemporaryTarget = async (deviceId: string) => {
  const status = deviceStore.getDeviceStatus(deviceId);
  if (status.kind !== 'running' && status.kind !== 'paused') {
    return true;
  }

  const device = deviceStore.devices.find((item) => item.id === deviceId);
  const approved = await confirm('当前设备正在执行。暂停当前任务后继续临时运行？', {
    title: '确认切换运行目标',
    okLabel: '继续',
    cancelLabel: '取消',
    kind: 'warning',
  });
  if (!approved) {
    return false;
  }

  try {
    if (status.kind === 'running') {
      await deviceStore.pauseDevice(deviceId);
    }
    showToast(`设备「${device?.data.deviceName || deviceId}」已暂停，开始临时运行`, 'success');
    return true;
  } catch (error) {
    showToast(error instanceof Error ? error.message : '暂停当前运行失败', 'error');
    return false;
  }
};

const handleRunTarget = async (deviceId: string, target: RunTarget) => {
  const error = await validateTemporaryRun(deviceId, target);
  if (error) {
    showToast(error, 'warning');
    return;
  }

  const canProceed = await prepareCurrentRunForTemporaryTarget(deviceId);
  if (!canProceed) {
    return;
  }

  try {
    const result = await runtimeService.runScriptTarget(deviceId, target);
    showToast(result, 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '临时运行失败', 'error');
  }
};

const handleStartDevice = async (deviceId: string) => {
  const error = validateDeviceQueueStart(deviceId);
  if (error) {
    showToast(error, 'warning');
    return;
  }

  try {
    await deviceStore.startDevice(deviceId);
  } catch (error) {
    showToast(error instanceof Error ? error.message : '启动失败', 'error');
  }
};

const handleStartAllDevices = async () => {
  for (const deviceId of deviceIds.value) {
    const error = validateDeviceQueueStart(deviceId);
    if (error) {
      showToast(error, 'warning');
      return;
    }
  }

  try {
    await deviceStore.startDevices(deviceIds.value);
  } catch (error) {
    showToast(error instanceof Error ? error.message : '批量启动失败', 'error');
  }
};

onMounted(async () => {
  await loadPageData();
});
</script>

<style scoped>
.task-device-tabs {
  display: flex;
  min-width: 0;
  max-width: 100%;
  gap: 0.35rem;
  overflow-x: auto;
  border-radius: 16px;
  border: 1px solid var(--app-border);
  background: color-mix(in srgb, var(--app-panel) 88%, transparent);
  padding: 0.3rem;
}

.task-device-tab {
  display: inline-flex;
  min-width: 0;
  max-width: 220px;
  align-items: center;
  gap: 0.5rem;
  border-radius: 12px;
  border: 1px solid transparent;
  padding: 0.55rem 0.75rem;
  color: var(--app-text-soft);
  transition: border-color 0.16s ease, background 0.16s ease, color 0.16s ease;
}

.task-device-tab:hover {
  color: var(--app-text-strong);
}

.task-device-tab-active {
  border-color: color-mix(in srgb, var(--app-accent) 30%, var(--app-border));
  background: color-mix(in srgb, var(--app-accent-soft) 58%, white);
  color: var(--app-text-strong);
}

.task-device-tab-meta {
  flex-shrink: 0;
  color: var(--app-text-faint);
  font-size: 0.76rem;
}

.task-device-tab-count {
  flex-shrink: 0;
  border-radius: 999px;
  background: var(--app-panel-muted);
  padding: 0.1rem 0.45rem;
  color: var(--app-text-soft);
  font-size: 0.72rem;
}
</style>
