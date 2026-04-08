<template>
  <div class="space-y-6">
    <AppPageHeader
      title="运行日志"
    />

    <SurfacePanel class="grid gap-3 xl:grid-cols-[220px_180px_minmax(0,1fr)_180px_auto]">
      <AppSelect v-model="selectedDeviceId" :options="deviceOptions" placeholder="全部设备" />
      <AppSelect v-model="selectedLevel" :options="levelOptions" placeholder="全部级别" />
      <input v-model.trim="searchText" class="app-input" placeholder="搜索日志内容" />
      <AppSelect
        v-if="selectedDeviceId"
        v-model="deviceLogLevel"
        :options="levelOptions.filter((item) => item.value)"
      />
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
            <span class="text-slate-500 font-sans tracking-wide pt-[1px]">{{ entry.time }} · {{ resolveDeviceName(entry.deviceId) }}</span>
            <div class="flex items-start gap-1.5 pt-[1px]" :class="levelClass(entry.level)">
              <AppIcon :name="levelIcon(entry.level)" :size="14" class="shrink-0 translate-y-[2px]" />
              <span class="text-xs font-semibold tracking-wider font-sans uppercase">{{ entry.level }}</span>
            </div>
            <span class="whitespace-pre-wrap break-all text-slate-100/90 leading-6">{{ entry.message }}</span>
          </div>
        </div>
      </div>
    </SurfacePanel>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
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

const deviceOptions = computed(() => [
  { label: '全部设备', value: '' },
  ...deviceStore.devices.map((device) => ({
    label: device.data.deviceName,
    value: device.id,
  })),
]);

const levelOptions = [
  { label: '全部级别', value: '' },
  { label: 'Debug', value: 'Debug' },
  { label: 'Info', value: 'Info' },
  { label: 'Warn', value: 'Warn' },
  { label: 'Error', value: 'Error' },
];

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

const levelIcon = (level: string) => {
  if (level === 'Error') return 'hexagon'; // x-octagon alternative if not available, wait 'hexagon' has no 'x' usually but let's use 'alert-triangle'
  if (level === 'Warn') return 'triangle-alert';
  if (level === 'Debug') return 'bug';
  return 'info';
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

watch(
  () => deviceLogLevel.value,
  async (level, previousLevel) => {
    if (!selectedDeviceId.value || level === previousLevel) {
      return;
    }
    await updateDeviceLogLevel();
  },
);

onMounted(async () => {
  await Promise.all([deviceStore.refreshAll(), logsStore.initListener()]);
});
</script>
