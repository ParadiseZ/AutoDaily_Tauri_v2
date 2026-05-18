import { invoke } from '@/utils/api';
import type { ScriptTransferDirection, ScriptTransferRecord } from '@/types/app/domain';

const normalizeRecord = (record: ScriptTransferRecord): ScriptTransferRecord => ({
  ...record,
  modelFileCount: Number(record.modelFileCount ?? 0) || 0,
  completedModelFileCount: Number(record.completedModelFileCount ?? 0) || 0,
  bytesTransferred: Number(record.bytesTransferred ?? 0) || 0,
  totalBytes: Number(record.totalBytes ?? 0) || 0,
});

export const scriptTransferService = {
  listRecords: async (options: {
    direction?: ScriptTransferDirection | null;
    localScriptId?: string | null;
    cloudScriptId?: string | null;
    limit?: number;
  }) => {
    const records = (await invoke('list_script_transfer_records_cmd', {
      direction: options.direction ?? null,
      localScriptId: options.localScriptId ?? null,
      cloudScriptId: options.cloudScriptId ?? null,
      limit: options.limit ?? 20,
    })) as ScriptTransferRecord[];

    return records.map(normalizeRecord);
  },
  deleteRecord: (recordId: string) =>
    invoke('delete_script_transfer_record_cmd', { recordId }) as Promise<void>,
  pauseRecord: (recordId: string) =>
    invoke('pause_script_transfer_record_cmd', { recordId }) as Promise<void>,
  resumeRecord: (recordId: string) =>
    invoke('resume_script_transfer_record_cmd', { recordId }) as Promise<void>,
  clearRecords: (options: {
    direction?: ScriptTransferDirection | null;
    localScriptId?: string | null;
    cloudScriptId?: string | null;
  }) =>
    invoke('clear_script_transfer_records_cmd', {
      direction: options.direction ?? null,
      localScriptId: options.localScriptId ?? null,
      cloudScriptId: options.cloudScriptId ?? null,
    }) as Promise<void>,
};
