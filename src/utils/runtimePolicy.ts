import type { AssignmentRecord, ScriptTableRecord } from '@/types/app/domain';
import type { DeviceTable } from '@/types/bindings/DeviceTable';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';

const normalizePlatform = (value: unknown) => (value === 'desktop' ? 'desktop' : 'android');

const isRunRecoveryTaskEnabled = (device: DeviceTable) =>
    device.data.executionPolicy?.timeoutAction === 'runRecoveryTask';

const isRunnableTask = (task: ScriptTaskTable) => task.rowType === 'task' && !task.isDeleted;

export const validateDeviceRuntimePlatform = (device: DeviceTable): string | null => {
    if (normalizePlatform(device.data.platform) !== 'desktop') {
        return null;
    }

    return `设备「${device.data.deviceName}」当前为 Desktop 平台，但本版本尚未实现 Desktop 运行时适配器。`;
};

export const validateScriptRecoveryTask = (
    script: ScriptTableRecord,
    tasks?: ScriptTaskTable[] | null,
): string | null => {
    const recoveryTaskId = script.data.runtimeSettings?.recoveryTaskId ?? null;
    if (!recoveryTaskId) {
        return `脚本「${script.data.name}」未配置恢复任务，当前设备不能使用“执行恢复任务”策略启动。`;
    }

    if (!tasks) {
        return null;
    }

    const matchedTask = tasks.find((task) => task.id === recoveryTaskId);
    if (!matchedTask || !isRunnableTask(matchedTask)) {
        return `脚本「${script.data.name}」配置的恢复任务不存在，或不是可执行 Task。`;
    }

    return null;
};

export const validateRunTargetRecoveryForDevice = (
    device: DeviceTable,
    script: ScriptTableRecord,
    tasks?: ScriptTaskTable[] | null,
): string | null => {
    if (isRunRecoveryTaskEnabled(device)) {
        return validateScriptRecoveryTask(script, tasks);
    }

    return null;
};

export const validateDeviceQueueRecoveryForDevice = (
    device: DeviceTable,
    assignments: AssignmentRecord[],
    scripts: ScriptTableRecord[],
): string | null => {
    if (!isRunRecoveryTaskEnabled(device)) {
        return null;
    }

    for (const assignment of assignments) {
        const script = scripts.find((item) => item.id === assignment.scriptId);
        if (!script) {
            continue;
        }

        const error = validateScriptRecoveryTask(script);
        if (error) {
            return error;
        }
    }

    return null;
};
