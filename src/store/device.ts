import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '../utils/api';
import { listen } from '@tauri-apps/api/event';
import type { DeviceTable } from '../types/bindings/DeviceTable';

export const useDeviceStore = defineStore('device', () => {
    // 1. 所有已配置的设备列表 (来自数据库 get_all_devices_cmd)
    const devices = ref<DeviceTable[]>([]);
    
    // 2. 在线的设备 ID 列表 (也就是已经拥有运行中子进程的设备 cmd_get_running_devices)
    const onlineDeviceIds = ref<string[]>([]);
    
    // 3. 设备的实时执行状态 (来自子进程 IPC 广播的 device-status)
    // 包含当前执行的脚本、详细运行状态等
    const deviceStatuses = ref<Record<string, { status: string, currentScript?: string, message?: string }>>({});

    const selectedDeviceId = ref<string | null>(null);

    // 计算属性: 在线的设备数量 (子进程存活)
    const onlineDevicesCount = computed(() => onlineDeviceIds.value.length);
    
    // 计算属性: 运行中的设备数量 (IPC 状态为执行相关的状态，这里暂时假定 Running 或执行中状态，根据具体枚举调整)
    const runningDevicesCount = computed(() => {
        return Object.values(deviceStatuses.value).filter(s => 
            s.status !== 'Idle' && s.status !== 'Error' && s.status !== 'Stopped'
        ).length;
    });

    // ========== API 调用 ==========
    // 加载配置的设备
    const loadDevices = async () => {
        try {
            const res = await invoke('get_all_devices_cmd');
            if (res) devices.value = res; 
        } catch (error) {
            console.error('Failed to load devices:', error);
        }
    };

    // 获取当前拥有子进程的设备 (即在线设备)
    const fetchOnlineDevices = async () => {
        try {
            const res = await invoke('cmd_get_running_devices');
            if (res) onlineDeviceIds.value = res;
        } catch (error) {
            console.error('Failed to fetch online/running process devices:', error);
        }
    };

    // ========== 进程级控制 (上下线) ==========
    const spawnDeviceProcess = async (deviceId: string) => {
        try {
            await invoke('cmd_spawn_device', { deviceId });
            await fetchOnlineDevices();
        } catch (error) {
            console.error(`Failed to spawn device ${deviceId}:`, error);
        }
    };

    const shutdownDeviceProcess = async (deviceId: string) => {
        try {
            await invoke('cmd_device_shutdown', { deviceId });
            await fetchOnlineDevices();
            delete deviceStatuses.value[deviceId];
        } catch (error) {
            console.error(`Failed to shutdown device ${deviceId}:`, error);
        }
    };

    // ========== 任务级控制 (向在线子进程发指令) ==========
    const sendTaskStart = async (deviceId: string) => {
        await invoke('cmd_device_start', { deviceId });
    };

    const sendTaskStop = async (deviceId: string) => {
        await invoke('cmd_device_stop', { deviceId });
    };

    const sendTaskPause = async (deviceId: string) => {
        await invoke('cmd_device_pause', { deviceId });
    };

    // ========== IPC 事件监听 (Tauri Events) ==========
    let _ipcInitialized = false;
    const initIpcListeners = async () => {
        if (_ipcInitialized) return;
        
        // 监听设备状态报告
        await listen('device-status', (event: any) => {
            const payload = event.payload;
            if (payload && payload.deviceId) {
                deviceStatuses.value[payload.deviceId] = {
                    status: payload.status,
                    currentScript: payload.currentScript,
                    message: payload.message
                };
            }
        });

        // 监听设备错误报告
        await listen('device-error', (event: any) => {
            const payload = event.payload;
            if (payload && payload.deviceId) {
                console.error(`Device ${payload.deviceId} IPC Error: [${payload.code}] ${payload.message}`);
                deviceStatuses.value[payload.deviceId] = {
                    status: 'Error',
                    message: payload.message
                };
            }
        });
        
        _ipcInitialized = true;
    };

    return {
        devices,
        onlineDeviceIds,
        deviceStatuses,
        selectedDeviceId,
        onlineDevicesCount,
        runningDevicesCount,
        loadDevices,
        fetchOnlineDevices,
        spawnDeviceProcess,
        shutdownDeviceProcess,
        sendTaskStart,
        sendTaskStop,
        sendTaskPause,
        initIpcListeners
    };
});
