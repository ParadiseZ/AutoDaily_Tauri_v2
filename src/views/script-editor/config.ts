/**
 * Script Editor Configuration
 * 
 * 将所有配置集中于此文件，方便维护和修改。
 * 类型与后端 nodes/ 目录对齐，保持前后端数据一致。
 */

export interface NodeTypeConfig {
    color: string;
    icon: string;
    display: string;
    displayCn: string;
    category: string;
    placeholder: string;
    description: string;
}

// ============================================================================
// NODE_TYPES：每个 key 对应一种可在工具箱/行为选择器中独立选择的操作
// 按后端 Action/FlowControl/DataHanding/TaskControl/VisionNode 枚举对齐
// ============================================================================
export const NODE_TYPES: Record<string, NodeTypeConfig> = {
    // ─── 互动行为 (Action) ───────────────────────────
    clickPoint: {
        color: 'bg-blue-500', icon: 'cursor', display: 'Click Point', displayCn: '坐标点击',
        category: 'basic', placeholder: 'Set click point...', description: '按坐标点击',
    },
    clickPercent: {
        color: 'bg-blue-400', icon: 'percent', display: 'Click Percent', displayCn: '百分比点击',
        category: 'basic', placeholder: 'Set click percent...', description: '按百分比位置点击',
    },
    clickLabel: {
        color: 'bg-blue-600', icon: 'target', display: 'Click Label', displayCn: '标签点击',
        category: 'basic', placeholder: 'Set label index...', description: '按 YOLO 标签索引点击',
    },
    clickTxt: {
        color: 'bg-blue-700', icon: 'type', display: 'Click Text', displayCn: '文字点击',
        category: 'basic', placeholder: 'Set text target...', description: '按 OCR 匹配文字点击',
    },
    swipePoint: {
        color: 'bg-cyan-500', icon: 'move', display: 'Swipe Point', displayCn: '坐标滑动',
        category: 'basic', placeholder: 'Set swipe gesture...', description: '按坐标滑动',
    },
    swipePercent: {
        color: 'bg-cyan-400', icon: 'move', display: 'Swipe Percent', displayCn: '百分比滑动',
        category: 'basic', placeholder: 'Set swipe percent...', description: '按百分比滑动',
    },
    swipeLabel: {
        color: 'bg-cyan-600', icon: 'target', display: 'Swipe Label', displayCn: '标签滑动',
        category: 'basic', placeholder: 'Set label swipe...', description: '按标签索引滑动',
    },
    swipeTxt: {
        color: 'bg-cyan-700', icon: 'type', display: 'Swipe Text', displayCn: '文字滑动',
        category: 'basic', placeholder: 'Set text swipe...', description: '按文字匹配滑动',
    },
    takeScreenshot: {
        color: 'bg-slate-500', icon: 'camera', display: 'Screenshot', displayCn: '截图',
        category: 'basic', placeholder: 'Save to variable...', description: '截图保存到变量',
    },
    reboot: {
        color: 'bg-red-600', icon: 'power', display: 'Reboot', displayCn: '重启设备',
        category: 'basic', placeholder: '', description: '重启设备',
    },
    launchApp: {
        color: 'bg-green-500', icon: 'smartphone', display: 'Launch App', displayCn: '启动应用',
        category: 'basic', placeholder: 'Set package name...', description: '启动指定应用',
    },
    stopApp: {
        color: 'bg-orange-500', icon: 'square', display: 'Stop App', displayCn: '停止应用',
        category: 'basic', placeholder: 'Set package name...', description: '停止指定应用',
    },

    // ─── 控制流程 (FlowControl) ─────────────────────
    waitMs: {
        color: 'bg-gray-500', icon: 'clock', display: 'Wait (ms)', displayCn: '延时等待',
        category: 'flow', placeholder: 'Set wait duration...', description: '等待指定毫秒',
    },
    if: {
        color: 'bg-yellow-500', icon: 'branch', display: 'IF', displayCn: '条件分支',
        category: 'flow', placeholder: 'Set condition...', description: '条件分支',
    },
    while: {
        color: 'bg-yellow-600', icon: 'rotate-cw', display: 'While', displayCn: '循环控制',
        category: 'flow', placeholder: 'While condition...', description: '符合条件时循环',
    },
    for: {
        color: 'bg-yellow-700', icon: 'list', display: 'For', displayCn: '遍历循环',
        category: 'flow', placeholder: 'For each...', description: '遍历循环',
    },
    sequence: {
        color: 'bg-green-600', icon: 'layers', display: 'Sequence', displayCn: '序列容器',
        category: 'flow', placeholder: 'Step Sequence...', description: '子步骤容器',
    },
    continue: {
        color: 'bg-amber-500', icon: 'skip-forward', display: 'Continue', displayCn: '继续循环',
        category: 'flow', placeholder: '', description: '跳过本次循环',
    },
    break: {
        color: 'bg-amber-600', icon: 'octagon', display: 'Break', displayCn: '跳出循环',
        category: 'flow', placeholder: '', description: '跳出当前循环',
    },
    link: {
        color: 'bg-violet-500', icon: 'external-link', display: 'Link Task', displayCn: '任务跳转',
        category: 'flow', placeholder: 'Set target task...', description: '跳转到指定任务',
    },
    addPolicies: {
        color: 'bg-violet-600', icon: 'plus-circle', display: 'Add Policies', displayCn: '追加策略集',
        category: 'flow', placeholder: 'Set policy set...', description: '动态追加策略集',
    },
    handlePolicySet: {
        color: 'bg-violet-700', icon: 'play-circle', display: 'Handle Policy Set', displayCn: '处理策略集',
        category: 'flow', placeholder: 'Set policy sets...', description: '处理指定策略集',
    },

    // ─── 视觉与解析 (VisionNode) ────────────────────
    visionSearch: {
        color: 'bg-indigo-500', icon: 'zap', display: 'Vision Logic', displayCn: '强化视觉搜索',
        category: 'vision', placeholder: 'Set vision rules...', description: '增强视觉元素查找',
    },
    ocr: {
        color: 'bg-purple-600', icon: 'type', display: 'OCR Text', displayCn: '文字提取',
        category: 'vision', placeholder: 'Configure OCR...', description: 'OCR 文字提取',
    },

    // ─── 数据与状态 (DataHanding + TaskControl) ──────
    setVar: {
        color: 'bg-emerald-500', icon: 'variable', display: 'Set Variable', displayCn: '变量赋值',
        category: 'data', placeholder: 'Set var using expr...', description: '设置内存变量',
    },
    getVar: {
        color: 'bg-emerald-600', icon: 'terminal', display: 'Get Variable', displayCn: '变量读取',
        category: 'data', placeholder: 'Get var...', description: '读取内存变量',
    },
    filter: {
        color: 'bg-emerald-700', icon: 'filter', display: 'Filter', displayCn: '数据过滤',
        category: 'data', placeholder: 'Set filter...', description: '过滤/映射数据',
    },
    setState: {
        color: 'bg-teal-500', icon: 'settings', display: 'Set State', displayCn: '设置状态',
        category: 'data', placeholder: 'Set Target State...', description: '设置策略/任务状态',
    },
    getState: {
        color: 'bg-teal-600', icon: 'eye', display: 'Get State', displayCn: '读取状态',
        category: 'data', placeholder: 'Get Target State...', description: '读取策略/任务状态',
    },

    // ─── 特殊节点（仅画布使用, 行为选择器中不显示）────
    start: {
        color: 'bg-emerald-600', icon: 'play', display: 'Start', displayCn: '开始',
        category: 'special', placeholder: '开始', description: '开始节点',
    },
    end: {
        color: 'bg-rose-600', icon: 'square', display: 'End', displayCn: '结束',
        category: 'special', placeholder: '结束', description: '结束节点',
    },
};

