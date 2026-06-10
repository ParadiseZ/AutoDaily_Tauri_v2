import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { deviceService } from '@/services/deviceService';
import { useRuntimeStore } from '@/store/runtime';
import {
    activeRuntimeProgressPhases,
    connectionLabels,
    connectionTones,
    getPendingActionStartLabel,
    getPendingActionStopLabel,
    isConnectionStatusKind,
    isDeviceLifecycleStatus,
    isDeviceRuntimeReconcileAction,
    isDeviceRuntimeReconcileJobType,
    isDeviceRuntimeReconcilePhase,
    pendingActionMessages,
    progressPhaseLabels,
    stopButtonPendingActions,
    toDeviceConnectionKind,
    toDeviceStatusKind,
} from '@/utils/deviceRuntime';
import type {
    DeviceRuntimeSnapshot,
    DeviceConnectionKind,
    DeviceConnectionStatus,
    DeviceRuntimeControlView,
    DeviceRuntimePresence,
    DeviceRuntimeProgressView,
    DeviceRuntimeReconcileAction,
    DeviceRuntimeReconcileEvent,
    DeviceStatusEvent,
    DeviceRuntimeStatus,
    DeviceRuntimeView,
    DeviceSummary,
} from '@/types/app/domain';
import type { DeviceTable } from '@/types/bindings/DeviceTable';

const emptyStatus: DeviceRuntimeStatus = {
    rawStatus: 'stopped',
    kind: 'stopped',
    currentScript: null,
    message: null,
};

const emptyConnectionStatus: DeviceConnectionStatus = {
    kind: 'unknown',
    rawStatus: null,
    message: null,
    at: null,
};

const toStatusEvent = (payload: unknown): DeviceStatusEvent | null => {
    if (!payload || typeof payload !== 'object') {
        return null;
    }

    const record = payload as Record<string, unknown>;
    if (typeof record.deviceId !== 'string' || !isDeviceLifecycleStatus(record.status)) {
        return null;
    }

    return {
        deviceId: record.deviceId,
        sessionId: typeof record.sessionId === 'string' ? record.sessionId : null,
        status: record.status,
        currentScript: typeof record.currentScriptId === 'string' ? record.currentScriptId : null,
        message: typeof record.message === 'string' ? record.message : null,
    };
};

const toConnectionEvent = (
    payload: unknown,
): { deviceId: string; status: DeviceConnectionKind; rawStatus: DeviceConnectionStatus['rawStatus']; message: string | null; at: string | null } | null => {
    if (!payload || typeof payload !== 'object') {
        return null;
    }

    const record = payload as Record<string, unknown>;
    if (typeof record.deviceId !== 'string' || !isConnectionStatusKind(record.status)) {
        return null;
    }

    return {
        deviceId: record.deviceId,
        status: toDeviceConnectionKind(record.status),
        rawStatus: record.status,
        message: typeof record.message === 'string' ? record.message : null,
        at: typeof record.at === 'string' ? record.at : null,
    };
};

const toRuntimeReconcileAction = (value: unknown): DeviceRuntimeReconcileAction => {
    return isDeviceRuntimeReconcileAction(value) ? value : null;
};

const toRuntimeReconcileEvent = (payload: unknown): DeviceRuntimeReconcileEvent | null => {
    if (!payload || typeof payload !== 'object') {
        return null;
    }

    const record = payload as Record<string, unknown>;
    if (
        typeof record.jobId !== 'string' ||
        !isDeviceRuntimeReconcileJobType(record.jobType) ||
        typeof record.deviceId !== 'string' ||
        !isDeviceRuntimeReconcilePhase(record.phase) ||
        typeof record.at !== 'string'
    ) {
        return null;
    }

    return {
        jobId: record.jobId,
        jobType: record.jobType,
        deviceId: record.deviceId,
        phase: record.phase,
        action: toRuntimeReconcileAction(record.action),
        message: typeof record.message === 'string' ? record.message : null,
        at: record.at,
    };
};

