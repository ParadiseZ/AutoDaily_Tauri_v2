import type { VarValue } from '@/types/bindings/VarValue';

const castVarValue = (value: unknown) => value as VarValue;

export interface VarValueDraft {
  kind: 'int' | 'float' | 'bool' | 'string';
  textValue: string;
  boolValue: boolean;
}

export type VarValueKind = VarValueDraft['kind'];

export const varValueTypeOptions = [
  { label: '整数', value: 'int', description: '32 位整数。' },
  { label: '浮点', value: 'float', description: '浮点数。' },
  { label: '布尔', value: 'bool', description: '真假值。' },
  { label: '文本', value: 'string', description: '字符串。' },
];

const isTaggedVarValue = (value: unknown): value is { type: VarValueKind; value: unknown } => {
  if (!value || typeof value !== 'object') {
    return false;
  }

  const record = value as Record<string, unknown>;
  return ['int', 'float', 'bool', 'string'].includes(String(record.type)) && 'value' in record;
};

const createDraft = (kind: VarValueKind, rawValue: unknown): VarValueDraft => {
  if (kind === 'bool') {
    const boolValue = typeof rawValue === 'boolean' ? rawValue : rawValue === 'true' || rawValue === 1;
    return {
      kind,
      textValue: boolValue ? 'true' : 'false',
      boolValue,
    };
  }

  return {
    kind,
    textValue:
      rawValue === null || rawValue === undefined
        ? kind === 'string'
          ? ''
          : '0'
        : String(rawValue),
    boolValue: Boolean(rawValue),
  };
};

export const parseVarValueDraft = (value: unknown, preferredKind?: VarValueKind): VarValueDraft => {
  if (isTaggedVarValue(value)) {
    return createDraft(value.type, value.value);
  }

  if (preferredKind) {
    return createDraft(preferredKind, value);
  }

  if (typeof value === 'boolean') {
    return {
      kind: 'bool',
      textValue: value ? 'true' : 'false',
      boolValue: value,
    };
  }

  if (typeof value === 'number') {
    const hasDecimal = !Number.isInteger(value);
    return {
      kind: hasDecimal ? 'float' : 'int',
      textValue: String(value),
      boolValue: value !== 0,
    };
  }

  return {
    kind: 'string',
    textValue: typeof value === 'string' ? value : '',
    boolValue: false,
  };
};

export const buildVarValue = (draft: VarValueDraft): VarValue => {
  switch (draft.kind) {
    case 'bool':
      return castVarValue(Boolean(draft.boolValue));
    case 'float':
      return castVarValue(Number(draft.textValue || '0'));
    case 'int':
      return castVarValue(Math.trunc(Number(draft.textValue || '0')));
    default:
      return castVarValue(draft.textValue);
  }
};
