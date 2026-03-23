import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { createBlankScript, normalizeScriptTable, scriptService } from '@/services/scriptService';
import { taskService } from '@/services/taskService';
import type {
    MarketPage,
    MarketScriptRecord,
    ScriptSearchInput,
    ScriptTableRecord,
    UserProfile,
} from '@/types/app/domain';

const defaultMarketQuery = (): ScriptSearchInput => ({
    page: 1,
    size: 12,
    keyword: '',
    author: '',
    runtimeType: '',
});

const emptyMarketPage = (): MarketPage<MarketScriptRecord> => ({
    records: [],
    total: 0,
    size: 12,
    current: 1,
});

export const useScriptStore = defineStore('script', () => {
    const scripts = ref<ScriptTableRecord[]>([]);
    const selectedScriptId = ref<string | null>(null);
    const loading = ref(false);
    const marketLoading = ref(false);
    const marketQuery = ref<ScriptSearchInput>(defaultMarketQuery());
    const marketPage = ref<MarketPage<MarketScriptRecord>>(emptyMarketPage());

    const selectedScript = computed(
        () => scripts.value.find((script) => script.id === selectedScriptId.value) ?? null,
    );

    const sortedScripts = computed(() =>
        [...scripts.value].sort((left, right) => {
            const leftTime = left.data.updateTime ? new Date(left.data.updateTime).getTime() : 0;
            const rightTime = right.data.updateTime ? new Date(right.data.updateTime).getTime() : 0;
            return rightTime - leftTime;
        }),
    );

    const loadScripts = async () => {
        loading.value = true;
        try {
            scripts.value = (await scriptService.listLocal()).map(normalizeScriptTable);
            if (!selectedScriptId.value && scripts.value.length > 0) {
                selectedScriptId.value = scripts.value[0].id;
            }
        } finally {
            loading.value = false;
        }
    };

    const selectScript = (scriptId: string) => {
        selectedScriptId.value = scriptId;
    };

    const createScript = async (name: string, userProfile: UserProfile | null) => {
        const id = await taskService.requestUuid();
        const script = createBlankScript(name, userProfile, id);
        await scriptService.saveLocal(script);
        await loadScripts();
        selectScript(id);
        return script;
    };

    const saveScript = async (script: ScriptTableRecord) => {
        await scriptService.saveLocal(script);
        await loadScripts();
    };

    const deleteScript = async (scriptId: string) => {
        await scriptService.removeLocal(scriptId);
        scripts.value = scripts.value.filter((script) => script.id !== scriptId);
        if (selectedScriptId.value === scriptId) {
            selectedScriptId.value = scripts.value[0]?.id ?? null;
        }
    };

    const uploadScript = (scriptId: string) => scriptService.uploadLocal(scriptId);

    const cloneScript = (sourceScriptId: string, currentUserId: string | null, overwriteCloudId: boolean) =>
        scriptService.cloneLocal(sourceScriptId, currentUserId, overwriteCloudId);

    const searchMarket = async (partial?: Partial<ScriptSearchInput>) => {
        marketLoading.value = true;
        try {
            marketQuery.value = {
                ...marketQuery.value,
                ...partial,
            };
            marketPage.value = await scriptService.searchMarket(marketQuery.value);
        } finally {
            marketLoading.value = false;
        }
    };

    const downloadMarketScript = (scriptId: string, currentUserId: string | null) =>
        scriptService.downloadMarketScript(scriptId, currentUserId);

    return {
        cloneScript,
        createScript,
        deleteScript,
        downloadMarketScript,
        loadScripts,
        loading,
        marketLoading,
        marketPage,
        marketQuery,
        saveScript,
        scripts,
        searchMarket,
        selectedScript,
        selectedScriptId,
        selectScript,
        sortedScripts,
        uploadScript,
    };
});
