
import { invoke } from '@tauri-apps/api/core';

export const useDevices = () => {
  const getAllDevices = async () => {
    const res = await invoke('get_all_devices_cmd');
    return Object.values(res);
  };

  const saveDevice = async (device) => {
    return await invoke('save_device_cmd', { device });
  };

  const deleteDevice = async (deviceId) => {
    return await invoke('delete_device_cmd', { deviceId });
  };

  const getUuidV7 = async () => {
    return await invoke('get_uuid_v7');
  };

  const getCpuCount = async () => {
    return await invoke('get_cpu_count_cmd');
  };

  return {
    getAllDevices,
    saveDevice,
    deleteDevice,
    getUuidV7,
    getCpuCount
  };
};