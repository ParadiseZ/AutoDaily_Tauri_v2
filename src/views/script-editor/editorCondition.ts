import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { LogicOp } from '@/types/bindings/LogicOp';
import type { CompareOp } from '@/types/bindings/CompareOp';
import {
  buildVarValue,
  parseVarValueDraft,
  varValueTypeOptions,
  type VarValueDraft,
  type VarValueKind,
} from '@/views/script-editor/editorVarValue';
import {
  COMPARE_OP,
  CONDITION_TYPE,
  createConditionNodeList,
  createStateStatus,
  createStateTarget,
  createStateTargetList,
  createStringList,
  POLICY_SET_RESULT_COMPARE_OP,
  POLICY_SET_RESULT_FIELD,
  STATE_STATUS_TYPE,
  TASK_CONTROL_TYPE,
} from '@/views/script-editor/editor-step/editorStepKinds';
import { buildCurrentTaskRuleRoot, countCurrentTaskRuleTasks } from '@/views/script-editor/editorCurrentTaskRule';

const castCondition = (value: unknown) => value as ConditionNode;

export const conditionTypeOptions = [
  { label: '表达式', value: CONDITION_TYPE.rawExpr, description: '直接编写 rhai 条件表达式。' },
  { label: '条件组', value: CONDITION_TYPE.group, description: '组合多个子条件。' },
  { label: '执行次数', value: CONDITION_TYPE.execNumCompare, description: '按任务或策略的执行次数判断。' },
  { label: '任务状态', value: CONDITION_TYPE.taskStatus, description: '按任务或策略完成/跳过状态判断。' },
  { label: '当前任务', value: CONDITION_TYPE.currentTaskIn, description: '判断当前执行任务是否属于指定列表。' },
  { label: '变量比较', value: CONDITION_TYPE.varCompare, description: '比较运行时变量或输入变量。' },
  { label: '判断数量大小', value: CONDITION_TYPE.visionCountCompare, description: '统计检测标签或 OCR 文字的匹配数量，并和指定数量比较。' },
  { label: '策略集结果', value: CONDITION_TYPE.policySetResult, description: '按策略集处理步骤输出的结果对象判断。' },
];

export const logicOpOptions = [
  { label: 'AND', value: 'And' satisfies LogicOp, description: '所有条件同时满足。' },
  { label: 'OR', value: 'Or' satisfies LogicOp, description: '任一条件满足即可。' },
  { label: 'NOT', value: 'Not' satisfies LogicOp, description: '对子条件结果取反。' },
];

export const stateTargetTypeOptions = [
  { label: '任务', value: createStateTarget().type, description: '引用任务状态。' },
  { label: '策略', value: createStateTarget('policy').type, description: '引用策略状态。' },
];

export const taskControlTypeOptions = [
  { label: '状态匹配', value: TASK_CONTROL_TYPE.setState, description: '检查目标当前状态。' },
];

export const stateStatusTypeOptions = [
  { label: '启用', value: STATE_STATUS_TYPE.enabled, description: '启用 / 禁用状态。' },
  { label: '完成', value: STATE_STATUS_TYPE.done, description: '完成状态。' },
  { label: '跳过', value: STATE_STATUS_TYPE.skip, description: '跳过状态。' },
];

export const policySetResultFieldOptions = [
  { label: 'matched', value: POLICY_SET_RESULT_FIELD.matched, description: '判断是否命中了策略。' },
  { label: 'policySetId', value: POLICY_SET_RESULT_FIELD.policySetId, description: '比较命中的策略集 id。' },
  { label: 'policyGroupId', value: POLICY_SET_RESULT_FIELD.policyGroupId, description: '比较命中的策略组 id。' },
  { label: 'policyId', value: POLICY_SET_RESULT_FIELD.policyId, description: '比较命中的策略 id。' },
];

export const policySetResultCompareOptions = [
  { label: '等于', value: POLICY_SET_RESULT_COMPARE_OP.eq, description: '与比较值相等。' },
  { label: '不等于', value: POLICY_SET_RESULT_COMPARE_OP.ne, description: '与比较值不相等。' },
];

