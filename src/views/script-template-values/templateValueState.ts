import type { JsonValue, ScriptTableRecord } from '@/types/app/domain';
import type { TaskCycle } from '@/types/bindings/TaskCycle';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { ScriptVariableDef } from '@/types/bindings/ScriptVariableDef';
import type { ScriptVariableValueType } from '@/types/bindings/ScriptVariableValueType';
import type { EditorInputEntry, EditorInputType } from '@/views/script-editor/editorVariables';
import { getVariableDisplayKey } from '@/views/script-editor/editorVariables';

export interface TemplateVariableEntry {
  id: string;
  key: string;
  displayKey: string;
  name: string;
  description: string;
  ownerTaskId: string | null;
  ownerTaskName: string;
  valueType: ScriptVariableValueType;
  defaultValue: JsonValue | null;
  defaultPreview: string;
  stringValue: string;
  booleanValue: boolean;
}

export interface TemplateTaskSettingEntry {
  taskId: string;
  enabled: boolean;
  defaultEnabled: boolean;
  taskCycle: TaskCycle;
  defaultTaskCycle: TaskCycle;
}

const isRecord = (value: unknown): value is Record<string, JsonValue> =>
  Boolean(value) && !Array.isArray(value) && typeof value === 'object';

const isTaskCycle = (value: JsonValue | undefined): value is TaskCycle => {
  if (value === 'everyRun' || value === 'daily' || value === 'weekly' || value === 'monthly') {
    return true;
  }

  if (!isRecord(value)) {
    return false;
  }

  return (
    (typeof value.weekDay === 'number' && Number.isFinite(value.weekDay)) ||
    (typeof value.monthDay === 'number' && Number.isFinite(value.monthDay))
  );
};

const normalizeTaskCycle = (value: TaskCycle): TaskCycle => {
  if (typeof value === 'string') {
    return value;
  }

  if ('weekDay' in value) {
    return { weekDay: Math.max(1, Math.min(7, Math.trunc(value.weekDay))) };
  }

  return { monthDay: Math.max(1, Math.min(31, Math.trunc(value.monthDay))) };
};

const sameTaskCycle = (left: TaskCycle, right: TaskCycle) =>
  JSON.stringify(normalizeTaskCycle(left)) === JSON.stringify(normalizeTaskCycle(right));

const fallbackValueByType = (valueType: ScriptVariableValueType): JsonValue => {
  switch (valueType) {
    case 'bool':
      return false;
    case 'int':
    case 'float':
      return 0;
    case 'list':
      return [];
    case 'json':
    case 'object':
      return {};
    default:
      return '';
  }
};

const stringifyValue = (value: JsonValue, valueType: ScriptVariableValueType) => {
  if (valueType === 'bool') {
    return typeof value === 'boolean' ? (value ? 'true' : 'false') : 'false';
  }

  if (valueType === 'int' || valueType === 'float') {
    return typeof value === 'number' ? String(value) : '0';
  }

  if (valueType === 'json' || valueType === 'list' || valueType === 'object') {
    return JSON.stringify(value, null, 2);
  }

  return typeof value === 'string' ? value : '';
};

const parseValue = (entry: TemplateVariableEntry): JsonValue => {
  if (entry.valueType === 'bool') {
    return entry.booleanValue;
  }

  if (entry.valueType === 'int') {
    const parsed = Number(entry.stringValue);
    if (!Number.isFinite(parsed)) {
      throw new Error(`变量 ${entry.name || entry.displayKey || entry.key} 的整数值无效。`);
    }
    return Math.trunc(parsed);
  }

  if (entry.valueType === 'float') {
    const parsed = Number(entry.stringValue);
    if (!Number.isFinite(parsed)) {
      throw new Error(`变量 ${entry.name || entry.displayKey || entry.key} 的浮点值无效。`);
    }
    return parsed;
  }

  if (entry.valueType === 'json' || entry.valueType === 'list' || entry.valueType === 'object') {
    return JSON.parse(entry.stringValue) as JsonValue;
  }

  return entry.stringValue;
};

const taskOrderIndex = (tasks: ScriptTaskTable[], taskId: string | null) => {
  if (!taskId) {
    return -1;
  }
  const matched = tasks.find((task) => task.id === taskId);
  return matched?.index ?? Number.MAX_SAFE_INTEGER;
};

const sortVariables = (tasks: ScriptTaskTable[], left: ScriptVariableDef, right: ScriptVariableDef) => {
  const taskDelta = taskOrderIndex(tasks, left.ownerTaskId) - taskOrderIndex(tasks, right.ownerTaskId);
  if (taskDelta !== 0) {
    return taskDelta;
  }
  if ((left.ownerTaskId ?? '') !== (right.ownerTaskId ?? '')) {
    return (left.ownerTaskId ?? '').localeCompare(right.ownerTaskId ?? '');
  }
  return left.name.localeCompare(right.name) || left.key.localeCompare(right.key);
};

