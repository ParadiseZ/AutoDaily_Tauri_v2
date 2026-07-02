import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { CurrentTaskRule } from '@/types/bindings/CurrentTaskRule';
import { LOGIC_OP } from '@/views/script-editor/editor-step/editorStepKinds';

type CurrentTaskCondition = Extract<ConditionNode, { type: 'currentTaskIn' }>;
type CurrentTaskRuleGroup = Extract<CurrentTaskRule, { type: 'group' }>;

export const currentTaskRuleTypeOptions = [
  { label: '任务', value: 'task' satisfies CurrentTaskRule['type'], description: '匹配当前执行任务。' },
  { label: '逻辑组', value: 'group' satisfies CurrentTaskRule['type'], description: '组合多个任务条件。' },
];

export const createCurrentTaskRule = (type: CurrentTaskRule['type'] = 'task'): CurrentTaskRule => {
  if (type === 'group') {
    return {
      type: 'group',
      op: LOGIC_OP.Or,
      items: [],
    };
  }

  return {
    type: 'task',
    target: '',
  };
};

export const buildCurrentTaskRuleRoot = (node: CurrentTaskCondition): CurrentTaskRuleGroup => {
  const items =
    node.items?.length
      ? node.items
      : (node.targets ?? []).map((target) => ({
          type: 'task',
          target,
        } satisfies CurrentTaskRule));

  return {
    type: 'group',
    op: node.op ?? LOGIC_OP.Or,
    items,
  };
};

export const serializeCurrentTaskRuleRoot = (
  root: CurrentTaskRuleGroup,
): Pick<CurrentTaskCondition, 'op' | 'items' | 'targets'> => ({
  op: root.op,
  items: root.items,
  targets: [],
});

export const countCurrentTaskRuleTasks = (rule: CurrentTaskRule): number => {
  if (rule.type === 'task') {
    return 1;
  }

  return rule.items.reduce((total, item) => total + countCurrentTaskRuleTasks(item), 0);
};

export const describeCurrentTaskRule = (rule: CurrentTaskRule): string => {
  if (rule.type === 'task') {
    return rule.target || '未指定任务';
  }

  return `${rule.op} · ${countCurrentTaskRuleTasks(rule)} 项任务`;
};