// ============================================================================
// NODE_CATEGORIES：工具栏和行为选择器弹窗的分组配置
// ============================================================================
export const NODE_CATEGORIES = [
    {
        key: 'basic',
        label: '互动行为',
        labelEn: 'Interaction',
        types: [
            'clickPoint', 'clickPercent', 'clickLabel', 'clickTxt',
            'swipePoint', 'swipePercent', 'swipeLabel', 'swipeTxt',
            'takeScreenshot', 'reboot', 'launchApp', 'stopApp',
        ],
    },
    {
        key: 'vision',
        label: '视觉与解析',
        labelEn: 'Vision & Inference',
        types: ['visionSearch', 'ocr'],
    },
    {
        key: 'flow',
        label: '控制与编排',
        labelEn: 'Control Flow',
        types: ['waitMs', 'if', 'while', 'for', 'sequence', 'continue', 'break', 'link', 'addPolicies', 'handlePolicySet'],
    },
    {
        key: 'data',
        label: '数据与状态',
        labelEn: 'Data & State',
        types: ['setVar', 'getVar', 'filter', 'setState', 'getState'],
    },
];

export const NODE_TEMPLATES: Record<string, any> = {
    template_1: {
        display: 'Vision Loop Template',
        displayCn: '视觉循环模板',
        description: 'Loop -> Screenshot -> Detect -> Click',
        nodes: [
            { type: 'while', label: '循环', position: { x: 0, y: 0 } },
            { type: 'takeScreenshot', label: '截图', position: { x: 0, y: 100 } },
            { type: 'visionSearch', label: '检测', position: { x: 0, y: 200 } },
            { type: 'if', label: '是否成功', position: { x: 0, y: 300 } },
            { type: 'clickPoint', label: '点击', position: { x: 0, y: 400 } },
        ],
        edges: [
            { sourceIdx: 0, targetIdx: 1, handle: 'loopStart' },
            { sourceIdx: 1, targetIdx: 2 },
            { sourceIdx: 2, targetIdx: 3 },
            { sourceIdx: 3, targetIdx: 4, handle: 'ifTrue' },
            { sourceIdx: 4, targetIdx: 0, targetHandle: 'loopEnd' }
        ]
    }
};

