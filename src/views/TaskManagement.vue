<template>
  <div class="space-y-6">
    <AppPageHeader
      title="任务管理"
    >
      <template #actions>
        <button class="app-button app-button-ghost group text-[var(--app-text-strong)]" type="button" @click="deviceStore.pauseDevices(deviceIds)">
          <AppIcon name="pause" :size="16" class="text-[var(--app-text-faint)] group-hover:text-[var(--app-text-strong)] transition-colors" />
          全部暂停
        </button>
        <button class="app-button app-button-warning shadow-md shadow-amber-500/10" type="button" @click="deviceStore.stopDevices(deviceIds)">
          <AppIcon name="square" :size="14" class="fill-current" />
          全部停止
        </button>
        <button class="app-button app-button-primary shadow-lg shadow-[var(--app-vibrant-blue)]/30 hover:shadow-[var(--app-vibrant-blue)]/50 transition-shadow" type="button" @click="handleStartAllDevices">
          <AppIcon name="play" :size="16" class="fill-current" />
          全部启动
        </button>
      </template>
    </AppPageHeader>

    <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
      <SurfacePanel class="space-y-1">
        <p class="app-stat-label">已配置设备</p>
        <p class="app-stat-value">{{ deviceStore.deviceSummary.total }}</p>
      </SurfacePanel>
      <SurfacePanel class="space-y-1">
        <p class="app-stat-label">在线设备</p>
        <p class="app-stat-value">{{ deviceStore.deviceSummary.online }}</p>
      </SurfacePanel>
      <SurfacePanel class="space-y-1">
        <p class="app-stat-label">运行中</p>
        <p class="app-stat-value">{{ deviceStore.deviceSummary.running }}</p>
      </SurfacePanel>
      <SurfacePanel class="space-y-1">
        <p class="app-stat-label">队列总数</p>
        <p class="app-stat-value">{{ totalAssignments }}</p>
      </SurfacePanel>
    </div>

    <EmptyState
      v-if="!deviceIds.length"
      title="还没有可调度设备"
      description="先去设备列表创建一台设备，配置连接方式和截图能力后，任务中心会自动接入。"
      icon="monitor-smartphone"
    />

    <div v-else class="grid gap-4 xl:grid-cols-[320px_minmax(0,1fr)]">
      <SurfacePanel class="space-y-3">
        <div>
          <p class="text-sm font-semibold text-[var(--app-text-strong)]">设备节点</p>
          <p class="text-xs text-[var(--app-text-faint)]">左侧快速切换设备，右侧只展开当前设备的完整运行上下文。</p>
        </div>

        <div class="space-y-2">
          <button
            v-for="device in orderedDevices"
            :key="device.id"
            type="button"
            class="app-list-item"
            :class="{ 'app-list-item-active': device.id === activeDevice?.id }"
            @click="deviceStore.selectedDeviceId = device.id"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ device.data.deviceName }}</p>
                <p class="mt-1 truncate text-xs text-[var(--app-text-faint)]">
                  {{ taskStore.assignmentsByDevice[device.id]?.length || 0 }} 条队列
                </p>
              </div>
              <span class="text-xs text-[var(--app-text-soft)]">
                {{ deviceStore.getDeviceStatus(device.id).kind === 'running' ? '运行中' : deviceStore.isDeviceOnline(device.id) ? '在线' : '离线' }}
              </span>
            </div>
          </button>
        </div>
      </SurfacePanel>

      <TaskDevicePanel
        v-if="activeDevice"
        :device="activeDevice"
        :status="deviceStore.getDeviceStatus(activeDevice.id)"
        :scripts="scriptStore.sortedScripts"
        :time-templates="taskStore.timeTemplates"
        :assignments="taskStore.assignmentsByDevice[activeDevice.id] ?? []"
        :schedules="taskStore.schedulesByDevice[activeDevice.id] ?? []"
        :recovery-checkpoint="runtimeStore.getRecoveryCheckpoint(activeDevice.id)"
        :recovery-event="runtimeStore.getLatestRecovery(activeDevice.id)"
        :loading-assignments="Boolean(taskStore.loadingAssignments[activeDevice.id])"
        :loading-schedules="Boolean(taskStore.loadingSchedules[activeDevice.id])"
        @add-script="handleAddScript"
        @remove-assignment="handleRemoveAssignment"
        @clear-schedules="handleClearSchedules"
        @start="handleStartDevice"
        @pause="deviceStore.pauseDevice"
        @stop="deviceStore.stopDevice"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import TaskDevicePanel from '@/views/task-management/TaskDevicePanel.vue';
import { useDeviceStore } from '@/store/device';
import { useRuntimeStore } from '@/store/runtime';
import { useScriptStore } from '@/store/script';
import { useTaskStore } from '@/store/task';
import { showToast } from '@/utils/toast';
import { validateDeviceQueueRecoveryForDevice, validateDeviceRuntimePlatform } from '@/utils/runtimePolicy';
import type { AssignmentRecord } from '@/types/app/domain';

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
const totalAssignments = computed(() =>
  Object.values(taskStore.assignmentsByDevice).reduce((count, items) => count + items.length, 0),
);

const loadPageData = async () => {
  await Promise.all([deviceStore.refreshAll(), scriptStore.loadScripts()]);
  await taskStore.hydrateForDevices(deviceIds.value);
  await runtimeStore.hydrateRecoveryCheckpoints(deviceIds.value);
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

const handleRemoveAssignment = async (deviceId: string, assignment: AssignmentRecord) => {
  try {
    await taskStore.removeAssignment(deviceId, assignment);
    showToast('脚本已从队列移除', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '移除失败', 'error');
  }
};

const handleClearSchedules = async (deviceId: string) => {
  try {
    await taskStore.clearSchedules(deviceId);
    runtimeStore.clearRecoveryState(deviceId);
    showToast('运行记录与恢复检查点已清空', 'success');
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
