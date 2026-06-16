import { taskService } from '@/services/taskService';
import { validateDeviceForm } from '@/views/device-list/deviceFormValidation';
import type { DeviceFormState, SystemPreferences } from '@/types/app/domain';
import type { DeviceTable } from '@/types/bindings/DeviceTable';

export const buildDeviceTableFromForm = async (
  form: DeviceFormState,
  preferences: SystemPreferences,
): Promise<DeviceTable> => {
  const normalized = validateDeviceForm(form);
  const captureMethodType = form.transportKind === 'emulatorTcp' ? form.capMethodType : 'adb';

  return {
    id: form.id ?? (await taskService.requestUuid()),
    data: {
      deviceName: form.deviceName,
      platform: form.platform,
      transportKind: form.transportKind,
      emulatorConnectMode: form.emulatorConnectMode,
      startupDelaySecs: Math.max(0, Math.floor(Number(form.startupDelaySecs) || 0)),
      connectAddress:
        form.transportKind === 'emulatorTcp' && form.emulatorConnectMode === 'tcpAddress'
          ? normalized.connectAddress
          : null,
      connectIdentifier:
        form.transportKind !== 'emulatorTcp' || form.emulatorConnectMode === 'identifier'
          ? normalized.connectIdentifier || null
          : null,
      adbPath:
        form.transportKind !== 'emulatorTcp' || form.emulatorConnectMode === 'identifier'
          ? normalized.adbPath || preferences.adbPath || null
          : null,
      adbServerConnect:
        form.transportKind === 'emulatorTcp' && form.emulatorConnectMode === 'tcpAddress'
          ? null
          : normalized.adbServerConnect || `${preferences.adbServerHost}:${preferences.adbServerPort}`,
      exePath: normalized.exePath || null,
      exeArgs: normalized.exeArgs || null,
      cores: form.cores,
      logLevel: form.logLevel,
      logToFile: form.logToFile,
      capMethod:
        captureMethodType === 'adb'
          ? { type: 'adb' }
          : { type: 'window', title: normalized.capMethodValue || form.deviceName },
      imageCompression: captureMethodType === 'adb' ? 'AdbOriginal' : 'WindowOriginal',
      enable: form.enable,
      autoStart: form.autoStart,
      executionPolicy: {
        actionWaitMs: Math.max(0, Math.floor(Number(form.actionWaitMs) || 0)),
        progressTimeoutEnabled: form.progressTimeoutEnabled,
        progressTimeoutMs: Math.max(1000, Math.floor(Number(form.progressTimeoutMs) || 30000)),
        timeoutAction: form.timeoutAction,
        timeoutNotifyChannels: [...form.timeoutNotifyChannels],
      },
    },
  };
};
