import type { Action } from '@/types/bindings/Action';
import type { ColorCompareMethod } from '@/types/bindings/ColorCompareMethod';
import type { CompareOp } from '@/types/bindings/CompareOp';
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { DataHanding } from '@/types/bindings/DataHanding';
import type { FilterMode } from '@/types/bindings/FilterMode';
import type { FlowControl } from '@/types/bindings/FlowControl';
import type { LogicOp } from '@/types/bindings/LogicOp';
import type { SearchRule } from '@/types/bindings/SearchRule';
import type { SearchScope } from '@/types/bindings/SearchScope';
import type { StateStatus } from '@/types/bindings/StateStatus';
import type { StateTarget } from '@/types/bindings/StateTarget';
import type { Step } from '@/types/bindings/Step';
import type { TaskControl } from '@/types/bindings/TaskControl';
import type { TaskCycle } from '@/types/bindings/TaskCycle';
import type { TaskRowType } from '@/types/bindings/TaskRowType';
import type { TaskTone } from '@/types/bindings/TaskTone';
import type { TaskTriggerMode } from '@/types/bindings/TaskTriggerMode';
import type { VisionNode } from '@/types/bindings/VisionNode';

type ActionMode = Extract<Action, { ac: 'click' }>['mode'] | Extract<Action, { ac: 'swipe' }>['mode'];
type PolicySetResultField = Extract<ConditionNode, { type: 'policySetResult' }>['field'];
type PolicySetResultCompareOp = Extract<ConditionNode, { type: 'policySetResult' }>['op'];
type ScalarTaskCycle = Extract<TaskCycle, string>;

export const STEP_OPS = ['sequence', 'action', 'dataHanding', 'flowControl', 'taskControl', 'vision'] as const satisfies readonly Step['op'][];
export const STEP_OP = {
  sequence: STEP_OPS[0],
  action: STEP_OPS[1],
  dataHanding: STEP_OPS[2],
  flowControl: STEP_OPS[3],
  taskControl: STEP_OPS[4],
  vision: STEP_OPS[5],
} as const;

export const ACTION_TYPES = ['click', 'swipe', 'capture', 'reboot', 'back', 'posAdd', 'posMinus', 'dropSetNext', 'launchApp', 'stopApp'] as const satisfies readonly Action['ac'][];
export const ACTION_TYPE = {
  click: ACTION_TYPES[0],
  swipe: ACTION_TYPES[1],
  capture: ACTION_TYPES[2],
  reboot: ACTION_TYPES[3],
  back: ACTION_TYPES[4],
  posAdd: ACTION_TYPES[5],
  posMinus: ACTION_TYPES[6],
  dropSetNext: ACTION_TYPES[7],
  launchApp: ACTION_TYPES[8],
  stopApp: ACTION_TYPES[9],
} as const;

export const ACTION_MODES = ['point', 'percent', 'txt', 'labelIdx', 'mixed'] as const satisfies readonly ActionMode[];
export const ACTION_MODE = {
  point: ACTION_MODES[0],
  percent: ACTION_MODES[1],
  txt: ACTION_MODES[2],
  labelIdx: ACTION_MODES[3],
  mixed: ACTION_MODES[4],
} as const;

export const FLOW_TYPES = [
  'if',
  'while',
  'forEach',
  'repeat',
  'continue',
  'break',
  'stopScript',
  'waitMs',
  'link',
  'addPolicies',
  'removePolicies',
  'bindPolicyGroup',
  'removePolicyGroup',
  'addPolicyGroups',
  'unloadPolicyGroup',
  'bindPolicy',
  'unloadPolicy',
  'handlePolicySet',
  'handlePolicy',
] as const satisfies readonly FlowControl['type'][];
export const FLOW_TYPE = {
  if: FLOW_TYPES[0],
  while: FLOW_TYPES[1],
  forEach: FLOW_TYPES[2],
  repeat: FLOW_TYPES[3],
  continue: FLOW_TYPES[4],
  break: FLOW_TYPES[5],
  stopScript: FLOW_TYPES[6],
  waitMs: FLOW_TYPES[7],
  link: FLOW_TYPES[8],
  addPolicies: FLOW_TYPES[9],
  removePolicies: FLOW_TYPES[10],
  bindPolicyGroup: FLOW_TYPES[11],
  removePolicyGroup: FLOW_TYPES[12],
  addPolicyGroups: FLOW_TYPES[13],
  unloadPolicyGroup: FLOW_TYPES[14],
  bindPolicy: FLOW_TYPES[15],
  unloadPolicy: FLOW_TYPES[16],
  handlePolicySet: FLOW_TYPES[17],
  handlePolicy: FLOW_TYPES[18],
} as const;

