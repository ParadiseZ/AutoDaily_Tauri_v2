import type { Step } from '@/types/bindings';
import type { VisionNode } from '@/types/bindings/VisionNode';
import {
  ACTION_MODE,
  ACTION_TYPE,
  COLOR_COMPARE_METHOD_TYPE,
  CONDITION_TYPE,
  createColorCompareMethod,
  createFilterMode,
  createRegionPoint,
  createSearchRuleList,
  createStateStatus,
  createStateTarget,
  createStateTargetList,
  createStepList,
  createStringList,
  DATA_TYPE,
  FLOW_TYPE,
  LOGIC_OP,
  SEARCH_RULE_TYPE,
  SEARCH_SCOPE,
  STATE_STATUS_TYPE,
  STATE_TARGET_TYPE,
  STEP_OP,
  TASK_CONTROL_TYPE,
  VISION_TYPE,
} from '@/views/script-editor/editor-step/editorStepKinds';
import { buildVarValue } from '@/views/script-editor/editorVarValue';

export interface EditorStepTemplate {
  id: string;
  icon: string;
  label: string;
  description: string;
  group: string;
  create: () => Step;
}

export const ACTION_SEQUENCE_TEMPLATE_IDS = [
  'launch-app',
  'stop-app',
  'click-point',
  'click-percent',
  'swipe-point',
  'swipe-percent',
  'back',
  'wait',
] as const;

export const isActionSequenceTemplateId = (templateId: string) =>
  ACTION_SEQUENCE_TEMPLATE_IDS.includes(templateId as (typeof ACTION_SEQUENCE_TEMPLATE_IDS)[number]);

const castStep = (value: unknown) => value as Step;

const createBaseStep = (partial: Record<string, unknown>) =>
  castStep({
    id: null,
    source_id: null,
    target_id: null,
    label: null,
    skip_flag: false,
    ...partial,
  });

const genSvg= (svg_name:string)=>{
    return '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">'
        + svg_name
        + '</svg>'
}

const SVG_ICONS = {
  capture: '<path d="M14.5 4h-5L7 7H4a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2V9a2 2 0 0 0-2-2h-3l-2.5-3z"/><circle cx="12" cy="13" r="3"/>',
  launch: '<rect x="3" y="3" width="18" height="18" rx="4"/><path d="M10 8l6 4-6 4v-8z"/>',
  stop: '<rect x="3" y="3" width="18" height="18" rx="4"/><rect x="9" y="9" width="6" height="6" rx="1"/>',
  click: '<path d="M4 4l7.07 17 2.51-7.39L21 11.07z"/>',
  swipe: '<path d="M5 12h14"/><path d="M12 5l7 7-7 7"/>',
  back: '<path d="M15 18l-6-6 6-6"/>',
  add: '<circle cx="12" cy="12" r="10"/><path d="M12 8v8"/><path d="M8 12h8"/>',
  minus: '<circle cx="12" cy="12" r="10"/><path d="M8 12h8"/>',
  next: '<path d="M6 9l6 6 6-6"/>',
  wait: '<circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/>',
  branch: '<path d="M6 3v12"/><circle cx="18" cy="6" r="3"/><circle cx="6" cy="18" r="3"/><path d="M18 9a9 9 0 0 1-9 9"/>',
  loop: '<path d="M21 12a9 9 0 1 1-9-9c2.5 0 4.8 1 6.5 2.5L21 8"/><path d="M21 2v6h-6"/>',
  forEach: '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/>',
  repeat: '<path d="M17 1l4 4-4 4"/><path d="M3 11V9a4 4 0 0 1 4-4h14"/><path d="M7 23l-4-4 4-4"/><path d="M21 13v2a4 4 0 0 1-4 4H3"/>',
  continue: '<polygon points="5 4 15 12 5 20 5 4"/><line x1="19" y1="5" x2="19" y2="19"/>',
  break: '<path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/><path d="M16 17l5-5-5-5"/><path d="M21 12H9"/>',
  stopScript: '<circle cx="12" cy="12" r="10"/><rect x="8" y="8" width="8" height="8" rx="1"/>',
  policySet: '<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>',
  policyGroup: '<path d="M3 7h18"/><path d="M3 12h18"/><path d="M3 17h18"/><circle cx="7" cy="7" r="1"/><circle cx="7" cy="12" r="1"/><circle cx="7" cy="17" r="1"/>',
  policy: '<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/>',
  setVar: '<path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/>',
  getVar: '<path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="M7 10l5 5 5-5"/><path d="M12 15V3"/>',
  code: '<path d="M16 18l6-6-6-6"/><path d="M8 6l-6 6 6 6"/><path d="M14 4l-4 16"/>',
  filter: '<polygon points="22 3 2 3 10 12.46 10 19 14 21 14 12.46 22 3"/>',
  color: '<circle cx="13.5" cy="6.5" r=".5"/><circle cx="17.5" cy="10.5" r=".5"/><circle cx="8.5" cy="7.5" r=".5"/><circle cx="6.5" cy="12.5" r=".5"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.926 0 1.648-.746 1.648-1.688 0-.437-.18-.836-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"/>',
  detect: '<path d="M3 11h4"/><path d="M17 11h4"/><path d="M11 3v4"/><path d="M11 17v4"/><rect x="6" y="6" width="10" height="10" rx="2"/>',
  ocr: '<path d="M4 6h16"/><path d="M4 12h10"/><path d="M4 18h7"/><path d="M18 12l2 2-4 4"/>',
  count: '<path d="M9 7h11"/><path d="M9 12h11"/><path d="M9 17h11"/><path d="M4 7h.01"/><path d="M4 12h.01"/><path d="M4 17h.01"/>',
  search: '<circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/>',
  state: '<path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><path d="M22 4L12 14.01l-3-3"/>',
  link: '<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/>',
  sequence: '<path d="M8 6h13"/><path d="M8 12h13"/><path d="M8 18h13"/><path d="M3 6h.01"/><path d="M3 12h.01"/><path d="M3 18h.01"/>'
};

