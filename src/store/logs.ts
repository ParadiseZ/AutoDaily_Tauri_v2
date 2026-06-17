import { listen } from '@tauri-apps/api/event';
import { defineStore } from 'pinia';
import { ref } from 'vue';
import { logsService } from '@/services/logsService';
import { getFromStore, logsSelectedDeviceKey, setToStore } from '@/store/store';
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

const buildLogKey = (entry: DeviceLogEntry) =>
    `${entry.deviceId}__${entry.time}__${entry.level}__${entry.message}`;

const sortLogs = (entries: DeviceLogEntry[]) =>
    [...entries].sort((left, right) => {
        const timeCompare = left.time.localeCompare(right.time);
        if (timeCompare !== 0) {
            return timeCompare;
        }
        const levelCompare = left.level.localeCompare(right.level);
        if (levelCompare !== 0) {
            return levelCompare;
        }
        return left.message.localeCompare(right.message);
    });

const mergeDeviceLogs = (current: DeviceLogEntry[], incoming: DeviceLogEntry[]) => {
    const merged = new Map(current.map((entry) => [buildLogKey(entry), entry]));
    incoming.forEach((entry) => {
        merged.set(buildLogKey(entry), entry);
    });
    return sortLogs(Array.from(merged.values())).slice(-MAX_LOGS_PER_DEVICE);
};

const groupLogsByDevice = (entries: DeviceLogEntry[]) => {
    const grouped = new Map<string, DeviceLogEntry[]>();
    entries.forEach((entry) => {
        const bucket = grouped.get(entry.deviceId) ?? [];
        bucket.push(entry);
        grouped.set(entry.deviceId, bucket);
    });
    return grouped;
};

export const useLogsStore = defineStore('logs', () => {
    const logsByDevice = ref<Record<string, DeviceLogEntry[]>>({});
    const allLogs = ref<DeviceLogEntry[]>([]);
    const listenerActive = ref(false);
    const persistedSelectionLoaded = ref(false);
    const historyLoadedAll = ref(false);
    const historyLoadedByDevice = ref<Record<string, true>>({});
    const selectedDeviceId = ref('');
    let detachListener: null | (() => void) = null;

    const mergeLogs = (entries: DeviceLogEntry[]) => {
        if (!entries.length) {
            return;
        }

        const grouped = groupLogsByDevice(entries);
        const nextLogs = { ...logsByDevice.value };
        grouped.forEach((groupEntries, deviceId) => {
            nextLogs[deviceId] = mergeDeviceLogs(nextLogs[deviceId] ?? [], groupEntries);
        });
        logsByDevice.value = nextLogs;
        allLogs.value = sortLogs(Object.values(nextLogs).flat());
    };

    const replaceLogs = (entries: DeviceLogEntry[], deviceId?: string | null) => {
        const nextLogs = { ...logsByDevice.value };

        if (deviceId) {
            nextLogs[deviceId] = sortLogs(entries).slice(-MAX_LOGS_PER_DEVICE);
        } else {
            Object.keys(nextLogs).forEach((currentDeviceId) => {
                nextLogs[currentDeviceId] = [];
            });

            groupLogsByDevice(entries).forEach((groupEntries, currentDeviceId) => {
                nextLogs[currentDeviceId] = sortLogs(groupEntries).slice(-MAX_LOGS_PER_DEVICE);
            });
        }

        logsByDevice.value = nextLogs;
        allLogs.value = sortLogs(Object.values(nextLogs).flat());
    };

    const appendLog = (entry: DeviceLogEntry) => {
        mergeLogs([entry]);
    };

    const ensurePersistedSelectionLoaded = async () => {
        if (persistedSelectionLoaded.value) {
            return;
        }

        const saved = await getFromStore<string>(logsSelectedDeviceKey);
        selectedDeviceId.value = typeof saved === 'string' ? saved : '';
        persistedSelectionLoaded.value = true;
    };

    const setSelectedDevice = async (deviceId: string) => {
        selectedDeviceId.value = deviceId;
        await setToStore(logsSelectedDeviceKey, deviceId);
    };

    const startListener = async () => {
        await ensurePersistedSelectionLoaded();
        if (listenerActive.value) {
            return;
        }

        detachListener = await listen('child-log', (event) => {
            const entry = normalizeLogEntry(event.payload);
            if (!entry) {
                return;
            }
            appendLog(entry);
        });

        listenerActive.value = true;
    };

    const stopListener = () => {
        detachListener?.();
        detachListener = null;
        listenerActive.value = false;
    };

    const ensureTodayLogsLoaded = async (deviceId?: string | null) => {
        await ensurePersistedSelectionLoaded();
        const targetDeviceId = deviceId ?? selectedDeviceId.value;

        if (targetDeviceId) {
            if (historyLoadedByDevice.value[targetDeviceId]) {
                return;
            }

            mergeLogs(await logsService.readToday(targetDeviceId));
            historyLoadedByDevice.value = {
                ...historyLoadedByDevice.value,
                [targetDeviceId]: true,
            };
            return;
        }

        if (historyLoadedAll.value) {
            return;
        }

        const entries = await logsService.readToday();
        mergeLogs(entries);
        historyLoadedAll.value = true;
        if (entries.length) {
            historyLoadedByDevice.value = entries.reduce<Record<string, true>>((acc, entry) => {
                acc[entry.deviceId] = true;
                return acc;
            }, { ...historyLoadedByDevice.value });
        }
    };

    const reloadTodayLogs = async (deviceId?: string | null) => {
        await ensurePersistedSelectionLoaded();
        const targetDeviceId = deviceId ?? selectedDeviceId.value;
        const entries = await logsService.readToday(targetDeviceId || null);

        replaceLogs(entries, targetDeviceId || null);

        if (targetDeviceId) {
            historyLoadedByDevice.value = {
                ...historyLoadedByDevice.value,
                [targetDeviceId]: true,
            };
            return;
        }

        historyLoadedAll.value = true;
        historyLoadedByDevice.value = entries.reduce<Record<string, true>>((acc, entry) => {
            acc[entry.deviceId] = true;
            return acc;
        }, { ...historyLoadedByDevice.value });
    };

    const getDeviceLogs = (deviceId: string) => logsByDevice.value[deviceId] ?? [];

    const clearLogs = async (deviceId?: string | null) => {
        await logsService.clearToday(deviceId);

        if (deviceId) {
            const nextLogs = {
                ...logsByDevice.value,
                [deviceId]: [],
            };
            logsByDevice.value = nextLogs;
            allLogs.value = sortLogs(Object.values(nextLogs).flat());
            return;
        }

        logsByDevice.value = {};
        allLogs.value = [];
    };

    return {
        clearLogs,
        ensurePersistedSelectionLoaded,
        ensureTodayLogsLoaded,
        getDeviceLogs,
        historyLoadedAll,
        historyLoadedByDevice,
        listenerActive,
        allLogs,
        logsByDevice,
        reloadTodayLogs,
        selectedDeviceId,
        setSelectedDevice,
        startListener,
        stopListener,
    };
});
