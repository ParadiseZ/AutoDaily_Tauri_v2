import { invoke } from '@/utils/api';
import type { ScriptTimeTemplateValuesDto } from '@/types/app/domain';

export const scriptTemplateValueService = {
    get: (
        deviceId: string,
        scriptId: string,
        timeTemplateId: string,
        accountId: string | null = null,
    ) =>
        invoke('get_script_time_template_values_cmd', {
            deviceId,
            scriptId,
            timeTemplateId,
            accountId,
        }) as Promise<ScriptTimeTemplateValuesDto | null>,
    save: (record: ScriptTimeTemplateValuesDto) =>
        invoke('save_script_time_template_values_cmd', { record }) as Promise<void>,
    remove: (
        deviceId: string,
        scriptId: string,
        timeTemplateId: string,
        accountId: string | null = null,
    ) =>
        invoke('delete_script_time_template_values_cmd', {
            deviceId,
            scriptId,
            timeTemplateId,
            accountId,
        }) as Promise<void>,
};
