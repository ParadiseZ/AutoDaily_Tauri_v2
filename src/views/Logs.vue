<template>
  <div class="flex h-full min-h-0 flex-col gap-4">
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

    <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
    <SurfacePanel v-if="!filteredLogs.length" class="space-y-4">
      <EmptyState
        :title="emptyLogTitle"
        :description="emptyLogDescription"
        icon="file-text"
      />
      <div class="grid gap-3 text-sm text-(--app-text-soft) md:grid-cols-3">
        <div class="rounded-[16px] border border-(--app-border) bg-(--app-panel-muted)/70 px-4 py-3">
          <p class="font-semibold text-(--app-text-strong)">先运行队列</p>
          <p class="mt-1 text-xs leading-5">设备开始执行后，日志会在这里追加。</p>
        </div>
        <div class="rounded-[16px] border border-(--app-border) bg-(--app-panel-muted)/70 px-4 py-3">
          <p class="font-semibold text-(--app-text-strong)">检查筛选</p>
          <p class="mt-1 text-xs leading-5">设备、级别或关键字过窄时会隐藏已有日志。</p>
        </div>
        <div class="rounded-[16px] border border-(--app-border) bg-(--app-panel-muted)/70 px-4 py-3">
          <p class="font-semibold text-(--app-text-strong)">当前会话</p>
          <p class="mt-1 text-xs leading-5">这里显示的是本次打开应用后收到的实时日志。</p>
        </div>
      </div>
    </SurfacePanel>

    <SurfacePanel v-else class="h-full overflow-hidden p-0">
      <div ref="logContainer" class="h-full overflow-y-auto bg-[#081019] px-5 py-4 font-mono text-xs text-slate-200">
        <div class="space-y-2">
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
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
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

const hasAnyLogs = computed(() => Object.values(logsStore.logsByDevice).some((entries) => entries.length > 0));
const hasActiveFilter = computed(() => Boolean(selectedDeviceId.value || selectedLevel.value || searchText.value));

const emptyLogTitle = computed(() => {
  if (hasAnyLogs.value && hasActiveFilter.value) {
    return '筛选后没有日志';
  }
  return '还没有运行日志';
});

const emptyLogDescription = computed(() => {
  if (hasAnyLogs.value && hasActiveFilter.value) {
    return '已有日志被当前设备、级别或关键字条件过滤了。';
  }
  return '启动设备队列或调试运行后，这里会显示实时日志。';
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
