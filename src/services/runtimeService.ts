import { invoke } from '@/utils/api';
import type { ResumeCheckpointRecord, RunTarget } from '@/types/app/domain';

export const runtimeService = {
    getRecoveryCheckpoint: (deviceId: string) =>
        invoke('get_recovery_checkpoint_by_device_cmd', { deviceId }) as Promise<ResumeCheckpointRecord | null>,
    syncDeviceSession: (deviceId: string) =>
        invoke('cmd_sync_device_runtime_session', { deviceId }) as Promise<string>,
    runScriptTarget: (deviceId: string, target: RunTarget) =>
        invoke('cmd_run_script_target', { deviceId, target }) as Promise<string>,
};
