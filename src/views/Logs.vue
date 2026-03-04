<template>
  <div class="h-full flex flex-col p-4">
    <!-- Top Bar -->
    <div class="flex gap-4 mb-4 items-center bg-base-100 p-2 rounded-lg shadow flex-wrap">
      <select class="select select-bordered select-sm" v-model="filterLevel">
        <option value="ALL">All Levels</option>
        <option value="Debug">Debug</option>
        <option value="Info">Info</option>
        <option value="Warn">Warn</option>
        <option value="Error">Error</option>
      </select>

      <select class="select select-bordered select-sm" v-model="filterDevice">
        <option value="ALL">All Devices</option>
        <option v-for="dev in devices" :key="dev.id" :value="dev.id">{{ dev.name }}</option>
      </select>

      <input
        type="text"
        placeholder="搜索日志内容..."
        class="input input-bordered input-sm flex-1 min-w-[120px]"
        v-model="filterText"
      />

      <div class="flex items-center gap-1 ml-auto">
        <span class="text-xs opacity-70">自动滚动</span>
        <input type="checkbox" class="toggle toggle-xs toggle-primary" v-model="autoScroll" />
      </div>

      <button class="btn btn-sm btn-ghost btn-outline" @click="clearLogs">清空</button>
    </div>

    <!-- Logs Container -->
    <div class="flex-1 overflow-x-auto flex gap-4">
      <div
        v-for="device in filteredDevices"
        :key="device.id"
        class="min-w-[300px] flex-1 bg-black text-green-400 p-2 rounded font-mono text-xs overflow-y-auto flex flex-col"
        ref="logContainers"
      >
        <div
          class="font-bold text-white border-b border-gray-700 mb-2 sticky top-0 bg-black pb-1 flex justify-between items-center"
        >
          <span>{{ device.name }}</span>
          <select
            class="select select-bordered select-xs bg-gray-800 text-white w-20"
            :value="device.logLevel || 'Info'"
            @change="handleChildLogLevelChange(device.id, $event.target.value)"
          >
            <option value="Debug">Debug</option>
            <option value="Info">Info</option>
            <option value="Warn">Warn</option>
            <option value="Error">Error</option>
          </select>
        </div>
        <div
          v-for="(log, idx) in getDeviceLogs(device.id)"
          :key="idx"
          class="whitespace-pre-wrap break-all hover:bg-gray-900"
        >
          <span :class="getLogClass(log.level)">[{{ log.time }}] [{{ log.level }}]</span> {{ log.message }}
        </div>
        <div v-if="getDeviceLogs(device.id).length === 0" class="text-gray-600 italic text-center mt-4">暂无日志</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

const filterLevel = ref('ALL');
const filterDevice = ref('ALL');
const filterText = ref('');
const autoScroll = ref(true);

const devices = ref([]);
const logs = ref({});
const logContainers = ref([]);

let unlisten = null;
const MAX_LOGS_PER_DEVICE = 500;

// Load devices
const loadDevices = async () => {
  try {
    const res = await invoke('get_all_devices_cmd');
    const devList = Object.values(res);
    devices.value = devList.map((d) => ({
      id: d.id,
      name: d.data.deviceName,
      logLevel: d.data.logLevel || 'Info',
    }));

    // Initialize empty log arrays for each device
    devList.forEach((d) => {
      if (!logs.value[d.id]) {
        logs.value[d.id] = [];
      }
    });
  } catch (e) {
    console.error('加载设备列表失败:', e);
  }
};

// Listen for child process logs via tauri events
const startLogListener = async () => {
  unlisten = await listen('child-log', (event) => {
    const { deviceId, level, message, time } = event.payload;

    if (!logs.value[deviceId]) {
      logs.value[deviceId] = [];
    }

    logs.value[deviceId].push({ time, level, message });

    // Limit log entries per device
    if (logs.value[deviceId].length > MAX_LOGS_PER_DEVICE) {
      logs.value[deviceId] = logs.value[deviceId].slice(-MAX_LOGS_PER_DEVICE);
    }

    // Auto scroll
    if (autoScroll.value) {
      nextTick(() => {
        if (logContainers.value) {
          logContainers.value.forEach((container) => {
            if (container) {
              container.scrollTop = container.scrollHeight;
            }
          });
        }
      });
    }
  });
};

// Filter devices
const filteredDevices = computed(() => {
  if (filterDevice.value === 'ALL') return devices.value;
  return devices.value.filter((d) => d.id === filterDevice.value);
});

// Get filtered logs for a device
const getDeviceLogs = (deviceId) => {
  const deviceLogs = logs.value[deviceId] || [];
  return deviceLogs.filter((log) => {
    if (filterLevel.value !== 'ALL' && log.level !== filterLevel.value) return false;
    if (filterText.value && !log.message.toLowerCase().includes(filterText.value.toLowerCase())) return false;
    return true;
  });
};

// Log level color classes
const getLogClass = (level) => {
  switch (level) {
    case 'Error':
      return 'text-red-500';
    case 'Warn':
      return 'text-yellow-500';
    case 'Info':
      return 'text-blue-400';
    case 'Debug':
      return 'text-gray-400';
    default:
      return 'text-gray-400';
  }
};

// Clear all logs
const clearLogs = () => {
  for (const key in logs.value) {
    logs.value[key] = [];
  }
};

// Update child process log level
const handleChildLogLevelChange = async (deviceId, level) => {
  try {
    await invoke('update_child_log_level_cmd', { deviceId, logLevel: level });
  } catch (e) {
    console.error('更新子进程日志级别失败:', e);
  }
};

onMounted(() => {
  loadDevices();
  startLogListener();
});

onUnmounted(() => {
  if (unlisten) {
    unlisten();
  }
});
</script>
