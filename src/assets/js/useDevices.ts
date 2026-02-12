
import { invoke } from '@tauri-apps/api/core';

import type {DeviceTable} from '@/types/bindings';

export const useDevices = () => {
  const getAllDevices = async () => {
    const res = await invoke<DeviceTable[]>('get_all_devices_cmd');
    return Object.values(res);
  };

  const saveDevice = async (device: DeviceTable & { id: string }) => {
    return await invoke('save_device_cmd', { device });
  };

  const deleteDevice = async (deviceId: string) => {
    return await invoke('delete_device_cmd', { deviceId });
  };

  const getUuidV7 = async (): Promise<string> => {
    return await invoke<string>('get_uuid_v7');
  };

  const getCpuCount = async () => {
    return await invoke<number>('get_cpu_count_cmd');
  };

  return {
    getAllDevices,
    saveDevice,
    deleteDevice,
    getUuidV7,
    getCpuCount
  };
};