import { invoke } from '@/utils/api';
import type { ScriptTable } from '@/types/bindings/ScriptTable';
import type { ScriptType } from '@/types/bindings/ScriptType';
import type {
    MarketPage,
    MarketScriptRecord,
    ScriptSearchInput,
    ScriptTableRecord,
    UserProfile,
} from '@/types/app/domain';

type RawScriptTable = ScriptTable & {
    data: ScriptTableRecord['data'] & {
        scriptType?: ScriptType;
        scriptTyCpe?: ScriptType;
    };
};

interface ApiEnvelope<T> {
    success: boolean;
    data?: T;
    message?: string;
}

const emptyMarketPage = (query: ScriptSearchInput): MarketPage<MarketScriptRecord> => ({
    records: [],
    total: 0,
    size: query.size,
    current: query.page,
});

export const normalizeScriptTable = (script: ScriptTable | ScriptTableRecord): ScriptTableRecord => {
    const raw = script as RawScriptTable;
    return {
        id: raw.id,
        data: {
            ...raw.data,
            scriptType: raw.data.scriptType ?? raw.data.scriptTyCpe ?? 'dev',
        },
    };
};

export const createBlankScript = (name: string, userProfile: UserProfile | null, id: string): ScriptTableRecord => ({
    id,
    data: {
        name,
        description: '',
        userId: userProfile?.id ?? '',
        userName: userProfile?.username ?? 'Local User',
        runtimeType: 'rhai',
        sponsorshipQr: null,
        sponsorshipUrl: null,
        contactInfo: null,
        imgDetModel: null,
        txtDetModel: null,
        txtRecModel: null,
        pkgName: null,
        createTime: new Date().toISOString(),
        updateTime: new Date().toISOString(),
        verName: '0.1.0',
        verNum: 1n,
        latestVer: 1n,
        downloadCount: 0n,
        scriptType: 'dev',
        isValid: true,
        allowClone: true,
        cloudId: null,
    },
});

export const scriptService = {
    listLocal: async (): Promise<ScriptTableRecord[]> => {
        const scripts = (await invoke('get_all_scripts_cmd')) as ScriptTable[];
        return scripts.map(normalizeScriptTable);
    },
    saveLocal: async (script: ScriptTableRecord): Promise<void> => {
        await invoke('save_script_cmd', { script });
    },
    removeLocal: async (scriptId: string): Promise<void> => {
        await invoke('delete_script_cmd', { scriptId });
    },
    uploadLocal: async (scriptId: string): Promise<ApiEnvelope<unknown>> =>
        (await invoke('backend_upload_script', { scriptId })) as ApiEnvelope<unknown>,
    cloneLocal: async (sourceScriptId: string, currentUserId: string | null, overwriteCloudId: boolean) =>
        (await invoke('clone_local_script_cmd', {
            sourceScriptId,
            currentUserId,
            overwriteCloudId,
        })) as ApiEnvelope<string>,
    searchMarket: async (query: ScriptSearchInput): Promise<MarketPage<MarketScriptRecord>> => {
        const response = (await invoke('backend_search_scripts', { req: query })) as ApiEnvelope<MarketPage<MarketScriptRecord>>;
        if (!response.success || !response.data) {
            return emptyMarketPage(query);
        }

        return response.data;
    },
    downloadMarketScript: async (scriptId: string, currentUserId: string | null) =>
        (await invoke('backend_download_script', { scriptId, currentUserId })) as ApiEnvelope<string>,
};
