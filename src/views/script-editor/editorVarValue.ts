import type { VarValue } from '@/types/bindings/VarValue';

export interface VarValueDraft {
  kind: VarValue['type'];
  textValue: string;
  boolValue: boolean;
}

export type VarValueKind = VarValue['type'];
type TaggedVarValue = Extract<VarValue, { type: VarValueKind }>;

const VAR_VALUE_KINDS = ['int', 'float', 'bool', 'string'] as const;

export const varValueTypeOptions = [
  { label: '整数', value: 'int', description: '32 位整数。' },
  { label: '浮点', value: 'float', description: '浮点数。' },
  { label: '布尔', value: 'bool', description: '真假值。' },
  { label: '文本', value: 'string', description: '字符串。' },
];

const isVarValueKind = (value: unknown): value is VarValueKind =>
  typeof value === 'string' && VAR_VALUE_KINDS.includes(value as VarValueKind);

const isTaggedVarValue = (value: unknown): value is TaggedVarValue => {
  if (!value || typeof value !== 'object') {
    return false;
  }

  const record = value as Record<string, unknown>;
  return isVarValueKind(record.type) && 'value' in record;
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

const buildTaggedVarValue = (kind: VarValueKind, rawValue: unknown): TaggedVarValue => {
  switch (kind) {
    case 'bool':
      return {
        type: 'bool',
        value: Boolean(rawValue),
      };
    case 'float':
      return {
        type: 'float',
        value: Number(rawValue ?? 0),
      };
    case 'int':
      return {
        type: 'int',
        value: Math.trunc(Number(rawValue ?? 0)),
      };
    default:
      return {
        type: 'string',
        value: rawValue === null || rawValue === undefined ? '' : String(rawValue),
      };
  }
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

export const buildVarValue = (draft: VarValueDraft): VarValue =>
  buildTaggedVarValue(draft.kind, draft.kind === 'bool' ? draft.boolValue : draft.textValue);
