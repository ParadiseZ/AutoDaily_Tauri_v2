import type { DeviceFormState } from '@/types/app/domain';

export interface NormalizedDeviceFormValues {
  connectAddress: string;
  connectIdentifier: string;
  adbServerConnect: string;
  adbPath: string;
  exePath: string;
  exeArgs: string;
  capMethodValue: string;
}

const hostPortPattern = /^([^:\s]+):(\d{1,5})$/;

const isValidIpv4 = (host: string) => {
  const parts = host.split('.');
  return parts.length === 4 && parts.every((part) => {
    if (!/^\d{1,3}$/.test(part)) {
      return false;
    }
    const value = Number(part);
    return value >= 0 && value <= 255;
  });
};

export const isValidIpv4Port = (value: string) => {
  const match = hostPortPattern.exec(value.trim());
  if (!match) {
    return false;
  }

  const [, host, portText] = match;
  const port = Number(portText);
  return isValidIpv4(host) && Number.isInteger(port) && port >= 1 && port <= 65535;
};

export const validateDeviceForm = (form: DeviceFormState): NormalizedDeviceFormValues => {
  const normalized: NormalizedDeviceFormValues = {
    connectAddress: form.connectAddress.trim(),
    connectIdentifier: form.connectIdentifier.trim(),
    adbServerConnect: form.adbServerConnect.trim(),
    adbPath: form.adbPath.trim(),
    exePath: form.exePath.trim(),
    exeArgs: form.exeArgs.trim(),
    capMethodValue: form.capMethodValue.trim(),
  };

  const emulatorUsesTcpAddress =
    form.transportKind === 'emulatorTcp' && form.emulatorConnectMode === 'tcpAddress';
  const needsIdentifier =
    form.transportKind !== 'emulatorTcp' ||
    (form.transportKind === 'emulatorTcp' && form.emulatorConnectMode === 'identifier');
  const supportsWindowCapture = form.transportKind === 'emulatorTcp';

  if (!supportsWindowCapture && form.capMethodType === 'window') {
    throw new Error('当前连接通道不支持窗口截图，只有模拟器连接可使用窗口截图。');
  }

  if (emulatorUsesTcpAddress && !isValidIpv4Port(normalized.connectAddress)) {
    throw new Error('TCP 地址格式应为 IP:端口，例如 127.0.0.1:5555');
  }

  if (needsIdentifier && !normalized.connectIdentifier) {
    throw new Error('请填写设备标识，例如 emulator-5554 或设备序列号。');
  }

  if (needsIdentifier && !normalized.adbPath) {
    throw new Error('当前连接方式需要填写 ADB 程序路径。');
  }

  if (needsIdentifier && !normalized.adbServerConnect) {
    throw new Error('当前连接方式需要填写 ADB Server 地址。');
  }

  if (needsIdentifier && !isValidIpv4Port(normalized.adbServerConnect)) {
    throw new Error('ADB Server 格式应为 IP:端口，例如 127.0.0.1:5037');
  }

  return normalized;
};
