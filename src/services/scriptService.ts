import { ApiEnvelope, createServerResponseError, invoke } from '@/utils/api';
import type { PolicyGroupTable } from '@/types/bindings/PolicyGroupTable';
import type { PolicySetTable } from '@/types/bindings/PolicySetTable';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { ScriptTable } from '@/types/bindings/ScriptTable';
import type { ScriptType } from '@/types/bindings/ScriptType';
import type { ScriptVariableCatalog } from '@/types/bindings/ScriptVariableCatalog';
import type {
    ScriptAuthorSeed,
    ScriptChangeLogRecord,
    ScriptCloudSummary,
    MarketPage,
    MarketScriptRecord,
    ScriptSearchInput,
    ScriptVersionPreflight,
    ScriptTableRecord,
} from '@/types/app/domain';

type RawScriptTable = ScriptTable & {
    data: ScriptTableRecord['data'] & {
        scriptType?: ScriptType;
        pkgName?: unknown;
        activityName?: unknown;
    };
};

const DEFAULT_SCRIPT_REQUIRED_FEATURES = ['onnxInference', 'runtime:rhai', 'device:android'];

type ScriptEditorSavePayload = {
    script: ScriptTableRecord;
    tasks: ScriptTaskTable[];
    policies: PolicyTable[];
    policyGroups: PolicyGroupTable[];
    policySets: PolicySetTable[];
    groupPolicyIdsByGroupId: Record<string, string[]>;
    setGroupIdsBySetId: Record<string, string[]>;
};

const emptyMarketPage = (query: ScriptSearchInput): MarketPage<MarketScriptRecord> => ({
    records: [],
    total: 0,
    size: query.size,
    current: query.page,
});

