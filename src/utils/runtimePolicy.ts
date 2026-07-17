import type { DeviceTable } from '@/types/bindings/DeviceTable';

const normalizePlatform = (value: unknown) => (value === 'desktop' ? 'desktop' : 'android');

export const validateDeviceRuntimePlatform = (device: DeviceTable): string | null => {
    if (normalizePlatform(device.data.platform) !== 'desktop') {
        return null;
    }

    return `设备「${device.data.deviceName}」当前为 Desktop 平台，但本版本尚未实现 Desktop 运行时适配器。`;
};

export const validateDeviceConnectionBootstrapConfig = (
    device: DeviceTable,
    isConnected: boolean,
): string | null => {
    if (normalizePlatform(device.data.platform) === 'desktop') {
        return null;
    }

    if (device.data.transportKind !== 'emulatorTcp') {
        return null;
    }

    if (device.data.emulatorConnectMode === 'identifier') {
        if (!device.data.connectIdentifier) {
            return `设备「${device.data.deviceName}」未配置模拟器设备标识，无法执行连接探测。`;
        }

        if (!device.data.adbPath?.trim()) {
            return `设备「${device.data.deviceName}」当前使用模拟器标识连接，但未配置 ADB 程序路径。`;
        }

        if (!device.data.adbServerConnect?.trim()) {
            return `设备「${device.data.deviceName}」当前使用模拟器标识连接，但未配置 ADB Server 地址。`;
        }
    } else if (!device.data.connectAddress) {
        return `设备「${device.data.deviceName}」未配置模拟器连接地址，无法执行连接探测。`;
    }

    if (!isConnected && !device.data.exePath?.trim()) {
        return `设备「${device.data.deviceName}」当前未连接，且未配置模拟器启动路径，点击运行时不会自动拉起模拟器。`;
    }

    return null;
};
