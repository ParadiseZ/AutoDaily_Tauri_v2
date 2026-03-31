import type { SearchRule } from '@/types/bindings/SearchRule';
import { LOGIC_OP, SEARCH_RULE_TYPE, SEARCH_SCOPE } from '@/views/script-editor/editor-step/editorStepKinds';

export const searchRuleTypeOptions = [
  { label: '关键字', value: SEARCH_RULE_TYPE.keyword, description: '按 OCR 文本关键字匹配。' },
  { label: '正则', value: SEARCH_RULE_TYPE.regex, description: '按正则表达式匹配文本。' },
  { label: 'YOLO 索引', value: SEARCH_RULE_TYPE.yoloIdx, description: '按视觉检测索引匹配。' },
  { label: '规则组', value: SEARCH_RULE_TYPE.group, description: '组合多个搜索规则。' },
];

export const logicOpOptions = [
  { label: 'AND', value: LOGIC_OP.And, description: '所有规则都满足。' },
  { label: 'OR', value: LOGIC_OP.Or, description: '任一规则满足即可。' },
  { label: 'NOT', value: LOGIC_OP.Not, description: '对组内规则取反。' },
];

export const searchScopeOptions = [
  { label: '全局', value: SEARCH_SCOPE.Global, description: '在全局结果中匹配。' },
  { label: '条目', value: SEARCH_SCOPE.Item, description: '在单个条目内匹配。' },
];

export const createSearchRule = (type: string): SearchRule => {
  switch (type) {
    case SEARCH_RULE_TYPE.regex:
      return { type: SEARCH_RULE_TYPE.regex, pattern: '.*' };
    case SEARCH_RULE_TYPE.yoloIdx:
      return { type: SEARCH_RULE_TYPE.yoloIdx, idx: 0 };
    case SEARCH_RULE_TYPE.group:
      return {
        type: SEARCH_RULE_TYPE.group,
        op: LOGIC_OP.And,
        scope: SEARCH_SCOPE.Global,
        items: [],
      };
    default:
      return { type: SEARCH_RULE_TYPE.keyword, pattern: '开始' };
  }
};

export const ensureRootGroupRule = (rule: SearchRule): SearchRule => {
  if (rule.type === SEARCH_RULE_TYPE.group) {
    return rule;
  }

  return {
    type: SEARCH_RULE_TYPE.group,
    op: LOGIC_OP.And,
    scope: SEARCH_SCOPE.Global,
    items: [rule],
  };
};

export const describeSearchRule = (rule: SearchRule): string => {
  switch (rule.type) {
    case SEARCH_RULE_TYPE.keyword:
      return `关键字: ${rule.pattern || '未填写'}`;
    case SEARCH_RULE_TYPE.regex:
      return `正则: ${rule.pattern || '未填写'}`;
    case SEARCH_RULE_TYPE.yoloIdx:
      return `YOLO 索引: ${String(rule.idx)}`;
    case SEARCH_RULE_TYPE.group:
      return `${rule.op} 组合 · ${rule.items.length} 项`;
    default:
      return '搜索规则';
  }
};
