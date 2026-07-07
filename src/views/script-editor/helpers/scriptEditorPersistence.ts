import type { ScriptTableRecord } from '@/types/app/domain';
import type { PolicyGroupTable } from '@/types/bindings/PolicyGroupTable';
import type { PolicySetTable } from '@/types/bindings/PolicySetTable';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import { buildUiData, parseUiSchema } from '@/views/script-editor/editorSchema';
import { parseInputEntries, syncInputVariableCatalog } from '@/views/script-editor/editorVariables';

export interface ScriptEditorSnapshots {
  script: string;
  tasks: string;
  policies: string;
  policyGroups: string;
  policySets: string;
  groupPolicies: string;
  setGroups: string;
}

type StableStringify = (value: unknown) => string;

const normalizeScriptForSnapshot = (script: ScriptTableRecord | null): ScriptTableRecord | null => {
  if (!script) {
    return script;
  }

  return {
    ...script,
    data: {
      ...script.data,
      // Dirty 比较按变量编辑器会写回的目录结构归一化，避免键/作用域回改后仍因目录排序或旧字段形态不同而保持脏状态。
      variableCatalog: syncInputVariableCatalog(
        script.data.variableCatalog,
        null,
        parseInputEntries(script.data.variableCatalog, null, {}),
      ),
    },
  };
};

const normalizeTaskForSnapshot = (task: ScriptTaskTable): ScriptTaskTable => ({
  ...task,
  data: {
    ...task.data,
    // Dirty 比较按编辑器会写回的 UI 结构归一化，避免旧 options 对象数组和新字符串数组语义相同却比较不等。
    uiData: buildUiData(parseUiSchema(task.data?.uiData ?? {})),
  },
});

const stringifyTasks = (tasks: ScriptTaskTable[], stableStringify: StableStringify) =>
  stableStringify(tasks.map(normalizeTaskForSnapshot));

export const buildScriptEditorSnapshots = ({
  script,
  tasks,
  policies,
  policyGroups,
  policySets,
  groupPolicyIdsByGroupId,
  setGroupIdsBySetId,
  stableStringify,
}: {
  script: ScriptTableRecord | null;
  tasks: ScriptTaskTable[];
  policies: PolicyTable[];
  policyGroups: PolicyGroupTable[];
  policySets: PolicySetTable[];
  groupPolicyIdsByGroupId: Record<string, string[]>;
  setGroupIdsBySetId: Record<string, string[]>;
  stableStringify: StableStringify;
}): ScriptEditorSnapshots => ({
  script: stableStringify(normalizeScriptForSnapshot(script)),
  tasks: stringifyTasks(tasks, stableStringify),
  policies: stableStringify(policies),
  policyGroups: stableStringify(policyGroups),
  policySets: stableStringify(policySets),
  groupPolicies: stableStringify(groupPolicyIdsByGroupId),
  setGroups: stableStringify(setGroupIdsBySetId),
});

export const hasDirtyScriptEditorState = ({
  script,
  tasks,
  policies,
  policyGroups,
  policySets,
  groupPolicyIdsByGroupId,
  setGroupIdsBySetId,
  snapshots,
  stableStringify,
}: {
  script: ScriptTableRecord | null;
  tasks: ScriptTaskTable[];
  policies: PolicyTable[];
  policyGroups: PolicyGroupTable[];
  policySets: PolicySetTable[];
  groupPolicyIdsByGroupId: Record<string, string[]>;
  setGroupIdsBySetId: Record<string, string[]>;
  snapshots: ScriptEditorSnapshots;
  stableStringify: StableStringify;
}) =>
  stableStringify(normalizeScriptForSnapshot(script)) !== snapshots.script
  || stringifyTasks(tasks, stableStringify) !== snapshots.tasks
  || stableStringify(policies) !== snapshots.policies
  || stableStringify(policyGroups) !== snapshots.policyGroups
  || stableStringify(policySets) !== snapshots.policySets
  || stableStringify(groupPolicyIdsByGroupId) !== snapshots.groupPolicies
  || stableStringify(setGroupIdsBySetId) !== snapshots.setGroups;