export const DATA_TYPES = ['setVar', 'getVar', 'filter', 'colorCompare', 'relativeFilter', 'rhai'] as const satisfies readonly DataHanding['type'][];
export const DATA_TYPE = {
  setVar: DATA_TYPES[0],
  getVar: DATA_TYPES[1],
  filter: DATA_TYPES[2],
  colorCompare: DATA_TYPES[3],
  relativeFilter: DATA_TYPES[4],
  rhai: DATA_TYPES[5],
} as const;

export const COLOR_COMPARE_METHOD_TYPES = ['oklabDistance'] as const satisfies readonly ColorCompareMethod['type'][];
export const COLOR_COMPARE_METHOD_TYPE = {
  oklabDistance: COLOR_COMPARE_METHOD_TYPES[0],
} as const;

export const VISION_TYPES = ['detect', 'ocr', 'countCompare', 'visionSearch'] as const;
export const VISION_TYPE = {
  detect: VISION_TYPES[0],
  ocr: VISION_TYPES[1],
  countCompare: VISION_TYPES[2],
  visionSearch: VISION_TYPES[3],
} as const;

export const TASK_CONTROL_TYPES = ['setState'] as const satisfies readonly TaskControl['type'][];
export const TASK_CONTROL_TYPE = {
  setState: TASK_CONTROL_TYPES[0],
} as const;

export const STATE_TARGET_TYPES = ['policy', 'task'] as const satisfies readonly StateTarget['type'][];
export const STATE_TARGET_TYPE = {
  policy: STATE_TARGET_TYPES[0],
  task: STATE_TARGET_TYPES[1],
} as const;

export const STATE_STATUS_TYPES = ['enabled', 'skip', 'done'] as const satisfies readonly StateStatus['type'][];
export const STATE_STATUS_TYPE = {
  enabled: STATE_STATUS_TYPES[0],
  skip: STATE_STATUS_TYPES[1],
  done: STATE_STATUS_TYPES[2],
} as const;

export const FILTER_MODE_TYPES = ['filter', 'map'] as const satisfies readonly FilterMode['type'][];
export const FILTER_MODE_TYPE = {
  filter: FILTER_MODE_TYPES[0],
  map: FILTER_MODE_TYPES[1],
} as const;

export const SEARCH_RULE_TYPES = ['txt', 'detLabel', 'group'] as const satisfies readonly SearchRule['type'][];
export const SEARCH_RULE_TYPE = {
  txt: SEARCH_RULE_TYPES[0],
  detLabel: SEARCH_RULE_TYPES[1],
  group: SEARCH_RULE_TYPES[2],
} as const;

export const LOGIC_OPS = ['And', 'Or', 'Not'] as const satisfies readonly LogicOp[];
export const LOGIC_OP = {
  And: LOGIC_OPS[0],
  Or: LOGIC_OPS[1],
  Not: LOGIC_OPS[2],
} as const;

export const SEARCH_SCOPES = ['Global', 'Item'] as const satisfies readonly SearchScope[];
export const SEARCH_SCOPE = {
  Global: SEARCH_SCOPES[0],
  Item: SEARCH_SCOPES[1],
} as const;

export const TASK_ROW_TYPES = ['task', 'title'] as const satisfies readonly TaskRowType[];
export const TASK_ROW_TYPE = {
  task: TASK_ROW_TYPES[0],
  title: TASK_ROW_TYPES[1],
} as const;

export const TASK_TRIGGER_MODES = ['rootOnly', 'linkOnly', 'rootAndLink'] as const satisfies readonly TaskTriggerMode[];
export const TASK_TRIGGER_MODE = {
  rootOnly: TASK_TRIGGER_MODES[0],
  linkOnly: TASK_TRIGGER_MODES[1],
  rootAndLink: TASK_TRIGGER_MODES[2],
} as const;

