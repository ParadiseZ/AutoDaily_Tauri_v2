import type { ADBConnectConfig } from '@/types/bindings/ADBConnectConfig';
import type { CapMethod } from '@/types/bindings/CapMethod';
import type { LogLevel } from '@/types/bindings/LogLevel';
import type { DeviceRuntimeStatus, ScriptTableRecord } from '@/types/app/domain';
import type { TaskCycle } from '@/types/bindings/TaskCycle';
import type { TaskRowType } from '@/types/bindings/TaskRowType';
import type { TaskTone } from '@/types/bindings/TaskTone';
import type { TaskTriggerMode } from '@/types/bindings/TaskTriggerMode';
import type { TimeTemplate } from '@/types/bindings/TimeTemplate';

export const formatDateTime = (value?: string | null) => {
    if (!value) {
        return '未记录';
    }

    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
        return value;
    }

    return new Intl.DateTimeFormat('zh-CN', {
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
    }).format(date);
};

export const formatDate = (value?: string | null) => {
    if (!value) {
        return '未记录';
    }

    const date = new Date(value);
    if (Number.isNaN(date.getTime())) {
        return value;
    }

    return new Intl.DateTimeFormat('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
    }).format(date);
};

export const formatNumberLike = (value: number | bigint | null | undefined) => {
    if (typeof value === 'bigint') {
        return value.toString();
    }
    if (typeof value === 'number') {
        return value.toString();
    }
    return '0';
};

export const formatRuntimeLabel = (runtimeType?: string | null) => {
    switch (runtimeType) {
        case 'rhai':
            return 'Rhai';
        case 'javaScript':
            return 'JavaScript';
        case 'lua':
            return 'Lua';
        case 'aIAndVision':
            return 'AI + Vision';
        case 'aI':
            return 'AI';
        default:
            return '未设置';
    }
};

export const formatPlatformLabel = (platform?: string | null) => {
    if (platform === 'desktop') {
        return '桌面程序';
    }

    return 'Android';
};

export const formatScriptType = (script: ScriptTableRecord) =>
    script.data.scriptType === 'published' ? '云端版本' : '本地开发';

export const formatConnectLabel = (config: ADBConnectConfig | null) => {
    if (!config) {
        return '未配置连接';
    }

    if ('directTcp' in config) {
        return config.directTcp || 'TCP 直连';
    }

    if ('directUsb' in config) {
        return `USB · ${config.directUsb.vendorId}:${config.directUsb.productId}`;
    }

    if ('serverConnectByIp' in config) {
        return `ADB 服务 · ${config.serverConnectByIp.clientConnect || '未设置地址'}`;
    }

    return `ADB 服务 · ${config.serverConnectByName.deviceName || '未设置名称'}`;
};

export const formatCaptureMethod = (method: CapMethod) =>
    typeof method === 'string' ? 'ADB 截图' : `窗口截取 · ${method.window}`;

export const formatStatusTone = (status: DeviceRuntimeStatus['kind'] | LogLevel) => {
    if (status === 'running' || status === 'Info') return 'info';
    if (status === 'idle') return 'neutral';
    if (status === 'paused' || status === 'Warn') return 'warning';
    if (status === 'error' || status === 'Error') return 'danger';
    if (status === 'Debug') return 'neutral';
    return 'success';
};

export const formatStatusLabel = (status: DeviceRuntimeStatus) => {
    switch (status.kind) {
        case 'running':
            return '运行中';
        case 'paused':
            return '已暂停';
        case 'error':
            return '异常';
        case 'idle':
            return '空闲';
        case 'stopped':
            return '离线';
        default:
            return status.rawStatus || '未知状态';
    }
};

export const formatTemplateWindow = (template: TimeTemplate | null | undefined) => {
    if (!template) {
        return '全天';
    }

    if (template.startTime && template.endTime) {
        return `${template.startTime} - ${template.endTime}`;
    }

    return template.name;
};

export const formatRecoveryPhaseLabel = (phase?: string | null) => {
    switch (phase) {
        case 'CheckpointPreparing':
            return '准备检查点';
        case 'CheckpointReady':
            return '检查点已保存';
        case 'RestartReady':
            return '可按检查点重启';
        case 'CheckpointLoaded':
            return '检查点已加载';
        default:
            return phase || '恢复事件';
    }
};

export const formatResumeModeLabel = (mode?: string | null) => {
    switch (mode) {
        case 'fromTaskStart':
            return '从任务起点';
        case 'fromStepStart':
            return '从步骤起点';
        case 'fromNextStep':
            return '从下一步骤';
        default:
            return mode || '未记录';
    }
};

export const formatTaskCycleLabel = (value: TaskCycle) => {
    if (value === 'everyRun') return '每次';
    if (value === 'daily') return '每日';
    if (value === 'weekly') return '每周';
    if (value === 'monthly') return '每月';
    if ('weekDay' in value) {
        return `周${['一', '二', '三', '四', '五', '六', '日'][Math.max(0, Math.min(6, value.weekDay - 1))]}`;
    }
    return `${value.monthDay} 日`;
};

export const formatTaskRowTypeLabel = (value: TaskRowType) => (value === 'title' ? '标题行' : '任务行');

export const formatTaskTriggerModeLabel = (value: TaskTriggerMode) => {
    if (value === 'rootOnly') return '一级循环';
    if (value === 'linkOnly') return '仅跳转';
    return '一级 + 跳转';
};

export const formatTaskToneLabel = (value: TaskTone) => {
    if (value === 'warning') return '警告';
    if (value === 'danger') return '严重';
    return '普通';
};
