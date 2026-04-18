import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import type { RuntimeProgressEvent, RuntimeScheduleEvent, RuntimeTimeoutEvent } from '@/types/app/domain';

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

const normalizeTimeoutEvent = (payload: unknown): RuntimeTimeoutEvent | null => {
    if (!payload || typeof payload !== 'object') {
        return null;
    }

    const record = payload as Record<string, unknown>;
    if (typeof record.deviceId !== 'string' || typeof record.message !== 'string' || typeof record.at !== 'string') {
        return null;
    }

    return {
        deviceId: record.deviceId,
        sessionId: typeof record.sessionId === 'string' ? record.sessionId : null,
        assignmentId: typeof record.assignmentId === 'string' ? record.assignmentId : null,
        scriptId: typeof record.scriptId === 'string' ? record.scriptId : null,
        taskId: typeof record.taskId === 'string' ? record.taskId : null,
        stepId: typeof record.stepId === 'string' ? record.stepId : null,
        message: record.message,
        at: record.at,
    };
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
        getScheduleEvents,
        getLatestTimeout,
        initIpcListeners,
        initialized,
        latestProgressByDevice,
        scheduleEventsByDevice,
        latestTimeoutByDevice,
    };
});
