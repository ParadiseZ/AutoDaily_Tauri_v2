import type { PolicyConditionRule } from '@/types/bindings/PolicyConditionRule';
import { LOGIC_OP } from '@/views/script-editor/editor-step/editorStepKinds';

export const POLICY_CONDITION_RULE_TYPE = {
  regex: 'regex',
  relative: 'relative',
  group: 'group',
} as const;

export const policyConditionRuleTypeOptions = [
  { label: '正则', value: POLICY_CONDITION_RULE_TYPE.regex, description: '按 OCR 文本缓冲区做正则精判。' },
  { label: '相对位置', value: POLICY_CONDITION_RULE_TYPE.relative, description: '按锚点和方向定位目标后做比较。' },
  { label: '规则组', value: POLICY_CONDITION_RULE_TYPE.group, description: '组合多个精判规则。' },
];

export const relativeAnchorTypeOptions = [
  { label: '文字锚点', value: 'ocrText', description: '用 OCR 文本作为相对位置锚点。' },
  { label: '标签锚点', value: 'detLabel', description: '用检测标签索引作为相对位置锚点。' },
];

export const relativeDirectionOptions = [
  { label: '右边', value: 'right', description: '寻找锚点右侧最近的候选目标。' },
  { label: '左边', value: 'left', description: '寻找锚点左侧最近的候选目标。' },
  { label: '下面', value: 'below', description: '寻找锚点下方最近的候选目标。' },
  { label: '上面', value: 'above', description: '寻找锚点上方最近的候选目标。' },
  { label: '附近', value: 'near', description: '寻找锚点附近最近的候选目标。' },
];

export const relativeTargetKindOptions = [
  { label: '文字', value: 'ocrText', description: '目标候选只从 OCR 文本里找。' },
  { label: '标签', value: 'detLabel', description: '目标候选只从检测标签里找。' },
  { label: '任意', value: 'any', description: '目标候选可来自文字或标签。' },
];

export const relativeValueTypeOptions = [
  { label: '文本', value: 'text', description: '直接比较目标文本。' },
  { label: '数字', value: 'number', description: '从目标文本里提取数字后比较。' },
  { label: '分数左值', value: 'fractionLeftNumber', description: '从 a/b 文本里提取左值比较。' },
  { label: '分数右值', value: 'fractionRightNumber', description: '从 a/b 文本里提取右值比较。' },
  { label: '标签名', value: 'label', description: '比较检测标签名称。' },
  { label: '标签索引', value: 'labelIndex', description: '比较检测标签索引。' },
];

export const relativeCompareOptions = [
  { label: '等于', value: 'eq', description: '值相等。' },
  { label: '不等于', value: 'ne', description: '值不相等。' },
  { label: '小于', value: 'lt', description: '左侧小于右侧。' },
  { label: '小于等于', value: 'le', description: '左侧小于等于右侧。' },
  { label: '大于', value: 'gt', description: '左侧大于右侧。' },
  { label: '大于等于', value: 'ge', description: '左侧大于右侧。' },
  { label: '包含', value: 'contains', description: '文本包含右侧值。' },
  { label: '不包含', value: 'notContains', description: '文本不包含右侧值。' },
];

export const logicOpOptions = [
  { label: 'AND', value: LOGIC_OP.And, description: '所有规则都满足。' },
  { label: 'OR', value: LOGIC_OP.Or, description: '任一规则满足即可。' },
  { label: 'NOT', value: LOGIC_OP.Not, description: '对组内规则取反。' },
];

export const createPolicyConditionRule = (type: string): PolicyConditionRule => {
  switch (type) {
    case POLICY_CONDITION_RULE_TYPE.relative:
      return {
        type: POLICY_CONDITION_RULE_TYPE.relative,
        anchor_type: 'ocrText',
        anchor_text: '结晶',
        anchor_idx: 0,
        direction: 'right',
        target_kind: 'ocrText',
        value_type: 'number',
        compare: 'gt',
        value: '5',
      };
    case POLICY_CONDITION_RULE_TYPE.group:
      return {
        type: POLICY_CONDITION_RULE_TYPE.group,
        op: LOGIC_OP.And,
        items: [],
      };
    default:
      return { type: POLICY_CONDITION_RULE_TYPE.regex, pattern: '.*' };
  }
};

export const describePolicyConditionRule = (rule: PolicyConditionRule): string => {
  switch (rule.type) {
    case POLICY_CONDITION_RULE_TYPE.regex:
      return `正则: ${rule.pattern || '未填写'}`;
    case POLICY_CONDITION_RULE_TYPE.relative:
      return `相对位置 · ${rule.anchor_type === 'ocrText' ? rule.anchor_text || '未填写锚点' : `标签#${rule.anchor_idx}`} ${rule.direction}`;
    case POLICY_CONDITION_RULE_TYPE.group:
      return `${rule.op} 组合 · ${rule.items.length} 项`;
    default:
      return '策略条件';
  }
};
