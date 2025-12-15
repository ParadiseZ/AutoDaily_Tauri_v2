<template>
  <div class="h-full flex flex-col p-4">
    <!-- Top Bar -->
    <div class="flex gap-4 mb-4 items-center bg-base-100 p-2 rounded-lg shadow">
      <select class="select select-bordered select-sm" v-model="filterLevel">
        <option value="ALL">All Levels</option>
        <option value="INFO">Info</option>
        <option value="WARN">Warn</option>
        <option value="ERROR">Error</option>
      </select>
      
      <select class="select select-bordered select-sm" v-model="filterDevice">
        <option value="ALL">All Devices</option>
        <option v-for="dev in devices" :key="dev.id" :value="dev.id">{{ dev.name }}</option>
      </select>
      
      <input type="text" placeholder="Filter logs..." class="input input-bordered input-sm flex-1" v-model="filterText" />
    </div>

    <!-- Logs Container -->
    <div class="flex-1 overflow-x-auto flex gap-4">
        <div v-for="device in filteredDevices" :key="device.id" class="min-w-[300px] flex-1 bg-black text-green-400 p-2 rounded font-mono text-xs overflow-y-auto flex flex-col">
            <div class="font-bold text-white border-b border-gray-700 mb-2 sticky top-0 bg-black pb-1">{{ device.name }}</div>
            <div v-for="(log, idx) in getDeviceLogs(device.id)" :key="idx" class="whitespace-pre-wrap break-all hover:bg-gray-900">
                <span :class="getLogClass(log.level)">[{{ log.time }}] [{{ log.level }}]</span> {{ log.message }}
            </div>
             <div v-if="getDeviceLogs(device.id).length === 0" class="text-gray-600 italic text-center mt-4">No logs</div>
        </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const filterLevel = ref('ALL');
const filterDevice = ref('ALL');
const filterText = ref('');

const devices = ref([]);
const logs = ref({});

// Load devices to populate dropdown and columns
const loadDevices = async () => {
    try {
        const res = await invoke('get_all_devices_cmd');
        const devList = Object.values(res);
        devices.value = devList.map(d => ({ id: d.deviceId, name: d.deviceName }));
        
        // Initialize logs for each device (mock)
        devList.forEach(d => {
            if (!logs.value[d.deviceId]) {
                logs.value[d.deviceId] = [
                    { time: new Date().toLocaleTimeString(), level: 'INFO', message: 'Log system initialized.' }
                ];
            }
        });
    } catch (e) {
        console.error(e);
        // Fallback mock
        devices.value = [{ id: 'mock1', name: 'Mock Device 1' }, { id: 'mock2', name: 'Mock Device 2' }];
        logs.value = {
            'mock1': [{ time: '12:00:00', level: 'INFO', message: 'Mock log 1' }],
            'mock2': [{ time: '12:00:00', level: 'WARN', message: 'Mock log 2' }]
        };
    }
};

const filteredDevices = computed(() => {
    if (filterDevice.value === 'ALL') return devices.value;
    return devices.value.filter(d => d.id === filterDevice.value);
});

const getDeviceLogs = (deviceId) => {
    const deviceLogs = logs.value[deviceId] || [];
    return deviceLogs.filter(log => {
        if (filterLevel.value !== 'ALL' && log.level !== filterLevel.value) return false;
        if (filterText.value && !log.message.toLowerCase().includes(filterText.value.toLowerCase())) return false;
        return true;
    });
};

const getLogClass = (level) => {
    switch(level) {
        case 'ERROR': return 'text-red-500';
        case 'WARN': return 'text-yellow-500';
        case 'INFO': return 'text-blue-400';
        case 'DEBUG': return 'text-gray-400';
        default: return 'text-gray-400';
    }
};

onMounted(() => {
    loadDevices();
});
</script>
