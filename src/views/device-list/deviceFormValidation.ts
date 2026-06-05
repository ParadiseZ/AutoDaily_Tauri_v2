import type { DeviceFormState } from '@/types/app/domain';

export interface NormalizedDeviceFormValues {
  connectAddress: string;
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
    adbServerConnect: form.adbServerConnect.trim(),
    adbPath: form.adbPath.trim(),
    exePath: form.exePath.trim(),
    exeArgs: form.exeArgs.trim(),
    capMethodValue: form.capMethodValue.trim(),
  };

  if (form.transportKind === 'emulatorTcp' && !isValidIpv4Port(normalized.connectAddress)) {
    throw new Error('TCP 地址格式应为 IP:端口，例如 127.0.0.1:5555');
  }

  if (form.transportKind !== 'emulatorTcp' && normalized.adbServerConnect && !isValidIpv4Port(normalized.adbServerConnect)) {
    throw new Error('ADB Server 格式应为 IP:端口，例如 127.0.0.1:5037');
  }

  return normalized;
};