export const editorStepTemplates: EditorStepTemplate[] = [
  {
    id: 'capture',
    icon: genSvg(SVG_ICONS.capture),
    label: '截图',
    description: '将当前画面写入变量，常用于后续视觉识别。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '截图',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.capture,
          output_var: 'latest_capture',
        },
      }),
  },
  {
    id: 'launch-app',
    icon: genSvg(SVG_ICONS.launch),
    label: '启动应用',
    description: '使用包名和 Activity 启动应用，适合作为任务开头。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '启动应用',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.launchApp,
          pkg_name: '',
          pkg_name_expr: null,
          activity_name: '',
          activity_name_expr: null,
        },
      }),
  },
  {
    id: 'stop-app',
    icon: genSvg(SVG_ICONS.stop),
    label: '停止应用',
    description: '主动停止目标包名，适合切换账号或重置状态。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '停止应用',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.stopApp,
          pkg_name: '',
          pkg_name_expr: null,
        },
      }),
  },
  {
    id: 'click-point',
    icon: genSvg(SVG_ICONS.click),
    label: '点击坐标',
    description: '按绝对坐标点击，适合固定布局。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '点击坐标',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.click,
          offset_x: 0,
          offset_y: 0,
          mode: ACTION_MODE.point,
          p: { x: 640, y: 360 },
          p_expr: null,
        },
      }),
  },
  {
    id: 'click-percent',
    icon: genSvg(SVG_ICONS.click),
    label: '点击百分比',
    description: '按相对坐标点击，适合多分辨率脚本。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '点击百分比',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.click,
          offset_x: 0,
          offset_y: 0,
          mode: ACTION_MODE.percent,
          p: { x: 0.5, y: 0.5 },
          p_expr: null,
        },
      }),
  },
  {
    id: 'click-text',
    icon: genSvg(SVG_ICONS.click),
    label: '点击文字',
    description: '搜索 OCR 文本后点击对应区域。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '点击文字',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.click,
          offset_x: 0,
          offset_y: 0,
          mode: ACTION_MODE.txt,
          input_var: 'runtime.searchHits',
          txt: '开始',
          enable_filter: true,
        },
      }),
  },
  {
    id: 'click-label',
    icon: genSvg(SVG_ICONS.click),
    label: '点击标签',
    description: '根据视觉标签索引点击对应目标。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '点击标签',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.click,
          offset_x: 0,
          offset_y: 0,
          mode: ACTION_MODE.labelIdx,
          input_var: 'runtime.detResults',
          idx: 0,
          idx_expr: null,
          enable_filter: true,
        },
      }),
  },
  {
    id: 'swipe-point',
    icon: genSvg(SVG_ICONS.swipe),
    label: '滑动坐标',
    description: '按绝对坐标执行滑动。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '滑动坐标',
        op: STEP_OP.action,
        exec_max: 0,
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
    icon: genSvg(SVG_ICONS.swipe),
    label: '滑动百分比',
    description: '按相对坐标执行滑动。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '滑动百分比',
        op: STEP_OP.action,
        exec_max: 0,
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
    icon: genSvg(SVG_ICONS.swipe),
    label: '滑动文字',
    description: '按 OCR 文字起止点执行滑动。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '滑动文字',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.swipe,
          mode: ACTION_MODE.txt,
          duration: 300,
          input_var: 'runtime.ocrResults',
          from: '开始',
          to: '结束',
        },
      }),
  },
  {
    id: 'swipe-label',
    icon: genSvg(SVG_ICONS.swipe),
    label: '滑动标签',
    description: '按视觉标签索引起止点执行滑动。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '滑动标签',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.swipe,
          mode: ACTION_MODE.labelIdx,
          duration: 300,
          input_var: 'runtime.detResults',
          from: 0,
          to: 1,
        },
      }),
  },
  {
    id: 'swipe-label-to-text',
    icon: genSvg(SVG_ICONS.swipe),
    label: '标签滑到文字',
    description: '以检测标签为起点、OCR 文字为终点执行滑动。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '标签滑到文字',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.swipe,
          mode: ACTION_MODE.mixed,
          duration: 300,
          from: {
            source: ACTION_MODE.labelIdx,
            input_var: 'runtime.detResults',
            idx: 0,
          },
          to: {
            source: ACTION_MODE.txt,
            input_var: 'runtime.ocrResults',
            value: '结束',
            value_expr: null,
          },
        },
      }),
  },
  {
    id: 'back',
    icon: genSvg(SVG_ICONS.back),
    label: '返回',
    description: '执行 Android 返回键动作。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '返回',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.back,
        },
      }),
  },
  {
    id: 'pos-add',
    icon: genSvg(SVG_ICONS.add),
    label: '点击索引加一',
    description: '运行时把指定策略的当前位置加一，用于多个相同目标时改下次点击命中。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '点击索引加一',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.posAdd,
          target: '',
        },
      }),
  },
  {
    id: 'pos-minus',
    icon: genSvg(SVG_ICONS.minus),
    label: '点击索引减一',
    description: '运行时把指定策略的当前位置减一，最低回到 0，不持久化。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: '点击索引减一',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.posMinus,
          target: '',
        },
      }),
  },
  {
    id: 'drop-set-next',
    icon: genSvg(SVG_ICONS.next),
    label: 'UI 变量下一个',
    description: '把指定任务的 Select/Radio 变量切换到下一个选项并写回模板变量。',
    group: '动作',
    create: () =>
      createBaseStep({
        label: 'UI 变量下一个',
        op: STEP_OP.action,
        exec_max: 0,
        a: {
          ac: ACTION_TYPE.dropSetNext,
          task: '',
          variable_id: '',
        },
      }),
  },
  {
    id: 'wait',
    icon: genSvg(SVG_ICONS.wait),
    label: '等待',
    description: '在关键跳转后留出稳定时间。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '等待',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.waitMs,
          ms: 1000,
          input_var: null,
          runtime_var: null,
        },
      }),
  },
  {
    id: 'if',
    icon: genSvg(SVG_ICONS.branch),
    label: 'if',//条件分支
    description: '创建一段最小化的条件判断骨架。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: 'if',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.if,
          con: {
            type: CONDITION_TYPE.rawExpr,
            expr: 'true',
          },
          then: createStepList(),
          else_steps: null,
        },
      }),
  },
  {
    id: 'while',
    icon: genSvg(SVG_ICONS.loop),
    label: 'while',//循环条件
    description: '根据 ConditionNode 条件循环执行子步骤。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: 'while',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.while,
          con: {
            type: CONDITION_TYPE.rawExpr,
            expr: 'true',
          },
          flow: createStepList(),
        },
      }),
  },
  {
    id: 'for-each',
    icon: genSvg(SVG_ICONS.forEach),
    label: 'forEach',//遍历循环
    description: '遍历输入变量中的数组元素，并把当前元素映射到运行时变量。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: 'forEach',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.forEach,
          input_var: 'runtime.items',
          item_var: 'runtime.item',
          index_var: 'runtime.itemIndex',
          flow: createStepList(),
        },
      }),
  },
  {
    id: 'repeat',
    icon: genSvg(SVG_ICONS.repeat),
    label: 'for',//次数循环
    description: '按绑定的数字变量重复执行子步骤，适合用 UI 变量控制执行次数。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '次数循环',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.repeat,
          count_expr: '',
          index_var: 'runtime.repeatIndex',
          flow: createStepList(),
        },
      }),
  },
  {
    id: 'continue',
    icon: genSvg(SVG_ICONS.continue),
    label: 'continue',//继续循环
    description: '在循环内部跳过后续步骤并继续下一轮。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: 'continue',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.continue,
        },
      }),
  },
  {
    id: 'break',
    icon: genSvg(SVG_ICONS.break),
    label: 'break',//break
    description: '在循环内部立即结束当前循环。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: 'break',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.break,
        },
      }),
  },
  {
    id: 'stop-script',
    icon: genSvg(SVG_ICONS.stopScript),
    label: '跳过脚本',
    description: '立即结束当前脚本执行，不继续后续任务迭代。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '跳过脚本',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.stopScript,
        },
      }),
  },
  {
    id: 'add-policies',
    icon: genSvg(SVG_ICONS.policySet),
    label: '追加策略集',
    description: '把源策略集里的策略组按顺序追加到目标策略集。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '追加策略集',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.addPolicies,
          source: '',
          target: '',
          top: false,
          reverse: false,
        },
      }),
  },
  {
    id: 'remove-policies',
    icon: genSvg(SVG_ICONS.policySet),
    label: '移除策略集',
    description: '从目标策略集中移除运行时追加的源策略集。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '移除策略集',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.removePolicies,
          source: '',
          target: '',
        },
      }),
  },
  {
    id: 'bind-policy-group',
    icon: genSvg(SVG_ICONS.policySet),
    label: '绑定策略组',
    description: '把一个策略组绑定到目标策略集，可控制顶部插入和逆序。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '绑定策略组',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.bindPolicyGroup,
          source: '',
          target: '',
          top: false,
          reverse: false,
        },
      }),
  },
  {
    id: 'remove-policy-group',
    icon: genSvg(SVG_ICONS.policySet),
    label: '移除策略组',
    description: '从目标策略集中移除运行时绑定的源策略组。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '移除策略组',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.removePolicyGroup,
          source: '',
          target: '',
        },
      }),
  },
  {
    id: 'add-policy-groups',
    icon: genSvg(SVG_ICONS.policyGroup),
    label: '追加策略组',
    description: '把一个策略组里的策略插入目标策略组，可控制顶部插入和逆序。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '追加策略组',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.addPolicyGroups,
          source: '',
          target: '',
          top: false,
          reverse: false,
        },
      }),
  },
  {
    id: 'unload-policy-group',
    icon: genSvg(SVG_ICONS.policyGroup),
    label: '卸载策略组',
    description: '从目标策略组中移除运行时追加的源策略组。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '卸载策略组',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.unloadPolicyGroup,
          source: '',
          target: '',
        },
      }),
  },
  {
    id: 'bind-policy',
    icon: genSvg(SVG_ICONS.policy),
    label: '绑定策略',
    description: '把一个策略绑定到目标策略组，可控制顶部插入和逆序。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '绑定策略',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.bindPolicy,
          source: '',
          target: '',
          top: false,
          reverse: false,
        },
      }),
  },
  {
    id: 'unload-policy',
    icon: genSvg(SVG_ICONS.policy),
    label: '卸载策略',
    description: '从目标策略组中移除运行时绑定的源策略。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '卸载策略',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.unloadPolicy,
          source: '',
          target: '',
        },
      }),
  },
  {
    id: 'handle-policy-set',
    icon: genSvg(SVG_ICONS.policySet),
    label: '处理策略集',
    description: '使用 DET / OCR 结果集执行策略集匹配，并把命中结果输出为 JSON 变量。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '处理策略集',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.handlePolicySet,
          target: createStringList(),
          det_input_var: 'runtime.detResults',
          ocr_input_var: 'runtime.ocrResults',
          search_hits_var: 'runtime.searchHits',
          out_var: 'runtime.policySetResult',
        },
      }),
  },
  {
    id: 'handle-policy',
    icon: genSvg(SVG_ICONS.policy),
    label: '处理策略',
    description: '执行指定策略并把结果输出为 JSON 变量。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '处理策略',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.handlePolicy,
          target: createStringList(),
          input_var: 'runtime.latestCapture',
          out_var: 'runtime.policyResult',
        },
      }),
  },
  {
    id: 'set-var',
    icon: genSvg(SVG_ICONS.setVar),
    label: '设置变量',
    description: '给已声明变量写入固定值。',
    group: '数据',
    create: () =>
      createBaseStep({
        label: '设置变量',
        op: STEP_OP.dataHanding,
        a: {
          type: DATA_TYPE.setVar,
          name: '',
          val: buildVarValue({
            kind: 'int',
            textValue: '0',
            boolValue: false,
          }),
          json_val: null,
          expr: null,
        },
      }),
  },
  {
    id: 'clear-vars',
    icon: genSvg(SVG_ICONS.minus),
    label: '清空变量',
    description: '按变量类型写入空值或移除图像引用。',
    group: '数据',
    create: () =>
      createBaseStep({
        label: '清空变量',
        op: STEP_OP.dataHanding,
        a: {
          type: DATA_TYPE.clearVars,
          names: createStringList(),
        },
      }),
  },
  {
    id: 'get-var',
    icon: genSvg(SVG_ICONS.getVar),
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
    id: 'rhai',
    icon: genSvg(SVG_ICONS.code),
    label: 'Rhai 代码',
    description: '执行一段预编译的 Rhai 代码块，可选写回输出变量。',
    group: '数据',
    create: () =>
      createBaseStep({
        label: 'Rhai 代码',
        op: STEP_OP.dataHanding,
        a: {
          type: DATA_TYPE.rhai,
          code: '// 直接读取 input / runtime\nruntime.value ?? 0',
          out_var: null,
        },
      }),
  },
  {
    id: 'filter-var',
    icon: genSvg(SVG_ICONS.filter),
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
          mode: createFilterMode(),
          logic_expr: 'true',
          region_top_left: createRegionPoint(),
          region_bottom_right: createRegionPoint(),
          then_steps: createStepList(),
        },
      }),
  },
  {
    id: 'color-compare',
    icon: genSvg(SVG_ICONS.color),
    label: '颜色筛选',
    description: '按 OCR 结果区域筛选字体色或背景色，并输出命中结果集。',
    group: '数据',
    create: () =>
      createBaseStep({
        label: '颜色筛选',
        op: STEP_OP.dataHanding,
        a: {
          type: DATA_TYPE.colorCompare,
          input_var: 'runtime.ocrResults',
          out_var: 'runtime.colorMatchedResults',
          target_text: null,
          is_font: true,
          target_color: {
            r: 255,
            g: 255,
            b: 255,
          },
          method: createColorCompareMethod(COLOR_COMPARE_METHOD_TYPE.oklabDistance, 0.05),
          region_top_left: createRegionPoint(),
          region_bottom_right: createRegionPoint(),
          then_steps: createStepList(),
        },
      }),
  },
  {
    id: 'vision-detect',
    icon: genSvg(SVG_ICONS.detect),
    label: '目标检测',
    description: '对截图变量执行目标检测，并输出检测结果。', 
    group: '视觉',
    create: () =>
      createBaseStep({
        label: '目标检测',
        op: STEP_OP.vision,
        a: {
          type: VISION_TYPE.detect,
          input_var: 'runtime.captureResult',
          out_var: 'runtime.detResults',
        },
      }),
  },
  {
    id: 'vision-ocr',
    icon: genSvg(SVG_ICONS.ocr),
    label: 'OCR',
    description: '对截图变量执行 OCR，并输出文字识别结果。',
    group: '视觉',
    create: () =>
      createBaseStep({
        label: 'OCR',
        op: STEP_OP.vision,
        a: {
          type: VISION_TYPE.ocr,
          input_var: 'runtime.captureResult',
          out_var: 'runtime.ocrResults',
        },
      }),
  },
  {
    id: 'vision-search',
    icon: genSvg(SVG_ICONS.search),
    label: '视觉搜索',
    description: '基于 OCR / YOLO 规则搜索目标并输出结果变量。',
    group: '视觉',
    create: () =>
      createBaseStep({
        label: '视觉搜索',
        op: STEP_OP.vision,
        a: {
          type: VISION_TYPE.visionSearch,
          det_res_var: null,
          ocr_res_var: null,
          rule: {
            type: SEARCH_RULE_TYPE.group,
            op: LOGIC_OP.And,
            scope: SEARCH_SCOPE.Global,
            items: createSearchRuleList(),
          },
          out_var: 'runtime.searchHits',
          out_det_var: null,
          out_ocr_var: null,
          then_steps: createStepList(),
        },
      }),
  },
  {
    id: 'set-task-state',
    icon: genSvg(SVG_ICONS.state),
    label: '设置状态',
    description: '设置任务或策略的 skip/done 状态。',
    group: '状态',
    create: () =>
      createBaseStep({
        label: '设置状态',
        op: STEP_OP.taskControl,
        a: {
          type: TASK_CONTROL_TYPE.setState,
          target: createStateTarget(STATE_TARGET_TYPE.task),
          targets: createStateTargetList(),
          status: createStateStatus(STATE_STATUS_TYPE.done, true),
        },
      }),
  },
  {
    id: 'link-task',
    icon: genSvg(SVG_ICONS.link),
    label: '跳转任务',
    description: '将执行流切换到另一个任务。',
    group: '流程',
    create: () =>
      createBaseStep({
        label: '跳转任务',
        op: STEP_OP.flowControl,
        a: {
          type: FLOW_TYPE.link,
          target: '',
        },
      }),
  },
  {
    id: 'sequence',
    icon: genSvg(SVG_ICONS.sequence),
    label: '动作序列',
    description: '将一组设备动作与显式等待合并为紧凑执行序列。',
    group: '容器',
    create: () =>
      createBaseStep({
        label: '动作序列',
        op: STEP_OP.sequence,
        steps: createStepList(),
      }),
  },
];