export const useDeviceStore = defineStore('device', () => {
    type DevicePendingAction = Exclude<DeviceRuntimeReconcileAction, null>;

    const devices = ref<DeviceTable[]>([]);
    const onlineDeviceIds = ref<string[]>([]);
    const deviceStatuses = ref<Record<string, DeviceRuntimeStatus>>({});
    const deviceConnectionStatuses = ref<Record<string, DeviceConnectionStatus>>({});
    const devicePendingActions = ref<Record<string, DevicePendingAction | null>>({});
    const selectedDeviceId = ref<string | null>(null);
    const loading = ref(false);
    const cpuCount = ref(0);
    const runtimeStore = useRuntimeStore();

    const deviceSummary = computed<DeviceSummary>(() => ({
        total: devices.value.length,
        enabled: devices.value.filter((device) => device.data.enable).length,
        online: devices.value.filter((device) => isDeviceOnline(device.id)).length,
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
        const runningIds = new Set(onlineDeviceIds.value);
        deviceConnectionStatuses.value = Object.fromEntries(
            Object.entries(deviceConnectionStatuses.value).filter(([deviceId]) => runningIds.has(deviceId)),
        );
    };

    const refreshAll = async () => {
        await Promise.all([loadDevices(), refreshRunningDevices(), loadCpuCount()]);
    };

    const bootstrapEnabledDeviceProcesses = async () => {
        const pendingDeviceIds = devices.value
            .filter((device) => device.data.enable && !onlineDeviceIds.value.includes(device.id))
            .map((device) => device.id);

        if (pendingDeviceIds.length === 0) {
            return;
        }

        pendingDeviceIds.forEach((deviceId) => setDevicePendingAction(deviceId, 'spawning'));
        try {
            await deviceService.bootstrapEnabledProcesses();
            await refreshRunningDevices();
        } catch (error) {
            console.error('[device bootstrap] 启动启用设备子进程失败', error);
        } finally {
            pendingDeviceIds.forEach((deviceId) => {
                if (devicePendingActions.value[deviceId] === 'spawning') {
                    setDevicePendingAction(deviceId, null);
                }
            });
        }
    };

    const saveDevice = async (device: DeviceTable) => {
        try {
            await deviceService.save(device);
            await Promise.all([loadDevices(), refreshRunningDevices()]);
        } catch (error) {
            throw error;
        }
    };

    const setDevicePendingAction = (deviceId: string, action: DevicePendingAction | null) => {
        devicePendingActions.value = {
            ...devicePendingActions.value,
            [deviceId]: action,
        };
    };

    const setDeviceConnectionStatus = (deviceId: string, status: DeviceConnectionStatus) => {
        deviceConnectionStatuses.value = {
            ...deviceConnectionStatuses.value,
            [deviceId]: status,
        };
    };

    const deleteDevice = async (deviceId: string) => {
        await deviceService.remove(deviceId);
        deviceStatuses.value = Object.fromEntries(
            Object.entries(deviceStatuses.value).filter(([currentId]) => currentId !== deviceId),
        );
        deviceConnectionStatuses.value = Object.fromEntries(
            Object.entries(deviceConnectionStatuses.value).filter(([currentId]) => currentId !== deviceId),
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
            deviceConnectionStatuses.value = {
                ...deviceConnectionStatuses.value,
                [deviceId]: emptyConnectionStatus,
            };
            await refreshRunningDevices();
        } finally {
            setDevicePendingAction(deviceId, null);
        }
    };

    const probeEnabledDeviceConnections = async (deviceIds?: string[]) => {
        const targetIds = (deviceIds ?? devices.value.map((device) => device.id))
            .filter((deviceId, index, current) => current.indexOf(deviceId) === index)
            .filter((deviceId) => {
                const device = devices.value.find((item) => item.id === deviceId);
                return Boolean(device?.data.enable) && onlineDeviceIds.value.includes(deviceId);
            });

        if (!targetIds.length) {
            return;
        }

        targetIds.forEach((deviceId) =>
            setDeviceConnectionStatus(deviceId, {
                kind: 'checking',
                rawStatus: 'deviceChecking',
                message: '正在检查设备连接',
                at: null,
            }),
        );

        try {
            await deviceService.probeConnections(targetIds);
        } catch (error) {
            console.error('[device connection] 发起连接探测失败', error);
            targetIds.forEach((deviceId) =>
                setDeviceConnectionStatus(deviceId, {
                    kind: 'disconnected',
                    rawStatus: 'deviceDisconnected',
                    message: error instanceof Error ? error.message : '发起连接探测失败',
                    at: null,
                }),
            );
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
        setDevicePendingAction(deviceId, 'starting');
        try {
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
        if (!onlineDeviceIds.value.includes(deviceId)) {
            return emptyStatus;
        }

        const status = deviceStatuses.value[deviceId];
        const connectionStatus = deviceConnectionStatuses.value[deviceId] ?? emptyConnectionStatus;

        if (connectionStatus.kind === 'disconnected') {
            return {
                rawStatus: 'deviceDisconnected',
                kind: 'error',
                currentScript: status?.currentScript ?? null,
                message: connectionStatus.message ?? status?.message ?? '设备连接不可用',
            };
        }

        if (connectionStatus.kind === 'checking') {
            return {
                rawStatus: 'deviceChecking',
                kind: 'unknown',
                currentScript: status?.currentScript ?? null,
                message: connectionStatus.message ?? status?.message ?? '正在检查设备连接',
            };
        }

        if (status) {
            return status;
        }

        if (connectionStatus.kind === 'connected') {
            return {
                rawStatus: 'idle',
                kind: 'idle',
                currentScript: null,
                message: connectionStatus.message ?? null,
            };
        }

        return {
            rawStatus: 'deviceUnknown',
            kind: 'unknown',
            currentScript: null,
            message: '等待连接状态检测',
        };
    };

    const getDeviceConnectionStatus = (deviceId: string): DeviceConnectionStatus =>
        deviceConnectionStatuses.value[deviceId] ?? emptyConnectionStatus;

    const hydrateRuntimeSnapshots = (snapshots: DeviceRuntimeSnapshot[]) => {
        const nextStatuses = { ...deviceStatuses.value };
        const nextConnections = { ...deviceConnectionStatuses.value };

        for (const snapshot of snapshots) {
            if (snapshot.status) {
                nextStatuses[snapshot.deviceId] = {
                    rawStatus: snapshot.status.status,
                    kind: toDeviceStatusKind(snapshot.status.status),
                    currentScript: snapshot.status.currentScriptId ?? null,
                    message: snapshot.status.message ?? null,
                };
            }

            if (snapshot.connection) {
                nextConnections[snapshot.deviceId] = {
                    kind: toDeviceConnectionKind(snapshot.connection.status),
                    rawStatus: snapshot.connection.status,
                    message: snapshot.connection.message ?? null,
                    at: snapshot.connection.at ?? null,
                };
            }
        }

        deviceStatuses.value = nextStatuses;
        deviceConnectionStatuses.value = nextConnections;
    };

    const getConnectionLabel = (status: DeviceConnectionStatus) => {
        return connectionLabels[status.kind];
    };

    const getConnectionTone = (
        status: DeviceConnectionStatus,
    ): DeviceRuntimeView['connectionTone'] => connectionTones[status.kind];

    const getPendingMessage = (deviceId: string) => {
        const action = getDevicePendingAction(deviceId);
        return action ? pendingActionMessages[action] : null;
    };

    const getProgressView = (deviceId: string): DeviceRuntimeProgressView => {
        const runtimeResult = runtimeStore.getRuntimeResult(deviceId);
        const connectionStatus = getDeviceConnectionStatus(deviceId);
        const latest = runtimeResult.latestProgress;
        const phase = latest?.phase ?? null;
        const fallbackMessage = getPendingMessage(deviceId) ?? connectionStatus.message ?? null;
        const message = latest?.message ?? fallbackMessage;

        let tone: DeviceRuntimeProgressView['tone'] = 'neutral';
        if (phase === 'failed' || phase === 'deviceDisconnected') tone = 'danger';
        else if (phase === 'childProcessExited' || phase === 'childProcessCrashed') tone = 'danger';
        else if (phase === 'completed' || phase === 'deviceConnected') tone = 'success';
        else if (phase === 'paused') tone = 'warning';
        else if (
            phase &&
            activeRuntimeProgressPhases.has(phase)
        ) tone = 'info';
        else if (connectionStatus.kind === 'checking') tone = 'info';

        return {
            phase,
            label:
                (phase ? progressPhaseLabels[phase] ?? phase : null) ??
                (connectionStatus.kind === 'checking' ? '连接准备中' : '暂无'),
            tone,
            message,
            at: latest?.at ?? connectionStatus.at ?? null,
        };
    };

    const isDeviceOnline = (deviceId: string) =>
        onlineDeviceIds.value.includes(deviceId) &&
        getDeviceConnectionStatus(deviceId).kind === 'connected';

    const getDevicePresence = (deviceId: string): DeviceRuntimePresence => {
        if (!onlineDeviceIds.value.includes(deviceId)) {
            return { label: '离线', tone: 'neutral' as const, icon: 'status-offline' };
        }

        const connectionStatus = getDeviceConnectionStatus(deviceId);
        if (connectionStatus.kind === 'connected') {
            return { label: '在线', tone: 'success' as const, icon: 'status-online' };
        }
        if (connectionStatus.kind === 'checking') {
            return { label: '检查中', tone: 'info' as const, icon: 'status-offline' };
        }
        if (connectionStatus.kind === 'disconnected') {
            return { label: '连接断开', tone: 'danger' as const, icon: 'status-offline' };
        }

        return { label: '待检测', tone: 'warning' as const, icon: 'status-offline' };
    };

    const getControlView = (deviceId: string): DeviceRuntimeControlView => {
        const status = getDeviceStatus(deviceId);
        const connectionStatus = getDeviceConnectionStatus(deviceId);
        const progress = getProgressView(deviceId);
        const pendingAction = getDevicePendingAction(deviceId);
        const showStopButton =
            status.kind === 'running' ||
            status.kind === 'paused' ||
            Boolean(pendingAction && stopButtonPendingActions.has(pendingAction)) ||
            Boolean(progress.phase && activeRuntimeProgressPhases.has(progress.phase)) ||
            connectionStatus.kind === 'checking';

        return {
            showStopButton,
            startLabel: getPendingActionStartLabel(pendingAction),
            stopLabel: getPendingActionStopLabel(pendingAction),
        };
    };

    const getDeviceRuntimeView = (deviceId: string): DeviceRuntimeView => {
        const status = getDeviceStatus(deviceId);
        const connectionStatus = getDeviceConnectionStatus(deviceId);
        return {
            status,
            connectionStatus,
            connectionLabel: getConnectionLabel(connectionStatus),
            connectionTone: getConnectionTone(connectionStatus),
            presence: getDevicePresence(deviceId),
            progress: getProgressView(deviceId),
            pendingMessage: getPendingMessage(deviceId),
            controls: getControlView(deviceId),
        };
    };

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
                    kind: toDeviceStatusKind(payload.status),
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
                    rawStatus: payload.status,
                    kind: 'error',
                    currentScript: payload.currentScript,
                    message: payload.message,
                },
            };
        });

        await listen('device-connection-status', (event) => {
            const payload = toConnectionEvent(event.payload);
            if (!payload) {
                return;
            }

            setDeviceConnectionStatus(payload.deviceId, {
                kind: payload.status,
                rawStatus: payload.rawStatus,
                message: payload.message,
                at: payload.at,
            });
        });

        await listen('device-runtime-reconcile', (event) => {
            const payload = toRuntimeReconcileEvent(event.payload);
            if (!payload) {
                return;
            }

            if (payload.phase === 'running' && payload.action) {
                setDevicePendingAction(payload.deviceId, payload.action);
            }

            if (payload.phase === 'succeeded' || payload.phase === 'failed') {
                if (
                    devicePendingActions.value[payload.deviceId] === 'syncing' ||
                    devicePendingActions.value[payload.deviceId] === payload.action
                ) {
                    setDevicePendingAction(payload.deviceId, null);
                }

                if (payload.phase === 'failed' && payload.message) {
                    console.error('[device runtime reconcile] 后台协调失败', payload);
                }

                void refreshRunningDevices();
            }
        });

        ipcInitialized = true;
    };

    return {
        bootstrapEnabledDeviceProcesses,
        cpuCount,
        deleteDevice,
        deviceConnectionStatuses,
        devicePendingActions,
        deviceStatuses,
        deviceSummary,
        devices,
        getDeviceConnectionStatus,
        getDevicePendingAction,
        getDevicePresence,
        getDeviceRuntimeView,
        getDeviceStatus,
        hydrateRuntimeSnapshots,
        initIpcListeners,
        isDeviceBusy,
        isDeviceOnline,
        loadCpuCount,
        loadDevices,
        loading,
        onlineDeviceIds,
        pauseDevice,
        pauseDevices,
        probeEnabledDeviceConnections,
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
