import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import type {
    RuntimeProgressEvent,
    RuntimeResultProjection,
    RuntimeScheduleEvent,
    RuntimeTimeoutEvent,
} from '@/types/app/domain';

const MAX_SCHEDULE_EVENTS = 50;

const normalizeProgressEvent = (payload: unknown): RuntimeProgressEvent | null => {
    if (!payload || typeof payload !== 'object') {
        return null;
    }

    const record = payload as Record<string, unknown>;
    if (
        typeof record.deviceId !== 'string' ||
        typeof record.phase !== 'string' ||
        typeof record.at !== 'string'
    ) {
        return null;
    }

    return {
        deviceId: record.deviceId,
        sessionId: typeof record.sessionId === 'string' ? record.sessionId : null,
        assignmentId: typeof record.assignmentId === 'string' ? record.assignmentId : null,
        scriptId: typeof record.scriptId === 'string' ? record.scriptId : null,
        taskId: typeof record.taskId === 'string' ? record.taskId : null,
        stepId: typeof record.stepId === 'string' ? record.stepId : null,
        phase: record.phase,
        message: typeof record.message === 'string' ? record.message : null,
        at: record.at,
    };
};

const normalizeScheduleEvent = (payload: unknown): RuntimeScheduleEvent | null => {
    if (!payload || typeof payload !== 'object') {
        return null;
    }

    const record = payload as Record<string, unknown>;
    if (
        typeof record.deviceId !== 'string' ||
        typeof record.status !== 'string' ||
        typeof record.at !== 'string'
    ) {
        return null;
    }

    return {
        deviceId: record.deviceId,
        sessionId: typeof record.sessionId === 'string' ? record.sessionId : null,
        executionId: typeof record.executionId === 'string' ? record.executionId : null,
        assignmentId: typeof record.assignmentId === 'string' ? record.assignmentId : null,
        scriptId: typeof record.scriptId === 'string' ? record.scriptId : null,
        taskId: typeof record.taskId === 'string' ? record.taskId : null,
        stepId: typeof record.stepId === 'string' ? record.stepId : null,
        status: record.status,
        message: typeof record.message === 'string' ? record.message : null,
        at: record.at,
    };
};

const normalizeTimeoutMetaValue = (value: string | undefined) => {
    if (!value || value === '<none>') {
        return null;
    }
    return value;
};

const parseTimeoutMessage = (message: string) => {
    const match = message.match(/^action=([^;]+);\s*page=([^;]+);\s*signature=([^;]+);\s*(.*)$/);
    if (!match) {
        return {
            timeoutAction: null,
            pageFingerprint: null,
            actionSignature: null,
            detail: message,
        };
    }

    return {
        timeoutAction: normalizeTimeoutMetaValue(match[1]?.trim()),
        pageFingerprint: normalizeTimeoutMetaValue(match[2]?.trim()),
        actionSignature: normalizeTimeoutMetaValue(match[3]?.trim()),
        detail: match[4]?.trim() || message,
    };
};

const normalizeTimeoutEvent = (payload: unknown): RuntimeTimeoutEvent | null => {
    if (!payload || typeof payload !== 'object') {
        return null;
    }

    const record = payload as Record<string, unknown>;
    if (typeof record.deviceId !== 'string' || typeof record.message !== 'string' || typeof record.at !== 'string') {
        return null;
    }

    const parsedMessage = parseTimeoutMessage(record.message);

    return {
        deviceId: record.deviceId,
        sessionId: typeof record.sessionId === 'string' ? record.sessionId : null,
        assignmentId: typeof record.assignmentId === 'string' ? record.assignmentId : null,
        scriptId: typeof record.scriptId === 'string' ? record.scriptId : null,
        taskId: typeof record.taskId === 'string' ? record.taskId : null,
        stepId: typeof record.stepId === 'string' ? record.stepId : null,
        ...parsedMessage,
        message: record.message,
        at: record.at,
    };
};

const parseRuntimeTime = (value: string | null | undefined) => {
    if (!value) {
        return 0;
    }
    const numeric = Number(value);
    if (Number.isFinite(numeric)) {
        return numeric;
    }
    const parsed = new Date(value).getTime();
    return Number.isFinite(parsed) ? parsed : 0;
};

const eventMatchesTimeoutScope = (event: RuntimeScheduleEvent, timeout: RuntimeTimeoutEvent) => {
    if (timeout.assignmentId && event.assignmentId !== timeout.assignmentId) {
        return false;
    }
    if (!timeout.assignmentId && timeout.scriptId && event.scriptId !== timeout.scriptId) {
        return false;
    }
    if (timeout.taskId && event.taskId && event.taskId !== timeout.taskId) {
        return false;
    }
    return true;
};

