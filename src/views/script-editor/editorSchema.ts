import type { JsonValue } from '@/types/app/domain';

export type EditorPanelId = 'basic' | 'inputs' | 'ui' | 'steps';
export type RawEditorSection = 'inputs' | 'ui' | 'steps';
export type UiFieldControl = 'checkbox' | 'radio' | 'select' | 'number' | 'slider' | 'text';

export interface EditorUiField {
  id: string;
  key: string;
  label: string;
  control: UiFieldControl;
  editable: boolean;
  checkboxStyle: 'checkbox' | 'switch';
  variableId: string;
  inputKey: string;
  description: string;
  placeholder: string;
  optionsText: string;
  min: number;
  max: number;
  step: number;
  numericMode: 'int' | 'float';
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
      ? '新复选'
      : control === 'radio'
        ? '新单选'
        : control === 'select'
          ? '新选择项'
          : control === 'number'
            ? '新数字输入'
            : control === 'slider'
              ? '新滑块'
            : '新文本输入',
  control,
  editable: control !== 'text',
  checkboxStyle: 'checkbox',
  variableId: '',
  inputKey: '',
  description: '',
  placeholder: control === 'text' ? '请绑定变量' : '',
  optionsText: control === 'radio' || control === 'select' ? '选项 1\n选项 2' : '',
  min: 0,
  max: 100,
  step: control === 'slider' ? 1 : 1,
  numericMode: 'int',
  extra: {},
});

export const uiFieldTemplates: Array<{ id: UiFieldControl; label: string; description: string }> = [
  { id: 'checkbox', label: 'Checkbox', description: '适合开关类配置。' },
  { id: 'radio', label: 'Radio', description: '适合少量互斥选项。' },
  { id: 'select', label: 'Select', description: '适合较长选项列表。' },
  { id: 'number', label: 'Number', description: '适合次数、阈值和索引。' },
  { id: 'slider', label: 'Slider', description: '适合范围值和阈值滑动调节。' },
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
      editable,
      checkboxStyle,
      variableId,
      inputKey,
      description,
      placeholder,
      options,
      min,
      max,
      step,
      numericMode,
      ...rest
    } = record;

    return {
      id: createEditorId('ui-field'),
      key: typeof key === 'string' ? key : '',
      label: typeof label === 'string' ? label : '',
      control: ['checkbox', 'radio', 'select', 'number', 'slider', 'text'].includes(String(control))
        ? (control as UiFieldControl)
        : 'text',
      editable: typeof editable === 'boolean' ? editable : control === 'text' ? false : true,
      checkboxStyle: checkboxStyle === 'switch' ? 'switch' : 'checkbox',
      variableId: typeof variableId === 'string' ? variableId : '',
      inputKey: typeof inputKey === 'string' ? inputKey : '',
      description: typeof description === 'string' ? description : '',
      placeholder: typeof placeholder === 'string' ? placeholder : '',
      optionsText: normalizeOptionsText(options),
      min: typeof min === 'number' ? min : 0,
      max: typeof max === 'number' ? max : 100,
      step: typeof step === 'number' && step > 0 ? step : 1,
      numericMode: numericMode === 'float' ? 'float' : 'int',
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
        ...(field.control === 'text' ? { editable: field.editable } : {}),
        ...(field.control === 'checkbox' ? { checkboxStyle: field.checkboxStyle } : {}),
        ...(field.variableId.trim() ? { variableId: field.variableId.trim() } : {}),
        inputKey: field.inputKey.trim(),
        ...(field.description.trim() ? { description: field.description.trim() } : {}),
        ...(field.placeholder.trim() ? { placeholder: field.placeholder.trim() } : {}),
        ...(field.control === 'slider' ? { min: field.min, max: field.max, step: field.step, numericMode: field.numericMode } : {}),
        ...(options.length ? { options } : {}),
      };
    });
  }

  return result;
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
    case 'slider':
      return 'Slider';
    default:
      return 'Text';
  }
};
