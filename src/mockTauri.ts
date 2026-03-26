import { mockConvertFileSrc, mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import type { LogConfig } from '@/types/app/domain';
import type { ScriptTable, ScriptTaskTable } from '@/types/bindings';

type StoreData = Record<string, unknown>;
type AssignmentMap = Record<string, unknown[]>;
type ScheduleMap = Record<string, unknown[]>;
type ScriptTaskMap = Record<string, ScriptTaskTable[]>;
type StoredScriptTable = Omit<ScriptTable, 'data'> & {
  data: Omit<ScriptTable['data'], 'downloadCount' | 'latestVer' | 'verNum'> & {
    downloadCount: number;
    latestVer: number;
    verNum: number;
  };
};

interface MockState {
  store: StoreData;
  scripts: StoredScriptTable[];
  scriptTasks: ScriptTaskMap;
  assignmentsByDevice: AssignmentMap;
  schedulesByDevice: ScheduleMap;
  devices: unknown[];
  timeTemplates: unknown[];
}

interface MockApi {
  getState: () => MockState;
  reset: () => MockState;
  seed: (partial: Partial<MockState>) => MockState;
}

declare global {
  interface Window {
    __AUTODAILY_MOCK__?: MockApi;
  }
}

const MOCK_STATE_KEY = 'autodaily.mock.state';
const STORE_RESOURCE_ID = 1;
const DEFAULT_LOG_CONFIG: LogConfig = {
  logLevel: 'Info',
  logDir: 'logs',
  retentionDays: 7,
};

const createDefaultState = (): MockState => ({
  store: {},
  scripts: [],
  scriptTasks: {},
  assignmentsByDevice: {},
  schedulesByDevice: {},
  devices: [],
  timeTemplates: [],
});

const isBrowserMockTarget =
  typeof window !== 'undefined' &&
  typeof navigator !== 'undefined' &&
  !navigator.userAgent.includes('Tauri');

if (isBrowserMockTarget && !(window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) {
  const readState = (): MockState => {
    const raw = window.localStorage.getItem(MOCK_STATE_KEY);
    if (!raw) {
      return createDefaultState();
    }

    try {
      const parsed = JSON.parse(raw) as Partial<MockState>;
      return {
        ...createDefaultState(),
        ...parsed,
        store: parsed.store ?? {},
        scripts: parsed.scripts ?? [],
        scriptTasks: parsed.scriptTasks ?? {},
        assignmentsByDevice: parsed.assignmentsByDevice ?? {},
        schedulesByDevice: parsed.schedulesByDevice ?? {},
        devices: parsed.devices ?? [],
        timeTemplates: parsed.timeTemplates ?? [],
      };
    } catch {
      return createDefaultState();
    }
  };

  const writeState = (state: MockState): MockState => {
    window.localStorage.setItem(MOCK_STATE_KEY, JSON.stringify(state));
    return state;
  };

  const updateState = (updater: (current: MockState) => MockState): MockState => {
    const next = updater(readState());
    return writeState(next);
  };

  const seedState = (partial: Partial<MockState>): MockState =>
    updateState((current) => ({
      ...current,
      ...partial,
      store: partial.store ?? current.store,
      scripts: partial.scripts ?? current.scripts,
      scriptTasks: partial.scriptTasks ?? current.scriptTasks,
      assignmentsByDevice: partial.assignmentsByDevice ?? current.assignmentsByDevice,
      schedulesByDevice: partial.schedulesByDevice ?? current.schedulesByDevice,
      devices: partial.devices ?? current.devices,
      timeTemplates: partial.timeTemplates ?? current.timeTemplates,
    }));

  const upsertScript = (scripts: StoredScriptTable[], script: StoredScriptTable) => {
    const next = [...scripts];
    const index = next.findIndex((item) => item.id === script.id);
    if (index >= 0) {
      next[index] = script;
    } else {
      next.push(script);
    }

    return next.sort((left, right) => {
      const leftTime = left.data.updateTime ? new Date(left.data.updateTime).getTime() : 0;
      const rightTime = right.data.updateTime ? new Date(right.data.updateTime).getTime() : 0;
      return rightTime - leftTime;
    });
  };

  const readStoreValue = (state: MockState, key: string) => {
    const exists = Object.prototype.hasOwnProperty.call(state.store, key);
    return [exists ? state.store[key] : null, exists] as const;
  };

  const buildUuid = () => {
    if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
      return crypto.randomUUID();
    }

    return `mock-${Date.now()}-${Math.random().toString(16).slice(2)}`;
  };

  mockWindows('main');
  mockConvertFileSrc('windows');
  mockIPC(
    (cmd, payload) => {
      const args =
        payload && typeof payload === 'object' && !Array.isArray(payload)
          ? (payload as Record<string, unknown>)
          : {};

      switch (cmd) {
        case 'plugin:store|load':
        case 'plugin:store|get_store':
          return STORE_RESOURCE_ID;
        case 'plugin:store|get': {
          const state = readState();
          return readStoreValue(state, String(args.key));
        }
        case 'plugin:store|set':
          updateState((current) => ({
            ...current,
            store: {
              ...current.store,
              [String(args.key)]: args.value,
            },
          }));
          return null;
        case 'plugin:store|has': {
          const state = readState();
          return Object.prototype.hasOwnProperty.call(state.store, String(args.key));
        }
        case 'plugin:store|delete':
          updateState((current) => {
            const nextStore = { ...current.store };
            delete nextStore[String(args.key)];
            return { ...current, store: nextStore };
          });
          return null;
        case 'plugin:store|clear':
        case 'plugin:store|reset':
          updateState((current) => ({ ...current, store: {} }));
          return null;
        case 'plugin:store|keys':
          return Object.keys(readState().store);
        case 'plugin:store|values':
          return Object.values(readState().store);
        case 'plugin:store|entries':
          return Object.entries(readState().store);
        case 'plugin:store|length':
          return Object.keys(readState().store).length;
        case 'plugin:store|reload':
        case 'plugin:store|save':
        case 'frontend_debug_log_cmd':
          return null;
        case 'backend_get_auth_session':
          return { success: false, data: null, message: 'No session' };
        case 'backend_get_profile':
          return { success: false, data: null, message: 'Profile unavailable in browser mock' };
        case 'backend_check_update':
          return { success: false, data: null, message: '未配置更新服务' };
        case 'backend_search_scripts':
          return {
            success: true,
            data: {
              records: [],
              total: 0,
              size: Number((args.req as { size?: number } | undefined)?.size ?? 12),
              current: Number((args.req as { page?: number } | undefined)?.page ?? 1),
            },
            message: null,
          };
        case 'get_log_config_cmd':
          return DEFAULT_LOG_CONFIG;
        case 'get_all_devices_cmd':
          return readState().devices;
        case 'cmd_get_running_devices':
          return [];
        case 'get_cpu_count_cmd':
          return 8;
        case 'get_assignments_by_device_cmd': {
          const state = readState();
          return state.assignmentsByDevice[String(args.deviceId)] ?? [];
        }
        case 'get_schedules_by_device_cmd': {
          const state = readState();
          return state.schedulesByDevice[String(args.deviceId)] ?? [];
        }
        case 'get_all_time_templates_cmd':
          return readState().timeTemplates;
        case 'get_all_scripts_cmd':
          return readState().scripts;
        case 'get_script_tasks_cmd': {
          const state = readState();
          return state.scriptTasks[String(args.scriptId)] ?? [];
        }
        case 'save_script_tasks_cmd':
          updateState((current) => ({
            ...current,
            scriptTasks: {
              ...current.scriptTasks,
              [String(args.scriptId)]: Array.isArray(args.tasks) ? (args.tasks as ScriptTaskTable[]) : [],
            },
          }));
          return null;
        case 'save_script_cmd':
          updateState((current) => ({
            ...current,
            scripts: upsertScript(current.scripts, args.script as StoredScriptTable),
          }));
          return null;
        case 'delete_script_cmd':
          updateState((current) => ({
            ...current,
            scripts: current.scripts.filter((script) => script.id !== args.scriptId),
            scriptTasks: Object.fromEntries(
              Object.entries(current.scriptTasks).filter(([scriptId]) => scriptId !== args.scriptId),
            ),
          }));
          return null;
        case 'clear_schedules_by_script_cmd':
          return null;
        case 'get_uuid_v7':
          return buildUuid();
        default:
          console.warn(`[Tauri Mock] unhandled command: ${cmd}`, payload);
          return null;
      }
    },
    { shouldMockEvents: true },
  );

  window.__AUTODAILY_MOCK__ = {
    getState: () => readState(),
    reset: () => writeState(createDefaultState()),
    seed: (partial) => seedState(partial),
  };

  if (!window.localStorage.getItem(MOCK_STATE_KEY)) {
    writeState(createDefaultState());
  }

  console.warn('Tauri environment not detected. Using browser mock backend.');
}
