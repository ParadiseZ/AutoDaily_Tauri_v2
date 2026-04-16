import { mockConvertFileSrc, mockIPC, mockWindows } from '@tauri-apps/api/mocks';
import type { LogConfig, ResumeCheckpointRecord, ScriptTimeTemplateValuesDto } from '@/types/app/domain';
import type { DeviceTable, PolicyGroupTable, PolicySetTable, PolicyTable, ScriptTable, ScriptTaskTable } from '@/types/bindings';

type StoreData = Record<string, unknown>;
type AssignmentMap = Record<string, unknown[]>;
type ScheduleMap = Record<string, unknown[]>;
type ScriptTaskMap = Record<string, ScriptTaskTable[]>;
type GroupPolicyMap = Record<string, string[]>;
type SetGroupMap = Record<string, string[]>;
type RuntimeProjectionMap = Record<string, unknown>;
type ScriptTemplateValueMap = Record<string, ScriptTimeTemplateValuesDto>;
type RecoveryCheckpointMap = Record<string, ResumeCheckpointRecord>;
type StoredDeviceTable = DeviceTable;
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
  policies: PolicyTable[];
  policyGroups: PolicyGroupTable[];
  policySets: PolicySetTable[];
  groupPolicies: GroupPolicyMap;
  setGroups: SetGroupMap;
  assignmentsByDevice: AssignmentMap;
  schedulesByDevice: ScheduleMap;
  runningDeviceIds: string[];
  runtimeProjections: RuntimeProjectionMap;
  scriptTemplateValues: ScriptTemplateValueMap;
  recoveryCheckpointsByDevice: RecoveryCheckpointMap;
  devices: StoredDeviceTable[];
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
  policies: [],
  policyGroups: [],
  policySets: [],
  groupPolicies: {},
  setGroups: {},
  assignmentsByDevice: {},
  schedulesByDevice: {},
  runningDeviceIds: [],
  runtimeProjections: {},
  scriptTemplateValues: {},
  recoveryCheckpointsByDevice: {},
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
        policies: parsed.policies ?? [],
        policyGroups: parsed.policyGroups ?? [],
        policySets: parsed.policySets ?? [],
        groupPolicies: parsed.groupPolicies ?? {},
        setGroups: parsed.setGroups ?? {},
        assignmentsByDevice: parsed.assignmentsByDevice ?? {},
        schedulesByDevice: parsed.schedulesByDevice ?? {},
        runningDeviceIds: parsed.runningDeviceIds ?? [],
        runtimeProjections: parsed.runtimeProjections ?? {},
        scriptTemplateValues: parsed.scriptTemplateValues ?? {},
        recoveryCheckpointsByDevice: parsed.recoveryCheckpointsByDevice ?? {},
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
      policies: partial.policies ?? current.policies,
      policyGroups: partial.policyGroups ?? current.policyGroups,
      policySets: partial.policySets ?? current.policySets,
      groupPolicies: partial.groupPolicies ?? current.groupPolicies,
      setGroups: partial.setGroups ?? current.setGroups,
      assignmentsByDevice: partial.assignmentsByDevice ?? current.assignmentsByDevice,
      schedulesByDevice: partial.schedulesByDevice ?? current.schedulesByDevice,
      runningDeviceIds: partial.runningDeviceIds ?? current.runningDeviceIds,
      runtimeProjections: partial.runtimeProjections ?? current.runtimeProjections,
      scriptTemplateValues: partial.scriptTemplateValues ?? current.scriptTemplateValues,
      recoveryCheckpointsByDevice:
        partial.recoveryCheckpointsByDevice ?? current.recoveryCheckpointsByDevice,
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

  const upsertById = <T extends { id: string }>(items: T[], nextItem: T) => {
    const next = [...items];
    const index = next.findIndex((item) => item.id === nextItem.id);
    if (index >= 0) {
      next[index] = nextItem;
    } else {
      next.push(nextItem);
    }
    return next;
  };

  const removeScriptFromState = (current: MockState, scriptId: string): MockState => {
    const removedGroupIds = new Set(
      current.policyGroups
        .filter((group) => group.scriptId === scriptId)
        .map((group) => group.id),
    );
    const removedSetIds = new Set(
      current.policySets
        .filter((set) => set.scriptId === scriptId)
        .map((set) => set.id),
    );

    return {
      ...current,
      scripts: current.scripts.filter((script) => script.id !== scriptId),
      scriptTasks: Object.fromEntries(
        Object.entries(current.scriptTasks).filter(([currentScriptId]) => currentScriptId !== scriptId),
      ),
      policies: current.policies.filter((policy) => policy.scriptId !== scriptId),
      policyGroups: current.policyGroups.filter((group) => group.scriptId !== scriptId),
      policySets: current.policySets.filter((set) => set.scriptId !== scriptId),
      groupPolicies: Object.fromEntries(
        Object.entries(current.groupPolicies).filter(([groupId]) => !removedGroupIds.has(groupId)),
      ),
      setGroups: Object.fromEntries(
        Object.entries(current.setGroups)
          .filter(([setId]) => !removedSetIds.has(setId))
          .map(([setId, groupIds]) => [
            setId,
            groupIds.filter((groupId) => !removedGroupIds.has(groupId)),
          ]),
      ),
      assignmentsByDevice: Object.fromEntries(
        Object.entries(current.assignmentsByDevice).map(([deviceId, assignments]) => [
          deviceId,
          assignments.filter((assignment) => (assignment as { scriptId?: unknown }).scriptId !== scriptId),
        ]),
      ),
      schedulesByDevice: Object.fromEntries(
        Object.entries(current.schedulesByDevice).map(([deviceId, items]) => [
          deviceId,
          items.filter((item) => (item as { scriptId?: unknown }).scriptId !== scriptId),
        ]),
      ),
      scriptTemplateValues: Object.fromEntries(
        Object.entries(current.scriptTemplateValues).filter(([, record]) => record.scriptId !== scriptId),
      ),
      recoveryCheckpointsByDevice: Object.fromEntries(
        Object.entries(current.recoveryCheckpointsByDevice).filter(([, checkpoint]) => checkpoint.scriptId !== scriptId),
      ),
    };
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

  const buildTemplateValueScopeKey = (
    deviceId: string | null | undefined,
    scriptId: string | null | undefined,
    timeTemplateId: string | null | undefined,
    accountId: string | null | undefined,
  ) => [deviceId ?? '', scriptId ?? '', timeTemplateId ?? '', accountId ?? ''].join('::');

  const findDevice = (state: MockState, deviceId: string) =>
    state.devices.find((device) => device.id === deviceId) ?? null;

  const findScript = (state: MockState, scriptId: string) =>
    state.scripts.find((script) => script.id === scriptId) ?? null;

  const isRunnableRecoveryTask = (state: MockState, scriptId: string, taskId: string) =>
    (state.scriptTasks[scriptId] ?? []).some(
      (task) => task.id === taskId && task.rowType === 'task' && !task.isDeleted,
    );

  const normalizeOptionalText = (value: unknown) =>
    typeof value === 'string' && value.trim().length > 0 ? value.trim() : null;

  const validateRecoveryPolicyForScript = (state: MockState, scriptId: string) => {
    const script = findScript(state, scriptId);
    if (!script) {
      return;
    }

    const recoveryTaskId = script.data.runtimeSettings?.recoveryTaskId ?? null;
    if (!recoveryTaskId) {
      throw new Error(`脚本「${script.data.name}」未配置恢复任务，无法使用 RunRecoveryTask 策略`);
    }

    if (!isRunnableRecoveryTask(state, scriptId, recoveryTaskId)) {
      throw new Error(`脚本「${script.data.name}」配置的恢复任务不存在，或不是可执行 Task`);
    }
  };

  const validateRestartAppPolicyForScript = (state: MockState, scriptId: string) => {
    const script = findScript(state, scriptId);
    if (!script) {
      return;
    }

    const pkgName = normalizeOptionalText(script.data.pkgName);
    const activityName = normalizeOptionalText((script.data as { activityName?: unknown }).activityName);
    if (!pkgName || !activityName) {
      throw new Error(`脚本「${script.data.name}」未配置全局包名或 Activity，无法使用 RestartApp 策略`);
    }
  };

  const validateTimeoutPolicyForRun = (state: MockState, deviceId: string, target?: unknown) => {
    const device = findDevice(state, deviceId);
    if (!device) {
      return;
    }

    const timeoutAction = device.data.executionPolicy?.timeoutAction;
    if (timeoutAction !== 'runRecoveryTask' && timeoutAction !== 'restartApp') {
      return;
    }

    const targetRecord = target && typeof target === 'object' ? (target as Record<string, unknown>) : null;
    const runType = typeof targetRecord?.type === 'string' ? targetRecord.type : 'deviceQueue';

    if (runType === 'deviceQueue') {
      for (const assignment of state.assignmentsByDevice[deviceId] ?? []) {
        const scriptId = typeof (assignment as { scriptId?: unknown }).scriptId === 'string'
          ? String((assignment as { scriptId?: unknown }).scriptId)
          : null;
        if (scriptId) {
          if (timeoutAction === 'runRecoveryTask') {
            validateRecoveryPolicyForScript(state, scriptId);
          } else {
            validateRestartAppPolicyForScript(state, scriptId);
          }
        }
      }
      return;
    }

    const scriptId = typeof targetRecord?.scriptId === 'string' ? targetRecord.scriptId : null;
    if (scriptId) {
      if (timeoutAction === 'runRecoveryTask') {
        validateRecoveryPolicyForScript(state, scriptId);
      } else {
        validateRestartAppPolicyForScript(state, scriptId);
      }
    }
  };

  const validateRuntimePlatformSupported = (state: MockState, deviceId: string) => {
    const device = findDevice(state, deviceId);
    if (!device) {
      return;
    }

    if (normalizePlatform(device.data.platform) === 'desktop') {
      throw new Error(`设备[${device.data.deviceName}]当前为 desktop 平台，但本版本尚未实现 Desktop 运行时适配器`);
    }
  };

  const normalizePlatform = (value: unknown) => (value === 'desktop' ? 'desktop' : 'android');

  const validateAssignmentPlatform = (state: MockState, deviceId: string, scriptId: string) => {
    const device = findDevice(state, deviceId);
    if (!device) {
      throw new Error(`设备不存在: ${deviceId}`);
    }

    const script = findScript(state, scriptId);
    if (!script) {
      throw new Error(`脚本不存在: ${scriptId}`);
    }

    const devicePlatform = normalizePlatform(device.data.platform);
    const scriptPlatform = normalizePlatform(script.data.platform);
    if (devicePlatform !== scriptPlatform) {
      throw new Error(`脚本平台不匹配，设备平台=${devicePlatform}，脚本平台=${scriptPlatform}`);
    }
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
        case 'get_vision_text_cache_config_cmd':
          return {
            enabled: false,
            dir: '',
            signatureGridSize: 8,
          };
        case 'get_all_devices_cmd':
          return readState().devices;
        case 'cmd_get_running_devices':
          return readState().runningDeviceIds;
        case 'cmd_is_device_running':
          return readState().runningDeviceIds.includes(String(args.deviceId));
        case 'cmd_spawn_device':
          validateRuntimePlatformSupported(readState(), String(args.deviceId));
          updateState((current) => ({
            ...current,
            runningDeviceIds: current.runningDeviceIds.includes(String(args.deviceId))
              ? current.runningDeviceIds
              : [...current.runningDeviceIds, String(args.deviceId)],
          }));
          return `设备[${String(args.deviceId)}]子进程已启动`;
        case 'cmd_device_shutdown':
          updateState((current) => ({
            ...current,
            runningDeviceIds: current.runningDeviceIds.filter((deviceId) => deviceId !== String(args.deviceId)),
          }));
          return `设备[${String(args.deviceId)}]子进程已关闭`;
        case 'cmd_restart_device_runtime':
          validateRuntimePlatformSupported(readState(), String(args.deviceId));
          validateTimeoutPolicyForRun(readState(), String(args.deviceId));
          updateState((current) => ({
            ...current,
            runningDeviceIds: current.runningDeviceIds.includes(String(args.deviceId))
              ? current.runningDeviceIds
              : [...current.runningDeviceIds, String(args.deviceId)],
          }));
          return `设备[${String(args.deviceId)}]子进程已按 checkpoint 流程重启并重新装填 session`;
        case 'cmd_device_start':
          validateRuntimePlatformSupported(readState(), String(args.deviceId));
          validateTimeoutPolicyForRun(readState(), String(args.deviceId));
          return `已向设备[${String(args.deviceId)}]发送启动命令`;
        case 'cmd_device_pause':
          return `已向设备[${String(args.deviceId)}]发送暂停命令`;
        case 'cmd_device_stop':
          return `已向设备[${String(args.deviceId)}]发送停止命令`;
        case 'cmd_sync_device_runtime_session':
          validateRuntimePlatformSupported(readState(), String(args.deviceId));
          validateTimeoutPolicyForRun(readState(), String(args.deviceId));
          return `已同步设备[${String(args.deviceId)}]运行会话`;
        case 'cmd_run_script_target':
          validateRuntimePlatformSupported(readState(), String(args.deviceId));
          validateTimeoutPolicyForRun(readState(), String(args.deviceId), args.target);
          return `已向设备[${String(args.deviceId)}]发送运行目标`;
        case 'get_cpu_count_cmd':
          return 8;
        case 'get_yolo_labels_cmd':
          return {
            0: '文本',
            1: '按钮',
            2: '图标',
            3: '高亮',
          };
        case 'get_assignments_by_device_cmd': {
          const state = readState();
          return state.assignmentsByDevice[String(args.deviceId)] ?? [];
        }
        case 'save_assignment_cmd':
          updateState((current) => {
            const assignment = args.assignment as {
              id: string;
              deviceId: string;
              scriptId: string;
              index: number;
            };
            validateAssignmentPlatform(current, assignment.deviceId, assignment.scriptId);
            const deviceId = String(assignment.deviceId);
            const currentAssignments = [...(current.assignmentsByDevice[deviceId] ?? [])];
            const nextAssignments = [
              ...currentAssignments.filter((item) => (item as { id?: unknown }).id !== assignment.id),
              assignment,
            ].sort(
              (left, right) =>
                Number((left as { index?: number }).index ?? 0) - Number((right as { index?: number }).index ?? 0),
            );
            return {
              ...current,
              assignmentsByDevice: {
                ...current.assignmentsByDevice,
                [deviceId]: nextAssignments,
              },
            };
          });
          return null;
        case 'delete_assignment_cmd':
          updateState((current) => ({
            ...current,
            assignmentsByDevice: Object.fromEntries(
              Object.entries(current.assignmentsByDevice).map(([deviceId, assignments]) => [
                deviceId,
                assignments.filter((assignment) => (assignment as { id?: unknown }).id !== args.assignmentId),
              ]),
            ),
          }));
          return null;
        case 'reorder_assignments_cmd':
          updateState((current) => {
            const deviceId = String(args.deviceId);
            const assignmentIds = Array.isArray(args.assignmentIds) ? (args.assignmentIds as string[]) : [];
            const currentAssignments = [...(current.assignmentsByDevice[deviceId] ?? [])];
            const assignmentMap = new Map(
              currentAssignments.map((assignment) => [String((assignment as { id?: unknown }).id ?? ''), assignment]),
            );
            const reordered = assignmentIds
              .map((id, index) => {
                const assignment = assignmentMap.get(id);
                if (!assignment) {
                  return null;
                }
                return {
                  ...assignment,
                  index,
                };
              })
              .filter(Boolean);
            return {
              ...current,
              assignmentsByDevice: {
                ...current.assignmentsByDevice,
                [deviceId]: reordered,
              },
            };
          });
          return null;
        case 'get_schedules_by_device_cmd': {
          const state = readState();
          return state.schedulesByDevice[String(args.deviceId)] ?? [];
        }
        case 'get_recovery_checkpoint_by_device_cmd': {
          const state = readState();
          return state.recoveryCheckpointsByDevice[String(args.deviceId)] ?? null;
        }
        case 'get_all_time_templates_cmd':
          return readState().timeTemplates;
        case 'get_script_time_template_values_cmd': {
          const state = readState();
          return state.scriptTemplateValues[
            buildTemplateValueScopeKey(
              String(args.deviceId),
              String(args.scriptId),
              String(args.timeTemplateId),
              typeof args.accountId === 'string' ? args.accountId : null,
            )
          ] ?? null;
        }
        case 'save_script_time_template_values_cmd':
          updateState((current) => {
            const record = args.record as ScriptTimeTemplateValuesDto;
            return {
              ...current,
              scriptTemplateValues: {
                ...current.scriptTemplateValues,
                [buildTemplateValueScopeKey(record.deviceId, record.scriptId, record.timeTemplateId, record.accountId ?? null)]: record,
              },
            };
          });
          return null;
        case 'delete_script_time_template_values_cmd':
          updateState((current) => {
            const next = { ...current.scriptTemplateValues };
            delete next[
              buildTemplateValueScopeKey(
                String(args.deviceId),
                String(args.scriptId),
                String(args.timeTemplateId),
                typeof args.accountId === 'string' ? args.accountId : null,
              )
            ];
            return {
              ...current,
              scriptTemplateValues: next,
            };
          });
          return null;
        case 'get_all_scripts_cmd':
          return readState().scripts;
        case 'save_device_cmd':
          updateState((current) => ({
            ...current,
            devices: upsertById(current.devices, args.device as StoredDeviceTable),
          }));
          return null;
        case 'delete_device_cmd':
          updateState((current) => ({
            ...current,
            devices: current.devices.filter((device) => device.id !== args.deviceId),
            assignmentsByDevice: Object.fromEntries(
              Object.entries(current.assignmentsByDevice).filter(([deviceId]) => deviceId !== args.deviceId),
            ),
            schedulesByDevice: Object.fromEntries(
              Object.entries(current.schedulesByDevice).filter(([deviceId]) => deviceId !== args.deviceId),
            ),
            recoveryCheckpointsByDevice: Object.fromEntries(
              Object.entries(current.recoveryCheckpointsByDevice).filter(([deviceId]) => deviceId !== args.deviceId),
            ),
            runtimeProjections: Object.fromEntries(
              Object.entries(current.runtimeProjections).filter(([deviceId]) => deviceId !== args.deviceId),
            ),
            runningDeviceIds: current.runningDeviceIds.filter((deviceId) => deviceId !== args.deviceId),
          }));
          return null;
        case 'get_script_tasks_cmd': {
          const state = readState();
          return state.scriptTasks[String(args.scriptId)] ?? [];
        }
        case 'get_all_policies_cmd': {
          const state = readState();
          return state.policies
            .filter((policy) => policy.scriptId === String(args.scriptId))
            .sort((left, right) => left.orderIndex - right.orderIndex);
        }
        case 'save_policy_cmd':
          updateState((current) => ({
            ...current,
            policies: upsertById(current.policies, args.policy as PolicyTable).sort((left, right) => left.orderIndex - right.orderIndex),
          }));
          return null;
        case 'delete_policy_cmd':
          updateState((current) => ({
            ...current,
            policies: current.policies.filter((policy) => policy.id !== args.id),
            groupPolicies: Object.fromEntries(
              Object.entries(current.groupPolicies).map(([groupId, policyIds]) => [groupId, policyIds.filter((policyId) => policyId !== args.id)]),
            ),
          }));
          return null;
        case 'get_all_policy_groups_cmd': {
          const state = readState();
          return state.policyGroups
            .filter((group) => group.scriptId === String(args.scriptId))
            .sort((left, right) => left.orderIndex - right.orderIndex);
        }
        case 'save_policy_group_cmd':
          updateState((current) => ({
            ...current,
            policyGroups: upsertById(current.policyGroups, args.group as PolicyGroupTable).sort((left, right) => left.orderIndex - right.orderIndex),
          }));
          return null;
        case 'delete_policy_group_cmd':
          updateState((current) => {
            const nextGroupPolicies = { ...current.groupPolicies };
            delete nextGroupPolicies[String(args.id)];
            return {
              ...current,
              policyGroups: current.policyGroups.filter((group) => group.id !== args.id),
              groupPolicies: nextGroupPolicies,
              setGroups: Object.fromEntries(
                Object.entries(current.setGroups).map(([setId, groupIds]) => [setId, groupIds.filter((groupId) => groupId !== args.id)]),
              ),
            };
          });
          return null;
        case 'get_group_policies_cmd': {
          const state = readState();
          return state.groupPolicies[String(args.groupId)] ?? [];
        }
        case 'update_group_policies_cmd':
          updateState((current) => ({
            ...current,
            groupPolicies: {
              ...current.groupPolicies,
              [String(args.groupId)]: Array.isArray(args.policyIds) ? (args.policyIds as string[]) : [],
            },
          }));
          return null;
        case 'get_all_policy_sets_cmd': {
          const state = readState();
          return state.policySets
            .filter((set) => set.scriptId === String(args.scriptId))
            .sort((left, right) => left.orderIndex - right.orderIndex);
        }
        case 'save_policy_set_cmd':
          updateState((current) => ({
            ...current,
            policySets: upsertById(current.policySets, args.set as PolicySetTable).sort((left, right) => left.orderIndex - right.orderIndex),
          }));
          return null;
        case 'delete_policy_set_cmd':
          updateState((current) => {
            const nextSetGroups = { ...current.setGroups };
            delete nextSetGroups[String(args.id)];
            return {
              ...current,
              policySets: current.policySets.filter((set) => set.id !== args.id),
              setGroups: nextSetGroups,
            };
          });
          return null;
        case 'get_set_groups_cmd': {
          const state = readState();
          return state.setGroups[String(args.setId)] ?? [];
        }
        case 'update_set_groups_cmd':
          updateState((current) => ({
            ...current,
            setGroups: {
              ...current.setGroups,
              [String(args.setId)]: Array.isArray(args.groupIds) ? (args.groupIds as string[]) : [],
            },
          }));
          return null;
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
          updateState((current) => removeScriptFromState(current, String(args.scriptId)));
          return null;
        case 'clone_local_script_cmd':
          {
            let clonedScriptId = '';
          updateState((current) => {
            const sourceScriptId = String(args.sourceScriptId);
            const overwriteCloudId = Boolean(args.overwriteCloudId);
            const sourceScript = current.scripts.find((script) => script.id === sourceScriptId);
            if (!sourceScript) {
              return current;
            }

            let nextState = current;
            const sourceCloudId = sourceScript.data.cloudId ?? sourceScript.id;
            const isPublished = sourceScript.data.scriptType === 'published';

            if (overwriteCloudId && isPublished) {
              const existingDev = nextState.scripts.find(
                (script) =>
                  script.id !== sourceScriptId &&
                  script.data.scriptType === 'dev' &&
                  script.data.cloudId === sourceCloudId,
              );
              if (existingDev) {
                nextState = removeScriptFromState(nextState, existingDev.id);
              }
            }

            const newScriptId = buildUuid();
            clonedScriptId = newScriptId;
            const policyIdMap = new Map<string, string>();
            const groupIdMap = new Map<string, string>();
            const setIdMap = new Map<string, string>();

            const nextScript: StoredScriptTable = {
              ...sourceScript,
              id: newScriptId,
              data: {
                ...sourceScript.data,
                name: `${sourceScript.data.name} (Clone)`,
                scriptType: 'dev',
                cloudId: overwriteCloudId && isPublished ? sourceCloudId : null,
                userId:
                  typeof args.currentUserId === 'string' && args.currentUserId.trim().length > 0
                    ? args.currentUserId
                    : sourceScript.data.userId,
              },
            };

            const sourcePolicies = nextState.policies.filter((policy) => policy.scriptId === sourceScriptId);
            const sourceGroups = nextState.policyGroups.filter((group) => group.scriptId === sourceScriptId);
            const sourceSets = nextState.policySets.filter((set) => set.scriptId === sourceScriptId);
            const sourceTasks = nextState.scriptTasks[sourceScriptId] ?? [];

            const clonedPolicies = sourcePolicies.map((policy) => {
              const nextId = buildUuid();
              policyIdMap.set(policy.id, nextId);
              return { ...policy, id: nextId, scriptId: newScriptId };
            });
            const clonedGroups = sourceGroups.map((group) => {
              const nextId = buildUuid();
              groupIdMap.set(group.id, nextId);
              return { ...group, id: nextId, scriptId: newScriptId };
            });
            const clonedSets = sourceSets.map((set) => {
              const nextId = buildUuid();
              setIdMap.set(set.id, nextId);
              return { ...set, id: nextId, scriptId: newScriptId };
            });
            const clonedTasks = sourceTasks.map((task) => ({
              ...task,
              id: buildUuid(),
              scriptId: newScriptId,
            }));

            const clonedGroupPolicies = Object.fromEntries(
              sourceGroups.map((group) => [
                groupIdMap.get(group.id) ?? group.id,
                (nextState.groupPolicies[group.id] ?? []).map((policyId) => policyIdMap.get(policyId) ?? policyId),
              ]),
            );
            const clonedSetGroups = Object.fromEntries(
              sourceSets.map((set) => [
                setIdMap.get(set.id) ?? set.id,
                (nextState.setGroups[set.id] ?? []).map((groupId) => groupIdMap.get(groupId) ?? groupId),
              ]),
            );

            return {
              ...nextState,
              scripts: upsertScript(nextState.scripts, nextScript),
              scriptTasks: {
                ...nextState.scriptTasks,
                [newScriptId]: clonedTasks,
              },
              policies: [...nextState.policies, ...clonedPolicies],
              policyGroups: [...nextState.policyGroups, ...clonedGroups],
              policySets: [...nextState.policySets, ...clonedSets],
              groupPolicies: {
                ...nextState.groupPolicies,
                ...clonedGroupPolicies,
              },
              setGroups: {
                ...nextState.setGroups,
                ...clonedSetGroups,
              },
            };
          });
          return {
            success: true,
            data: clonedScriptId,
            message: '复制成功',
          };
          }
        case 'clear_schedules_cmd':
          updateState((current) => ({
            ...current,
            schedulesByDevice: {
              ...current.schedulesByDevice,
              [String(args.deviceId)]: [],
            },
            recoveryCheckpointsByDevice: Object.fromEntries(
              Object.entries(current.recoveryCheckpointsByDevice).filter(([deviceId]) => deviceId !== args.deviceId),
            ),
          }));
          return null;
        case 'clear_schedules_by_script_cmd':
          updateState((current) => ({
            ...current,
            schedulesByDevice: Object.fromEntries(
              Object.entries(current.schedulesByDevice).map(([deviceId, items]) => [
                deviceId,
                items.filter((item) => (item as { scriptId?: unknown }).scriptId !== args.scriptId),
              ]),
            ),
            recoveryCheckpointsByDevice: Object.fromEntries(
              Object.entries(current.recoveryCheckpointsByDevice).filter(([, checkpoint]) => checkpoint.scriptId !== args.scriptId),
            ),
          }));
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
