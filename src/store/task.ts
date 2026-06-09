import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { taskService } from '@/services/taskService';
import type { AssignmentRecord, AssignmentSchedule, DeviceScriptSchedule, JsonValue } from '@/types/app/domain';
import type { DeviceScriptAssignment } from '@/types/bindings/DeviceScriptAssignment';
import type { TimeTemplate } from '@/types/bindings/TimeTemplate';

const normalizeAssignment = (assignment: DeviceScriptAssignment): AssignmentRecord => ({
    ...assignment,
    accountData: assignment.accountData ?? null,
});

export const useTaskStore = defineStore('task', () => {
    const assignmentsByDevice = ref<Record<string, AssignmentRecord[]>>({});
    const assignmentSchedulesByDevice = ref<Record<string, AssignmentSchedule[]>>({});
    const schedulesByDevice = ref<Record<string, DeviceScriptSchedule[]>>({});
    const timeTemplates = ref<TimeTemplate[]>([]);
    const loadingAssignments = ref<Record<string, boolean>>({});
    const loadingSchedules = ref<Record<string, boolean>>({});

    const hasTemplates = computed(() => timeTemplates.value.length > 0);

    const assertRealTimeTemplate = (timeTemplateId: string | null) => {
        if (!timeTemplateId) {
            throw new Error('追加队列任务必须选择真实时间模板。');
        }

        if (!timeTemplates.value.some((template) => template.id === timeTemplateId)) {
            throw new Error('所选时间模板不存在或已失效，请重新选择。');
        }
    };

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
            const [assignmentSchedules, items] = await Promise.all([
                taskService.listAssignmentSchedules(deviceId),
                taskService.listSchedules(deviceId),
            ]);
            assignmentSchedulesByDevice.value = {
                ...assignmentSchedulesByDevice.value,
                [deviceId]: assignmentSchedules,
            };
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
        assertRealTimeTemplate(timeTemplateId);
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

    const updateAssignment = async (assignment: DeviceScriptAssignment) => {
        if (assignment.timeTemplateId) {
            assertRealTimeTemplate(assignment.timeTemplateId);
        }
        await taskService.saveAssignment(assignment);
        await loadAssignments(assignment.deviceId);
        return normalizeAssignment(assignment);
    };

    const removeAssignment = async (deviceId: string, assignment: AssignmentRecord) => {
        await taskService.deleteAssignment(assignment.id);
        await loadAssignments(deviceId);
    };

    const detachAssignmentTemplate = async (assignment: AssignmentRecord) => {
        await updateAssignment({
            id: assignment.id,
            deviceId: assignment.deviceId,
            scriptId: assignment.scriptId,
            timeTemplateId: null,
            accountData: assignment.accountData ?? null,
            index: assignment.index,
        });
    };

    const saveTimeTemplate = async (template: TimeTemplate) => {
        await taskService.saveTimeTemplate(template);
        await loadTimeTemplates();
    };

    const deleteTimeTemplate = async (templateId: string) => {
        await taskService.deleteTimeTemplate(templateId);
        await loadTimeTemplates();
        const deviceIds = Object.keys(assignmentsByDevice.value);
        await Promise.all(deviceIds.map((deviceId) => loadAssignments(deviceId)));
    };

    const clearSchedules = async (deviceId: string) => {
        await taskService.clearSchedules(deviceId);
        assignmentSchedulesByDevice.value = {
            ...assignmentSchedulesByDevice.value,
            [deviceId]: [],
        };
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
        assignmentSchedulesByDevice,
        clearSchedules,
        clearSchedulesByScript,
        createAssignment,
        deleteTimeTemplate,
        detachAssignmentTemplate,
        hasTemplates,
        hydrateForDevices,
        loadAssignments,
        loadingAssignments,
        loadingSchedules,
        loadSchedules,
        loadTimeTemplates,
        removeAssignment,
        saveTimeTemplate,
        schedulesByDevice,
        timeTemplates,
        updateAssignment,
    };
});
