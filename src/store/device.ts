import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { deviceService } from '@/services/deviceService';
import type { DeviceRuntimeStatus, DeviceStatusEvent, DeviceSummary } from '@/types/app/domain';
import type { DeviceTable } from '@/types/bindings/DeviceTable';

const emptyStatus: DeviceRuntimeStatus = {
    rawStatus: 'Stopped',
    kind: 'stopped',
    currentScript: null,
    message: null,
};

const toStatusKind = (status: string): DeviceRuntimeStatus['kind'] => {
    const normalized = status.toLowerCase();
    if (normalized.includes('run') || normalized.includes('work') || normalized.includes('busy')) return 'running';
    if (normalized.includes('pause')) return 'paused';
    if (normalized.includes('stop') || normalized.includes('shutdown')) return 'stopped';
    if (normalized.includes('error') || normalized.includes('fail')) return 'error';
    if (normalized.includes('idle') || normalized.includes('wait')) return 'idle';
    return 'unknown';
};

const toStatusEvent = (payload: unknown): DeviceStatusEvent | null => {
    if (!payload || typeof payload !== 'object') {
        return null;
    }

    const record = payload as Record<string, unknown>;
    if (typeof record.deviceId !== 'string' || typeof record.status !== 'string') {
        return null;
    }

    return {
        deviceId: record.deviceId,
        status: record.status,
        currentScript: typeof record.currentScript === 'string' ? record.currentScript : null,
        message: typeof record.message === 'string' ? record.message : null,
    };
};

export const useDeviceStore = defineStore('device', () => {
    const devices = ref<DeviceTable[]>([]);
    const onlineDeviceIds = ref<string[]>([]);
    const deviceStatuses = ref<Record<string, DeviceRuntimeStatus>>({});
    const selectedDeviceId = ref<string | null>(null);
    const loading = ref(false);
    const cpuCount = ref(0);

    const deviceSummary = computed<DeviceSummary>(() => ({
        total: devices.value.length,
        enabled: devices.value.filter((device) => device.data.enable).length,
        online: onlineDeviceIds.value.length,
        running: Object.values(deviceStatuses.value).filter((item) => item.kind === 'running').length,
    }));

    const selectedDevice = computed(() =>
        devices.value.find((device) => device.id === selectedDeviceId.value) ?? null,
    );

    const loadDevices = async () => {
        loading.value = true;
        try {
            devices.value = await deviceService.list();
            if (!selectedDeviceId.value && devices.value.length > 0) {
                selectedDeviceId.value = devices.value[0].id;
            }
        } finally {
            loading.value = false;
        }
    };

    const loadCpuCount = async () => {
        cpuCount.value = await deviceService.getCpuCount();
    };

    const refreshRunningDevices = async () => {
        onlineDeviceIds.value = await deviceService.getRunningDeviceIds();
    };

    const refreshAll = async () => {
        await Promise.all([loadDevices(), refreshRunningDevices(), loadCpuCount()]);
    };

    const saveDevice = async (device: DeviceTable) => {
        await deviceService.save(device);
        await loadDevices();
    };

    const deleteDevice = async (deviceId: string) => {
        await deviceService.remove(deviceId);
        deviceStatuses.value = Object.fromEntries(
            Object.entries(deviceStatuses.value).filter(([currentId]) => currentId !== deviceId),
        );
        await loadDevices();
        await refreshRunningDevices();
    };

    const spawnDeviceProcess = async (deviceId: string) => {
        await deviceService.spawn(deviceId);
        await refreshRunningDevices();
    };

    const shutdownDeviceProcess = async (deviceId: string) => {
        await deviceService.shutdown(deviceId);
        deviceStatuses.value = {
            ...deviceStatuses.value,
            [deviceId]: emptyStatus,
        };
        await refreshRunningDevices();
    };

    const sendTaskStart = async (deviceId: string) => {
        await deviceService.start(deviceId);
    };

    const sendTaskStop = async (deviceId: string) => {
        await deviceService.stop(deviceId);
    };

    const sendTaskPause = async (deviceId: string) => {
        await deviceService.pause(deviceId);
    };

    const startDevice = async (deviceId: string) => {
        if (!onlineDeviceIds.value.includes(deviceId)) {
            await spawnDeviceProcess(deviceId);
        }
        await sendTaskStart(deviceId);
    };

    const pauseDevice = async (deviceId: string) => {
        if (!onlineDeviceIds.value.includes(deviceId)) {
            return;
        }
        await sendTaskPause(deviceId);
    };

    const stopDevice = async (deviceId: string) => {
        if (!onlineDeviceIds.value.includes(deviceId)) {
            return;
        }
        await sendTaskStop(deviceId);
    };

    const startDevices = async (deviceIds: string[]) => {
        await Promise.all(deviceIds.map(startDevice));
        await refreshRunningDevices();
    };

    const pauseDevices = async (deviceIds: string[]) => {
        await Promise.all(deviceIds.map(pauseDevice));
    };

    const stopDevices = async (deviceIds: string[]) => {
        await Promise.all(deviceIds.map(stopDevice));
    };

    const shutdownDevices = async (deviceIds: string[]) => {
        await Promise.all(deviceIds.map(shutdownDeviceProcess));
        await refreshRunningDevices();
    };

    const getDeviceStatus = (deviceId: string): DeviceRuntimeStatus => {
        const status = deviceStatuses.value[deviceId];
        if (status) {
            return status;
        }

        return onlineDeviceIds.value.includes(deviceId)
            ? { rawStatus: 'Idle', kind: 'idle', currentScript: null, message: null }
            : emptyStatus;
    };

    const isDeviceOnline = (deviceId: string) => onlineDeviceIds.value.includes(deviceId);

    let ipcInitialized = false;
    const initIpcListeners = async () => {
        if (ipcInitialized) {
            return;
        }

        await listen('device-status', (event) => {
            const payload = toStatusEvent(event.payload);
            if (!payload) {
                return;
            }

            deviceStatuses.value = {
                ...deviceStatuses.value,
                [payload.deviceId]: {
                    rawStatus: payload.status,
                    kind: toStatusKind(payload.status),
                    currentScript: payload.currentScript,
                    message: payload.message,
                },
            };
        });

        await listen('device-error', (event) => {
            const payload = toStatusEvent(event.payload);
            if (!payload) {
                return;
            }

            deviceStatuses.value = {
                ...deviceStatuses.value,
                [payload.deviceId]: {
                    rawStatus: payload.status || 'Error',
                    kind: 'error',
                    currentScript: payload.currentScript,
                    message: payload.message,
                },
            };
        });

        ipcInitialized = true;
    };

    return {
        cpuCount,
        deleteDevice,
        deviceStatuses,
        deviceSummary,
        devices,
        getDeviceStatus,
        initIpcListeners,
        isDeviceOnline,
        loadCpuCount,
        loadDevices,
        loading,
        onlineDeviceIds,
        pauseDevice,
        pauseDevices,
        refreshAll,
        refreshRunningDevices,
        saveDevice,
        selectedDevice,
        selectedDeviceId,
        sendTaskPause,
        sendTaskStart,
        sendTaskStop,
        shutdownDeviceProcess,
        shutdownDevices,
        spawnDeviceProcess,
        startDevice,
        startDevices,
        stopDevice,
        stopDevices,
    };
});
