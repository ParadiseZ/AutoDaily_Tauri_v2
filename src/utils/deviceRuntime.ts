import type { ConnectionStatusKind } from '@/types/bindings/ConnectionStatusKind';
import type { DeviceLifecycleStatus } from '@/types/bindings/DeviceLifecycleStatus';
import type { DeviceRuntimeProgressPhase } from '@/types/bindings/DeviceRuntimeProgressPhase';
import type { DeviceRuntimeReconcileAction } from '@/types/bindings/DeviceRuntimeReconcileAction';
import type { DeviceRuntimeReconcileJobType } from '@/types/bindings/DeviceRuntimeReconcileJobType';
import type { DeviceRuntimeReconcilePhase } from '@/types/bindings/DeviceRuntimeReconcilePhase';
import type {
    DeviceConnectionKind,
    DeviceRuntimeStatus,
    DeviceRuntimeView,
} from '@/types/app/domain';

export const deviceLifecycleStatusValues = [
    'initializing',
    'loaded',
    'idle',
    'running',
    'paused',
    'stopping',
    'stopped',
    'error',
] as const satisfies readonly DeviceLifecycleStatus[];

export const connectionStatusKindValues = [
    'deviceUnknown',
    'deviceChecking',
    'shellProbeChecking',
    'emulatorStarting',
    'emulatorWaiting',
    'deviceConnected',
    'deviceDisconnected',
] as const satisfies readonly ConnectionStatusKind[];

export const deviceRuntimeReconcileActionValues = [
    'spawning',
    'starting',
    'pausing',
    'stopping',
    'shuttingDown',
    'restarting',
    'syncing',
] as const satisfies readonly DeviceRuntimeReconcileAction[];

export const deviceRuntimeReconcilePhaseValues = [
    'queued',
    'running',
    'succeeded',
    'failed',
] as const satisfies readonly DeviceRuntimeReconcilePhase[];

export const deviceRuntimeReconcileJobTypeValues = [
    'deviceConfig',
    'deviceSessionRefresh',
] as const satisfies readonly DeviceRuntimeReconcileJobType[];

export const progressPhaseLabels = {
    idle: '空闲',
    loading: '加载中',
    planning: '规划中',
    childRuntimeStarting: '启动运行时',
    childIpcWaiting: '等待运行时 IPC',
    childIpcReady: '运行时已就绪',
    deviceChecking: '准备连接',
    shellProbeChecking: '连接探测中',
    emulatorStarting: '模拟器启动中',
    emulatorWaiting: '等待模拟器',
    deviceConnected: '设备已连接',
    deviceDisconnected: '设备已断开',
    executing: '执行中',
    paused: '已暂停',
    completed: '已完成',
    failed: '失败',
    childProcessExited: '运行时已退出',
    childProcessCrashed: '运行时异常退出',
} as const satisfies Record<DeviceRuntimeProgressPhase, string>;

export const activeRuntimeProgressPhases = new Set<DeviceRuntimeProgressPhase>([
    'loading',
    'planning',
    'childRuntimeStarting',
    'childIpcWaiting',
    'childIpcReady',
    'deviceChecking',
    'shellProbeChecking',
    'emulatorStarting',
    'emulatorWaiting',
    'executing',
    'paused',
]);

export const stopButtonPendingActions = new Set<DeviceRuntimeReconcileAction>([
    'starting',
    'stopping',
    'shuttingDown',
]);

export const connectionLabels = {
    unknown: '未检测',
    checking: '连接准备中',
    connected: '已连接',
    disconnected: '未连接',
} as const satisfies Record<DeviceConnectionKind, string>;

export const connectionTones = {
    unknown: 'warning',
    checking: 'info',
    connected: 'success',
    disconnected: 'danger',
} as const satisfies Record<DeviceConnectionKind, DeviceRuntimeView['connectionTone']>;

export const pendingActionMessages = {
    spawning: '正在启动设备子进程...',
    restarting: '正在重启设备子进程...',
    syncing: '正在同步设备运行时...',
    starting: '正在启动设备队列...',
    pausing: '正在暂停当前设备...',
    stopping: '正在停止当前设备...',
    shuttingDown: '正在关闭设备子进程...',
} as const satisfies Record<DeviceRuntimeReconcileAction, string>;

export const pendingActionStartLabels = {
    spawning: '正在启动子进程...',
    restarting: '正在重启子进程...',
    syncing: '正在同步运行时...',
    starting: '正在启动队列...',
} as const satisfies Partial<Record<DeviceRuntimeReconcileAction, string>>;

export const pendingActionStopLabels = {
    stopping: '正在停止...',
    shuttingDown: '正在关闭子进程...',
    starting: '准备中...',
} as const satisfies Partial<Record<DeviceRuntimeReconcileAction, string>>;

export const getPendingActionStartLabel = (action: DeviceRuntimeReconcileAction | null) => {
    switch (action) {
        case 'spawning':
        case 'restarting':
        case 'syncing':
        case 'starting':
            return pendingActionStartLabels[action];
        default:
            return '运行';
    }
};

export const getPendingActionStopLabel = (action: DeviceRuntimeReconcileAction | null) => {
    switch (action) {
        case 'stopping':
        case 'shuttingDown':
        case 'starting':
            return pendingActionStopLabels[action];
        default:
            return '停止';
    }
};

export const isDeviceLifecycleStatus = (value: unknown): value is DeviceLifecycleStatus =>
    typeof value === 'string' &&
    (deviceLifecycleStatusValues as readonly string[]).includes(value);

export const isConnectionStatusKind = (value: unknown): value is ConnectionStatusKind =>
    typeof value === 'string' &&
    (connectionStatusKindValues as readonly string[]).includes(value);

export const isDeviceRuntimeReconcileAction = (value: unknown): value is DeviceRuntimeReconcileAction =>
    typeof value === 'string' &&
    (deviceRuntimeReconcileActionValues as readonly string[]).includes(value);

export const isDeviceRuntimeReconcilePhase = (value: unknown): value is DeviceRuntimeReconcilePhase =>
    typeof value === 'string' &&
    (deviceRuntimeReconcilePhaseValues as readonly string[]).includes(value);

export const isDeviceRuntimeReconcileJobType = (value: unknown): value is DeviceRuntimeReconcileJobType =>
    typeof value === 'string' &&
    (deviceRuntimeReconcileJobTypeValues as readonly string[]).includes(value);

export const toDeviceStatusKind = (status: DeviceLifecycleStatus): DeviceRuntimeStatus['kind'] => {
    switch (status) {
        case 'running':
            return 'running';
        case 'paused':
            return 'paused';
        case 'stopping':
        case 'stopped':
            return 'stopped';
        case 'error':
            return 'error';
        case 'initializing':
        case 'loaded':
        case 'idle':
            return 'idle';
    }
};

export const toDeviceConnectionKind = (status: ConnectionStatusKind): DeviceConnectionKind => {
    switch (status) {
        case 'deviceChecking':
        case 'shellProbeChecking':
        case 'emulatorStarting':
        case 'emulatorWaiting':
            return 'checking';
        case 'deviceConnected':
            return 'connected';
        case 'deviceDisconnected':
            return 'disconnected';
        case 'deviceUnknown':
            return 'unknown';
    }
};
