import { defineStore } from 'pinia';
import { ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import type { DeviceLogEntry } from '@/types/app/domain';

const MAX_LOGS_PER_DEVICE = 500;

const normalizeLogEntry = (payload: unknown): DeviceLogEntry | null => {
    if (!payload || typeof payload !== 'object') {
        return null;
    }

    const record = payload as Record<string, unknown>;
    if (
        typeof record.deviceId !== 'string' ||
        typeof record.level !== 'string' ||
        typeof record.message !== 'string' ||
        typeof record.time !== 'string'
    ) {
        return null;
    }

    return {
        deviceId: record.deviceId,
        level: record.level as DeviceLogEntry['level'],
        message: record.message,
        time: record.time,
    };
};

export const useLogsStore = defineStore('logs', () => {
    const logsByDevice = ref<Record<string, DeviceLogEntry[]>>({});
    const initialized = ref(false);

    const appendLog = (entry: DeviceLogEntry) => {
        const current = logsByDevice.value[entry.deviceId] ?? [];
        const next = [...current, entry].slice(-MAX_LOGS_PER_DEVICE);
        logsByDevice.value = {
            ...logsByDevice.value,
            [entry.deviceId]: next,
        };
    };

    const initListener = async () => {
        if (initialized.value) {
            return;
        }

        await listen('child-log', (event) => {
            const entry = normalizeLogEntry(event.payload);
            if (!entry) {
                return;
            }
            appendLog(entry);
        });

        initialized.value = true;
    };

    const getDeviceLogs = (deviceId: string) => logsByDevice.value[deviceId] ?? [];

    const clearLogs = (deviceId?: string) => {
        if (deviceId) {
            logsByDevice.value = {
                ...logsByDevice.value,
                [deviceId]: [],
            };
            return;
        }

        logsByDevice.value = {};
    };

    return {
        clearLogs,
        getDeviceLogs,
        initListener,
        initialized,
        logsByDevice,
    };
});
