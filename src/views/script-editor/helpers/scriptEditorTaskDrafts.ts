import type { PolicyGroupTable } from '@/types/bindings/PolicyGroupTable';
import type { PolicySetTable } from '@/types/bindings/PolicySetTable';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import { createStepList, TASK_CYCLE_VALUE, TASK_ROW_TYPE, TASK_TONE, TASK_TRIGGER_MODE } from '@/views/script-editor/editor-step/editorStepKinds';
import { cloneJson } from '@/views/script-editor/editorSchema';

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
      name: name || `新任务 ${index + 1}`,
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
