import { invoke } from '@/utils/api';
import type { DeviceScriptSchedule } from '@/types/bindings/DeviceScriptSchedule';
import type { DeviceScriptAssignment } from '@/types/bindings/DeviceScriptAssignment';
import type { TimeTemplate } from '@/types/bindings/TimeTemplate';

export const taskService = {
    listAssignments: (deviceId: string) =>
        invoke('get_assignments_by_device_cmd', { deviceId }) as Promise<DeviceScriptAssignment[]>,
    saveAssignment: (assignment: DeviceScriptAssignment) =>
        invoke('save_assignment_cmd', { assignment }) as Promise<void>,
    deleteAssignment: (assignmentId: string) =>
        invoke('delete_assignment_cmd', { assignmentId }) as Promise<void>,
    reorderAssignments: (deviceId: string, assignmentIds: string[]) =>
        invoke('reorder_assignments_cmd', { deviceId, assignmentIds }) as Promise<void>,
    listSchedules: (deviceId: string) =>
        invoke('get_schedules_by_device_cmd', { deviceId }) as Promise<DeviceScriptSchedule[]>,
    clearSchedules: (deviceId: string) =>
        invoke('clear_schedules_cmd', { deviceId }) as Promise<void>,
    clearSchedulesByScript: (scriptId: string) =>
        invoke('clear_schedules_by_script_cmd', { scriptId }) as Promise<void>,
    listTimeTemplates: () => invoke('get_all_time_templates_cmd') as Promise<TimeTemplate[]>,
    saveTimeTemplate: (template: TimeTemplate) =>
        invoke('save_time_template_cmd', { template }) as Promise<void>,
    deleteTimeTemplate: (templateId: string) =>
        invoke('delete_time_template_cmd', { templateId }) as Promise<void>,
    requestUuid: () => invoke('get_uuid_v7') as Promise<string>,
};
