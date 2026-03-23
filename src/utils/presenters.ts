import type { ADBConnectConfig } from '@/types/bindings/ADBConnectConfig';
import type { CapMethod } from '@/types/bindings/CapMethod';
import type { LogLevel } from '@/types/bindings/LogLevel';
import type { DeviceRuntimeStatus, ScriptTableRecord } from '@/types/app/domain';
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
