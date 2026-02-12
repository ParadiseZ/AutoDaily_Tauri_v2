import type {Ref} from 'vue';
import {ref} from 'vue';
import {invoke} from '@tauri-apps/api/core';
import type {ScriptInfo, ScriptTable} from '@/types/bindings';

export const useScripts = () => {
    const scripts: Ref<ScriptTable[]> = ref([]);
    const selectedScript: Ref<ScriptTable | null> = ref(null);
    const selectedTemplate: Ref<string | null> = ref(null);

    const getAllScripts = async () => {
        try {
            scripts.value = await invoke<ScriptTable[]>('get_all_scripts_cmd');
            return scripts.value;
        } catch (e) {
            console.error('Failed to fetch scripts:', e);
            return [];
        }
    };

    const saveScript = async (scriptData: ScriptInfo & { id?: string }) => {
        try {
            const id = scriptData.id || await invoke<string>('get_uuid_v7');
            const data = { ...scriptData };
            delete (data as any).id;

            const scriptTable: ScriptTable = {
                id,
                data
            };

            await invoke('save_script_cmd', { script: scriptTable });
            await getAllScripts(); // Refresh list
            return true;
        } catch (e) {
            console.error('Failed to save script:', e);
            throw e;
        }
    };

    const deleteScript = async (script: ScriptTable) => {
        try {
            await invoke('delete_script_cmd', { scriptId: script.id });
            await getAllScripts();
            if (selectedScript.value?.id === script.id) {
                selectedScript.value = null;
            }
            return true;
        } catch (e) {
            console.error('Failed to delete script:', e);
            return false;
        }
    };

    const selectScript = (script: ScriptTable) => {
        selectedScript.value = script;
        // Note: ScriptInfo doesn't actually have templates in the bindings, 
        // but the UI currently expects it. We'll handle it cautiously.
        const data = script.data as any;
        if (data.templates && data.templates.length > 0) {
            selectedTemplate.value = data.templates[0];
        } else {
            selectedTemplate.value = null;
        }
    };

    const editScript = (script: ScriptTable) => {
        console.log('Edit script:', script.data.name);
    };

    return {
        scripts,
        selectedScript,
        selectedTemplate,
        getAllScripts,
        saveScript,
        deleteScript,
        selectScript,
        editScript
    };
};