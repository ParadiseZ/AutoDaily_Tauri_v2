import { invoke } from '@tauri-apps/api/core';

/** 设备脚本分配（队列定义） — 对应后端 DeviceScriptAssignment */
export interface DeviceScriptAssignment {
  id: string;
  deviceId: string;
  scriptId: string;
  timeTemplateId: string | null;
  accountData: any;
  index: number;
}

/** 调度记录 — 对应后端 DeviceScriptSchedule */
export interface DeviceScriptSchedule {
  id: string;
  deviceId: string;
  scriptId: string;
  taskId: string;
  taskCycle: string;
  status: string;
  startedAt: string;
  completedAt: string | null;
  message: string | null;
}

export const useAssignments = () => {
  /** 获取指定设备的所有脚本分配（按 index 排序） */
  const getByDevice = async (deviceId: string): Promise<DeviceScriptAssignment[]> => {
    return await invoke<DeviceScriptAssignment[]>('get_assignments_by_device_cmd', { deviceId });
  };

  /** 保存（新增或更新）脚本分配 */
  const save = async (assignment: DeviceScriptAssignment): Promise<void> => {
    await invoke('save_assignment_cmd', { assignment });
  };

  /** 删除脚本分配 */
  const remove = async (assignmentId: string): Promise<void> => {
    await invoke('delete_assignment_cmd', { assignmentId });
  };

  /** 批量更新排序顺序 */
  const reorder = async (deviceId: string, assignmentIds: string[]): Promise<void> => {
    await invoke('reorder_assignments_cmd', { deviceId, assignmentIds });
  };

  /** 获取指定设备的调度记录 */
  const getSchedulesByDevice = async (deviceId: string): Promise<DeviceScriptSchedule[]> => {
    return await invoke<DeviceScriptSchedule[]>('get_schedules_by_device_cmd', { deviceId });
  };

  /** 清除指定设备的调度记录 */
  const clearSchedules = async (deviceId: string): Promise<void> => {
    await invoke('clear_schedules_cmd', { deviceId });
  };

  /** 清除指定脚本的调度记录 */
  const clearSchedulesByScript = async (scriptId: string): Promise<void> => {
    await invoke('clear_schedules_by_script_cmd', { scriptId });
  };

  return {
    getByDevice,
    save,
    remove,
    reorder,
    getSchedulesByDevice,
    clearSchedules,
    clearSchedulesByScript,
  };
};