export const compareOpOptions = [
  { label: '等于', value: COMPARE_OP.eq, description: '值相等。' },
  { label: '不等于', value: COMPARE_OP.ne, description: '值不相等。' },
  { label: '小于', value: COMPARE_OP.lt, description: '左侧小于右侧。' },
  { label: '小于等于', value: COMPARE_OP.le, description: '左侧小于等于右侧。' },
  { label: '大于', value: COMPARE_OP.gt, description: '左侧大于右侧。' },
  { label: '大于等于', value: COMPARE_OP.ge, description: '左侧大于等于右侧。' },
  { label: '包含', value: COMPARE_OP.contains, description: '字符串包含目标值。' },
  { label: '不包含', value: COMPARE_OP.notContains, description: '字符串不包含目标值。' },
];

export const createConditionNode = (type: ConditionNode['type'] = CONDITION_TYPE.rawExpr): ConditionNode => {
  switch (type) {
    case CONDITION_TYPE.group:
      return castCondition({
        type: CONDITION_TYPE.group,
        op: 'And' satisfies LogicOp,
        items: createConditionNodeList(),
      });
    case CONDITION_TYPE.execNumCompare:
      return castCondition({
        type: CONDITION_TYPE.execNumCompare,
        target: createStateTarget(),
        op: COMPARE_OP.ge,
      });
    case CONDITION_TYPE.taskStatus:
      return castCondition({
        type: CONDITION_TYPE.taskStatus,
        a: {
          type: TASK_CONTROL_TYPE.setState,
          target: createStateTarget(),
          targets: createStateTargetList(),
          status: createStateStatus(),
        },
      });
    case CONDITION_TYPE.currentTaskIn:
      return castCondition({
        type: CONDITION_TYPE.currentTaskIn,
        op: 'Or' satisfies LogicOp,
        items: [],
        targets: createStringList(),
      });
    case CONDITION_TYPE.varCompare:
      return castCondition({
        type: CONDITION_TYPE.varCompare,
        var_name: 'runtime.ocr_text',
        op: COMPARE_OP.contains satisfies CompareOp,
        value: buildVarValue({
          kind: 'string',
          textValue: '开始',
          boolValue: false,
        }),
      });
    case CONDITION_TYPE.visionCountCompare:
      return castCondition({
        type: CONDITION_TYPE.visionCountCompare,
        input_var: 'runtime.ocrResults',
        target_value: null,
        op: COMPARE_OP.ge satisfies CompareOp,
        expected_count: 1,
      });
    case CONDITION_TYPE.policySetResult:
      return castCondition({
        type: CONDITION_TYPE.policySetResult,
        result_var: 'runtime.policySetResult',
        field: POLICY_SET_RESULT_FIELD.policyId,
        op: POLICY_SET_RESULT_COMPARE_OP.eq,
        value_bool: true,
        value_id: '',
      });
    case CONDITION_TYPE.colorCompare:
      return castCondition({
        type: CONDITION_TYPE.colorCompare,
        txt_target: '',
        is_font: true,
        r: 255,
        g: 255,
        b: 255,
      });
    default:
      return castCondition({
        type: CONDITION_TYPE.rawExpr,
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
      return `当前任务 · ${countCurrentTaskRuleTasks(buildCurrentTaskRuleRoot(node))} 项`;
    case 'colorCompare':
      return `${node.is_font ? '字体色' : '背景色'} · ${node.txt_target || '未指定目标'}`;
    case 'varCompare':
      return `变量 ${node.var_name || '未命名'} · ${node.op}`;
    case 'visionCountCompare':
      return `数量比较 · ${node.input_var || '未绑定变量'} · ${node.op} ${node.expected_count}`;
    case 'policySetResult':
      return `策略集结果 · ${node.field}`;
    case 'colorCompare':
      return `${node.is_font ? '字体色' : '背景色'} · ${node.txt_target || '未指定目标'}`;
    default:
      return '条件';
  }
};

export { buildVarValue, parseVarValueDraft, varValueTypeOptions };
export type { VarValueDraft, VarValueKind };
