
import { invoke } from '@tauri-apps/api/core';

export const useDevices = () => {
  const getAllDevices = async () => {
    return await invoke('get_all_devices_cmd');
  };

  return {
    getAllDevices,
  };
};