
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export const useScripts = () => {
    const scripts = ref([]);
    const selectedScript = ref(null);
    const selectedTemplate = ref(null);

    const getAllScripts = async () => {
        try {
            const res = await invoke('get_all_scripts_cmd');
            // Backend returns Vec<ScriptTable>, which has { id, data: ScriptInfo }
            // We map it to a flatter structure for the UI
            scripts.value = res.map(item => ({
                id: item.id,
                ...item.data
            }));
            return scripts.value;
        } catch (e) {
            console.error('Failed to fetch scripts:', e);
            return [];
        }
    };

    const saveScript = async (scriptData) => {
        try {
            // If it's a new script, it might not have an id yet
            // Backend's ScriptTable needs an id and data (ScriptInfo)
            const scriptTable = {
                id: scriptData.id || await invoke('get_uuid_v7'),
                data: { ...scriptData }
            };
            delete scriptTable.data.id; // Remove id from data part to match Rust struct

            await invoke('save_script_cmd', { script: scriptTable });
            await getAllScripts(); // Refresh list
            return true;
        } catch (e) {
            console.error('Failed to save script:', e);
            throw e;
        }
    };

    const deleteScript = async (script) => {
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

    const selectScript = (script) => {
        selectedScript.value = script;
        if (script.templates && script.templates.length > 0) {
            selectedTemplate.value = script.templates[0].id;
        } else {
            selectedTemplate.value = null;
        }
    };

    const editScript = (script) => {
        console.log('Edit script:', script.name);
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
