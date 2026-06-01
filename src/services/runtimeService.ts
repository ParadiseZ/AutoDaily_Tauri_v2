import { invoke } from '@/utils/api';
import type { RunTarget } from '@/types/app/domain';

type InvokeRunTarget =
    | 'deviceQueue'
    | { fullScript: { script_id: string } }
    | { task: { script_id: string; task_id: string } }
    | { policyGroup: { script_id: string; policy_group_id: string } }
    | { policySet: { script_id: string; policy_set_id: string } }
    | { policy: { script_id: string; policy_id: string } };

const serializeRunTarget = (target: RunTarget): InvokeRunTarget => {
    switch (target.type) {
        case 'deviceQueue':
            return 'deviceQueue';
        case 'fullScript':
            return {
                fullScript: {
                    script_id: target.scriptId,
                },
            };
        case 'task':
            return {
                task: {
                    script_id: target.scriptId,
                    task_id: target.taskId,
                },
            };
        case 'policyGroup':
            return {
                policyGroup: {
                    script_id: target.scriptId,
                    policy_group_id: target.policyGroupId,
                },
            };
        case 'policySet':
            return {
                policySet: {
                    script_id: target.scriptId,
                    policy_set_id: target.policySetId,
                },
            };
        case 'policy':
            return {
                policy: {
                    script_id: target.scriptId,
                    policy_id: target.policyId,
                },
            };
    }
};

export const runtimeService = {
    syncDeviceSession: (deviceId: string) =>
        invoke('cmd_sync_device_runtime_session', { deviceId }) as Promise<string>,
    runScriptTarget: (deviceId: string, target: RunTarget) =>
        invoke('cmd_run_script_target', { deviceId, target: serializeRunTarget(target) }) as Promise<string>,
};
