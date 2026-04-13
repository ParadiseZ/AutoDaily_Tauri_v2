import type { Step } from '@/types/bindings';
import {
  ACTION_MODE,
  ACTION_TYPE,
  DATA_TYPE,
  FILTER_MODE_TYPE,
  FLOW_TYPE,
  STATE_STATUS_TYPE,
  STATE_TARGET_TYPE,
  STEP_OP,
  TASK_CONTROL_TYPE,
  VISION_TYPE,
} from '@/views/script-editor/editor-step/editorStepKinds';

export interface EditorStepTemplate {
  id: string;
  label: string;
  description: string;
  group: string;
  create: () => Step;
}

const castStep = (value: unknown) => value as Step;

const createBaseStep = (partial: Record<string, unknown>) =>
  castStep({
    id: null,
    source_id: null,
    target_id: null,
    label: null,
    skip_flag: false,
    exec_cur: 0,
    exec_max: 1,
    ...partial,
  });

export const editorStepTemplates: EditorStepTemplate[] = [
  {
    id: 'capture',
    label: '截图',
    description: '将当前画面写入变量，常用于后续视觉识别。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '截图',
        op: STEP_OP.action,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          ac: ACTION_TYPE.capture,
          output_var: 'latest_capture',
        },
      }),
  },
  {
    id: 'launch-app',
    label: '启动应用',
    description: '使用包名和 Activity 启动应用，适合作为任务开头。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '启动应用',
        op: STEP_OP.action,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          ac: ACTION_TYPE.launchApp,
          pkg_name: '',
          activity_name: '',
        },
      }),
  },
  {
    id: 'stop-app',
    label: '停止应用',
    description: '主动停止目标包名，适合切换账号或重置状态。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '停止应用',
        op: STEP_OP.action,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          ac: ACTION_TYPE.stopApp,
          pkg_name: '',
        },
      }),
  },
  {
    id: 'click-point',
    label: '点击坐标',
    description: '按绝对坐标点击，适合固定布局。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '点击坐标',
        op: STEP_OP.action,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          ac: ACTION_TYPE.click,
          mode: ACTION_MODE.point,
          p: { x: 640, y: 360 },
        },
      }),
  },
  {
    id: 'click-percent',
    label: '点击百分比',
    description: '按相对坐标点击，适合多分辨率脚本。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '点击百分比',
        op: STEP_OP.action,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          ac: ACTION_TYPE.click,
          mode: ACTION_MODE.percent,
          p: { x: 0.5, y: 0.5 },
        },
      }),
  },
  {
    id: 'click-text',
    label: '点击文字',
    description: '搜索 OCR 文本后点击对应区域。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '点击文字',
        op: STEP_OP.action,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          ac: ACTION_TYPE.click,
          mode: ACTION_MODE.txt,
          txt: '开始',
        },
      }),
  },
  {
    id: 'click-label',
    label: '点击标签',
    description: '根据视觉标签索引点击对应目标。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '点击标签',
        op: STEP_OP.action,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          ac: ACTION_TYPE.click,
          mode: ACTION_MODE.labelIdx,
          idx: 0,
        },
      }),
  },
  {
    id: 'swipe-point',
    label: '滑动坐标',
    description: '按绝对坐标执行滑动。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '滑动坐标',
        op: STEP_OP.action,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          ac: ACTION_TYPE.swipe,
          mode: ACTION_MODE.point,
          duration: 300,
          from: { x: 640, y: 560 },
          to: { x: 640, y: 180 },
        },
      }),
  },
  {
    id: 'swipe-percent',
    label: '滑动百分比',
    description: '按相对坐标执行滑动。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '滑动百分比',
        op: STEP_OP.action,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          ac: ACTION_TYPE.swipe,
          mode: ACTION_MODE.percent,
          duration: 300,
          from: { x: 0.5, y: 0.75 },
          to: { x: 0.5, y: 0.25 },
        },
      }),
  },
  {
    id: 'swipe-text',
    label: '滑动文字',
    description: '按 OCR 文字起止点执行滑动。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '滑动文字',
        op: STEP_OP.action,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          ac: ACTION_TYPE.swipe,
          mode: ACTION_MODE.txt,
          duration: 300,
          from: '开始',
          to: '结束',
        },
      }),
  },
  {
    id: 'swipe-label',
    label: '滑动标签',
    description: '按视觉标签索引起止点执行滑动。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '滑动标签',
        op: STEP_OP.action,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          ac: ACTION_TYPE.swipe,
          mode: ACTION_MODE.labelIdx,
          duration: 300,
          from: 0,
          to: 1,
        },
      }),
  },
  {
    id: 'reboot',
    label: '重启设备',
    description: '执行设备重启动作。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '重启设备',
        op: STEP_OP.action,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          ac: ACTION_TYPE.reboot,
        },
      }),
  },
  {
    id: 'wait',
    label: '等待',
    description: '在关键跳转后留出稳定时间。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '等待',
        op: STEP_OP.flowControl,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          type: FLOW_TYPE.waitMs,
          ms: 1000,
        },
      }),
  },
  {
    id: 'if',
    label: '条件分支',
    description: '创建一段最小化的条件判断骨架。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '条件分支',
        op: STEP_OP.flowControl,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          type: FLOW_TYPE.if,
          con: {
            type: 'rawExpr',
            expr: 'true',
          },
          then: [],
          else_steps: null,
        },
      }),
  },
  {
    id: 'while',
    label: '循环条件',
    description: '根据 ConditionNode 条件循环执行子步骤。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '循环条件',
        op: STEP_OP.flowControl,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          type: FLOW_TYPE.while,
          con: {
            type: 'rawExpr',
            expr: 'true',
          },
          flow: [],
        },
      }),
  },
  {
    id: 'for',
    label: '遍历循环',
    description: '根据 ConditionNode 条件执行 for 流程。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '遍历循环',
        op: STEP_OP.flowControl,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          type: FLOW_TYPE.for,
          con: {
            type: 'rawExpr',
            expr: 'true',
          },
          flow: [],
        },
      }),
  },
  {
    id: 'continue',
    label: '继续循环',
    description: '在循环内部跳过后续步骤并继续下一轮。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '继续循环',
        op: STEP_OP.flowControl,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          type: FLOW_TYPE.continue,
        },
      }),
  },
  {
    id: 'break',
    label: '跳出循环',
    description: '在循环内部立即结束当前循环。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '跳出循环',
        op: STEP_OP.flowControl,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          type: FLOW_TYPE.break,
        },
      }),
  },
  {
    id: 'handle-policy-set',
    label: '处理策略集',
    description: '执行策略集匹配并把命中结果输出为 JSON 变量。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '处理策略集',
        op: STEP_OP.flowControl,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          type: FLOW_TYPE.handlePolicySet,
          target: [],
          input_var: 'runtime.latestCapture',
          out_var: 'runtime.policySetResult',
        },
      }),
  },
  {
    id: 'handle-policy',
    label: '处理策略',
    description: '执行指定策略并把结果输出为 JSON 变量。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '处理策略',
        op: STEP_OP.flowControl,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          type: FLOW_TYPE.handlePolicy,
          target: [],
          input_var: 'runtime.latestCapture',
          out_var: 'runtime.policyResult',
        },
      }),
  },
  {
    id: 'set-var',
    label: '设置变量',
    description: '向任务上下文写入一个变量。',
    group: '数据',
    create: () =>
      createBaseStep({
        label: '设置变量',
        op: STEP_OP.dataHanding,
        a: {
          type: DATA_TYPE.setVar,
          name: '',
          val: null,
          expr: null,
        },
      }),
  },
  {
    id: 'get-var',
    label: '读取变量',
    description: '从上下文读取变量并写回当前步骤使用。',
    group: '兼容',
    create: () =>
      createBaseStep({
        label: '读取变量',
        op: STEP_OP.dataHanding,
        a: {
          type: DATA_TYPE.getVar,
          name: '',
          default_val: null,
        },
      }),
  },
  {
    id: 'filter-var',
    label: '过滤变量',
    description: '对输入变量执行过滤或映射逻辑。',
    group: '数据',
    create: () =>
      createBaseStep({
        label: '过滤变量',
        op: STEP_OP.dataHanding,
        a: {
          type: DATA_TYPE.filter,
          input_var: 'items',
          out_name: 'filtered_items',
          mode: {
            type: FILTER_MODE_TYPE.filter,
          },
          logic_expr: 'true',
          then_steps: [],
        },
      }),
  },
  {
    id: 'vision-search',
    label: '视觉搜索',
    description: '基于 OCR / YOLO 规则搜索目标并输出结果变量。',
    group: '视觉',
    create: () =>
      createBaseStep({
        label: '视觉搜索',
        op: STEP_OP.vision,
        a: {
          type: VISION_TYPE.visionSearch,
          rule: {
            type: 'group',
            op: 'And',
            scope: 'Global',
            items: [],
          },
          out_var: 'vision_hit',
          then_steps: [],
        },
      }),
  },
  {
    id: 'set-task-state',
    label: '设置状态',
    description: '设置任务或策略的 skip/done 状态。',
    group: '状态',
    create: () =>
      createBaseStep({
        label: '设置状态',
        op: STEP_OP.taskControl,
        a: {
          type: TASK_CONTROL_TYPE.setState,
          target: {
            type: STATE_TARGET_TYPE.task,
            id: '',
          },
          status: {
            type: STATE_STATUS_TYPE.done,
            value: true,
          },
        },
      }),
  },
  {
    id: 'link-task',
    label: '跳转任务',
    description: '将执行流切换到另一个任务。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '跳转任务',
        op: STEP_OP.flowControl,
        cur_exec_num: 0,
        max_exec_num: 1,
        a: {
          type: FLOW_TYPE.link,
          target: '',
        },
      }),
  },
  {
    id: 'sequence',
    label: '子序列',
    description: '在当前任务内嵌套一段顺序步骤。',
    group: '容器',
    create: () =>
      createBaseStep({
        label: '子序列',
        op: STEP_OP.sequence,
        steps: [],
      }),
  },
];