export const createTemplateVariableEntries = (
  script: ScriptTableRecord,
  tasks: ScriptTaskTable[],
  storedVariables: JsonValue,
): TemplateVariableEntry[] => {
  const taskNameMap = new Map(tasks.map((task) => [task.id, task.name || '未命名任务']));
  const stored = isRecord(storedVariables) ? storedVariables : {};

  return [...script.data.variableCatalog.variables]
    .filter((variable) => variable.namespace === 'input' && variable.persisted)
    .sort((left, right) => sortVariables(tasks, left, right))
    .map((variable) => {
      const displayKey = getVariableDisplayKey(variable.key, variable.namespace);
      const candidateKeys = [variable.id, variable.key, displayKey, `input.${displayKey}`];
      const storedKey = candidateKeys.find((key) => Object.prototype.hasOwnProperty.call(stored, key));
      const value =
        storedKey
          ? stored[storedKey]
          : variable.defaultValue ?? fallbackValueByType(variable.valueType);

      return {
        id: variable.id,
        key: variable.key,
        displayKey,
        name: variable.name || displayKey,
        description: variable.description || '',
        ownerTaskId: variable.ownerTaskId,
        ownerTaskName: variable.ownerTaskId ? taskNameMap.get(variable.ownerTaskId) ?? '未命名任务' : '脚本级',
        valueType: variable.valueType,
        defaultValue: variable.defaultValue,
        defaultPreview: formatTemplateVariableDefault(variable.defaultValue),
        stringValue: stringifyValue(value, variable.valueType),
        booleanValue: typeof value === 'boolean' ? value : false,
      };
    });
};

export const buildTemplateVariablePayload = (entries: TemplateVariableEntry[]) =>
  Object.fromEntries(entries.map((entry) => [entry.id, parseValue(entry)] satisfies [string, JsonValue]));

export const createTemplateTaskSettingEntries = (
  tasks: ScriptTaskTable[],
  storedSettings: JsonValue,
): TemplateTaskSettingEntry[] => {
  const stored = isRecord(storedSettings) ? storedSettings : {};

  return tasks
    .filter((task) => task.rowType === 'task' && !task.isDeleted)
    .map((task) => {
      const rawSetting = stored[task.id];
      const setting = isRecord(rawSetting) ? rawSetting : {};
      return {
        taskId: task.id,
        enabled: typeof setting.enabled === 'boolean' ? setting.enabled : task.defaultEnabled,
        defaultEnabled: task.defaultEnabled,
        taskCycle: isTaskCycle(setting.taskCycle) ? normalizeTaskCycle(setting.taskCycle) : task.defaultTaskCycle,
        defaultTaskCycle: task.defaultTaskCycle,
      };
    });
};

export const buildTemplateTaskSettingsPayload = (entries: TemplateTaskSettingEntry[]) =>
  Object.fromEntries(
    entries
      .map((entry) => {
        const payload: Record<string, JsonValue> = {};
        if (entry.enabled !== entry.defaultEnabled) {
          payload.enabled = entry.enabled;
        }
        if (!sameTaskCycle(entry.taskCycle, entry.defaultTaskCycle)) {
          payload.taskCycle = normalizeTaskCycle(entry.taskCycle) as JsonValue;
        }
        return [entry.taskId, payload] satisfies [string, Record<string, JsonValue>];
      })
      .filter(([, payload]) => Object.keys(payload).length > 0),
  );

export const updateTemplateTaskSetting = (
  entries: TemplateTaskSettingEntry[],
  taskId: string,
  enabled: boolean,
) => entries.map((entry) => (entry.taskId === taskId ? { ...entry, enabled } : entry));

export const updateTemplateTaskCycleSetting = (
  entries: TemplateTaskSettingEntry[],
  taskId: string,
  taskCycle: TaskCycle,
) => entries.map((entry) => (entry.taskId === taskId ? { ...entry, taskCycle: normalizeTaskCycle(taskCycle) } : entry));

const mapTemplateValueTypeToInputType = (valueType: ScriptVariableValueType): EditorInputType => {
  switch (valueType) {
    case 'int':
      return 'int';
    case 'float':
      return 'float';
    case 'bool':
      return 'bool';
    case 'image':
      return 'image';
    case 'json':
    case 'list':
    case 'object':
      return 'json';
    default:
      return 'string';
  }
};

export const buildTemplateEditorInputs = (entries: TemplateVariableEntry[]): EditorInputEntry[] =>
  entries.map((entry) => ({
    id: entry.id,
    key: entry.displayKey,
    name: entry.name,
    description: entry.description,
    namespace: 'input',
    type: mapTemplateValueTypeToInputType(entry.valueType),
    stringValue: entry.stringValue,
    booleanValue: entry.booleanValue,
    sourceStepId: null,
  }));

export const updateTemplateEntryFromEditorInput = (
  entries: TemplateVariableEntry[],
  entryId: string,
  field: 'stringValue' | 'booleanValue',
  value: string | boolean,
) =>
  entries.map((entry) => {
    if (entry.id !== entryId) {
      return entry;
    }

    if (field === 'booleanValue') {
      return {
        ...entry,
        booleanValue: Boolean(value),
      };
    }

    return {
      ...entry,
      stringValue: String(value),
    };
  });

export const formatTemplateVariableDefault = (value: JsonValue | null) => {
  if (value === null) {
    return '未设置';
  }
  if (typeof value === 'boolean') {
    return value ? '开' : '关';
  }
  if (typeof value === 'number' || typeof value === 'string') {
    return String(value);
  }
  if (Array.isArray(value)) {
    return `${value.length} 项`;
  }
  return '对象';
};
