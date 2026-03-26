import type { JsonValue } from '@/types/app/domain';

export type EditorInputType = 'string' | 'number' | 'boolean' | 'json';
export type EditorPanelId = 'basic' | 'inputs' | 'ui' | 'steps';
export type RawEditorSection = 'inputs' | 'ui' | 'steps';
export type UiFieldControl = 'checkbox' | 'radio' | 'select' | 'number' | 'text';

export interface EditorInputEntry {
  id: string;
  key: string;
  type: EditorInputType;
  stringValue: string;
  booleanValue: boolean;
}

export interface EditorUiField {
  id: string;
  key: string;
  label: string;
  control: UiFieldControl;
  inputKey: string;
  description: string;
  placeholder: string;
  optionsText: string;
  extra: Record<string, JsonValue>;
}

export interface EditorUiSchema {
  layout: 'horizontal' | 'vertical';
  fields: EditorUiField[];
  extras: Record<string, JsonValue>;
}

const createEditorId = (prefix: string) => `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;

const isRecord = (value: unknown): value is Record<string, JsonValue> =>
  Boolean(value) && !Array.isArray(value) && typeof value === 'object';

export const stableStringify = (value: unknown) =>
  JSON.stringify(value, (_key, item) => (typeof item === 'bigint' ? Number(item) : item), 2);

export const cloneJson = <T>(value: T): T => JSON.parse(stableStringify(value)) as T;

const inferInputType = (value: JsonValue): EditorInputType => {
  if (typeof value === 'boolean') {
    return 'boolean';
  }

  if (typeof value === 'number') {
    return 'number';
  }

  if (typeof value === 'string') {
    return 'string';
  }

  return 'json';
};

const stringifyInputValue = (value: JsonValue, type: EditorInputType) => {
  if (type === 'string') {
    return typeof value === 'string' ? value : '';
  }

  if (type === 'number') {
    return typeof value === 'number' ? String(value) : '0';
  }

  if (type === 'boolean') {
    return '';
  }

  return stableStringify(value);
};

export const createInputEntry = (type: EditorInputType = 'string'): EditorInputEntry => ({
  id: createEditorId('input'),
  key: '',
  type,
  stringValue: type === 'number' ? '0' : '',
  booleanValue: false,
});

export const parseInputEntries = (value: JsonValue): EditorInputEntry[] => {
  if (!isRecord(value)) {
    return [];
  }

  return Object.entries(value).map(([key, item]) => {
    const type = inferInputType(item);
    return {
      id: createEditorId('input'),
      key,
      type,
      stringValue: stringifyInputValue(item, type),
      booleanValue: typeof item === 'boolean' ? item : false,
    };
  });
};

const parseInputValue = (entry: EditorInputEntry): JsonValue => {
  if (entry.type === 'boolean') {
    return entry.booleanValue;
  }

  if (entry.type === 'number') {
    const parsed = Number(entry.stringValue);
    if (!Number.isFinite(parsed)) {
      throw new Error(`输入变量 ${entry.key || '未命名'} 的数字值无效。`);
    }
    return parsed;
  }

  if (entry.type === 'json') {
    return JSON.parse(entry.stringValue) as JsonValue;
  }

  return entry.stringValue;
};

export const buildInputJson = (entries: EditorInputEntry[]): Record<string, JsonValue> => {
  const result: Record<string, JsonValue> = {};

  for (const entry of entries) {
    const key = entry.key.trim();
    if (!key) {
      continue;
    }

    result[key] = parseInputValue(entry);
  }

  return result;
};

const normalizeOptionsText = (value: unknown) => {
  if (!Array.isArray(value)) {
    return '';
  }

  return value
    .map((item) => {
      if (typeof item === 'string') {
        return item;
      }

      if (isRecord(item) && typeof item.label === 'string') {
        return item.label;
      }

      return '';
    })
    .filter(Boolean)
    .join('\n');
};

export const createUiField = (control: UiFieldControl): EditorUiField => ({
  id: createEditorId('ui-field'),
  key: '',
  label:
    control === 'checkbox'
      ? '新开关'
      : control === 'radio'
        ? '新单选'
        : control === 'select'
          ? '新选择项'
          : control === 'number'
            ? '新数字输入'
            : '新文本输入',
  control,
  inputKey: '',
  description: '',
  placeholder: control === 'text' ? '请输入内容' : '',
  optionsText: control === 'radio' || control === 'select' ? '选项 1\n选项 2' : '',
  extra: {},
});

export const uiFieldTemplates: Array<{ id: UiFieldControl; label: string; description: string }> = [
  { id: 'checkbox', label: 'Checkbox', description: '适合开关类配置。' },
  { id: 'radio', label: 'Radio', description: '适合少量互斥选项。' },
  { id: 'select', label: 'Select', description: '适合较长选项列表。' },
  { id: 'number', label: 'Number', description: '适合次数、阈值和索引。' },
  { id: 'text', label: 'Text', description: '适合字符串输入。' },
];

export const createUiSchema = (): EditorUiSchema => ({
  layout: 'horizontal',
  fields: [],
  extras: {},
});

export const parseUiSchema = (value: JsonValue): EditorUiSchema => {
  if (!isRecord(value)) {
    return createUiSchema();
  }

  const extras = Object.fromEntries(
    Object.entries(value).filter(([key]) => key !== 'title' && key !== 'hint' && key !== 'layout' && key !== 'fields'),
  );

  const fieldsSource = Array.isArray(value.fields) ? value.fields : [];
  const fields = fieldsSource.map((item) => {
    const record = isRecord(item) ? item : {};
    const {
      key,
      label,
      control,
      inputKey,
      description,
      placeholder,
      options,
      ...rest
    } = record;

    return {
      id: createEditorId('ui-field'),
      key: typeof key === 'string' ? key : '',
      label: typeof label === 'string' ? label : '',
      control: ['checkbox', 'radio', 'select', 'number', 'text'].includes(String(control))
        ? (control as UiFieldControl)
        : 'text',
      inputKey: typeof inputKey === 'string' ? inputKey : '',
      description: typeof description === 'string' ? description : '',
      placeholder: typeof placeholder === 'string' ? placeholder : '',
      optionsText: normalizeOptionsText(options),
      extra: rest,
    } satisfies EditorUiField;
  });

  return {
    layout: value.layout === 'vertical' ? 'vertical' : 'horizontal',
    fields,
    extras,
  };
};

export const buildUiData = (schema: EditorUiSchema): Record<string, JsonValue> => {
  const result: Record<string, JsonValue> = {
    ...schema.extras,
  };

  result.layout = schema.layout;

  if (schema.fields.length) {
    result.fields = schema.fields.map((field) => {
      const options = field.optionsText
        .split('\n')
        .map((item) => item.trim())
        .filter(Boolean);

      return {
        ...field.extra,
        key: field.key.trim() || field.inputKey.trim() || field.label.trim(),
        label: field.label.trim() || field.key.trim(),
        control: field.control,
        inputKey: field.inputKey.trim(),
        ...(field.description.trim() ? { description: field.description.trim() } : {}),
        ...(field.placeholder.trim() ? { placeholder: field.placeholder.trim() } : {}),
        ...(options.length ? { options } : {}),
      };
    });
  }

  return result;
};

export const getInputTypeLabel = (type: EditorInputType) => {
  switch (type) {
    case 'boolean':
      return '布尔';
    case 'number':
      return '数字';
    case 'json':
      return 'JSON';
    default:
      return '文本';
  }
};

export const getUiControlLabel = (control: UiFieldControl) => {
  switch (control) {
    case 'checkbox':
      return 'Checkbox';
    case 'radio':
      return 'Radio';
    case 'select':
      return 'Select';
    case 'number':
      return 'Number';
    default:
      return 'Text';
  }
};
