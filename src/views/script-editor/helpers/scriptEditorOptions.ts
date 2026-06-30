import type { PolicyGroupTable } from '@/types/bindings/PolicyGroupTable';
import type { PolicySetTable } from '@/types/bindings/PolicySetTable';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { ScriptVariableCatalog } from '@/types/bindings/ScriptVariableCatalog';
import type { EditorNamedItem } from '@/views/script-editor/editor-policy/editorPolicy';
import { TASK_ROW_TYPE, TASK_TRIGGER_MODE } from '@/views/script-editor/editor-step/editorStepKinds';
import type { EditorReferenceOption, EditorTaskUiVariableOption } from '@/views/script-editor/editorReferences';
import { parseUiSchema } from '@/views/script-editor/editorSchema';
import type { ScriptEditorRelationMap } from '@/views/script-editor/helpers/scriptEditorRelations';

export const buildPolicyItems = (policies: PolicyTable[]): EditorNamedItem[] =>
  policies.map((policy) => ({
    id: policy.id,
    title: policy.data.name,
    subtitle: `${policy.data.afterAction.length} 个命中步骤 · ${policy.data.beforeAction.length} 个全局步骤`,
    badge: String(policy.orderIndex + 1),
  }));

export const buildPolicyGroupItems = (
  groups: PolicyGroupTable[],
  groupPolicyIdsByGroupId: ScriptEditorRelationMap,
): EditorNamedItem[] =>
  groups.map((group) => ({
    id: group.id,
    title: group.data.name,
    subtitle: `${(groupPolicyIdsByGroupId[group.id] ?? []).length} 个策略`,
    badge: String(group.orderIndex + 1),
  }));

export const buildPolicySetItems = (
  sets: PolicySetTable[],
  setGroupIdsBySetId: ScriptEditorRelationMap,
): EditorNamedItem[] =>
  sets.map((set) => ({
    id: set.id,
    title: set.data.name,
    subtitle: `${(setGroupIdsBySetId[set.id] ?? []).length} 个策略组`,
    badge: String(set.orderIndex + 1),
  }));

const describeTaskReferenceTriggerMode = (mode: ScriptTaskTable['triggerMode']) => {
  switch (mode) {
    case TASK_TRIGGER_MODE.linkOnly:
      return '仅跳转';
    case TASK_TRIGGER_MODE.rootAndLink:
      return '循环 + 跳转';
    default:
      return '仅循环';
  }
};

export const buildTaskReferenceOptions = (tasks: ScriptTaskTable[]): EditorReferenceOption[] =>
  tasks
    .filter((task) => task.rowType === TASK_ROW_TYPE.task)
    .map((task) => ({
      label: task.name,
      value: task.id,
      description: `${describeTaskReferenceTriggerMode(task.triggerMode)} · ${task.defaultEnabled ? '默认启用' : '默认关闭'}`,
    }));

export const buildTaskUiVariableOptions = (
  tasks: ScriptTaskTable[],
  variableCatalog: ScriptVariableCatalog | null | undefined,
): EditorTaskUiVariableOption[] => {
  const variables = new Map((variableCatalog?.variables ?? []).map((variable) => [variable.id, variable]));
  const result: EditorTaskUiVariableOption[] = [];

  for (const task of tasks.filter((item) => item.rowType === TASK_ROW_TYPE.task && !item.isDeleted)) {
    const uiSchema = parseUiSchema(task.data.uiData ?? {});
    for (const field of uiSchema.fields) {
      if (field.control !== 'select' && field.control !== 'radio') {
        continue;
      }
      const variableId = field.variableId?.trim();
      if (!variableId) {
        continue;
      }
      const variable = variables.get(variableId);
      if (!variable || variable.namespace !== 'input' || !variable.persisted) {
        continue;
      }
      const options = field.optionsText
        .split('\n')
        .map((item) => item.trim())
        .filter(Boolean);
      if (!options.length) {
        continue;
      }
      result.push({
        taskId: task.id,
        taskLabel: task.name || '未命名任务',
        variableId,
        label: `${task.name || '未命名任务'} · ${field.label || variable.name || variable.key}`,
        description: `${variable.name || variable.key} · ${field.control === 'select' ? 'Select' : 'Radio'} · ${options.length} 个选项`,
        options,
      });
    }
  }

  return result;
};

export const buildPolicyReferenceOptions = (policies: PolicyTable[]): EditorReferenceOption[] =>
  policies.map((policy) => ({
    label: policy.data.name,
    value: policy.id,
    description: `${policy.data.afterAction.length} 个命中步骤 · ${policy.data.beforeAction.length} 个全局步骤`,
  }));

export const buildPolicyGroupReferenceOptions = (
  groups: PolicyGroupTable[],
  groupPolicyIdsByGroupId: ScriptEditorRelationMap,
): EditorReferenceOption[] =>
  groups.map((group) => ({
    label: group.data.name,
    value: group.id,
    description: `${(groupPolicyIdsByGroupId[group.id] ?? []).length} 个策略`,
  }));

export const buildPolicySetReferenceOptions = (
  sets: PolicySetTable[],
  setGroupIdsBySetId: ScriptEditorRelationMap,
): EditorReferenceOption[] =>
  sets.map((set) => ({
    label: set.data.name,
    value: set.id,
    description: `${(setGroupIdsBySetId[set.id] ?? []).length} 个策略组`,
  }));

export const buildAssignedRelationItems = <T extends { id: string; data: { name: string; note?: string | null } }>(
  items: T[],
  assignedIds: string[],
): EditorNamedItem[] =>
  assignedIds
    .map((id) => items.find((item) => item.id === id))
    .filter((item): item is T => Boolean(item))
    .map((item) => ({
      id: item.id,
      title: item.data.name,
      subtitle: item.data.note || '未填写备注',
    }));

export const buildUnassignedRelationItems = <T extends { id: string; data: { name: string; note?: string | null } }>(
  items: T[],
  assignedIds: string[],
): EditorNamedItem[] => {
  const assigned = new Set(assignedIds);
  return items
    .filter((item) => !assigned.has(item.id))
    .map((item) => ({
      id: item.id,
      title: item.data.name,
      subtitle: item.data.note || '未填写备注',
    }));
};