const resolveTimeoutActionResult = (
    timeout: RuntimeTimeoutEvent | null,
    schedules: RuntimeScheduleEvent[],
    progress: RuntimeProgressEvent | null,
): RuntimeResultProjection['timeoutActionResult'] => {
    if (!timeout) {
        return 'none';
    }

    const timeoutAt = parseRuntimeTime(timeout.at);
    const followUp = [...schedules]
        .reverse()
        .find((event) => parseRuntimeTime(event.at) >= timeoutAt && eventMatchesTimeoutScope(event, timeout));

    if (followUp?.status === 'Skipped') {
        return 'skipped';
    }
    if (followUp?.status === 'Success') {
        return timeout.timeoutAction === 'RunRecoveryTask' ? 'recovered' : 'skipped';
    }
    if (followUp?.status === 'Failed') {
        return 'failed';
    }
    if (timeout.timeoutAction === 'StopExecution' && (progress?.phase === 'Idle' || progress?.phase === 'Failed')) {
        return 'stopped';
    }

    return 'pending';
};

const latestRuntimeTimestamp = (
    progress: RuntimeProgressEvent | null,
    schedule: RuntimeScheduleEvent | null,
    timeout: RuntimeTimeoutEvent | null,
) => {
    const latest = [progress?.at, schedule?.at, timeout?.at]
        .filter((value): value is string => Boolean(value))
        .sort((left, right) => parseRuntimeTime(right) - parseRuntimeTime(left))[0];
    return latest ?? null;
};

export const useRuntimeStore = defineStore('runtime', () => {
    const initialized = ref(false);
    const latestProgressByDevice = ref<Record<string, RuntimeProgressEvent | null>>({});
    const scheduleEventsByDevice = ref<Record<string, RuntimeScheduleEvent[]>>({});
    const latestTimeoutByDevice = ref<Record<string, RuntimeTimeoutEvent | null>>({});

    const appendScheduleEvent = (entry: RuntimeScheduleEvent) => {
        const current = scheduleEventsByDevice.value[entry.deviceId] ?? [];
        const next = [...current, entry].slice(-MAX_SCHEDULE_EVENTS);
        scheduleEventsByDevice.value = {
            ...scheduleEventsByDevice.value,
            [entry.deviceId]: next,
        };
    };

    const initIpcListeners = async () => {
        if (initialized.value) {
            return;
        }

        await listen('device-progress', async (event) => {
            const payload = normalizeProgressEvent(event.payload);
            if (!payload) {
                return;
            }

            latestProgressByDevice.value = {
                ...latestProgressByDevice.value,
                [payload.deviceId]: payload,
            };
        });

        await listen('device-schedule', (event) => {
            const payload = normalizeScheduleEvent(event.payload);
            if (!payload) {
                return;
            }

            appendScheduleEvent(payload);
        });

        await listen('device-timeout', (event) => {
            const payload = normalizeTimeoutEvent(event.payload);
            if (!payload) {
                return;
            }

            latestTimeoutByDevice.value = {
                ...latestTimeoutByDevice.value,
                [payload.deviceId]: payload,
            };
        });

        initialized.value = true;
    };

    const getLatestProgress = (deviceId: string) => latestProgressByDevice.value[deviceId] ?? null;
    const getScheduleEvents = (deviceId: string) => scheduleEventsByDevice.value[deviceId] ?? [];
    const getLatestTimeout = (deviceId: string) => latestTimeoutByDevice.value[deviceId] ?? null;
    const getRuntimeResult = (deviceId: string): RuntimeResultProjection => {
        const latestProgress = getLatestProgress(deviceId);
        const schedules = getScheduleEvents(deviceId);
        const latestSchedule = schedules.at(-1) ?? null;
        const latestTimeout = getLatestTimeout(deviceId);

        return {
            deviceId,
            latestProgress,
            latestSchedule,
            latestTimeout,
            timeoutActionResult: resolveTimeoutActionResult(latestTimeout, schedules, latestProgress),
            updatedAt: latestRuntimeTimestamp(latestProgress, latestSchedule, latestTimeout),
        };
    };

    const clearTimeoutState = (deviceId?: string) => {
        if (deviceId) {
            latestTimeoutByDevice.value = {
                ...latestTimeoutByDevice.value,
                [deviceId]: null,
            };
            return;
        }

        latestTimeoutByDevice.value = {};
    };

    const clearRuntimeState = (deviceId?: string) => {
        if (deviceId) {
            latestProgressByDevice.value = {
                ...latestProgressByDevice.value,
                [deviceId]: null,
            };
            scheduleEventsByDevice.value = {
                ...scheduleEventsByDevice.value,
                [deviceId]: [],
            };
            clearTimeoutState(deviceId);
            return;
        }

        latestProgressByDevice.value = {};
        scheduleEventsByDevice.value = {};
        clearTimeoutState();
    };

    return {
        clearTimeoutState,
        clearRuntimeState,
        getLatestProgress,
        getRuntimeResult,
        getScheduleEvents,
        getLatestTimeout,
        initIpcListeners,
        initialized,
        latestProgressByDevice,
        scheduleEventsByDevice,
        latestTimeoutByDevice,
    };
});
