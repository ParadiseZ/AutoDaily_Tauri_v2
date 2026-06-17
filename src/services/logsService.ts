import { invoke } from '@/utils/api';
import type { DeviceLogEntry } from '@/types/app/domain';

export const logsService = {
    readToday: (deviceId?: string | null) =>
        invoke('read_today_device_logs_cmd', { deviceId: deviceId || null }) as Promise<DeviceLogEntry[]>,
    clearToday: (deviceId?: string | null) =>
        invoke('clear_today_device_logs_cmd', { deviceId: deviceId || null }) as Promise<string>,
};
