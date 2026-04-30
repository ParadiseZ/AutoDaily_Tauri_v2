<template>
  <div class="flex h-full min-h-0 flex-col gap-4">
    <div class="flex min-w-0 flex-nowrap items-center gap-3">
      <div v-if="deviceIds.length" class="editor-panel-tabs task-device-tabs">
        <button
          v-for="device in orderedDevices"
          :key="device.id"
          type="button"
          class="editor-panel-tab task-device-tab"
          :class="{ 'editor-panel-tab-active': device.id === activeDevice?.id }"
          @click="deviceStore.selectedDeviceId = device.id"
        >
          <span class="min-w-0 flex-1 truncate font-semibold">{{ device.data.deviceName }}</span>
          <StatusBadge :label="formatStatusLabel(deviceStore.getDeviceStatus(device.id))" :tone="formatStatusTone(deviceStore.getDeviceStatus(device.id).kind)" />
          <span class="task-device-tab-count">{{ taskStore.assignmentsByDevice[device.id]?.length || 0 }}</span>
        </button>
      </div>

      <div v-if="deviceIds.length" class="flex shrink-0 flex-wrap items-center justify-end gap-2">
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
import AppIcon from '@/components/shared/AppIcon.vue';
import AppDialog from '@/components/shared/AppDialog.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import StatusBadge from '@/components/shared/StatusBadge.vue';
import { requestAppConfirm } from '@/services/appDialogService';
import { runtimeService } from '@/services/runtimeService';
import TaskDevicePanel from '@/views/task-management/TaskDevicePanel.vue';
import ScriptTemplateValuePanel from '@/views/script-template-values/ScriptTemplateValuePanel.vue';
import { useDeviceStore } from '@/store/device';
import { useRuntimeStore } from '@/store/runtime';
import { useScriptStore } from '@/store/script';
import { useTaskStore } from '@/store/task';
import { formatStatusLabel, formatStatusTone, formatTemplateWindow } from '@/utils/presenters';
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
  const script = scriptStore.sortedScripts.find((item) => item.id === assignment.scriptId);
  const approved = await requestAppConfirm({
    title: '移除队列脚本',
    message: `确认从当前设备队列移除「${script?.data.name || assignment.scriptId}」？`,
    confirmText: '移除',
    tone: 'danger',
  });
  if (!approved) {
    return;
  }

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
  const approved = await requestAppConfirm({
    title: '确认切换运行目标',
    message: '当前设备正在执行。暂停当前任务后继续临时运行？',
    confirmText: '继续',
    cancelText: '取消',
    tone: 'warning',
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
  flex: 1 1 auto;
  min-width: 0;
  overflow-x: auto;
  overflow-y: hidden;
}

.task-device-tab {
  display: inline-flex;
  min-width: 180px;
  max-width: 280px;
  align-items: center;
  gap: 0.5rem;
  padding-right: 0.75rem;
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
