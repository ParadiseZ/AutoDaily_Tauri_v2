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
  { label: '当前任务', value: 'currentTaskIn', description: '判断当前执行任务是否属于指定列表。' },
  { label: '变量比较', value: 'varCompare', description: '比较运行时变量或输入变量。' },
  { label: '策略集结果', value: 'policySetResult', description: '按策略集处理步骤输出的结果对象判断。' },
  { label: '策略条件', value: 'policyCondition', description: '基于图像做视觉精判，可用于策略或任务步骤。' },
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
  { label: '状态匹配', value: 'setState', description: '检查目标当前状态。' },
];

export const stateStatusTypeOptions = [
  { label: '启用', value: 'enabled', description: '启用 / 禁用状态。' },
  { label: '完成', value: 'done', description: '完成状态。' },
  { label: '跳过', value: 'skip', description: '跳过状态。' },
];

export const policySetResultFieldOptions = [
  { label: 'matched', value: 'matched', description: '判断是否命中了策略。' },
  { label: 'policySetId', value: 'policySetId', description: '比较命中的策略集 id。' },
  { label: 'policyGroupId', value: 'policyGroupId', description: '比较命中的策略组 id。' },
  { label: 'policyId', value: 'policyId', description: '比较命中的策略 id。' },
];

export const policySetResultCompareOptions = [
  { label: '等于', value: 'eq', description: '与比较值相等。' },
  { label: '不等于', value: 'ne', description: '与比较值不相等。' },
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
        target: {
          type: 'task',
          id: '',
        },
        op: 'ge',
      });
    case 'taskStatus':
      return castCondition({
        type: 'taskStatus',
        a: {
          type: 'setState',
          target: {
            type: 'task',
            id: '',
          },
          targets: [],
          status: {
            type: 'done',
            value: true,
          },
        },
      });
    case 'currentTaskIn':
      return castCondition({
        type: 'currentTaskIn',
        targets: [],
      });
    case 'varCompare':
      return castCondition({
        type: 'varCompare',
        var_name: 'runtime.ocr_text',
        op: 'contains' satisfies CompareOp,
        value: castVarValue('开始'),
      });
    case 'policySetResult':
      return castCondition({
        type: 'policySetResult',
        result_var: 'runtime.policySetResult',
        field: 'policyId',
        op: 'eq',
        value_bool: true,
        value_id: '',
      });
    case 'policyCondition':
      return castCondition({
        type: 'policyCondition',
        input_var: null,
        rule: {
          type: 'regex',
          pattern: '.*',
        },
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
      return `执行次数 · ${node.target.type}:${node.target.id || '未指定'} · ${node.op}`;
    case 'taskStatus':
      return `状态匹配 · ${node.a.target.type}:${node.a.target.id || '未指定'}`;
    case 'currentTaskIn':
      return `当前任务 · ${node.targets.length} 项`;
    case 'colorCompare':
      return `${node.is_font ? '字体色' : '背景色'} · ${node.txt_target || '未指定目标'}`;
    case 'varCompare':
      return `变量 ${node.var_name || '未命名'} · ${node.op}`;
    case 'policySetResult':
      return `策略集结果 · ${node.field}`;
    case 'policyCondition':
      return `策略条件 · ${node.rule.type}`;
    default:
      return '条件';
  }
};

export { buildVarValue, parseVarValueDraft, varValueTypeOptions };
export type { VarValueDraft, VarValueKind };
