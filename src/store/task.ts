import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { taskService } from '@/services/taskService';
import type { AssignmentRecord, JsonValue } from '@/types/app/domain';
import type { DeviceScriptAssignment } from '@/types/bindings/DeviceScriptAssignment';
import type { DeviceScriptSchedule } from '@/types/bindings/DeviceScriptSchedule';
import type { TimeTemplate } from '@/types/bindings/TimeTemplate';

const normalizeAssignment = (assignment: DeviceScriptAssignment): AssignmentRecord => ({
    ...assignment,
    accountData: assignment.accountData ?? null,
});

export const useTaskStore = defineStore('task', () => {
    const assignmentsByDevice = ref<Record<string, AssignmentRecord[]>>({});
    const schedulesByDevice = ref<Record<string, DeviceScriptSchedule[]>>({});
    const timeTemplates = ref<TimeTemplate[]>([]);
    const loadingAssignments = ref<Record<string, boolean>>({});
    const loadingSchedules = ref<Record<string, boolean>>({});

    const hasTemplates = computed(() => timeTemplates.value.length > 0);

    const setLoadingFlag = (
        target: typeof loadingAssignments | typeof loadingSchedules,
        deviceId: string,
        value: boolean,
    ) => {
        target.value = {
            ...target.value,
            [deviceId]: value,
        };
    };

    const loadAssignments = async (deviceId: string) => {
        setLoadingFlag(loadingAssignments, deviceId, true);
        try {
            const items = await taskService.listAssignments(deviceId);
            assignmentsByDevice.value = {
                ...assignmentsByDevice.value,
                [deviceId]: items.map(normalizeAssignment),
            };
            return assignmentsByDevice.value[deviceId];
        } finally {
            setLoadingFlag(loadingAssignments, deviceId, false);
        }
    };

    const loadSchedules = async (deviceId: string) => {
        setLoadingFlag(loadingSchedules, deviceId, true);
        try {
            const items = await taskService.listSchedules(deviceId);
            schedulesByDevice.value = {
                ...schedulesByDevice.value,
                [deviceId]: items,
            };
            return items;
        } finally {
            setLoadingFlag(loadingSchedules, deviceId, false);
        }
    };

    const loadTimeTemplates = async () => {
        timeTemplates.value = await taskService.listTimeTemplates();
    };

    const hydrateForDevices = async (deviceIds: string[]) => {
        await loadTimeTemplates();
        await Promise.all(deviceIds.flatMap((deviceId) => [loadAssignments(deviceId), loadSchedules(deviceId)]));
    };

    const createAssignment = async (
        deviceId: string,
        scriptId: string,
        timeTemplateId: string | null,
        accountData: JsonValue = null,
    ) => {
        const currentAssignments = assignmentsByDevice.value[deviceId] ?? [];
        const assignment: DeviceScriptAssignment = {
            id: await taskService.requestUuid(),
            deviceId,
            scriptId,
            timeTemplateId,
            accountData,
            index: currentAssignments.length,
        };

        await taskService.saveAssignment(assignment);

        await loadAssignments(deviceId);
        return normalizeAssignment(assignment);
    };

    const removeAssignment = async (deviceId: string, assignment: AssignmentRecord) => {
        await taskService.deleteAssignment(assignment.id);
        await loadAssignments(deviceId);
    };

    const clearSchedules = async (deviceId: string) => {
        await taskService.clearSchedules(deviceId);
        schedulesByDevice.value = {
            ...schedulesByDevice.value,
            [deviceId]: [],
        };
    };

    const clearSchedulesByScript = async (scriptId: string) => {
        await taskService.clearSchedulesByScript(scriptId);
    };

    return {
        assignmentsByDevice,
        clearSchedules,
        clearSchedulesByScript,
        createAssignment,
        hasTemplates,
        hydrateForDevices,
        loadAssignments,
        loadingAssignments,
        loadingSchedules,
        loadSchedules,
        loadTimeTemplates,
        removeAssignment,
        schedulesByDevice,
        timeTemplates,
    };
});
