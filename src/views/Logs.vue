<template>
  <div class="space-y-6">
    <AppPageHeader
      eyebrow="Diagnostics"
      title="运行日志"
      description="日志页保持低干扰和长时间可读，过滤条件压在顶部，正文只做排查所需的最小高亮。"
    />

    <SurfacePanel class="grid gap-3 xl:grid-cols-[220px_180px_minmax(0,1fr)_180px_auto]">
      <select v-model="selectedDeviceId" class="app-select">
        <option value="">全部设备</option>
        <option v-for="device in deviceStore.devices" :key="device.id" :value="device.id">
          {{ device.data.deviceName }}
        </option>
      </select>
      <select v-model="selectedLevel" class="app-select">
        <option value="">全部级别</option>
        <option value="Debug">Debug</option>
        <option value="Info">Info</option>
        <option value="Warn">Warn</option>
        <option value="Error">Error</option>
      </select>
      <input v-model.trim="searchText" class="app-input" placeholder="搜索日志内容" />
      <select v-if="selectedDeviceId" v-model="deviceLogLevel" class="app-select" @change="updateDeviceLogLevel">
        <option value="Debug">Debug</option>
        <option value="Info">Info</option>
        <option value="Warn">Warn</option>
        <option value="Error">Error</option>
      </select>
      <button class="app-button app-button-ghost" type="button" @click="logsStore.clearLogs(selectedDeviceId || undefined)">
        清空
      </button>
    </SurfacePanel>

    <SurfacePanel class="overflow-hidden p-0">
      <div ref="logContainer" class="h-[calc(100vh-260px)] overflow-y-auto bg-[#081019] px-5 py-4 font-mono text-xs text-slate-200">
        <div v-if="!filteredLogs.length" class="flex h-full items-center justify-center text-sm text-slate-400">
          暂无符合条件的日志，运行设备后这里会持续滚动追加。
        </div>
        <div v-else class="space-y-2">
          <div
            v-for="entry in filteredLogs"
            :key="`${entry.deviceId}-${entry.time}-${entry.message}`"
            class="grid gap-1 border-b border-white/5 pb-2 md:grid-cols-[160px_90px_1fr]"
          >
            <span class="text-slate-500">{{ entry.time }} · {{ resolveDeviceName(entry.deviceId) }}</span>
            <span :class="levelClass(entry.level)">[{{ entry.level }}]</span>
            <span class="whitespace-pre-wrap break-all text-slate-100">{{ entry.message }}</span>
          </div>
        </div>
      </div>
    </SurfacePanel>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import { useDeviceStore } from '@/store/device';
import { useLogsStore } from '@/store/logs';
import { deviceService } from '@/services/deviceService';
import { showToast } from '@/utils/toast';
import type { LogLevel } from '@/types/bindings/LogLevel';

const deviceStore = useDeviceStore();
const logsStore = useLogsStore();
const selectedDeviceId = ref('');
const selectedLevel = ref('');
const searchText = ref('');
const deviceLogLevel = ref<LogLevel>('Info');
const logContainer = ref<HTMLDivElement | null>(null);

const filteredLogs = computed(() => {
  const pool = selectedDeviceId.value
    ? logsStore.getDeviceLogs(selectedDeviceId.value)
    : Object.values(logsStore.logsByDevice).flat();

  return pool.filter((entry) => {
    if (selectedLevel.value && entry.level !== selectedLevel.value) {
      return false;
    }
    if (searchText.value && !entry.message.toLowerCase().includes(searchText.value.toLowerCase())) {
      return false;
    }
    return true;
  });
});

const resolveDeviceName = (deviceId: string) => {
  return deviceStore.devices.find((device) => device.id === deviceId)?.data.deviceName || deviceId;
};

const levelClass = (level: string) => {
  if (level === 'Error') return 'text-red-400';
  if (level === 'Warn') return 'text-amber-300';
  if (level === 'Info') return 'text-sky-300';
  return 'text-slate-400';
};

const updateDeviceLogLevel = async () => {
  if (!selectedDeviceId.value) {
    return;
  }

  try {
    await deviceService.updateChildLogLevel(selectedDeviceId.value, deviceLogLevel.value);
    showToast('设备日志级别已更新', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '更新失败', 'error');
  }
};

watch(
  () => filteredLogs.value.length,
  async () => {
    await nextTick();
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight;
    }
  },
);

watch(
  () => selectedDeviceId.value,
  (deviceId) => {
    if (!deviceId) {
      deviceLogLevel.value = 'Info';
      return;
    }
    deviceLogLevel.value =
      deviceStore.devices.find((device) => device.id === deviceId)?.data.logLevel || 'Info';
  },
  { immediate: true },
);

onMounted(async () => {
  await Promise.all([deviceStore.refreshAll(), logsStore.initListener()]);
});
</script>