export const createStepFromTemplate = (templateId: string) =>
  editorStepTemplates.find((template) => template.id === templateId)?.create() ?? null;

export const describeStepTitle = (step: Step) => {
  if (step.op === STEP_OP.sequence) {
    return '顺序步骤';
  }

  if (step.op === STEP_OP.action) {
    if (step.a.ac === ACTION_TYPE.capture) return '截图';
    if (step.a.ac === ACTION_TYPE.launchApp) return '启动应用';
    if (step.a.ac === ACTION_TYPE.stopApp) return '停止应用';
    if (step.a.ac === ACTION_TYPE.reboot) return '重启设备';
    if (step.a.ac === ACTION_TYPE.click) {
      if (step.a.mode === ACTION_MODE.percent) return '点击百分比';
      if (step.a.mode === ACTION_MODE.txt) return '点击文字';
      if (step.a.mode === ACTION_MODE.labelIdx) return '点击标签';
      return '点击坐标';
    }
    if (step.a.ac === ACTION_TYPE.swipe) {
      if (step.a.mode === ACTION_MODE.percent) return '滑动百分比';
      if (step.a.mode === ACTION_MODE.txt) return '滑动文字';
      if (step.a.mode === ACTION_MODE.labelIdx) return '滑动标签';
      return '滑动坐标';
    }
    return '动作';
  }

  if (step.op === STEP_OP.flowControl) {
    if (step.a.type === FLOW_TYPE.waitMs) return '等待';
    if (step.a.type === FLOW_TYPE.link) return '跳转任务';
    if (step.a.type === FLOW_TYPE.handlePolicySet) return '处理策略集';
    if (step.a.type === FLOW_TYPE.handlePolicy) return '处理策略';
    if (step.a.type === FLOW_TYPE.if) return '条件分支';
    if (step.a.type === FLOW_TYPE.while) return 'While';
    if (step.a.type === FLOW_TYPE.for) return 'For';
    if (step.a.type === FLOW_TYPE.continue) return '继续循环';
    if (step.a.type === FLOW_TYPE.break) return '跳出循环';
    return '流程控制';
  }

  if (step.op === STEP_OP.dataHanding) {
    if (step.a.type === DATA_TYPE.setVar) return '设置变量';
    if (step.a.type === DATA_TYPE.getVar) return '读取变量';
    if (step.a.type === DATA_TYPE.filter) return '过滤变量';
    return '数据处理';
  }

  if (step.op === STEP_OP.taskControl) {
    if (step.a.type === TASK_CONTROL_TYPE.setState) return '设置状态';
    if (step.a.type === TASK_CONTROL_TYPE.getState) return '读取状态';
    return '状态控制';
  }

  if (step.op === STEP_OP.vision) {
    if (step.a.type === VISION_TYPE.visionSearch) return '视觉搜索';
    return '视觉步骤';
  }

  return '步骤';
};

