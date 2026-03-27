export const STEP_OPS = ['sequence', 'action', 'dataHanding', 'flowControl', 'taskControl', 'vision'] as const;
export const STEP_OP = {
  sequence: STEP_OPS[0],
  action: STEP_OPS[1],
  dataHanding: STEP_OPS[2],
  flowControl: STEP_OPS[3],
  taskControl: STEP_OPS[4],
  vision: STEP_OPS[5],
} as const;

export const ACTION_TYPES = ['click', 'swipe', 'capture', 'reboot', 'launchApp', 'stopApp'] as const;
export const ACTION_TYPE = {
  click: ACTION_TYPES[0],
  swipe: ACTION_TYPES[1],
  capture: ACTION_TYPES[2],
  reboot: ACTION_TYPES[3],
  launchApp: ACTION_TYPES[4],
  stopApp: ACTION_TYPES[5],
} as const;

export const ACTION_MODES = ['point', 'percent', 'txt', 'labelIdx'] as const;
export const ACTION_MODE = {
  point: ACTION_MODES[0],
  percent: ACTION_MODES[1],
  txt: ACTION_MODES[2],
  labelIdx: ACTION_MODES[3],
} as const;

export const FLOW_TYPES = ['if', 'while', 'for', 'continue', 'break', 'waitMs', 'link', 'addPolicies', 'handlePolicySet'] as const;
export const FLOW_TYPE = {
  if: FLOW_TYPES[0],
  while: FLOW_TYPES[1],
  for: FLOW_TYPES[2],
  continue: FLOW_TYPES[3],
  break: FLOW_TYPES[4],
  waitMs: FLOW_TYPES[5],
  link: FLOW_TYPES[6],
  addPolicies: FLOW_TYPES[7],
  handlePolicySet: FLOW_TYPES[8],
} as const;

export const DATA_TYPES = ['setVar', 'getVar', 'filter'] as const;
export const DATA_TYPE = {
  setVar: DATA_TYPES[0],
  getVar: DATA_TYPES[1],
  filter: DATA_TYPES[2],
} as const;

export const VISION_TYPES = ['visionSearch'] as const;
export const VISION_TYPE = {
  visionSearch: VISION_TYPES[0],
} as const;
