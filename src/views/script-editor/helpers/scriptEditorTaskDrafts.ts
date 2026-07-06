import type { PolicyGroupTable } from '@/types/bindings/PolicyGroupTable';
import type { PolicySetTable } from '@/types/bindings/PolicySetTable';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import { createStepList, TASK_CYCLE_VALUE, TASK_ROW_TYPE, TASK_TONE, TASK_TRIGGER_MODE } from '@/views/script-editor/editor-step/editorStepKinds';
import { createSearchRule } from '@/views/script-editor/editorSearchRule';
import { cloneJson } from '@/views/script-editor/editorSchema';
import { normalizePolicy, normalizePolicyGroup, normalizePolicySet } from '@/views/script-editor/editor-policy/editorPolicy';

const findNextOrdinalName = (names: string[], prefix: string) => {
  const used = new Set(
    names
      .map((name) => name.match(new RegExp(`^${prefix} (\\d+)$`)))
      .map((match) => (match ? Number(match[1]) : null))
      .filter((value): value is number => value !== null && Number.isInteger(value) && value > 0),
  );
  let next = 1;
  while (used.has(next)) {
    next += 1;
  }
  return `${prefix} ${next}`;
};

export const normalizeTask = (task: ScriptTaskTable, index: number, scriptId: string): ScriptTaskTable => {
  const rowType = task.rowType ?? TASK_ROW_TYPE.task;
  const isTitle = rowType === TASK_ROW_TYPE.title;
  return {
    ...task,
    scriptId: task.scriptId || scriptId,
    name: task.name ?? `任务 ${index + 1}`,
    description: task.description ?? '',
    rowType,
    triggerMode: task.triggerMode ?? TASK_TRIGGER_MODE.linkOnly,
    recordSchedule: isTitle ? false : task.recordSchedule ?? true,
    sectionId: isTitle ? null : task.sectionId ?? null,
    indentLevel: isTitle ? 0 : Math.max(0, Math.min(8, Number(task.indentLevel ?? 1))),
    defaultTaskCycle: task.defaultTaskCycle ?? TASK_CYCLE_VALUE.everyRun,
    execMax: isTitle ? 0 : Math.max(0, Number(task.execMax ?? 0)),
    showEnabledToggle: isTitle ? false : task.showEnabledToggle ?? true,
    defaultEnabled: task.defaultEnabled ?? true,
    taskTone: isTitle ? TASK_TONE.normal : task.taskTone ?? TASK_TONE.normal,
    isHidden: Boolean(task.isHidden),
    index,
    createdAt: task.createdAt || new Date().toISOString(),
    updatedAt: task.updatedAt || new Date().toISOString(),
    deletedAt: task.deletedAt ?? null,
    isDeleted: Boolean(task.isDeleted),
    data: {
      uiData: task.data?.uiData ?? {},
      variables: task.data?.variables ?? {},
      steps: Array.isArray(task.data?.steps) ? task.data.steps : createStepList(),
    },
  };
};

export const buildTaskDraft = async ({
  name,
  requestUuid,
  scriptId,
  tasks,
}: {
  name?: string;
  requestUuid: () => Promise<string>;
  scriptId: string;
  tasks: ScriptTaskTable[];
}): Promise<ScriptTaskTable> => {
  const index = tasks.length;
  return normalizeTask(
    {
      id: await requestUuid(),
      scriptId,
      name: name || findNextOrdinalName(tasks.map((task) => task.name || ''), '新任务'),
      description: '',
      rowType: TASK_ROW_TYPE.task,
      triggerMode: TASK_TRIGGER_MODE.linkOnly,
      recordSchedule: true,
      sectionId: tasks.filter((task) => task.rowType === TASK_ROW_TYPE.title).at(-1)?.id ?? null,
      indentLevel: 1,
      defaultTaskCycle: TASK_CYCLE_VALUE.everyRun,
      execMax: 0,
      showEnabledToggle: true,
      defaultEnabled: true,
      taskTone: TASK_TONE.normal,
      isHidden: false,
      data: {
        uiData: {},
        variables: {},
        steps: createStepList(),
      },
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      deletedAt: null,
      isDeleted: false,
      index,
    },
    index,
    scriptId,
  );
};

export const buildTaskSavePayload = (tasks: ScriptTaskTable[], scriptId: string) =>
  tasks.map((task, index) => ({
    ...cloneJson(task),
    scriptId,
    index,
  }));

