import type { JsonValue } from '@/types/app/domain';
import type { ScriptVariableCatalog } from '@/types/bindings/ScriptVariableCatalog';
import type { ScriptVariableDef } from '@/types/bindings/ScriptVariableDef';
import type { ScriptVariableNamespace } from '@/types/bindings/ScriptVariableNamespace';
import type { ScriptVariableSourceType } from '@/types/bindings/ScriptVariableSourceType';
import type { ScriptVariableValueType } from '@/types/bindings/ScriptVariableValueType';
import type { Step } from '@/types/bindings/Step';

export type EditorInputType = 'string' | 'int' | 'float' | 'bool' | 'json';

export interface EditorInputEntry {
  id: string;
  key: string;
  name: string;
  description: string;
  namespace: ScriptVariableNamespace;
  type: EditorInputType;
  stringValue: string;
  booleanValue: boolean;
}

export interface EditorVariableOption {
  id: string;
  key: string;
  label: string;
  namespace: ScriptVariableNamespace;
  valueType: ScriptVariableValueType;
  sourceType: ScriptVariableSourceType;
  readable: boolean;
  writable: boolean;
  uiBindable: boolean;
  description: string;
}

const NAMESPACE_PREFIX: Record<ScriptVariableNamespace, string> = {
  input: 'input.',
  runtime: 'runtime.',
  system: 'system.',
};

