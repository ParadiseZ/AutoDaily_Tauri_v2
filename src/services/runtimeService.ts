import { invoke } from '@/utils/api';
import type { RunTarget } from '@/types/app/domain';

export const runtimeService = {
    syncDeviceSession: (deviceId: string) =>
        invoke('cmd_sync_device_runtime_session', { deviceId }) as Promise<string>,
    runScriptTarget: (deviceId: string, target: RunTarget) =>
        invoke('cmd_run_script_target', { deviceId, target }) as Promise<string>,
};