export const createStepFromTemplate = (templateId: string) =>
  editorStepTemplates.find((template) => template.id === templateId)?.create() ?? null;

export const describeStepTitle = (step: Step) => {
  const visionStep = step.op === STEP_OP.vision ? (step.a as VisionNode) : null;

  if (step.op === STEP_OP.sequence) {
    return '动作序列';
  }

  if (step.op === STEP_OP.action) {
    if (step.a.ac === ACTION_TYPE.capture) return '截图';
    if (step.a.ac === ACTION_TYPE.launchApp) return '启动应用';
    if (step.a.ac === ACTION_TYPE.stopApp) return '停止应用';
    if (step.a.ac === ACTION_TYPE.reboot) return '重启应用';
    if (step.a.ac === ACTION_TYPE.back) return '返回';
    if (step.a.ac === ACTION_TYPE.posAdd) return '点击索引加一';
    if (step.a.ac === ACTION_TYPE.posMinus) return '点击索引减一';
    if (step.a.ac === ACTION_TYPE.dropSetNext) return 'UI 变量下一个';
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
      if (step.a.mode === ACTION_MODE.mixed) return '混合目标滑动';
      return '滑动坐标';
    }
    return '动作';
  }

  if (step.op === STEP_OP.flowControl) {
    if (step.a.type === FLOW_TYPE.waitMs) return '等待';
    if (step.a.type === FLOW_TYPE.link) return '跳转任务';
    if (step.a.type === FLOW_TYPE.addPolicies) return '追加策略集';
    if (step.a.type === FLOW_TYPE.removePolicies) return '移除策略集';
    if (step.a.type === FLOW_TYPE.bindPolicyGroup) return '绑定策略组';
    if (step.a.type === FLOW_TYPE.removePolicyGroup) return '移除策略组';
    if (step.a.type === FLOW_TYPE.addPolicyGroups) return '追加策略组';
    if (step.a.type === FLOW_TYPE.unloadPolicyGroup) return '卸载策略组';
    if (step.a.type === FLOW_TYPE.bindPolicy) return '绑定策略';
    if (step.a.type === FLOW_TYPE.unloadPolicy) return '卸载策略';
    if (step.a.type === FLOW_TYPE.handlePolicySet) return '处理策略集';
    if (step.a.type === FLOW_TYPE.handlePolicy) return '处理策略';
    if (step.a.type === FLOW_TYPE.if) return '条件分支';
    if (step.a.type === FLOW_TYPE.while) return 'While';
    if (step.a.type === FLOW_TYPE.forEach) return '遍历循环';
    if (step.a.type === FLOW_TYPE.repeat) return '次数循环';
    if (step.a.type === FLOW_TYPE.continue) return '继续循环';
    if (step.a.type === FLOW_TYPE.break) return '跳出循环';
    if (step.a.type === FLOW_TYPE.stopScript) return '跳过脚本';
    return '流程控制';
  }

  if (step.op === STEP_OP.dataHanding) {
    if (step.a.type === DATA_TYPE.setVar) return '设置变量';
    if (step.a.type === DATA_TYPE.clearVars) return '清空变量';
    if (step.a.type === DATA_TYPE.getVar) return '读取变量';
    if (step.a.type === DATA_TYPE.rhai) return 'Rhai 代码';
    if (step.a.type === DATA_TYPE.filter) return '过滤变量';
    if (step.a.type === DATA_TYPE.colorCompare) return '颜色筛选';
    if (step.a.type === DATA_TYPE.relativeFilter) return '相对位置筛选';
    return '数据处理';
  }

  if (step.op === STEP_OP.taskControl) {
    if (step.a.type === TASK_CONTROL_TYPE.setState) return '设置状态';
    return '状态控制';
  }

  if (visionStep) {
    if (visionStep.type === VISION_TYPE.detect) return '目标检测';
    if (visionStep.type === VISION_TYPE.ocr) return 'OCR';
    if (visionStep.type === VISION_TYPE.countCompare) return '判断数量大小';
    if (visionStep.type === VISION_TYPE.visionSearch) return '视觉搜索';
    return '视觉步骤';
  }

  return '步骤';
};

