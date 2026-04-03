import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { LogicOp } from '@/types/bindings/LogicOp';
import type { CompareOp } from '@/types/bindings/CompareOp';
import type { VarValue } from '@/types/bindings/VarValue';
import {
  buildVarValue,
  parseVarValueDraft,
  varValueTypeOptions,
  type VarValueDraft,
  type VarValueKind,
} from '@/views/script-editor/editorVarValue';

const castCondition = (value: unknown) => value as ConditionNode;
const castVarValue = (value: unknown) => value as VarValue;

export const conditionTypeOptions = [
  { label: '表达式', value: 'rawExpr', description: '直接编写 rhai 条件表达式。' },
  { label: '条件组', value: 'group', description: '组合多个子条件。' },
  { label: '执行次数', value: 'execNumCompare', description: '按任务或策略的执行次数判断。' },
  { label: '任务状态', value: 'taskStatus', description: '按任务或策略完成/跳过状态判断。' },
  { label: '变量比较', value: 'varCompare', description: '比较运行时变量或输入变量。' },
  { label: '颜色匹配', value: 'colorCompare', description: '按 OCR 目标的颜色判断。' },
];

export const logicOpOptions = [
  { label: 'AND', value: 'And', description: '所有条件同时满足。' },
  { label: 'OR', value: 'Or', description: '任一条件满足即可。' },
  { label: 'NOT', value: 'Not', description: '对子条件结果取反。' },
];

export const stateTargetTypeOptions = [
  { label: '任务', value: 'task', description: '引用任务状态。' },
  { label: '策略', value: 'policy', description: '引用策略状态。' },
];

export const taskControlTypeOptions = [
  { label: '读取状态', value: 'getState', description: '检查目标当前状态。' },
  { label: '设置状态', value: 'setState', description: '设置目标状态。' },
];

export const stateStatusTypeOptions = [
  { label: '启用', value: 'enabled', description: '启用 / 禁用状态。' },
  { label: '完成', value: 'done', description: '完成状态。' },
  { label: '跳过', value: 'skip', description: '跳过状态。' },
];

export const compareOpOptions = [
  { label: '等于', value: 'eq', description: '值相等。' },
  { label: '不等于', value: 'ne', description: '值不相等。' },
  { label: '小于', value: 'lt', description: '左侧小于右侧。' },
  { label: '小于等于', value: 'le', description: '左侧小于等于右侧。' },
  { label: '大于', value: 'gt', description: '左侧大于右侧。' },
  { label: '大于等于', value: 'ge', description: '左侧大于等于右侧。' },
  { label: '包含', value: 'contains', description: '字符串包含目标值。' },
  { label: '不包含', value: 'notContains', description: '字符串不包含目标值。' },
];

export const createConditionNode = (type: string = 'rawExpr'): ConditionNode => {
  switch (type) {
    case 'group':
      return castCondition({
        type: 'group',
        op: 'And' satisfies LogicOp,
        items: [],
      });
    case 'execNumCompare':
      return castCondition({
        type: 'execNumCompare',
        a: {
          type: 'task',
          id: '',
        },
      });
    case 'taskStatus':
      return castCondition({
        type: 'taskStatus',
        a: {
          type: 'getState',
          target: {
            type: 'task',
            id: '',
          },
          status: {
            type: 'done',
            value: true,
          },
        },
      });
    case 'varCompare':
      return castCondition({
        type: 'varCompare',
        var_name: 'runtime.ocr_text',
        op: 'contains' satisfies CompareOp,
        value: castVarValue('开始'),
      });
    case 'colorCompare':
      return castCondition({
        type: 'colorCompare',
        txt_target: '',
        is_font: true,
        r: 255,
        g: 255,
        b: 255,
      });
    default:
      return castCondition({
        type: 'rawExpr',
        expr: 'true',
      });
  }
};

export const describeConditionNode = (node: ConditionNode) => {
  switch (node.type) {
    case 'rawExpr':
      return node.expr || '空表达式';
    case 'group':
      return `${node.op} · ${node.items.length} 项`;
    case 'execNumCompare':
      return `执行次数 · ${node.a.type}:${node.a.id || '未指定'}`;
    case 'taskStatus':
      return `${node.a.type === 'getState' ? '读取状态' : '设置状态'} · ${node.a.target.type}:${node.a.target.id || '未指定'}`;
    case 'colorCompare':
      return `${node.is_font ? '字体色' : '背景色'} · ${node.txt_target || '未指定目标'}`;
    case 'varCompare':
      return `变量 ${node.var_name || '未命名'} · ${node.op}`;
    default:
      return '条件';
  }
};

export { buildVarValue, parseVarValueDraft, varValueTypeOptions };
export type { VarValueDraft, VarValueKind };
