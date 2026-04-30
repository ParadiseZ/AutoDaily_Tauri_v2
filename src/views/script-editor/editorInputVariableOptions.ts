import type { ScriptVariableSourceType } from '@/types/bindings/ScriptVariableSourceType';
import type { ScriptVariableValueType } from '@/types/bindings/ScriptVariableValueType';
import type { EditorInputEntry, EditorInputType, EditorVariableOption } from '@/views/script-editor/editorVariables';

const inputTypeToValueType = (type: EditorInputType): ScriptVariableValueType => {
  if (type === 'int') return 'int';
  if (type === 'float') return 'float';
  if (type === 'bool') return 'bool';
  if (type === 'json') return 'json';
  if (type === 'image') return 'image';
  return 'string';
};

const buildNamespacedKey = (entry: EditorInputEntry) => {
  const storageKey = entry.key.trim();
  if (entry.namespace === 'runtime') return `runtime.${storageKey}`;
  if (entry.namespace === 'system') return `system.${storageKey}`;
  return `input.${storageKey}`;
};

export const inputEntryToVariableOption = (
  entry: EditorInputEntry,
  ownerTaskId: string | null,
): EditorVariableOption => {
  const storageKey = entry.key.trim();
  const uiBindable = entry.namespace === 'input' && entry.type !== 'json' && entry.type !== 'image';

  return {
    id: entry.id,
    key: buildNamespacedKey(entry),
    label: entry.name.trim() || storageKey || '未命名输入',
    namespace: entry.namespace,
    valueType: inputTypeToValueType(entry.type),
    defaultValue: null,
    sourceType: 'manual' satisfies ScriptVariableSourceType,
    ownerTaskId,
    sourceStepId: entry.sourceStepId,
    readable: true,
    writable: entry.namespace !== 'system',
    uiBindable,
    description: entry.description.trim(),
  };
};
