import type { ScriptTableRecord } from '@/types/app/domain';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { TaskCycle } from '@/types/bindings/TaskCycle';
import type { TaskRowType } from '@/types/bindings/TaskRowType';
import type { TaskTone } from '@/types/bindings/TaskTone';
import type { TaskTriggerMode } from '@/types/bindings/TaskTriggerMode';
import { TASK_ROW_TYPE, TASK_TONE } from '@/views/script-editor/editor-step/editorStepKinds';
import { buildUiData, type EditorUiSchema } from '@/views/script-editor/editorSchema';
import { syncInputVariableCatalog, type EditorInputEntry } from '@/views/script-editor/editorVariables';

export const shouldSkipTaskMetaSync = ({
  hasCurrentTask,
  hydrating,
  rowType,
  allowTitle = true,
}: {
  hasCurrentTask: boolean;
  hydrating: boolean;
  rowType?: TaskRowType;
  allowTitle?: boolean;
}) => !hasCurrentTask || hydrating || (!allowTitle && rowType === TASK_ROW_TYPE.title);

export const applyTaskName = (task: ScriptTaskTable, value: string) => {
  task.name = value.trim();
  return task;
};

export const applyTaskRowType = (task: ScriptTaskTable, value: TaskRowType) => {
  task.rowType = value;
  const forceBasicPanel = value === TASK_ROW_TYPE.title;
  if (forceBasicPanel) {
    task.recordSchedule = false;
    task.sectionId = null;
    task.indentLevel = 0;
    task.execMax = 0;
    task.showEnabledToggle = false;
    task.taskTone = TASK_TONE.normal;
  }
  return { task, forceBasicPanel };
};

export const applyTaskTriggerMode = (task: ScriptTaskTable, value: TaskTriggerMode) => {
  task.triggerMode = value;
  return task;
};

export const applyTaskHidden = (task: ScriptTaskTable, value: boolean) => {
  task.isHidden = value;
  return task;
};

export const applyTaskRecordSchedule = (task: ScriptTaskTable, value: boolean) => {
  task.recordSchedule = value;
  return task;
};

export const applyTaskSectionId = (task: ScriptTaskTable, value: string | null) => {
  task.sectionId = value;
  return task;
};

export const applyTaskIndentLevel = (task: ScriptTaskTable, value: number | string) => {
  task.indentLevel = Math.max(0, Math.min(8, Number(value || 0)));
  return task;
};

export const applyTaskDefaultTaskCycle = (task: ScriptTaskTable, value: TaskCycle) => {
  task.defaultTaskCycle = value;
  return task;
};

export const applyTaskExecMax = (task: ScriptTaskTable, value: number | string) => {
  task.execMax = Math.max(0, Number(value) || 0);
  return task;
};

export const applyTaskShowEnabledToggle = (task: ScriptTaskTable, value: boolean) => {
  task.showEnabledToggle = value;
  return task;
};

export const applyTaskDefaultEnabled = (task: ScriptTaskTable, value: boolean) => {
  task.defaultEnabled = value;
  return task;
};

export const applyTaskTone = (task: ScriptTaskTable, value: TaskTone) => {
  task.taskTone = value;
  return task;
};

export const syncTaskInputEntries = ({
  draftScript,
  currentTask,
  uiSchema,
  entries,
}: {
  draftScript: ScriptTableRecord | null;
  currentTask: ScriptTaskTable | null;
  uiSchema: EditorUiSchema;
  entries: EditorInputEntry[];
}) => {
  const nextCatalog = syncInputVariableCatalog(draftScript?.data.variableCatalog, null, entries);
  const nextDraftScript = draftScript
    ? {
        ...draftScript,
        data: {
          ...draftScript.data,
          variableCatalog: nextCatalog,
        },
      }
    : draftScript;

  if (!currentTask) {
    return {
      draftScript: nextDraftScript,
      uiSchema,
    };
  }

  const inputKeyById = new Map(
    entries
      .filter((entry) => entry.namespace === 'input' && entry.key.trim())
      .map((entry) => [entry.id, entry.key.trim()]),
  );

  let changedUiBinding = false;
  const fields = uiSchema.fields.map((field) => {
    const nextInputKey = field.variableId ? inputKeyById.get(field.variableId) : null;
    if (!nextInputKey || field.inputKey === nextInputKey) {
      return field;
    }
    changedUiBinding = true;
    return {
      ...field,
      inputKey: nextInputKey,
      key: field.key === field.inputKey ? nextInputKey : field.key,
    };
  });

  return {
    draftScript: nextDraftScript,
    uiSchema: changedUiBinding
      ? {
          ...uiSchema,
          fields,
        }
      : uiSchema,
  };
};

export const applyTaskUiSchema = (task: ScriptTaskTable, value: EditorUiSchema) => {
  task.data.uiData = buildUiData(value);
  return task;
};