export const TASK_TONES = ['normal', 'warning', 'danger'] as const satisfies readonly TaskTone[];
export const TASK_TONE = {
  normal: TASK_TONES[0],
  warning: TASK_TONES[1],
  danger: TASK_TONES[2],
} as const;

export const TASK_CYCLE_VALUES = ['everyRun', 'daily', 'weekly', 'monthly'] as const satisfies readonly ScalarTaskCycle[];
export const TASK_CYCLE_VALUE = {
  everyRun: TASK_CYCLE_VALUES[0],
  daily: TASK_CYCLE_VALUES[1],
  weekly: TASK_CYCLE_VALUES[2],
  monthly: TASK_CYCLE_VALUES[3],
  weekDay: 'weekDay',
  monthDay: 'monthDay',
} as const;

export const CONDITION_TYPES = ['rawExpr', 'group', 'execNumCompare', 'taskStatus', 'currentTaskIn', 'varCompare', 'policySetResult', 'colorCompare'] as const satisfies readonly ConditionNode['type'][];
export const CONDITION_TYPE = {
  rawExpr: CONDITION_TYPES[0],
  group: CONDITION_TYPES[1],
  execNumCompare: CONDITION_TYPES[2],
  taskStatus: CONDITION_TYPES[3],
  currentTaskIn: CONDITION_TYPES[4],
  varCompare: CONDITION_TYPES[5],
  policySetResult: CONDITION_TYPES[6],
  colorCompare: CONDITION_TYPES[7],
} as const;

export const POLICY_SET_RESULT_FIELDS = ['matched', 'policySetId', 'policyGroupId', 'policyId'] as const satisfies readonly PolicySetResultField[];
export const POLICY_SET_RESULT_FIELD = {
  matched: POLICY_SET_RESULT_FIELDS[0],
  policySetId: POLICY_SET_RESULT_FIELDS[1],
  policyGroupId: POLICY_SET_RESULT_FIELDS[2],
  policyId: POLICY_SET_RESULT_FIELDS[3],
} as const;

export const POLICY_SET_RESULT_COMPARE_OPS = ['eq', 'ne'] as const satisfies readonly PolicySetResultCompareOp[];
export const POLICY_SET_RESULT_COMPARE_OP = {
  eq: POLICY_SET_RESULT_COMPARE_OPS[0],
  ne: POLICY_SET_RESULT_COMPARE_OPS[1],
} as const;

export const COMPARE_OPS = ['eq', 'ne', 'lt', 'le', 'gt', 'ge', 'contains', 'notContains'] as const satisfies readonly CompareOp[];
export const COMPARE_OP = {
  eq: COMPARE_OPS[0],
  ne: COMPARE_OPS[1],
  lt: COMPARE_OPS[2],
  le: COMPARE_OPS[3],
  gt: COMPARE_OPS[4],
  ge: COMPARE_OPS[5],
  contains: COMPARE_OPS[6],
  notContains: COMPARE_OPS[7],
} as const;

export const createStepList = (): Step[] => [];
export const createSearchRuleList = (): SearchRule[] => [];
export const createConditionNodeList = (): ConditionNode[] => [];
export const createStringList = (): string[] => [];
export const createStateTargetList = (): StateTarget[] => [];

export const createTaskCycleWeekDay = (weekDay = 1): TaskCycle => ({ weekDay });
export const createTaskCycleMonthDay = (monthDay = 1): TaskCycle => ({ monthDay });
export const createFilterMode = (type: FilterMode['type'] = FILTER_MODE_TYPE.filter): FilterMode => ({ type });
export const createRegionPoint = (mode: 'point' | 'percent' = ACTION_MODE.point, x = 0, y = 0) => ({
  mode,
  p: { x, y },
});
export const createColorCompareMethod = (
  type: ColorCompareMethod['type'] = COLOR_COMPARE_METHOD_TYPE.oklabDistance,
  threshold = 0,
): ColorCompareMethod => ({
  type,
  threshold,
});
export const createStateTarget = (type: StateTarget['type'] = STATE_TARGET_TYPE.task, id = ''): StateTarget => ({ type, id });
export const createStateStatus = (type: StateStatus['type'] = STATE_STATUS_TYPE.done, value = true): StateStatus => ({ type, value });
