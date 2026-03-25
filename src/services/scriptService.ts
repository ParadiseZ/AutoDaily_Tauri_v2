import { invoke } from '@/utils/api';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { ScriptTable } from '@/types/bindings/ScriptTable';
import type { ScriptType } from '@/types/bindings/ScriptType';
import type {
    ScriptAuthorSeed,
    MarketPage,
    MarketScriptRecord,
    ScriptSearchInput,
    ScriptTableRecord,
} from '@/types/app/domain';

type RawScriptTable = ScriptTable & {
    data: ScriptTableRecord['data'] & {
        scriptType?: ScriptType;
    };
};

interface ApiEnvelope<T> {
    success: boolean;
    data?: T;
    message?: string;
}

type ScriptTablePayload = {
    id: string;
    data: Omit<ScriptTableRecord['data'], 'verNum' | 'latestVer' | 'downloadCount'> & {
        verNum: number;
        latestVer: number;
        downloadCount: number;
    };
};

const emptyMarketPage = (query: ScriptSearchInput): MarketPage<MarketScriptRecord> => ({
    records: [],
    total: 0,
    size: query.size,
    current: query.page,
});

const toSafeNumber = (value: bigint | number | string | null | undefined, fallback = 0) => {
    if (typeof value === 'number' && Number.isFinite(value)) {
        return value;
    }

    if (typeof value === 'bigint') {
        return Number(value);
    }

    if (typeof value === 'string') {
        const parsed = Number(value);
        return Number.isFinite(parsed) ? parsed : fallback;
    }

    return fallback;
};

export const normalizeScriptTable = (script: ScriptTable | ScriptTableRecord): ScriptTableRecord => {
    const raw = script as RawScriptTable;
    return {
        id: raw.id,
        data: {
            ...raw.data,
            scriptType: raw.data.scriptType ?? 'dev',
            verNum: toSafeNumber(raw.data.verNum, 1),
            latestVer: toSafeNumber(raw.data.latestVer, 1),
            downloadCount: toSafeNumber(raw.data.downloadCount, 0),
        },
    };
};

export const createBlankScript = (
    name: string,
    author: { userId: string; userName: string },
    id: string,
): ScriptTableRecord => ({
    id,
    data: {
        name,
        description: '',
        userId: author.userId,
        userName: author.userName,
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
        verNum: 1,
        latestVer: 1,
        downloadCount: 0,
        scriptType: 'dev',
        isValid: true,
        allowClone: true,
        cloudId: null,
    },
});

const serializeScriptTable = (script: ScriptTableRecord): ScriptTablePayload => ({
    id: script.id,
    data: {
        ...script.data,
        verNum: toSafeNumber(script.data.verNum, 1),
        latestVer: toSafeNumber(script.data.latestVer, 1),
        downloadCount: toSafeNumber(script.data.downloadCount, 0),
    },
});

export const createScriptName = (index: number) => `未命名脚本 ${index}`;

export const createEditableScript = async (
    requestUuid: () => Promise<string>,
    author: ScriptAuthorSeed,
    name: string,
): Promise<ScriptTableRecord> =>
    createBlankScript(
        name,
        {
            userId: author.userId?.trim() || (await requestUuid()),
            userName: author.userName?.trim() || 'Guest',
        },
        await requestUuid(),
    );

export const scriptService = {
    listLocal: async (): Promise<ScriptTableRecord[]> => {
        const scripts = (await invoke('get_all_scripts_cmd')) as ScriptTable[];
        return scripts.map(normalizeScriptTable);
    },
    listTasks: (scriptId: string) => invoke('get_script_tasks_cmd', { scriptId }) as Promise<ScriptTaskTable[]>,
    saveLocal: async (script: ScriptTableRecord): Promise<void> => {
        await invoke('save_script_cmd', { script: serializeScriptTable(script) });
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
    convertLocalImageToDataUrl: async (imagePath: string): Promise<string> => {
        const base64 = (await invoke('convert_img_to_base64_cmd', { imgPath: imagePath })) as string;
        return `data:image/png;base64,${base64}`;
    },
    downloadMarketScript: async (scriptId: string, currentUserId: string | null) =>
        (await invoke('backend_download_script', { scriptId, currentUserId })) as ApiEnvelope<string>,
};