export const collectSnapshotIds = <T extends { id: string }>(snapshot: string) =>
  new Set(((JSON.parse(snapshot || '[]') as T[]) ?? []).map((item) => item.id));

export const loadScriptEditorData = async ({
  sourceScript,
  loadScriptTasks,
  listPolicies,
  listPolicyGroups,
  listPolicySets,
  getGroupPolicies,
  getSetGroups,
  normalizeTask,
  buildTaskDraft,
  normalizePolicy,
  normalizePolicyGroup,
  normalizePolicySet,
  cloneScript,
  stableStringify,
}: {
  sourceScript: ScriptTableRecord;
  loadScriptTasks: (scriptId: string) => Promise<ScriptTaskTable[]>;
  listPolicies: (scriptId: string) => Promise<PolicyTable[]>;
  listPolicyGroups: (scriptId: string) => Promise<PolicyGroupTable[]>;
  listPolicySets: (scriptId: string) => Promise<PolicySetTable[]>;
  getGroupPolicies: (groupId: string) => Promise<string[]>;
  getSetGroups: (setId: string) => Promise<string[]>;
  normalizeTask: (task: ScriptTaskTable, index: number) => ScriptTaskTable;
  buildTaskDraft: (name?: string) => Promise<ScriptTaskTable>;
  normalizePolicy: (policy: PolicyTable, index: number) => PolicyTable;
  normalizePolicyGroup: (group: PolicyGroupTable, index: number) => PolicyGroupTable;
  normalizePolicySet: (set: PolicySetTable, index: number) => PolicySetTable;
  cloneScript: (script: ScriptTableRecord) => ScriptTableRecord;
  stableStringify: StableStringify;
}) => {
  const draftScript = cloneScript(sourceScript);
  const [loadedTasks, loadedPolicies, loadedPolicyGroups, loadedPolicySets] = await Promise.all([
    loadScriptTasks(sourceScript.id),
    listPolicies(sourceScript.id),
    listPolicyGroups(sourceScript.id),
    listPolicySets(sourceScript.id),
  ]);

  const draftTasks = loadedTasks.length
    ? loadedTasks.map((task, index) => normalizeTask(task, index))
    : [await buildTaskDraft('主任务 1')];
  const draftPolicies = loadedPolicies.map((policy, index) => normalizePolicy(policy, index));
  const draftPolicyGroups = loadedPolicyGroups.map((group, index) => normalizePolicyGroup(group, index));
  const draftPolicySets = loadedPolicySets.map((set, index) => normalizePolicySet(set, index));

  const [groupRelations, setRelations] = await Promise.all([
    Promise.all(draftPolicyGroups.map(async (group) => [group.id, await getGroupPolicies(group.id)] as const)),
    Promise.all(draftPolicySets.map(async (set) => [set.id, await getSetGroups(set.id)] as const)),
  ]);

  const groupPolicyIdsByGroupId = Object.fromEntries(groupRelations);
  const setGroupIdsBySetId = Object.fromEntries(setRelations);
  const snapshots = buildScriptEditorSnapshots({
    script: draftScript,
    tasks: loadedTasks.length ? draftTasks : [],
    policies: draftPolicies,
    policyGroups: draftPolicyGroups,
    policySets: draftPolicySets,
    groupPolicyIdsByGroupId,
    setGroupIdsBySetId,
    stableStringify,
  });

  return {
    draftScript,
    draftTasks,
    draftPolicies,
    draftPolicyGroups,
    draftPolicySets,
    groupPolicyIdsByGroupId,
    setGroupIdsBySetId,
    snapshots,
    saveTime: sourceScript.data.updateTime || null,
  };
};

