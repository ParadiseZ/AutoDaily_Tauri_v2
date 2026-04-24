import {
  buildInputJson,
  getVariableDisplayKey,
  getVariableValueTypeLabel,
  type EditorInputEntry,
  type EditorVariableOption,
} from '@/views/script-editor/editorVariables';
import type { EditorUiField } from '@/views/script-editor/editorSchema';

export const buildUiBindOptions = (variableOptions: EditorVariableOption[]) => [
  { label: '未绑定', value: null, description: '纯展示字段或说明文本。' },
  ...variableOptions
    .filter((entry) => entry.uiBindable)
    .map((entry) => ({
      label: entry.label || entry.key || '未命名输入',
      value: entry.id,
      description: `${getVariableDisplayKey(entry.key, entry.namespace)} · ${entry.namespace === 'input' ? 'Input' : entry.namespace === 'runtime' ? 'Runtime' : 'System'} · ${getVariableValueTypeLabel(entry.valueType)}`,
    })),
];

export const findBoundInputEntry = (field: EditorUiField, inputEntries: EditorInputEntry[]) => {
  if (field.variableId) {
    const byId = inputEntries.find((entry) => entry.id === field.variableId);
    if (byId) {
      return byId;
    }
  }

  const inputKeys = [field.inputKey, field.key]
    .map((key) => key.trim())
    .filter(Boolean);

  for (const inputKey of inputKeys) {
    const matched = inputEntries.find((entry) => {
      const displayKey = getVariableDisplayKey(entry.key, entry.namespace);
      return entry.key === inputKey || displayKey === inputKey || `input.${displayKey}` === inputKey;
    });

    if (matched) {
      return matched;
    }
  }

  return null;
};

export const resolvePreviewValue = (field: EditorUiField, inputEntries: EditorInputEntry[]) => {
  try {
    const inputs = buildInputJson(inputEntries);
    const entry = findBoundInputEntry(field, inputEntries);
    return entry ? inputs[entry.key] ?? null : null;
  } catch {
    return null;
  }
};

export const parseFieldOptions = (field: EditorUiField) =>
  field.optionsText
    .split('\n')
    .map((item) => item.trim())
    .filter(Boolean);

export const resolveNumberPreview = (field: EditorUiField, inputEntries: EditorInputEntry[]) => {
  const value = resolvePreviewValue(field, inputEntries);
  return value === null || value === undefined || value === '' ? '0' : String(value);
};

export const resolveSelectPreview = (field: EditorUiField, inputEntries: EditorInputEntry[]) => {
  const options = parseFieldOptions(field);
  const value = resolvePreviewValue(field, inputEntries);
  if (value !== null && value !== undefined && String(value).trim()) {
    return String(value);
  }
  return options[0] ?? '请选择';
};

export const getPreviewOptions = (field: EditorUiField, inputEntries: EditorInputEntry[]) => {
  const options = parseFieldOptions(field);
  if (options.length) {
    return options;
  }

  const preview = resolveSelectPreview(field, inputEntries);
  return preview ? [preview] : ['请选择'];
};

export const resolveTextPreview = (field: EditorUiField, inputEntries: EditorInputEntry[]) => {
  const value = resolvePreviewValue(field, inputEntries);
  if (value !== null && value !== undefined && String(value).trim()) {
    return String(value);
  }
  return field.label || field.placeholder || field.description || '文本';
};

export const isUiFieldPreviewInteractive = (field: EditorUiField, inputEntries: EditorInputEntry[]) => {
  const entry = findBoundInputEntry(field, inputEntries);
  if (!entry || entry.namespace !== 'input') {
    return false;
  }

  return field.control === 'text' ? field.editable : true;
};