export const SOURCE_HANDLE: Record<string, { label: string | null, animated: boolean }> = {
    'ifTrue': { label: '是', animated: true },
    'ifFalse': { label: '否', animated: true },
    'loopStart': { label: '内含流程', animated: true },
    'out': { label: null, animated: false }
}

export const TARGET_HANDLE: Record<string, { label: string | null, animated: boolean }> = {
    'loopEnd': { label: '子流交汇', animated: true },
    'in': { label: null, animated: false }
}

export const DEFAULT_FALLBACK_STRATEGIES = [
    { target: 'back_button', action: 'click', label: '尝试点击返回' },
    { target: 'close_button', action: 'click', label: '尝试点击关闭' },
    { target: 'confirm_button', action: 'click', label: '尝试点击确认' },
];

export const NODE_DATA_DEFAULTS: Record<string, any> = {
    targetType: 'coordinates',
    x: 0,
    y: 0,
    target: '',
    duration: 1000,
    randomize: false,
    searchType: 'image',
    confidence: 80,
    timeout: 5000,
    count: 3,
    loopType: 'count',
    breakCondition: '',
    maxRetries: 3,
    targetTaskId: null,
    waitForComplete: true,
    delayBefore: 0,
    delayAfter: 0,
    condition: '',
};

// ============================================================================
// getNodeDefaults: 每种类型创建时的默认 Step 数据
// 与后端 serde tag 格式严格一致
// ============================================================================
export function getNodeDefaults(op: string): any {
    const base = { type: op, label: '', skip_flag: false, exec_cur: 0, exec_max: 0 };
    switch (op) {
        // ─── Action ───
        case 'clickPoint':    return { ...base, op: 'action', a: { ac: 'click', mode: 'Point', p: { x: 0, y: 0 } } };
        case 'clickPercent':  return { ...base, op: 'action', a: { ac: 'click', mode: 'Percent', p: { x: 0.5, y: 0.5 } } };
        case 'clickLabel':    return { ...base, op: 'action', a: { ac: 'click', mode: 'LabelIdx', idx: null } };
        case 'clickTxt':      return { ...base, op: 'action', a: { ac: 'click', mode: 'Txt', txt: null } };
        case 'swipePoint':    return { ...base, op: 'action', a: { ac: 'swipe', mode: 'Point', duration: 500, from: { x: 0, y: 0 }, to: { x: 0, y: 0 } } };
        case 'swipePercent':  return { ...base, op: 'action', a: { ac: 'swipe', mode: 'Percent', duration: 500, from: { x: 0.3, y: 0.5 }, to: { x: 0.7, y: 0.5 } } };
        case 'swipeLabel':    return { ...base, op: 'action', a: { ac: 'swipe', mode: 'LabelIdx', duration: 500, from: 0, to: 1 } };
        case 'swipeTxt':      return { ...base, op: 'action', a: { ac: 'swipe', mode: 'Txt', duration: 500, from: null, to: null } };
        case 'takeScreenshot':return { ...base, op: 'action', a: { ac: 'capture', output_var: 'last_capture' } };
        case 'reboot':        return { ...base, op: 'action', a: { ac: 'reboot' } };
        case 'launchApp':     return { ...base, op: 'action', a: { ac: 'launchApp', pkg_name: '' } };
        case 'stopApp':       return { ...base, op: 'action', a: { ac: 'stopApp', pkg_name: '' } };

        // ─── FlowControl ───
        case 'waitMs':        return { ...base, op: 'flowControl', a: { type: 'waitMs', ms: 1000 } };
        case 'if':            return { ...base, op: 'flowControl', a: { type: 'if', con: { type: 'group', op: 'And', items: [] }, then: [], else_steps: null } };
        case 'while':         return { ...base, op: 'flowControl', a: { type: 'while', con: { type: 'group', op: 'And', items: [] }, flow: [] } };
        case 'for':           return { ...base, op: 'flowControl', a: { type: 'for', con: { type: 'group', op: 'And', items: [] }, flow: [] } };
        case 'sequence':      return { ...base, op: 'sequence', steps: [], reverse: false };
        case 'continue':      return { ...base, op: 'flowControl', a: { type: 'continue' } };
        case 'break':         return { ...base, op: 'flowControl', a: { type: 'break' } };
        case 'link':          return { ...base, op: 'flowControl', a: { type: 'link', target: '' } };
        case 'addPolicies':   return { ...base, op: 'flowControl', a: { type: 'addPolicies', source: '', target: '' } };
        case 'handlePolicySet': return { ...base, op: 'flowControl', a: { type: 'handlePolicySet', target: [] } };

        // ─── Vision ───
        case 'visionSearch':  return { ...base, op: 'vision', a: { type: 'visionSearch', rule: { type: 'group', op: 'And', scope: 'Global', items: [] }, out_var: 'search_result', then_steps: [] } };
        case 'ocr':           return { ...base, op: 'vision', a: { type: 'ocr' } };

        // ─── DataHanding ───
        case 'setVar':        return { ...base, op: 'dataHanding', a: { type: 'setVar', name: '', val: null, expr: null } };
        case 'getVar':        return { ...base, op: 'dataHanding', a: { type: 'getVar', name: '', default_val: null } };
        case 'filter':        return { ...base, op: 'dataHanding', a: { type: 'filter', input_var: '', out_name: '', mode: { type: 'filter' }, logic_expr: '', then_steps: null } };

        // ─── TaskControl ───
        case 'setState':      return { ...base, op: 'taskControl', a: { type: 'setState', target: { type: 'Policy', id: '' }, status: { type: 'Skip', value: false } } };
        case 'getState':      return { ...base, op: 'taskControl', a: { type: 'getState', target: { type: 'Policy', id: '' }, status: { type: 'Skip', value: false } } };

        default: return { ...base, op };
    }
}