export const describeStepMeta = (step: Step) => {
  const visionStep = step.op === STEP_OP.vision ? (step.a as VisionNode) : null;

  if (step.op === STEP_OP.sequence) {
    return `动作序列 · ${step.steps.length} 个子步骤`;
  }

  if (step.op === STEP_OP.action) {
    if (step.a.ac === ACTION_TYPE.capture) return `截图写入 ${step.a.output_var}`;
    if (step.a.ac === ACTION_TYPE.launchApp) {
      const pkg = step.a.pkg_name_expr || step.a.pkg_name || '未指定包名';
      const activity = step.a.activity_name_expr || step.a.activity_name || '未指定 Activity';
      return `启动 ${pkg}/${activity}`;
    }
    if (step.a.ac === ACTION_TYPE.stopApp) return `停止 ${step.a.pkg_name_expr || step.a.pkg_name || '未指定包名'}`;
    if (step.a.ac === ACTION_TYPE.reboot) return '旧版重启应用动作，请改用停止应用 + 启动应用';
    if (step.a.ac === ACTION_TYPE.back) return 'Android 返回键';
    if (step.a.ac === ACTION_TYPE.posAdd) return `调整策略 ${step.a.target || '未指定'} · +1`;
    if (step.a.ac === ACTION_TYPE.posMinus) return `调整策略 ${step.a.target || '未指定'} · -1`;
    if (step.a.ac === ACTION_TYPE.dropSetNext) return `任务 ${step.a.task || '未指定'} · 变量 ${step.a.variable_id || '未指定'}`;
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
      case FLOW_TYPE.forEach:
        return `遍历 ${step.a.input_var || '未指定输入'} -> ${step.a.item_var || '未指定元素变量'}`;
      case FLOW_TYPE.repeat:
        return step.a.count_expr ? `按 ${step.a.count_expr} 循环` : '绑定次数变量';
      case FLOW_TYPE.waitMs:
        if (step.a.runtime_var) {
          return `等待 OCR 倒计时 · ${step.a.runtime_var} · 兜底 ${String(step.a.ms)} ms`;
        }
        if (step.a.input_var) {
          return `等待输入变量 · ${step.a.input_var} · 兜底 ${String(step.a.ms)} ms`;
        }
        return `等待 ${String(step.a.ms)} ms`;
      case FLOW_TYPE.link:
        return `跳转➡️[${step.a.target || '未指定'}]`;
      case FLOW_TYPE.addPolicies:
        return `策略集 ${step.a.source || '未指定'} -> ${step.a.target || '未指定'}${step.a.top ? ' · 顶部插入' : ' · 底部插入'}${step.a.reverse ? ' · 逆序' : ''}`;
      case FLOW_TYPE.removePolicies:
        return `移除策略集 ${step.a.source || '未指定'} -> ${step.a.target || '未指定'}`;
      case FLOW_TYPE.bindPolicyGroup:
        return `策略组 ${step.a.source || '未指定'} -> 策略集 ${step.a.target || '未指定'}${step.a.top ? ' · 顶部插入' : ' · 底部插入'}${step.a.reverse ? ' · 逆序' : ''}`;
      case FLOW_TYPE.removePolicyGroup:
        return `移除策略组 ${step.a.source || '未指定'} -> 策略集 ${step.a.target || '未指定'}`;
      case FLOW_TYPE.addPolicyGroups:
        return `策略组 ${step.a.source || '未指定'} -> 策略组 ${step.a.target || '未指定'}${step.a.top ? ' · 顶部插入' : ' · 底部插入'}${step.a.reverse ? ' · 逆序' : ''}`;
      case FLOW_TYPE.unloadPolicyGroup:
        return `卸载策略组 ${step.a.source || '未指定'} -> 策略组 ${step.a.target || '未指定'}`;
      case FLOW_TYPE.bindPolicy:
        return `策略 ${step.a.source || '未指定'} -> 策略组 ${step.a.target || '未指定'}${step.a.top ? ' · 顶部插入' : ' · 底部插入'}${step.a.reverse ? ' · 逆序' : ''}`;
      case FLOW_TYPE.unloadPolicy:
        return `卸载策略 ${step.a.source || '未指定'} -> 策略组 ${step.a.target || '未指定'}`;
      case FLOW_TYPE.continue:
        return '继续下一轮循环';
      case FLOW_TYPE.break:
        return '跳出当前循环';
      case FLOW_TYPE.stopScript:
        return '结束当前脚本执行';
      case FLOW_TYPE.handlePolicySet:
        return `处理 ${step.a.target.length} 个策略集 · DET ${step.a.det_input_var || '未指定'} · OCR ${step.a.ocr_input_var || '未指定'} · Hits ${step.a.search_hits_var || '未指定'} -> ${step.a.out_var || '未指定输出'}`;
      case FLOW_TYPE.handlePolicy:
        return `处理 ${step.a.target.length} 个策略 · ${step.a.input_var || '未指定输入'} -> ${step.a.out_var || '未指定输出'}`;
      default:
        return '流程控制';
    }
  }

  if (step.op === STEP_OP.dataHanding) {
    switch (step.a.type) {
      case DATA_TYPE.setVar:
        return `写入变量 ${step.a.name || '未命名变量'}`;
      case DATA_TYPE.clearVars:
        return step.a.names.length ? `清空 ${step.a.names.length} 个变量` : '清空变量列表';
      case DATA_TYPE.getVar:
        return `读取变量 ${step.a.name || '未命名变量'}`;
      case DATA_TYPE.rhai:
        return step.a.out_var?.trim()
          ? `Rhai 代码 -> ${step.a.out_var.trim()}`
          : 'Rhai 代码块';
      case DATA_TYPE.filter:
        return `过滤 ${step.a.input_var || '未命名输入'} -> ${step.a.out_name || '未命名输出'}`;
      case DATA_TYPE.colorCompare:
        return `颜色筛选 ${step.a.input_var || '未命名输入'} -> ${step.a.out_var || '未命名输出'}`;
      case DATA_TYPE.relativeFilter:
        return `相对位置 ${step.a.input_var || '未命名输入'} -> ${step.a.out_var || '未命名输出'}`;
      default:
        return '数据处理';
    }
  }

  if (visionStep) {
    if (visionStep.type === VISION_TYPE.detect) {
      return `目标检测 ${visionStep.input_var || '未命名输入'} -> ${visionStep.out_var || '未命名输出'}`;
    }
    if (visionStep.type === VISION_TYPE.ocr) {
      return `OCR ${visionStep.input_var || '未命名输入'} -> ${visionStep.out_var || '未命名输出'}`;
    }
    if (visionStep.type === VISION_TYPE.countCompare) {
      return `数量比较 ${visionStep.input_var || '未命名输入'} -> ${visionStep.out_var || '未命名输出'}`;
    }
    return `视觉搜索 -> ${visionStep.out_var || '未命名输出'}`;
  }

  if (step.op === STEP_OP.taskControl) {
    return `设置状态 · ${step.a.target.type}:${step.a.target.id || '未指定'}`;
  }

  return '未识别步骤';
};

export const describeStep = (step: Step) => step.label?.trim() || describeStepTitle(step);

