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
    if (normalized.includes('load')) return 'idle';
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
        sessionId: typeof record.sessionId === 'string' ? record.sessionId : null,
        status: record.status,
        currentScript: typeof record.currentScript === 'string' ? record.currentScript : null,
        message: typeof record.message === 'string' ? record.message : null,
    };
};

export const useDeviceStore = defineStore('device', () => {
    type DevicePendingAction =
        | 'spawning'
        | 'starting'
        | 'pausing'
        | 'stopping'
        | 'shuttingDown'
        | 'restarting';

    const devices = ref<DeviceTable[]>([]);
    const onlineDeviceIds = ref<string[]>([]);
    const deviceStatuses = ref<Record<string, DeviceRuntimeStatus>>({});
    const devicePendingActions = ref<Record<string, DevicePendingAction | null>>({});
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

    const resolveSavePendingAction = (nextDevice: DeviceTable): DevicePendingAction | null => {
        const previous = devices.value.find((device) => device.id === nextDevice.id) ?? null;
        const isRunning = onlineDeviceIds.value.includes(nextDevice.id);
        const nextEnabled = nextDevice.data.enable;

        if (!nextEnabled) {
            return isRunning ? 'shuttingDown' : null;
        }

        if (!isRunning) {
            return 'spawning';
        }

        if (previous && previous.data.cores.join(',') !== nextDevice.data.cores.join(',')) {
            return 'restarting';
        }

        return null;
    };

    const saveDevice = async (device: DeviceTable) => {
        const pendingAction = resolveSavePendingAction(device);
        if (pendingAction) {
            setDevicePendingAction(device.id, pendingAction);
        }
        try {
            await deviceService.save(device);
            await Promise.all([loadDevices(), refreshRunningDevices()]);
        } finally {
            if (pendingAction) {
                setDevicePendingAction(device.id, null);
            }
        }
    };

    const setDevicePendingAction = (deviceId: string, action: DevicePendingAction | null) => {
        devicePendingActions.value = {
            ...devicePendingActions.value,
            [deviceId]: action,
        };
    };

    const deleteDevice = async (deviceId: string) => {
        await deviceService.remove(deviceId);
        deviceStatuses.value = Object.fromEntries(
            Object.entries(deviceStatuses.value).filter(([currentId]) => currentId !== deviceId),
        );
        devicePendingActions.value = Object.fromEntries(
            Object.entries(devicePendingActions.value).filter(([currentId]) => currentId !== deviceId),
        );
        await loadDevices();
        await refreshRunningDevices();
    };

    const spawnDeviceProcess = async (deviceId: string) => {
        if (isDeviceBusy(deviceId)) {
            return;
        }
        setDevicePendingAction(deviceId, 'spawning');
        try {
            await deviceService.spawn(deviceId);
            await refreshRunningDevices();
        } finally {
            setDevicePendingAction(deviceId, null);
        }
    };

    const shutdownDeviceProcess = async (deviceId: string) => {
        if (isDeviceBusy(deviceId)) {
            return;
        }
        setDevicePendingAction(deviceId, 'shuttingDown');
        try {
            await deviceService.shutdown(deviceId);
            deviceStatuses.value = {
                ...deviceStatuses.value,
                [deviceId]: emptyStatus,
            };
            await refreshRunningDevices();
        } finally {
            setDevicePendingAction(deviceId, null);
        }
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
        if (isDeviceBusy(deviceId)) {
            return;
        }
        try {
            if (!onlineDeviceIds.value.includes(deviceId)) {
                setDevicePendingAction(deviceId, 'spawning');
                await deviceService.spawn(deviceId);
                await refreshRunningDevices();
            }
            setDevicePendingAction(deviceId, 'starting');
            await sendTaskStart(deviceId);
            await refreshRunningDevices();
        } finally {
            setDevicePendingAction(deviceId, null);
        }
    };

    const pauseDevice = async (deviceId: string) => {
        if (isDeviceBusy(deviceId)) {
            return;
        }
        if (!onlineDeviceIds.value.includes(deviceId)) {
            return;
        }
        setDevicePendingAction(deviceId, 'pausing');
        try {
            await sendTaskPause(deviceId);
        } finally {
            setDevicePendingAction(deviceId, null);
        }
    };

    const stopDevice = async (deviceId: string) => {
        if (isDeviceBusy(deviceId)) {
            return;
        }
        if (!onlineDeviceIds.value.includes(deviceId)) {
            return;
        }
        setDevicePendingAction(deviceId, 'stopping');
        try {
            await sendTaskStop(deviceId);
        } finally {
            setDevicePendingAction(deviceId, null);
        }
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
    const getDevicePendingAction = (deviceId: string) => devicePendingActions.value[deviceId] ?? null;
    const isDeviceBusy = (deviceId: string) => Boolean(getDevicePendingAction(deviceId));

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
        devicePendingActions,
        deviceStatuses,
        deviceSummary,
        devices,
        getDevicePendingAction,
        getDeviceStatus,
        initIpcListeners,
        isDeviceBusy,
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
