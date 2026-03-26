import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { createEditableScript, normalizeScriptTable, scriptService } from '@/services/scriptService';
import { taskService } from '@/services/taskService';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type {
    MarketPage,
    MarketScriptRecord,
    ScriptAuthorSeed,
    ScriptSearchInput,
    ScriptTableRecord,
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

const normalizeScriptTasks = (tasks: ScriptTaskTable[]) =>
    [...tasks].sort((left, right) => {
        if (left.index !== right.index) {
            return left.index - right.index;
        }

        return new Date(left.createdAt).getTime() - new Date(right.createdAt).getTime();
    });

export const useScriptStore = defineStore('script', () => {
    const scripts = ref<ScriptTableRecord[]>([]);
    const selectedScriptId = ref<string | null>(null);
    const loading = ref(false);
    const marketLoading = ref(false);
    const taskLoading = ref<Record<string, boolean>>({});
    const marketQuery = ref<ScriptSearchInput>(defaultMarketQuery());
    const marketPage = ref<MarketPage<MarketScriptRecord>>(emptyMarketPage());
    const tasksByScriptId = ref<Record<string, ScriptTaskTable[]>>({});

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

    const saveScript = async (script: ScriptTableRecord) => {
        await scriptService.saveLocal(script);
        await loadScripts();
        selectScript(script.id);
    };

    const prepareScript = (author: ScriptAuthorSeed, name: string) =>
        createEditableScript(taskService.requestUuid, author, name);

    const loadScriptTasks = async (scriptId: string) => {
        taskLoading.value = {
            ...taskLoading.value,
            [scriptId]: true,
        };
        try {
            const tasks = normalizeScriptTasks(await scriptService.listTasks(scriptId));
            tasksByScriptId.value = {
                ...tasksByScriptId.value,
                [scriptId]: tasks,
            };
            return tasks;
        } finally {
            taskLoading.value = {
                ...taskLoading.value,
                [scriptId]: false,
            };
        }
    };

    const saveScriptTasks = async (scriptId: string, tasks: ScriptTaskTable[]) => {
        const normalized = normalizeScriptTasks(tasks);
        await scriptService.saveTasks(scriptId, normalized);
        tasksByScriptId.value = {
            ...tasksByScriptId.value,
            [scriptId]: normalized,
        };
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
        deleteScript,
        downloadMarketScript,
        loadScripts,
        loadScriptTasks,
        loading,
        marketLoading,
        marketPage,
        marketQuery,
        prepareScript,
        saveScript,
        saveScriptTasks,
        scripts,
        searchMarket,
        selectedScript,
        selectedScriptId,
        selectScript,
        sortedScripts,
        taskLoading,
        tasksByScriptId,
        uploadScript,
    };
});
