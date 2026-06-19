import type { PolicyConditionRule } from '@/types/bindings/PolicyConditionRule';
import {
  createPolicyConditionRuleList,
  LOGIC_OP,
  POLICY_CONDITION_RULE_TYPE,
  RELATIVE_ANCHOR_TYPE,
  RELATIVE_COMPARE_OP,
  RELATIVE_DIRECTION,
  RELATIVE_TARGET_KIND,
  RELATIVE_VALUE_TYPE,
} from '@/views/script-editor/editor-step/editorStepKinds';

export { POLICY_CONDITION_RULE_TYPE };

export const policyConditionRuleTypeOptions = [
  { label: '正则', value: POLICY_CONDITION_RULE_TYPE.regex, description: '按 OCR 文本缓冲区做正则精判。' },
  { label: '相对位置', value: POLICY_CONDITION_RULE_TYPE.relative, description: '按锚点和方向定位目标后做比较。' },
  { label: '逻辑组', value: POLICY_CONDITION_RULE_TYPE.group, description: '组合多个精判规则。' },
];

export const relativeAnchorTypeOptions = [
  { label: '文字锚点', value: RELATIVE_ANCHOR_TYPE.ocrText, description: '用 OCR 文本作为相对位置锚点。' },
  { label: '标签锚点', value: RELATIVE_ANCHOR_TYPE.detLabel, description: '用检测标签索引作为相对位置锚点。' },
];

export const relativeDirectionOptions = [
  { label: '右边', value: RELATIVE_DIRECTION.right, description: '寻找锚点右侧最近的候选目标。' },
  { label: '左边', value: RELATIVE_DIRECTION.left, description: '寻找锚点左侧最近的候选目标。' },
  { label: '下面', value: RELATIVE_DIRECTION.below, description: '寻找锚点下方最近的候选目标。' },
  { label: '上面', value: RELATIVE_DIRECTION.above, description: '寻找锚点上方最近的候选目标。' },
  { label: '附近', value: RELATIVE_DIRECTION.near, description: '寻找锚点附近最近的候选目标。' },
];

export const relativeTargetKindOptions = [
  { label: '文字', value: RELATIVE_TARGET_KIND.ocrText, description: '目标候选只从 OCR 文本里找。' },
  { label: '标签', value: RELATIVE_TARGET_KIND.detLabel, description: '目标候选只从检测标签里找。' },
  { label: '任意', value: RELATIVE_TARGET_KIND.any, description: '目标候选可来自文字或标签。' },
];

export const relativeValueTypeOptions = [
  { label: '文本', value: RELATIVE_VALUE_TYPE.text, description: '直接比较目标文本。' },
  { label: '数字', value: RELATIVE_VALUE_TYPE.number, description: '从目标文本里提取数字后比较。' },
  { label: '分数左值', value: RELATIVE_VALUE_TYPE.fractionLeftNumber, description: '从 a/b 文本里提取左值比较。' },
  { label: '分数右值', value: RELATIVE_VALUE_TYPE.fractionRightNumber, description: '从 a/b 文本里提取右值比较。' },
  { label: '标签名', value: RELATIVE_VALUE_TYPE.label, description: '比较检测标签名称。' },
  { label: '标签索引', value: RELATIVE_VALUE_TYPE.labelIndex, description: '比较检测标签索引。' },
];

export const relativeCompareOptions = [
  { label: '等于', value: RELATIVE_COMPARE_OP.eq, description: '值相等。' },
  { label: '不等于', value: RELATIVE_COMPARE_OP.ne, description: '值不相等。' },
  { label: '小于', value: RELATIVE_COMPARE_OP.lt, description: '左侧小于右侧。' },
  { label: '小于等于', value: RELATIVE_COMPARE_OP.le, description: '左侧小于等于右侧。' },
  { label: '大于', value: RELATIVE_COMPARE_OP.gt, description: '左侧大于右侧。' },
  { label: '大于等于', value: RELATIVE_COMPARE_OP.ge, description: '左侧大于右侧。' },
  { label: '包含', value: RELATIVE_COMPARE_OP.contains, description: '文本包含右侧值。' },
  { label: '不包含', value: RELATIVE_COMPARE_OP.notContains, description: '文本不包含右侧值。' },
];

export const logicOpOptions = [
  { label: 'AND', value: LOGIC_OP.And, description: '所有规则都满足。' },
  { label: 'OR', value: LOGIC_OP.Or, description: '任一规则满足即可。' },
  { label: 'NOT', value: LOGIC_OP.Not, description: '对组内规则取反。' },
];

export const createPolicyConditionRule = (type: PolicyConditionRule['type']): PolicyConditionRule => {
  switch (type) {
    case POLICY_CONDITION_RULE_TYPE.relative:
      return {
        type: POLICY_CONDITION_RULE_TYPE.relative,
        anchor_type: RELATIVE_ANCHOR_TYPE.ocrText,
        anchor_text: '结晶',
        anchor_idx: 0,
        direction: RELATIVE_DIRECTION.right,
        target_kind: RELATIVE_TARGET_KIND.ocrText,
        value_type: RELATIVE_VALUE_TYPE.number,
        compare: RELATIVE_COMPARE_OP.gt,
        value: '5',
        max_offset_x: null,
        max_offset_y: null,
        target_index: null,
      };
    case POLICY_CONDITION_RULE_TYPE.group:
      return {
        type: POLICY_CONDITION_RULE_TYPE.group,
        op: LOGIC_OP.And,
        items: createPolicyConditionRuleList(),
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
