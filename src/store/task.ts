import { defineStore } from 'pinia';
import { invoke } from '../utils/api';
// import type { DeviceScriptAssignment } from '../types/bindings/DeviceScriptAssignment'; // Example

export const useTaskStore = defineStore('task', () => {
    
    // 获取指定设备的运行分配 (任务)
    const loadAssignments = async (deviceId: string) => {
        try {
            const res = await invoke('get_assignments_by_device_cmd', { deviceId });
            return res;
        } catch (error) {
            console.error(`Failed to load assignments for device ${deviceId}:`, error);
            return [];
        }
    };

    // 保存分配
    const saveAssignment = async (assignment: any) => {
        try {
            await invoke('save_assignment_cmd', { assignment });
        } catch (error) {
            console.error('Failed to save assignment:', error);
            throw error;
        }
    };

    return {
        loadAssignments,
        saveAssignment,
    };
});