const createEmptyVariableCatalog = (): ScriptVariableCatalog => ({
    version: 1,
    variables: [],
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

const stripScriptInfoAppTarget = <T extends ScriptTableRecord['data']>(value: T) => {
    const data = { ...value } as T & { pkgName?: unknown; activityName?: unknown };
    delete data.pkgName;
    delete data.activityName;
    return data;
};

export const normalizeScriptTable = (script: ScriptTable | ScriptTableRecord): ScriptTableRecord => {
    const raw = script as RawScriptTable;
    const data = stripScriptInfoAppTarget(raw.data);
    return {
        id: raw.id,
        data: {
            ...data,
            contentMd: raw.data.contentMd ?? null,
            scriptType: raw.data.scriptType ?? 'dev',
            platform: raw.data.platform ?? 'android',
            minAppVersion: raw.data.minAppVersion ?? null,
            minRuntimeSchema: raw.data.minRuntimeSchema == null ? null : toSafeNumber(raw.data.minRuntimeSchema, 1),
            requiredFeatures: Array.isArray(raw.data.requiredFeatures) && raw.data.requiredFeatures.length
                ? raw.data.requiredFeatures
                : [...DEFAULT_SCRIPT_REQUIRED_FEATURES],
            variableCatalog: raw.data.variableCatalog ?? createEmptyVariableCatalog(),
            runtimeSettings: {
                recoveryTaskId: raw.data.runtimeSettings?.recoveryTaskId ?? null,
                clickRandomOffset: Math.max(0, toSafeNumber(raw.data.runtimeSettings?.clickRandomOffset, 0)),
            },
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
        contentMd: '',
        userId: author.userId,
        userName: author.userName,
        runtimeType: 'rhai',
        platform: 'android',
        sponsorshipQr: null,
        sponsorshipUrl: null,
        contactInfo: null,
        imgDetModel: null,
        txtDetModel: null,
        txtRecModel: null,
        createTime: new Date().toISOString(),
        updateTime: new Date().toISOString(),
        verName: '0.1.0',
        verNum: 1,
        latestVer: 1,
        downloadCount: 0,
        scriptType: 'dev',
        isValid: true,
        allowClone: true,
        minAppVersion: null,
        minRuntimeSchema: null,
        requiredFeatures: [...DEFAULT_SCRIPT_REQUIRED_FEATURES],
        variableCatalog: createEmptyVariableCatalog(),
        cloudId: null,
        runtimeSettings: {
            recoveryTaskId: null,
            clickRandomOffset: 0,
        },
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
    listPolicies: (scriptId: string) => invoke('get_all_policies_cmd', { scriptId }) as Promise<PolicyTable[]>,
    savePolicy: (policy: PolicyTable) => invoke('save_policy_cmd', { policy }) as Promise<void>,
    removePolicy: (id: string) => invoke('delete_policy_cmd', { id }) as Promise<void>,
    listPolicyGroups: (scriptId: string) => invoke('get_all_policy_groups_cmd', { scriptId }) as Promise<PolicyGroupTable[]>,
    savePolicyGroup: (group: PolicyGroupTable) => invoke('save_policy_group_cmd', { group }) as Promise<void>,
    removePolicyGroup: (id: string) => invoke('delete_policy_group_cmd', { id }) as Promise<void>,
    getGroupPolicies: (groupId: string) => invoke('get_group_policies_cmd', { groupId }) as Promise<string[]>,
    updateGroupPolicies: (groupId: string, policyIds: string[]) =>
        invoke('update_group_policies_cmd', { groupId, policyIds }) as Promise<void>,
    listPolicySets: (scriptId: string) => invoke('get_all_policy_sets_cmd', { scriptId }) as Promise<PolicySetTable[]>,
    savePolicySet: (set: PolicySetTable) => invoke('save_policy_set_cmd', { set }) as Promise<void>,
    removePolicySet: (id: string) => invoke('delete_policy_set_cmd', { id }) as Promise<void>,
    getSetGroups: (setId: string) => invoke('get_set_groups_cmd', { setId }) as Promise<string[]>,
    updateSetGroups: (setId: string, groupIds: string[]) =>
        invoke('update_set_groups_cmd', { setId, groupIds }) as Promise<void>,
    saveEditorBundle: (payload: ScriptEditorSavePayload) =>
        invoke('save_script_editor_cmd', { payload }) as Promise<void>,
    saveLocal: async (script: ScriptTableRecord): Promise<void> => {
        await invoke('save_script_cmd', { script });
    },
    removeLocal: async (scriptId: string): Promise<void> => {
        await invoke('delete_script_cmd', { scriptId });
    },
    uploadLocal: async (scriptId: string): Promise<ApiEnvelope<unknown>> =>
        (await invoke('backend_upload_script', { scriptId })) as ApiEnvelope<unknown>,
    cloneLocal: async (sourceScriptId: string, overwriteCloudId: boolean) =>
        (await invoke('clone_local_script_cmd', {
            sourceScriptId,
            overwriteCloudId,
        })) as ApiEnvelope<string>,
    searchMarket: async (query: ScriptSearchInput): Promise<MarketPage<MarketScriptRecord>> => {
        const response = (await invoke('backend_search_scripts', { req: query })) as ApiEnvelope<MarketPage<MarketScriptRecord>>;
        if (!response.success) {
            throw createServerResponseError('backend_search_scripts', response);
        }
        if (!response.success || !response.data) {
            return emptyMarketPage(query);
        }

        return response.data;
    },
    listChangeLogs: async (scriptId: string, fromVersion?: number | null): Promise<ScriptChangeLogRecord[]> => {
        const response = (await invoke('backend_get_script_change_logs', {
            scriptId,
            fromVersion: fromVersion ?? null,
        })) as ApiEnvelope<ScriptChangeLogRecord[]>;
        if (!response.success) {
            throw createServerResponseError('backend_get_script_change_logs', response);
        }
        return response.data ?? [];
    },
    getCloudSummary: async (scriptId: string): Promise<ScriptCloudSummary | null> => {
        const response = (await invoke('backend_get_script_cloud_summary', {
            scriptId,
        })) as ApiEnvelope<ScriptCloudSummary | null>;
        if (!response.success) {
            throw createServerResponseError('backend_get_script_cloud_summary', response);
        }
        return response.data ?? null;
    },
    preflightDownloadMarketScript: async (
        scriptId: string,
        verName: string | null,
        verNum: number | null,
    ): Promise<ScriptVersionPreflight> => {
        const response = (await invoke('backend_preflight_download_script', {
            scriptId,
            verName,
            verNum,
        })) as ApiEnvelope<ScriptVersionPreflight>;
        if (!response.success || !response.data) {
            throw createServerResponseError('backend_preflight_download_script', response);
        }
        return response.data;
    },
    preflightUploadLocalScript: async (scriptId: string): Promise<ScriptVersionPreflight> => {
        const response = (await invoke('backend_preflight_upload_script', {
            scriptId,
        })) as ApiEnvelope<ScriptVersionPreflight>;
        if (!response.success || !response.data) {
            throw createServerResponseError('backend_preflight_upload_script', response);
        }
        return response.data;
    },
    convertLocalImageToDataUrl: async (imagePath: string): Promise<string> => {
        const base64 = (await invoke('convert_img_to_base64_cmd', { imgPath: imagePath })) as string;
        return `data:image/png;base64,${base64}`;
    },
    downloadMarketScript: async (
        scriptId: string,
        runtimeType: string,
        replaceLocalScriptId: string | null = null,
        scriptName: string | null = null,
    ) =>
        (await invoke('backend_download_script', {
            scriptId,
            runtimeType,
            replaceLocalScriptId,
            scriptName,
        })) as ApiEnvelope<string>,
    getYoloLabels: async (path: string): Promise<Array<{ index: number; label: string }>> => {
        const labels = (await invoke('get_yolo_labels_cmd', { path })) as Record<string, string>;
        return Object.entries(labels)
            .map(([index, label]) => ({
                index: Number(index),
                label,
            }))
            .filter((item) => Number.isFinite(item.index))
            .sort((left, right) => left.index - right.index);
    },
};


