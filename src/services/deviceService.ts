import { invoke } from '@/utils/api';
import type { DeviceTable } from '@/types/bindings/DeviceTable';
import type { LogLevel } from '@/types/bindings/LogLevel';

export const deviceService = {
    list: () => invoke('get_all_devices_cmd') as Promise<DeviceTable[]>,
    save: (device: DeviceTable) => invoke('save_device_cmd', { device }) as Promise<void>,
    remove: (deviceId: string) => invoke('delete_device_cmd', { deviceId }) as Promise<void>,
    getCpuCount: () => invoke('get_cpu_count_cmd') as Promise<number>,
    getRunningDeviceIds: () => invoke('cmd_get_running_devices') as Promise<string[]>,
    isRunning: (deviceId: string) => invoke('cmd_is_device_running', { deviceId }) as Promise<boolean>,
    prepareCapture: (deviceId: string) =>
        invoke('cmd_prepare_device_capture', { deviceId }) as Promise<string>,
    spawn: (deviceId: string) => invoke('cmd_spawn_device', { deviceId }) as Promise<string>,
    shutdown: (deviceId: string) => invoke('cmd_device_shutdown', { deviceId }) as Promise<string>,
    start: (deviceId: string) => invoke('cmd_device_start', { deviceId }) as Promise<string>,
    pause: (deviceId: string) => invoke('cmd_device_pause', { deviceId }) as Promise<string>,
    stop: (deviceId: string) => invoke('cmd_device_stop', { deviceId }) as Promise<string>,
    restartRuntime: (deviceId: string) =>
        invoke('cmd_restart_device_runtime', { deviceId }) as Promise<string>,
    updateChildLogLevel: (deviceId: string, logLevel: LogLevel) =>
        invoke('update_child_log_level_cmd', { deviceId, logLevel }) as Promise<string>,
};