export const buildPolicyDraft = async ({
  name,
  requestUuid,
  scriptId,
  policies,
}: {
  name?: string;
  requestUuid: () => Promise<string>;
  scriptId: string;
  policies: PolicyTable[];
}): Promise<PolicyTable> =>
  normalizePolicy(
    {
      id: await requestUuid(),
      scriptId,
      orderIndex: policies.length,
      data: {
        name: name || findNextOrdinalName(policies.map((policy) => policy.data?.name || ''), '策略'),
        note: '',
        logPrint: null,
        curPos: 0,
        skipFlag: false,
        execMax: 1,
        beforeAction: createStepList(),
        cond: createSearchRule('group'),
        afterAction: createStepList(),
      },
    },
    policies.length,
  );

export const buildPolicyGroupDraft = async ({
  name,
  requestUuid,
  scriptId,
  groups,
}: {
  name?: string;
  requestUuid: () => Promise<string>;
  scriptId: string;
  groups: PolicyGroupTable[];
}): Promise<PolicyGroupTable> =>
  normalizePolicyGroup(
    {
      id: await requestUuid(),
      scriptId,
      orderIndex: groups.length,
      data: {
        name: name || findNextOrdinalName(groups.map((group) => group.data?.name || ''), '策略组'),
        note: '',
      },
    },
    groups.length,
  );

export const buildPolicySetDraft = async ({
  name,
  requestUuid,
  scriptId,
  sets,
}: {
  name?: string;
  requestUuid: () => Promise<string>;
  scriptId: string;
  sets: PolicySetTable[];
}): Promise<PolicySetTable> =>
  normalizePolicySet(
    {
      id: await requestUuid(),
      scriptId,
      orderIndex: sets.length,
      data: {
        name: name || findNextOrdinalName(sets.map((set) => set.data?.name || ''), '策略集'),
        note: '',
      },
    },
    sets.length,
  );

export const duplicateTaskDraft = async ({
  task,
  requestUuid,
  scriptId,
  tasks,
}: {
  task: ScriptTaskTable;
  requestUuid: () => Promise<string>;
  scriptId: string;
  tasks: ScriptTaskTable[];
}) =>
  normalizeTask(
    {
      ...cloneJson(task),
      id: await requestUuid(),
      scriptId,
      name: `${task.name} 副本`,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    },
    tasks.length,
    scriptId,
  );

export const duplicatePolicyDraft = async ({
  policy,
  requestUuid,
  scriptId,
  policies,
}: {
  policy: PolicyTable;
  requestUuid: () => Promise<string>;
  scriptId: string;
  policies: PolicyTable[];
}) =>
  normalizePolicy(
    {
      ...cloneJson(policy),
      id: await requestUuid(),
      scriptId,
      data: {
        ...cloneJson(policy.data),
        name: `${policy.data.name} 副本`,
      },
      orderIndex: policies.length,
    },
    policies.length,
  );

export const duplicatePolicyGroupDraft = async ({
  group,
  requestUuid,
  scriptId,
  groups,
  relatedPolicyIds,
}: {
  group: PolicyGroupTable;
  requestUuid: () => Promise<string>;
  scriptId: string;
  groups: PolicyGroupTable[];
  relatedPolicyIds: string[];
}) => {
  const duplicateId = await requestUuid();
  return {
    item: normalizePolicyGroup(
      {
        ...cloneJson(group),
        id: duplicateId,
        scriptId,
        data: {
          ...cloneJson(group.data),
          name: `${group.data.name} 副本`,
        },
        orderIndex: groups.length,
      },
      groups.length,
    ),
    relatedPolicyIds: [...relatedPolicyIds],
  };
};

export const duplicatePolicySetDraft = async ({
  set,
  requestUuid,
  scriptId,
  sets,
  relatedGroupIds,
}: {
  set: PolicySetTable;
  requestUuid: () => Promise<string>;
  scriptId: string;
  sets: PolicySetTable[];
  relatedGroupIds: string[];
}) => {
  const duplicateId = await requestUuid();
  return {
    item: normalizePolicySet(
      {
        ...cloneJson(set),
        id: duplicateId,
        scriptId,
        data: {
          ...cloneJson(set.data),
          name: `${set.data.name} 副本`,
        },
        orderIndex: sets.length,
      },
      sets.length,
    ),
    relatedGroupIds: [...relatedGroupIds],
  };
};

export const buildPolicySavePayload = (policies: PolicyTable[], scriptId: string) =>
  policies.map((policy, index) => ({
    ...cloneJson(policy),
    scriptId,
    orderIndex: index,
  }));

export const buildPolicyGroupSavePayload = (groups: PolicyGroupTable[], scriptId: string) =>
  groups.map((group, index) => ({
    ...cloneJson(group),
    scriptId,
    orderIndex: index,
  }));

export const buildPolicySetSavePayload = (sets: PolicySetTable[], scriptId: string) =>
  sets.map((set, index) => ({
    ...cloneJson(set),
    scriptId,
    orderIndex: index,
  }));