/**
 * 所有 StepItemEditor / ActionSequenceEditor 支持的虚拟操作类型列表。
 * 新增类型时只需在这里和 NODE_TYPES 各加一行即可。
 */
export const SUPPORTED_STEP_OPS: string[] = Object.keys(NODE_TYPES).filter(
    k => !['start', 'end'].includes(k)
);

/**
 * 从后端嵌套结构 step 推导出前端统一的虚拟操作类型 key（对应 NODE_TYPES 的 key）。
 * 这是 StepItemEditor 渲染的核心依据，所有组件统一使用此函数。
 */
export function getStepVirtualOp(step: any): string {
    if (!step || !step.op) return 'unknown';
    if (step.op === 'sequence') return 'sequence';

    if (step.op === 'action') {
        const a = step.a;
        if (!a) return 'unknown';
        if (a.ac === 'click') {
            if (a.mode === 'Point') return 'clickPoint';
            if (a.mode === 'Percent') return 'clickPercent';
            if (a.mode === 'LabelIdx') return 'clickLabel';
            if (a.mode === 'Txt') return 'clickTxt';
            return 'clickPoint'; // fallback
        }
        if (a.ac === 'swipe') {
            if (a.mode === 'Point') return 'swipePoint';
            if (a.mode === 'Percent') return 'swipePercent';
            if (a.mode === 'LabelIdx') return 'swipeLabel';
            if (a.mode === 'Txt') return 'swipeTxt';
            return 'swipePoint'; // fallback
        }
        if (a.ac === 'capture') return 'takeScreenshot';
        if (a.ac === 'reboot') return 'reboot';
        if (a.ac === 'launchApp') return 'launchApp';
        if (a.ac === 'stopApp') return 'stopApp';
        return a.ac || 'unknown';
    }

    if (step.op === 'flowControl') {
        const a = step.a;
        if (!a) return 'unknown';
        return a.type || 'unknown'; // 'if' | 'while' | 'for' | 'waitMs' | 'continue' | 'break' | 'link' | 'addPolicies' | 'handlePolicySet'
    }

    if (step.op === 'dataHanding') {
        const a = step.a;
        if (!a) return 'unknown';
        return a.type || 'unknown'; // 'setVar' | 'getVar' | 'filter'
    }

    if (step.op === 'taskControl') {
        const a = step.a;
        if (!a) return 'unknown';
        return a.type || 'unknown'; // 'setState' | 'getState'
    }

    if (step.op === 'vision') {
        const a = step.a;
        if (!a) return 'unknown';
        return a.type || 'unknown'; // 'visionSearch' | 'ocr'
    }

    return step.op;
}

