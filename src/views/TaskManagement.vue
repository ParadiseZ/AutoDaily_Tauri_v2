<template>
  <div class="space-y-6">
    <AppPageHeader
      eyebrow="Workspace"
      title="任务管理"
      description="把多设备的运行状态、脚本队列与最近执行结果放在同一工作台里，减少切页和上下文切换。"
    >
      <template #actions>
        <button class="app-button app-button-ghost" type="button" @click="deviceStore.pauseDevices(deviceIds)">
          全部暂停
        </button>
        <button class="app-button app-button-warning" type="button" @click="deviceStore.stopDevices(deviceIds)">
          全部停止
        </button>
        <button class="app-button app-button-primary" type="button" @click="deviceStore.startDevices(deviceIds)">
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
      :icon="MonitorSmartphone"
    />

    <div v-else class="space-y-4">
      <TaskDevicePanel
        v-for="device in orderedDevices"
        :key="device.id"
        :device="device"
        :status="deviceStore.getDeviceStatus(device.id)"
        :scripts="scriptStore.sortedScripts"
        :time-templates="taskStore.timeTemplates"
        :assignments="taskStore.assignmentsByDevice[device.id] ?? []"
        :schedules="taskStore.schedulesByDevice[device.id] ?? []"
        :loading-assignments="Boolean(taskStore.loadingAssignments[device.id])"
        :loading-schedules="Boolean(taskStore.loadingSchedules[device.id])"
        @add-script="handleAddScript"
        @remove-assignment="handleRemoveAssignment"
        @clear-schedules="taskStore.clearSchedules"
        @start="deviceStore.startDevice"
        @pause="deviceStore.pauseDevice"
        @stop="deviceStore.stopDevice"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { MonitorSmartphone } from 'lucide-vue-next';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import TaskDevicePanel from '@/views/task-management/TaskDevicePanel.vue';
import { useDeviceStore } from '@/store/device';
import { useScriptStore } from '@/store/script';
import { useTaskStore } from '@/store/task';
import { showToast } from '@/utils/toast';
import type { AssignmentRecord } from '@/types/app/domain';

const deviceStore = useDeviceStore();
const scriptStore = useScriptStore();
const taskStore = useTaskStore();

const deviceIds = computed(() => deviceStore.devices.map((device) => device.id));
const orderedDevices = computed(() =>
  [...deviceStore.devices].sort((left, right) => Number(right.data.enable) - Number(left.data.enable)),
);
const totalAssignments = computed(() =>
  Object.values(taskStore.assignmentsByDevice).reduce((count, items) => count + items.length, 0),
);

const loadPageData = async () => {
  await Promise.all([deviceStore.refreshAll(), scriptStore.loadScripts()]);
  await taskStore.hydrateForDevices(deviceIds.value);
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

onMounted(async () => {
  await loadPageData();
});
</script>
