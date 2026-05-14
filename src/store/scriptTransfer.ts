import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { listen } from '@tauri-apps/api/event';
import { scriptTransferService } from '@/services/scriptTransferService';
import type {
  ScriptTransferDirection,
  ScriptTransferProgressEvent,
  ScriptTransferRecord,
} from '@/types/app/domain';

const normalizeProgressEvent = (payload: unknown): ScriptTransferProgressEvent | null => {
  if (!payload || typeof payload !== 'object') {
    return null;
  }

  const record = payload as Record<string, unknown>;
  if (typeof record.id !== 'string' || typeof record.direction !== 'string' || typeof record.status !== 'string') {
    return null;
  }

  return {
    id: record.id,
    direction: record.direction as ScriptTransferDirection,
    localScriptId: typeof record.localScriptId === 'string' ? record.localScriptId : null,
    cloudScriptId: typeof record.cloudScriptId === 'string' ? record.cloudScriptId : null,
    scriptName: typeof record.scriptName === 'string' ? record.scriptName : null,
    status: record.status as ScriptTransferProgressEvent['status'],
    modelFileCount: Number(record.modelFileCount ?? 0) || 0,
    completedModelFileCount: Number(record.completedModelFileCount ?? 0) || 0,
    currentFileName: typeof record.currentFileName === 'string' ? record.currentFileName : null,
    latestFileName: typeof record.latestFileName === 'string' ? record.latestFileName : null,
    bytesTransferred: Number(record.bytesTransferred ?? 0) || 0,
    totalBytes: Number(record.totalBytes ?? 0) || 0,
    latestMessage: typeof record.latestMessage === 'string' ? record.latestMessage : null,
    errorMessage: typeof record.errorMessage === 'string' ? record.errorMessage : null,
    startedAt: typeof record.startedAt === 'string' ? record.startedAt : '',
    finishedAt: typeof record.finishedAt === 'string' ? record.finishedAt : null,
    updatedAt: typeof record.updatedAt === 'string' ? record.updatedAt : '',
  };
};

const eventToRecord = (event: ScriptTransferProgressEvent): ScriptTransferRecord => ({
  id: event.id,
  direction: event.direction,
  localScriptId: event.localScriptId,
  cloudScriptId: event.cloudScriptId,
  scriptName: event.scriptName,
  status: event.status,
  modelFileCount: event.modelFileCount,
  completedModelFileCount: event.completedModelFileCount,
  latestFileName: event.latestFileName ?? event.currentFileName,
  bytesTransferred: event.bytesTransferred,
  totalBytes: event.totalBytes,
  latestMessage: event.latestMessage,
  errorMessage: event.errorMessage,
  startedAt: event.startedAt,
  finishedAt: event.finishedAt,
  createdAt: event.startedAt,
  updatedAt: event.updatedAt,
});

const recordUpdatedAt = (record: ScriptTransferRecord) =>
  new Date(record.updatedAt || record.finishedAt || record.startedAt || 0).getTime();

export const useScriptTransferStore = defineStore('scriptTransfer', () => {
  const initialized = ref(false);
  const recordsById = ref<Record<string, ScriptTransferRecord>>({});
  const lastProgressEventById = ref<Record<string, ScriptTransferProgressEvent>>({});

  const upsertRecord = (record: ScriptTransferRecord) => {
    recordsById.value = {
      ...recordsById.value,
      [record.id]: record,
    };
  };

  const initListener = async () => {
    if (initialized.value) {
      return;
    }

    await listen('script-transfer', (event) => {
      const payload = normalizeProgressEvent(event.payload);
      if (!payload) {
        return;
      }

      lastProgressEventById.value = {
        ...lastProgressEventById.value,
        [payload.id]: payload,
      };
      upsertRecord(eventToRecord(payload));
    });

    initialized.value = true;
  };

  const loadRecords = async (options: {
    direction?: ScriptTransferDirection | null;
    localScriptId?: string | null;
    cloudScriptId?: string | null;
    limit?: number;
  }) => {
    const records = await scriptTransferService.listRecords(options);
    const next = { ...recordsById.value };
    for (const record of records) {
      next[record.id] = record;
    }
    recordsById.value = next;
    return records;
  };

  const deleteRecord = async (recordId: string) => {
    await scriptTransferService.deleteRecord(recordId);
    const nextRecords = { ...recordsById.value };
    const nextEvents = { ...lastProgressEventById.value };
    delete nextRecords[recordId];
    delete nextEvents[recordId];
    recordsById.value = nextRecords;
    lastProgressEventById.value = nextEvents;
  };

  const clearRecords = async (options: {
    direction?: ScriptTransferDirection | null;
    localScriptId?: string | null;
    cloudScriptId?: string | null;
  }) => {
    await scriptTransferService.clearRecords(options);
    const nextRecords = Object.fromEntries(
      Object.entries(recordsById.value).filter(([, record]) => {
        if (options.direction && record.direction !== options.direction) return true;
        if (options.localScriptId && record.localScriptId !== options.localScriptId) return true;
        if (options.cloudScriptId && record.cloudScriptId !== options.cloudScriptId) return true;
        return false;
      }),
    );
    const nextEvents = Object.fromEntries(
      Object.entries(lastProgressEventById.value).filter(([, record]) => {
        if (options.direction && record.direction !== options.direction) return true;
        if (options.localScriptId && record.localScriptId !== options.localScriptId) return true;
        if (options.cloudScriptId && record.cloudScriptId !== options.cloudScriptId) return true;
        return false;
      }),
    );
    recordsById.value = nextRecords;
    lastProgressEventById.value = nextEvents;
  };

  const allRecords = computed(() =>
    Object.values(recordsById.value).sort((left, right) => recordUpdatedAt(right) - recordUpdatedAt(left)),
  );

  const getRecordsForScope = (options: {
    direction: ScriptTransferDirection;
    localScriptId?: string | null;
    cloudScriptId?: string | null;
  }) =>
    allRecords.value.filter((record) => {
      if (record.direction !== options.direction) {
        return false;
      }
      if (options.localScriptId) {
        return record.localScriptId === options.localScriptId;
      }
      if (options.cloudScriptId) {
        return record.cloudScriptId === options.cloudScriptId;
      }
      return false;
    });

  const getLatestProgressEvent = (recordId: string) => lastProgressEventById.value[recordId] ?? null;

  return {
    allRecords,
    clearRecords,
    deleteRecord,
    getLatestProgressEvent,
    getRecordsForScope,
    initListener,
    initialized,
    lastProgressEventById,
    loadRecords,
    recordsById,
  };
});