export const describeStepMeta = (step: Step) => {
  if (step.op === STEP_OP.sequence) {
    return `顺序容器 · ${step.steps.length} 个子步骤`;
  }

  if (step.op === STEP_OP.action) {
    if (step.a.ac === ACTION_TYPE.capture) return `截图写入 ${step.a.output_var}`;
    if (step.a.ac === ACTION_TYPE.launchApp) {
      return `启动 ${step.a.pkg_name || '未指定包名'}/${step.a.activity_name || '未指定 Activity'}`;
    }
    if (step.a.ac === ACTION_TYPE.stopApp) return `停止 ${step.a.pkg_name || '未指定包名'}`;
    if (step.a.ac === ACTION_TYPE.reboot) return '重启设备';
    if (step.a.ac === ACTION_TYPE.click) return `点击 · ${step.a.mode}`;
    if (step.a.ac === ACTION_TYPE.swipe) return `滑动 · ${step.a.mode}`;
    return '动作';
  }

  if (step.op === STEP_OP.flowControl) {
    switch (step.a.type) {
      case FLOW_TYPE.if:
        return `条件分支 · then ${step.a.then.length} 步`;
      case FLOW_TYPE.while:
        return `循环 · ${step.a.flow.length} 步`;
      case FLOW_TYPE.for:
        return `遍历 · ${step.a.flow.length} 步`;
      case FLOW_TYPE.waitMs:
        return `等待 ${String(step.a.ms)} ms`;
      case FLOW_TYPE.link:
        return `跳转到 ${step.a.target || '未指定任务'}`;
      case FLOW_TYPE.continue:
        return '继续下一轮循环';
      case FLOW_TYPE.break:
        return '跳出当前循环';
      case FLOW_TYPE.handlePolicySet:
        return `处理 ${step.a.target.length} 个策略集 · ${step.a.input_var || '未指定输入'} -> ${step.a.out_var || '未指定输出'}`;
      case FLOW_TYPE.handlePolicy:
        return `处理 ${step.a.target.length} 个策略 · ${step.a.input_var || '未指定输入'} -> ${step.a.out_var || '未指定输出'}`;
      case FLOW_TYPE.addPolicies:
        return `追加策略 ${step.a.source || '未命名'} -> ${step.a.target || '未命名'}`;
      default:
        return '流程控制';
    }
  }

  if (step.op === STEP_OP.dataHanding) {
    switch (step.a.type) {
      case DATA_TYPE.setVar:
        return `写入变量 ${step.a.name || '未命名变量'}`;
      case DATA_TYPE.getVar:
        return `读取变量 ${step.a.name || '未命名变量'}`;
      case DATA_TYPE.filter:
        return `过滤 ${step.a.input_var || '未命名输入'} -> ${step.a.out_name || '未命名输出'}`;
      default:
        return '数据处理';
    }
  }

  if (step.op === STEP_OP.vision) {
    return `视觉搜索 -> ${step.a.out_var || '未命名输出'}`;
  }

  if (step.op === STEP_OP.taskControl) {
    return `${step.a.type === TASK_CONTROL_TYPE.setState ? '设置' : '读取'}状态 · ${step.a.target.type}:${step.a.target.id || '未指定'}`;
  }

  return '未识别步骤';
};

export const describeStep = (step: Step) => step.label?.trim() || describeStepTitle(step);