export const savePrimaryScriptEditorData = async ({
  script,
  tasks,
  policies,
  policyGroups,
  policySets,
  groupPolicyIdsByGroupId,
  setGroupIdsBySetId,
  saveScriptEditorBundle,
}: {
  script: ScriptTableRecord;
  tasks: ScriptTaskTable[];
  policies: PolicyTable[];
  policyGroups: PolicyGroupTable[];
  policySets: PolicySetTable[];
  groupPolicyIdsByGroupId: Record<string, string[]>;
  setGroupIdsBySetId: Record<string, string[]>;
  saveScriptEditorBundle: (payload: {
    script: ScriptTableRecord;
    tasks: ScriptTaskTable[];
    policies: PolicyTable[];
    policyGroups: PolicyGroupTable[];
    policySets: PolicySetTable[];
    groupPolicyIdsByGroupId: Record<string, string[]>;
    setGroupIdsBySetId: Record<string, string[]>;
  }) => Promise<void>;
}) => {
  await saveScriptEditorBundle({
    script,
    tasks,
    policies,
    policyGroups,
    policySets,
    groupPolicyIdsByGroupId,
    setGroupIdsBySetId,
  });
  return script;
};

export const removeDeletedScriptEditorData = async ({
  sourcePolicySnapshot,
  sourcePolicyGroupSnapshot,
  sourcePolicySetSnapshot,
  policies,
  policyGroups,
  policySets,
  removePolicy,
  removePolicyGroup,
  removePolicySet,
}: {
  sourcePolicySnapshot: string;
  sourcePolicyGroupSnapshot: string;
  sourcePolicySetSnapshot: string;
  policies: PolicyTable[];
  policyGroups: PolicyGroupTable[];
  policySets: PolicySetTable[];
  removePolicy: (id: string) => Promise<void>;
  removePolicyGroup: (id: string) => Promise<void>;
  removePolicySet: (id: string) => Promise<void>;
}) => {
  const sourcePolicyIds = collectSnapshotIds<PolicyTable>(sourcePolicySnapshot);
  const sourcePolicyGroupIds = collectSnapshotIds<PolicyGroupTable>(sourcePolicyGroupSnapshot);
  const sourcePolicySetIds = collectSnapshotIds<PolicySetTable>(sourcePolicySetSnapshot);
  const nextPolicyIds = new Set(policies.map((item) => item.id));
  const nextPolicyGroupIds = new Set(policyGroups.map((item) => item.id));
  const nextPolicySetIds = new Set(policySets.map((item) => item.id));

  await Promise.all([
    ...Array.from(sourcePolicyIds).filter((id) => !nextPolicyIds.has(id)).map((id) => removePolicy(id)),
    ...Array.from(sourcePolicyGroupIds).filter((id) => !nextPolicyGroupIds.has(id)).map((id) => removePolicyGroup(id)),
    ...Array.from(sourcePolicySetIds).filter((id) => !nextPolicySetIds.has(id)).map((id) => removePolicySet(id)),
  ]);
};

export const updateScriptEditorRelations = async ({
  policyGroups,
  policySets,
  groupPolicyIdsByGroupId,
  setGroupIdsBySetId,
  updateGroupPolicies,
  updateSetGroups,
}: {
  policyGroups: PolicyGroupTable[];
  policySets: PolicySetTable[];
  groupPolicyIdsByGroupId: Record<string, string[]>;
  setGroupIdsBySetId: Record<string, string[]>;
  updateGroupPolicies: (groupId: string, policyIds: string[]) => Promise<void>;
  updateSetGroups: (setId: string, groupIds: string[]) => Promise<void>;
}) => {
  await Promise.all([
    ...policyGroups.map((group) => updateGroupPolicies(group.id, groupPolicyIdsByGroupId[group.id] ?? [])),
    ...policySets.map((set) => updateSetGroups(set.id, setGroupIdsBySetId[set.id] ?? [])),
  ]);
};
