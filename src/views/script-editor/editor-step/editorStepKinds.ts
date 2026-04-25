export const STEP_OPS = ['sequence', 'action', 'dataHanding', 'flowControl', 'taskControl', 'vision'] as const;
export const STEP_OP = {
  sequence: STEP_OPS[0],
  action: STEP_OPS[1],
  dataHanding: STEP_OPS[2],
  flowControl: STEP_OPS[3],
  taskControl: STEP_OPS[4],
  vision: STEP_OPS[5],
} as const;

export const ACTION_TYPES = ['click', 'swipe', 'capture', 'reboot', 'back', 'posAdd', 'posMinus', 'dropSetNext', 'launchApp', 'stopApp'] as const;
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

export const ACTION_MODES = ['point', 'percent', 'txt', 'labelIdx'] as const;
export const ACTION_MODE = {
  point: ACTION_MODES[0],
  percent: ACTION_MODES[1],
  txt: ACTION_MODES[2],
  labelIdx: ACTION_MODES[3],
} as const;

export const FLOW_TYPES = ['if', 'while', 'forEach', 'continue', 'break', 'waitMs', 'link', 'addPolicies', 'handlePolicySet', 'handlePolicy'] as const;
export const FLOW_TYPE = {
  if: FLOW_TYPES[0],
  while: FLOW_TYPES[1],
  forEach: FLOW_TYPES[2],
  continue: FLOW_TYPES[3],
  break: FLOW_TYPES[4],
  waitMs: FLOW_TYPES[5],
  link: FLOW_TYPES[6],
  addPolicies: FLOW_TYPES[7],
  handlePolicySet: FLOW_TYPES[8],
  handlePolicy: FLOW_TYPES[9],
} as const;

export const DATA_TYPES = ['setVar', 'getVar', 'filter', 'colorCompare'] as const;
export const DATA_TYPE = {
  setVar: DATA_TYPES[0],
  getVar: DATA_TYPES[1],
  filter: DATA_TYPES[2],
  colorCompare: DATA_TYPES[3],
} as const;

export const COLOR_COMPARE_METHOD_TYPES = ['oklabDistance'] as const;
export const COLOR_COMPARE_METHOD_TYPE = {
  oklabDistance: COLOR_COMPARE_METHOD_TYPES[0],
} as const;

export const VISION_TYPES = ['visionSearch'] as const;
export const VISION_TYPE = {
  visionSearch: VISION_TYPES[0],
} as const;

export const TASK_CONTROL_TYPES = ['setState'] as const;
export const TASK_CONTROL_TYPE = {
  setState: TASK_CONTROL_TYPES[0],
} as const;

export const STATE_TARGET_TYPES = ['policy', 'task'] as const;
export const STATE_TARGET_TYPE = {
  policy: STATE_TARGET_TYPES[0],
  task: STATE_TARGET_TYPES[1],
} as const;

export const STATE_STATUS_TYPES = ['enabled', 'skip', 'done'] as const;
export const STATE_STATUS_TYPE = {
  enabled: STATE_STATUS_TYPES[0],
  skip: STATE_STATUS_TYPES[1],
  done: STATE_STATUS_TYPES[2],
} as const;

export const FILTER_MODE_TYPES = ['filter', 'map'] as const;
export const FILTER_MODE_TYPE = {
  filter: FILTER_MODE_TYPES[0],
  map: FILTER_MODE_TYPES[1],
} as const;

export const SEARCH_RULE_TYPES = ['txt', 'detLabel', 'group'] as const;
export const SEARCH_RULE_TYPE = {
  txt: SEARCH_RULE_TYPES[0],
  detLabel: SEARCH_RULE_TYPES[1],
  group: SEARCH_RULE_TYPES[2],
} as const;

export const LOGIC_OPS = ['And', 'Or', 'Not'] as const;
export const LOGIC_OP = {
  And: LOGIC_OPS[0],
  Or: LOGIC_OPS[1],
  Not: LOGIC_OPS[2],
} as const;

export const SEARCH_SCOPES = ['Global', 'Item'] as const;
export const SEARCH_SCOPE = {
  Global: SEARCH_SCOPES[0],
  Item: SEARCH_SCOPES[1],
} as const;
