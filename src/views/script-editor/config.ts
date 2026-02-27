/**
 * Script Editor Configuration
 * 
 * 将所有配置集中于此文件，方便维护和修改。
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

export const NODE_TYPES: Record<string, NodeTypeConfig> = {
    // Basic Nodes
    clickAction: {
        color: 'bg-blue-500',
        icon: 'cursor',
        display: 'Click Action',
        displayCn: '点击操作',
        category: 'basic',
        placeholder: 'Set click target...',
        description: 'Click on a target',
    },
    waitMs: {
        color: 'bg-gray-500',
        icon: 'clock',
        display: 'Wait (ms)',
        displayCn: '延时等待',
        category: 'basic',
        placeholder: 'Set wait duration...',
        description: 'Wait for duration',
    },
    swipePoint: {
        color: 'bg-cyan-500',
        icon: 'move',
        display: 'Swipe',
        displayCn: '坐标滑动',
        category: 'basic',
        placeholder: 'Set swipe gesture...',
        description: 'Swipe gesture by points',
    },

    // Condition & Flow Nodes
    if: {
        color: 'bg-yellow-500',
        icon: 'branch',
        display: 'IF',
        displayCn: '条件分支',
        category: 'condition',
        placeholder: 'Set condition...',
        description: 'If condition met, then...',
    },
    while: {
        color: 'bg-yellow-600',
        icon: 'rotate-cw',
        display: 'While',
        displayCn: '循环控制',
        category: 'condition',
        placeholder: 'While condition...',
        description: 'Loop while condition is met',
    },
    sequence: {
        color: 'bg-green-600',
        icon: 'layers',
        display: 'Sequence',
        displayCn: '序列容器',
        category: 'condition',
        placeholder: 'Step Sequence...',
        description: 'Container for nested steps',
    },

    // Vision Nodes
    takeScreenshot: {
        color: 'bg-slate-500',
        icon: 'camera',
        display: 'Screenshot',
        displayCn: '截图',
        category: 'vision',
        placeholder: 'Save to variable...',
        description: 'Capture screen to variable',
    },
    visionSearch: {
        color: 'bg-indigo-500',
        icon: 'zap',
        display: 'Vision Logic',
        displayCn: '强化视觉搜索',
        category: 'vision',
        placeholder: 'Set vision rules...',
        description: 'Advanced vision element finder',
    },
    ocr: {
        color: 'bg-purple-600',
        icon: 'type',
        display: 'OCR Text',
        displayCn: '文字提取',
        category: 'vision',
        placeholder: 'Configure OCR...',
        description: 'Extract text via OCR',
    },

    // Data Nodes
    setVar: {
        color: 'bg-emerald-500',
        icon: 'variable',
        display: 'Set Variable',
        displayCn: '变量赋值',
        category: 'data',
        placeholder: 'Set var using expr...',
        description: 'Set memory variable',
    },
    getVar: {
        color: 'bg-emerald-600',
        icon: 'terminal',
        display: 'Get Variable',
        displayCn: '变量读取',
        category: 'data',
        placeholder: 'Get var...',
        description: 'Read memory variable',
    },

    setState: {
        color: 'bg-teal-500',
        icon: 'settings',
        display: 'Set State',
        displayCn: '状态设置',
        category: 'data',
        placeholder: 'Set Target State...',
        description: 'Set Policy/Task state',
    },

    // Special Nodes
    start: {
        color: 'bg-emerald-600',
        icon: 'play',
        display: 'Start',
        displayCn: '开始',
        category: 'special',
        placeholder: '开始',
        description: 'Start node',
    },
    end: {
        color: 'bg-rose-600',
        icon: 'square',
        display: 'End',
        displayCn: '结束',
        category: 'special',
        placeholder: '结束',
        description: 'End node',
    },
};

export const NODE_CATEGORIES = [
    {
        key: 'basic',
        label: '互动行为',
        labelEn: 'Interaction',
        types: ['clickAction', 'waitMs', 'swipePoint'],
    },
    {
        key: 'vision',
        label: '视觉与解析',
        labelEn: 'Vision & Inference',
        types: ['visionSearch', 'ocr', 'takeScreenshot'],
    },
    {
        key: 'condition',
        label: '控制与编排',
        labelEn: 'Control Flow',
        types: ['if', 'while', 'sequence'],
    },
    {
        key: 'data',
        label: '数据与状态',
        labelEn: 'Data & State',
        types: ['setVar', 'getVar', 'setState'],
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
            { type: 'clickAction', label: '点击', position: { x: 0, y: 400 } },
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

export function getNodeDefaults(op: string): any {
    const base = { type: op, label: '', skip_flag: false, exec_cur: 0, exec_max: 0 };
    switch (op) {
        case 'clickAction': return { ...base, op: 'action', a: { ac: 'click', mode: 'point', p: { x: 0, y: 0 } } };
        case 'waitMs': return { ...base, op: 'flowControl', a: { type: 'waitMs', ms: 1000 } };
        case 'swipePoint': return { ...base, op: 'action', a: { ac: 'swipe', mode: 'point', duration: 1000, from: { x: 0, y: 0 }, to: { x: 0, y: 0 } } };
        case 'if': return { ...base, op: 'flowControl', a: { type: 'if', con: { type: 'group', op: 'And', scope: 'Global', items: [] }, then: [], elseSteps: null } };
        case 'while': return { ...base, op: 'flowControl', a: { type: 'while', con: { type: 'group', op: 'And', scope: 'Global', items: [] }, flow: [] } };
        case 'sequence': return { ...base, op: 'sequence', steps: [], reverse: false };
        case 'visionSearch': return { ...base, op: 'vision', a: { type: 'visionSearch', rule: { type: 'group', op: 'And', scope: 'Global', items: [] }, outVar: 'search_result', thenSteps: [] } };
        case 'takeScreenshot': return { ...base, op: 'action', a: { ac: 'capture', outputVar: 'last_capture' } };
        case 'setVar': return { ...base, op: 'dataHanding', a: { type: 'setVar', name: '', val: null, expr: null } };
        case 'getVar': return { ...base, op: 'dataHanding', a: { type: 'getVar', name: '', defaultVal: null } };
        case 'setState': return { ...base, op: 'taskControl', a: { type: 'setState', target: { type: 'Policy', id: '' }, status: { type: 'Skip', value: false } } };
        case 'ocr': return { ...base, op: 'vision', a: { type: 'ocr' } };
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
        if (a.ac === 'click') return 'clickAction';
        if (a.ac === 'swipe') {
            if (a.mode === 'point') return 'swipePoint';
            if (a.mode === 'percent') return 'swipePercent';
            if (a.mode === 'txt') return 'swipeTxt';
            if (a.mode === 'labelIdx') return 'swipeLabelIdx';
            return 'swipePoint';
        }
        if (a.ac === 'capture') return 'takeScreenshot';
        return a.ac;
    }
    if (step.op === 'dataHanding') {
        const a = step.a;
        if (!a) return 'unknown';
        return a.type || 'unknown';
    }
    if (step.op === 'flowControl') {
        const a = step.a;
        if (!a) return 'unknown';
        return a.type || 'unknown';
    }
    if (step.op === 'taskControl') {
        const a = step.a;
        if (!a) return 'unknown';
        return a.type || 'unknown';
    }
    if (step.op === 'vision') {
        const a = step.a;
        if (!a) return 'unknown';
        return a.type || 'unknown';
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
export function isLoopNode(type: string): boolean { return ['loop', 'while', 'for'].includes(type); }
export function isEndNode(type: string): boolean { return type === 'end'; }
