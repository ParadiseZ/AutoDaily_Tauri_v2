import type { JsonValue, ScriptTableRecord } from '@/types/app/domain';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { ScriptVariableDef } from '@/types/bindings/ScriptVariableDef';
import type { ScriptVariableValueType } from '@/types/bindings/ScriptVariableValueType';
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

const isRecord = (value: unknown): value is Record<string, JsonValue> =>
  Boolean(value) && !Array.isArray(value) && typeof value === 'object';

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
      const value =
        Object.prototype.hasOwnProperty.call(stored, variable.id)
          ? stored[variable.id]
          : variable.defaultValue ?? fallbackValueByType(variable.valueType);

      return {
        id: variable.id,
        key: variable.key,
        displayKey: getVariableDisplayKey(variable.key, variable.namespace),
        name: variable.name || getVariableDisplayKey(variable.key, variable.namespace),
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