export const THEMES = [
    'dark', 'light', 'cupcake', 'bumblebee', 'emerald', 'corporate',
    'synthwave', 'retro', 'cyberpunk', 'valentine', 'halloween', 'garden',
    'forest', 'aqua', 'lofi', 'pastel', 'fantasy', 'wireframe',
    'black', 'luxury', 'dracula', 'cmyk', 'autumn', 'business',
    'acid', 'lemonade', 'night', 'coffee', 'winter', 'dim', 'nord', 'sunset',
];

export const DEFAULT_EDITOR_THEME = 'light';
export const DEFAULT_APP_THEME = 'light';

export function getNodeTypeConfig(type: string): NodeTypeConfig {
    return NODE_TYPES[type] || {
        color: 'bg-neutral',
        icon: 'box',
        display: 'Node',
        displayCn: '节点',
        category: 'special',
        placeholder: '无描述',
        description: '',
    };
}

export function getNodeColor(type: string): string { return getNodeTypeConfig(type).color; }
export function getNodeDisplay(type: string, lang: 'en' | 'cn' = 'en'): string {
    const config = getNodeTypeConfig(type);
    return lang === 'cn' ? config.displayCn : config.display;
}
export function getNodeIcon(type: string): string { return getNodeTypeConfig(type).icon; }
export function getNodePlaceholder(type: string): string { return getNodeTypeConfig(type).placeholder; }
export function getNodeDescription(type: string): string { return getNodeTypeConfig(type).description || ''; }
export function isStartNode(type: string): boolean { return type === 'start'; }
export function isConditionNode(type: string): boolean { return type === 'if'; }
export function isLoopNode(type: string): boolean { return ['while', 'for'].includes(type); }
export function isEndNode(type: string): boolean { return type === 'end'; }
