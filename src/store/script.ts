import { defineStore } from 'pinia';
import { computed, ref, shallowRef } from 'vue';
import { createEditableScript, scriptService } from '@/services/scriptService';
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
    const scripts = shallowRef<ScriptTableRecord[]>([]);
    const selectedScriptId = ref<string | null>(null);
    const loading = ref(false);
    const marketLoading = ref(false);
    const marketAppending = ref(false);
    const taskLoading = ref<Record<string, boolean>>({});
    const marketQuery = ref<ScriptSearchInput>(defaultMarketQuery());
    const marketPage = ref<MarketPage<MarketScriptRecord>>(emptyMarketPage());
    const tasksByScriptId = ref<Record<string, ScriptTaskTable[]>>({});

    const selectedScript = computed<ScriptTableRecord | null>(
        () => {
            if (!selectedScriptId.value) {
                return null;
            }

            for (const script of scripts.value) {
                if (script.id === selectedScriptId.value) {
                    return script;
                }
            }

            return null;
        },
    );

    const sortedScripts = computed<ScriptTableRecord[]>(() =>
        [...scripts.value].sort((left, right) => {
            const leftTime = left.data.updateTime ? new Date(left.data.updateTime).getTime() : 0;
            const rightTime = right.data.updateTime ? new Date(right.data.updateTime).getTime() : 0;
            return rightTime - leftTime;
        }),
    );
    const hasMoreMarket = computed(() => marketPage.value.records.length < marketPage.value.total);

    const loadScripts = async () => {
        loading.value = true;
        try {
            scripts.value = await scriptService.listLocal();
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

    const deleteScript = async (scriptId: string) => {
        await scriptService.removeLocal(scriptId);
        scripts.value = scripts.value.filter((script) => script.id !== scriptId);
        if (selectedScriptId.value === scriptId) {
            selectedScriptId.value = scripts.value[0]?.id ?? null;
        }
    };

    const uploadScript = (scriptId: string) => scriptService.uploadLocal(scriptId);

    const cloneScript = (sourceScriptId: string, overwriteCloudId: boolean) =>
        scriptService.cloneLocal(sourceScriptId, overwriteCloudId);

    const searchMarket = async (
        partial?: Partial<ScriptSearchInput>,
        options?: {
            append?: boolean;
        },
    ) => {
        marketLoading.value = true;
        marketAppending.value = Boolean(options?.append);
        try {
            const nextQuery = {
                ...marketQuery.value,
                ...partial,
            };
            const nextPage = await scriptService.searchMarket(nextQuery);
            marketQuery.value = nextQuery;
            marketPage.value = options?.append
                ? {
                    ...nextPage,
                    records: [...marketPage.value.records, ...nextPage.records],
                }
                : nextPage;
        } finally {
            marketLoading.value = false;
            marketAppending.value = false;
        }
    };

    const loadMoreMarket = async () => {
        if (marketLoading.value || !hasMoreMarket.value) {
            return;
        }

        await searchMarket(
            {
                page: marketPage.value.current + 1,
            },
            { append: true },
        );
    };

    const downloadMarketScript = (
        scriptId: string,
        runtimeType: string,
        replaceLocalScriptId: string | null = null,
        scriptName: string | null = null,
    ) => scriptService.downloadMarketScript(scriptId, runtimeType, replaceLocalScriptId, scriptName);

    return {
        cloneScript,
        deleteScript,
        downloadMarketScript,
        loadScripts,
        loadScriptTasks,
        loadMoreMarket,
        loading,
        hasMoreMarket,
        marketAppending,
        marketLoading,
        marketPage,
        marketQuery,
        prepareScript,
        saveScript,
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