const createEditorId = (prefix: string) => `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;

const isRecord = (value: unknown): value is Record<string, JsonValue> =>
  Boolean(value) && !Array.isArray(value) && typeof value === 'object';

const createEmptyCatalog = (): ScriptVariableCatalog => ({
  version: 1,
  variables: [],
});

const toStorageKey = (key: string) => {
  const trimmed = key.trim();
  for (const prefix of Object.values(NAMESPACE_PREFIX)) {
    if (trimmed.startsWith(prefix)) {
      return trimmed.slice(prefix.length);
    }
  }
  return trimmed;
};

const toCatalogKey = (key: string, namespace: ScriptVariableNamespace) => `${NAMESPACE_PREFIX[namespace]}${key.trim()}`;

const inferInputType = (value: JsonValue): EditorInputType => {
  if (typeof value === 'boolean') return 'bool';
  if (typeof value === 'number') return Number.isInteger(value) ? 'int' : 'float';
  if (typeof value === 'string') return 'string';
  return 'json';
};

const mapValueTypeToInputType = (valueType: ScriptVariableValueType): EditorInputType => {
  switch (valueType) {
    case 'int':
      return 'int';
    case 'float':
      return 'float';
    case 'bool':
      return 'bool';
    case 'json':
    case 'list':
    case 'object':
      return 'json';
    default:
      return 'string';
  }
};

const mapInputTypeToValueType = (valueType: EditorInputType): ScriptVariableValueType => {
  switch (valueType) {
    case 'int':
      return 'int';
    case 'float':
      return 'float';
    case 'bool':
      return 'bool';
    case 'json':
      return 'json';
    default:
      return 'string';
  }
};

const getNamespaceRules = (namespace: ScriptVariableNamespace, type: EditorInputType) => {
  if (namespace === 'runtime') {
    return {
      readable: true,
      writable: true,
      persisted: false,
      uiBindable: false,
    };
  }

  if (namespace === 'system') {
    return {
      readable: true,
      writable: false,
      persisted: false,
      uiBindable: false,
    };
  }

  return {
    readable: true,
    writable: true,
    persisted: true,
    uiBindable: type !== 'json',
  };
};

const stringifyInputValue = (value: JsonValue, type: EditorInputType) => {
  if (type === 'bool') {
    return typeof value === 'boolean' ? (value ? 'true' : 'false') : 'false';
  }

  if (type === 'int' || type === 'float') {
    return typeof value === 'number' ? String(value) : '0';
  }

  if (type === 'string') {
    return typeof value === 'string' ? value : '';
  }

  return JSON.stringify(value, null, 2);
};

const parseInputValue = (entry: EditorInputEntry): JsonValue => {
  if (entry.type === 'bool') {
    return entry.booleanValue;
  }

  if (entry.type === 'int') {
    const parsed = Number(entry.stringValue);
    if (!Number.isFinite(parsed)) {
      throw new Error(`输入变量 ${entry.key || '未命名'} 的整数值无效。`);
    }
    return Math.trunc(parsed);
  }

  if (entry.type === 'float') {
    const parsed = Number(entry.stringValue);
    if (!Number.isFinite(parsed)) {
      throw new Error(`输入变量 ${entry.key || '未命名'} 的浮点值无效。`);
    }
    return parsed;
  }

  if (entry.type === 'json') {
    return JSON.parse(entry.stringValue) as JsonValue;
  }

  return entry.stringValue;
};

const buildVariableDef = (entry: EditorInputEntry, ownerTaskId: string): ScriptVariableDef => {
  const storageKey = entry.key.trim();
  const rules = getNamespaceRules(entry.namespace, entry.type);
  const defaultValue = entry.namespace === 'input' ? parseInputValue(entry) : null;

  return {
    id: entry.id || createEditorId('input'),
    key: toCatalogKey(storageKey, entry.namespace),
    name: entry.name.trim() || storageKey || '未命名输入',
    namespace: entry.namespace,
    valueType: mapInputTypeToValueType(entry.type),
    ownerTaskId,
    sourceType: 'manual' satisfies ScriptVariableSourceType,
    sourceStepId: null,
    readable: rules.readable,
    writable: rules.writable,
    persisted: rules.persisted,
    uiBindable: rules.uiBindable,
    defaultValue,
    description: entry.description.trim(),
  };
};

const compareVariables = (left: ScriptVariableDef, right: ScriptVariableDef) =>
  left.namespace.localeCompare(right.namespace) || left.name.localeCompare(right.name) || left.key.localeCompare(right.key);

const createDerivedRuntimeVariable = (
  key: string,
  valueType: ScriptVariableValueType,
  ownerTaskId: string | null,
  sourceStepId: string | null,
): ScriptVariableDef => ({
  id: `derived-${key}`,
  key,
  name: key.replace(/^runtime\./, ''),
  namespace: 'runtime',
  valueType,
  ownerTaskId,
  sourceType: 'stepOutput',
  sourceStepId,
  readable: true,
  writable: true,
  persisted: false,
  uiBindable: false,
  defaultValue: null,
  description: '步骤运行时产出的临时变量，用于后续条件、步骤或表达式引用。',
});

const collectDerivedRuntimeVariables = (
  steps: Step[],
  ownerTaskId: string | null,
  bucket: Map<string, ScriptVariableDef>,
) => {
  for (const step of steps) {
    if (step.op === 'sequence') {
      collectDerivedRuntimeVariables(step.steps, ownerTaskId, bucket);
      continue;
    }

    if (step.op === 'action' && step.a.ac === 'capture' && step.a.output_var?.trim()) {
      const key = step.a.output_var.trim();
      bucket.set(key, createDerivedRuntimeVariable(key, 'string', ownerTaskId, step.id));
      continue;
    }

    if (step.op === 'vision' && step.a.type === 'visionSearch') {
      if (step.a.out_var?.trim()) {
        const key = step.a.out_var.trim();
        bucket.set(key, createDerivedRuntimeVariable(key, 'json', ownerTaskId, step.id));
      }
      collectDerivedRuntimeVariables(step.a.then_steps, ownerTaskId, bucket);
      continue;
    }

    if (step.op === 'dataHanding' && step.a.type === 'filter') {
      if (step.a.out_name?.trim()) {
        const key = step.a.out_name.trim();
        bucket.set(key, createDerivedRuntimeVariable(key, 'json', ownerTaskId, step.id));
      }
      collectDerivedRuntimeVariables(step.a.then_steps, ownerTaskId, bucket);
      continue;
    }

    if (step.op === 'flowControl') {
      if (step.a.type === 'if') {
        collectDerivedRuntimeVariables(step.a.then, ownerTaskId, bucket);
        collectDerivedRuntimeVariables(step.a.else_steps ?? [], ownerTaskId, bucket);
        continue;
      }

      if (step.a.type === 'while' || step.a.type === 'for') {
        collectDerivedRuntimeVariables(step.a.flow, ownerTaskId, bucket);
      }
    }
  }
};

export const editorInputTypeOptions = [
  { label: '文本', value: 'string', description: '普通字符串。' },
  { label: '整数', value: 'int', description: '次数、索引和数量。' },
  { label: '浮点', value: 'float', description: '阈值、比例和精度值。' },
  { label: '布尔', value: 'bool', description: '开关状态。' },
  { label: 'JSON', value: 'json', description: '复杂对象或数组。' },
];

export const createInputEntry = (type: EditorInputType = 'int'): EditorInputEntry => ({
  id: createEditorId('input'),
  key: '',
  name: '',
  description: '',
  namespace: 'input',
  type,
  stringValue: type === 'string' ? '' : type === 'json' ? '{}' : '0',
  booleanValue: false,
});

export const parseInputEntries = (
  catalog: ScriptVariableCatalog | null | undefined,
  ownerTaskId: string,
  value: JsonValue,
): EditorInputEntry[] => {
  const catalogValue = catalog ?? createEmptyCatalog();
  const variables = isRecord(value) ? value : {};
  const defs = catalogValue.variables.filter((item) => item.sourceType === 'manual' && item.ownerTaskId === ownerTaskId);
  const matchedKeys = new Set<string>();
  const entries: EditorInputEntry[] = [];

  for (const def of defs) {
    const storageKey = toStorageKey(def.key);
    matchedKeys.add(storageKey);
    const actualValue = def.namespace === 'input' ? variables[storageKey] ?? def.defaultValue ?? '' : def.defaultValue ?? '';
    const type = mapValueTypeToInputType(def.valueType);
    entries.push({
      id: def.id,
      key: storageKey,
      name: def.name || storageKey,
      description: def.description || '',
      namespace: def.namespace,
      type,
      stringValue: stringifyInputValue(actualValue, type),
      booleanValue: typeof actualValue === 'boolean' ? actualValue : false,
    });
  }

  for (const [storageKey, item] of Object.entries(variables)) {
    if (matchedKeys.has(storageKey)) {
      continue;
    }

    const type = inferInputType(item);
    entries.push({
      id: createEditorId('input'),
      key: storageKey,
      name: storageKey,
      description: '',
      namespace: 'input',
      type,
      stringValue: stringifyInputValue(item, type),
      booleanValue: typeof item === 'boolean' ? item : false,
    });
  }

  return entries;
};

export const buildInputJson = (entries: EditorInputEntry[]): Record<string, JsonValue> => {
  const result: Record<string, JsonValue> = {};

  for (const entry of entries) {
    if (entry.namespace !== 'input') continue;
    const key = entry.key.trim();
    if (!key) continue;
    result[key] = parseInputValue(entry);
  }

  return result;
};

export const syncInputVariableCatalog = (
  catalog: ScriptVariableCatalog | null | undefined,
  ownerTaskId: string,
  entries: EditorInputEntry[],
): ScriptVariableCatalog => {
  const currentCatalog = catalog ?? createEmptyCatalog();
  const preserved = currentCatalog.variables.filter((item) => !(item.sourceType === 'manual' && item.ownerTaskId === ownerTaskId));
  const nextDefs = entries
    .filter((entry) => entry.key.trim())
    .map((entry) => buildVariableDef(entry, ownerTaskId));

  return {
    version: currentCatalog.version || 1,
    variables: [...preserved, ...nextDefs].sort(compareVariables),
  };
};

const isVariableVisibleForTask = (variable: ScriptVariableDef, ownerTaskId: string | null) =>
  variable.ownerTaskId === null || variable.ownerTaskId === ownerTaskId;

const createVariableOptions = (
  variables: ScriptVariableDef[],
  capability: 'read' | 'write' | 'ui',
) =>
  variables
    .filter((item) => {
      if (capability === 'write') return item.writable;
      if (capability === 'ui') return item.uiBindable;
      return item.readable;
    })
    .sort(compareVariables)
    .map((item) => ({
      id: item.id,
      key: item.key,
      label: item.name || item.key,
      namespace: item.namespace,
      valueType: item.valueType,
      sourceType: item.sourceType,
      readable: item.readable,
      writable: item.writable,
      uiBindable: item.uiBindable,
      description: item.description,
    }));

export const listVariableOptions = (
  catalog: ScriptVariableCatalog | null | undefined,
  ownerTaskId: string | null,
  steps: Step[] = [],
  capability: 'read' | 'write' | 'ui' = 'read',
  includeDerivedRuntime = true,
): EditorVariableOption[] => {
  const currentCatalog = catalog ?? createEmptyCatalog();
  const derivedRuntime = new Map<string, ScriptVariableDef>();
  if (includeDerivedRuntime) {
    collectDerivedRuntimeVariables(steps, ownerTaskId, derivedRuntime);
  }
  const catalogKeys = new Set(currentCatalog.variables.map((item) => item.key));
  const combined = [
    ...currentCatalog.variables,
    ...Array.from(derivedRuntime.values()).filter((item) => !catalogKeys.has(item.key)),
  ];

  return createVariableOptions(
    combined.filter((item) => isVariableVisibleForTask(item, ownerTaskId)),
    capability,
  );
};

export const listAllVariableOptions = (
  catalog: ScriptVariableCatalog | null | undefined,
  steps: Step[] = [],
  capability: 'read' | 'write' | 'ui' = 'read',
  includeDerivedRuntime = true,
): EditorVariableOption[] => {
  const currentCatalog = catalog ?? createEmptyCatalog();
  const derivedRuntime = new Map<string, ScriptVariableDef>();
  if (includeDerivedRuntime) {
    collectDerivedRuntimeVariables(steps, null, derivedRuntime);
  }
  const catalogKeys = new Set(currentCatalog.variables.map((item) => item.key));
  return createVariableOptions(
    [
      ...currentCatalog.variables,
      ...Array.from(derivedRuntime.values()).filter((item) => !catalogKeys.has(item.key)),
    ],
    capability,
  );
};

export const getInputTypeLabel = (type: EditorInputType) => {
  switch (type) {
    case 'bool':
      return '布尔';
    case 'int':
      return '整数';
    case 'float':
      return '浮点';
    case 'json':
      return 'JSON';
    default:
      return '文本';
  }
};
