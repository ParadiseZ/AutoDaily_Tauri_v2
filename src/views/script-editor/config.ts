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
    click: {
        color: 'bg-blue-500',
        icon: 'cursor',
        display: 'Click',
        displayCn: '点击',
        category: 'basic',
        placeholder: 'Set click target...',
        description: 'Click on a target',
    },
    wait: {
        color: 'bg-gray-500',
        icon: 'clock',
        display: 'Wait',
        displayCn: '等待',
        category: 'basic',
        placeholder: 'Set wait duration...',
        description: 'Wait for duration',
    },
    swipe: {
        color: 'bg-cyan-500',
        icon: 'move',
        display: 'Swipe',
        displayCn: '滑动',
        category: 'basic',
        placeholder: 'Set swipe gesture...',
        description: 'Swipe gesture',
    },

    // Condition Nodes
    if: {
        color: 'bg-yellow-500',
        icon: 'branch',
        display: 'IF Found',
        displayCn: '判断',
        category: 'condition',
        placeholder: 'Set search target...',
        description: 'If condition met, then...',
    },

    // Vision Nodes
    capture: {
        color: 'bg-slate-500',
        icon: 'camera',
        display: 'Screenshot',
        displayCn: '截图',
        category: 'vision',
        placeholder: 'Save to variable...',
        description: 'Capture screen to variable',
    },
    detect: {
        color: 'bg-purple-500',
        icon: 'target',
        display: 'Find Image',
        displayCn: '目标检测',
        category: 'vision',
        placeholder: 'Select image...',
        description: 'Locate image on screen',
    },
    ocr: {
        color: 'bg-violet-500',
        icon: 'type',
        display: 'OCR',
        displayCn: '文字识别',
        category: 'vision',
        placeholder: 'Set OCR region...',
        description: 'Recognize text',
    },
    vision_logic: {
        color: 'bg-pink-600',
        icon: 'zap',
        display: 'Vision Logic',
        displayCn: '视觉逻辑',
        category: 'vision',
        placeholder: 'Configure vision rules...',
        description: 'Advanced search logic (Text + Object + Color)',
    },

    // Data Nodes
    variable: {
        color: 'bg-orange-500',
        icon: 'variable',
        display: 'Variable',
        displayCn: '变量',
        category: 'data',
        placeholder: 'Expression...',
        description: 'Process data / set variable',
    },
    filter: {
        color: 'bg-orange-400',
        icon: 'filter',
        display: 'Filter/Map',
        displayCn: '数据过滤',
        category: 'data',
        placeholder: 'Filter or transform...',
        description: 'Filter or Map array data',
    },

    // Control Flow Nodes
    loop: {
        color: 'bg-green-500',
        icon: 'repeat',
        display: 'Loop',
        displayCn: '循环',
        category: 'control',
        placeholder: 'Configure loop...',
        description: 'Repeat N times',
    },
    fallback: {
        color: 'bg-red-500',
        icon: 'alert-triangle',
        display: 'Fallback',
        displayCn: '回调',
        category: 'control',
        placeholder: 'Fallback actions',
        description: 'Retry actions when all conditions fail',
    },
    subflow: {
        color: 'bg-pink-500',
        icon: 'git-branch',
        display: 'Sub-Flow',
        displayCn: '子流程',
        category: 'control',
        placeholder: 'Select sub-flow...',
        description: "Call another task's flow",
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
        label: '基本',
        labelEn: 'Basic',
        types: ['click', 'wait', 'swipe'],
    },
    {
        key: 'condition',
        label: '条件逻辑',
        labelEn: 'Conditions',
        types: ['if'],
    },
    {
        key: 'vision',
        label: '视觉',
        labelEn: 'Vision',
        types: ['capture', 'detect', 'ocr', 'vision_logic'],
    },
    {
        key: 'data',
        label: '数据处理',
        labelEn: 'Data',
        types: ['variable', 'filter'],
    },
    {
        key: 'control',
        label: '控制流',
        labelEn: 'Control Flow',
        types: ['loop', 'fallback', 'subflow'],
    },
];

export const NODE_TEMPLATES: Record<string, any> = {
    template_1: {
        display: 'Vision Loop Template',
        displayCn: '视觉循环模板',
        description: 'Loop -> Screenshot -> Detect -> Click',
        nodes: [
            { type: 'loop', label: '循环', position: { x: 0, y: 0 } },
            { type: 'screenshot', label: '截图', position: { x: 0, y: 100 } },
            { type: 'detect', label: '检测', position: { x: 0, y: 200 } },
            { type: 'if', label: '是否成功', position: { x: 0, y: 300 } },
            { type: 'click', label: '点击', position: { x: 0, y: 400 } },
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
    'loopStart': { label: '循环开始', animated: true },
    'out': { label: null, animated: false }
}

export const TARGET_HANDLE: Record<string, { label: string | null, animated: boolean }> = {
    'loopEnd': { label: '循环结束', animated: true },
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

export function getNodeDefaults(type: string): any {
    const base = { ...NODE_DATA_DEFAULTS };
    switch (type) {
        case 'click': return { type, targetType: base.targetType, x: base.x, y: base.y, target: base.target };
        case 'wait': return { type, duration: base.duration, randomize: base.randomize };
        case 'swipe': return { type, startX: 0, startY: 0, endX: 0, endY: 0, duration: base.duration };
        case 'if': return { type, searchType: base.searchType, target: base.target, confidence: base.confidence, timeout: base.timeout };
        case 'detect': return { type, imagePath: '', confidence: base.confidence, resultVar: '' };
        case 'ocr': return { type, regionX: null, regionY: null, regionW: null, regionH: null, resultVar: '' };
        case 'vision_logic': return { type, rule: { type: 'Group', op: 'And', scope: 'Global', items: [] }, outputVar: 'search_results' };
        case 'loop': return { type, count: base.count, loopType: base.loopType, breakCondition: base.breakCondition };
        case 'fallback': return { type, maxRetries: base.maxRetries, strategies: DEFAULT_FALLBACK_STRATEGIES.map(s => ({ ...s })) };
        case 'subflow': return { type, targetTaskId: base.targetTaskId, waitForComplete: base.waitForComplete };
        case 'capture': return { type, outputVar: 'last_capture' };
        case 'variable': return { type, varName: '', opType: 'set', expression: '' };
        case 'filter': return { type, sourceVar: '', targetVar: '', mode: 'filter', logic: '' };
        case 'macro_1': return { type, screenshot: true, detectTarget: '', confidence: 80, clickType: 'coordinates', postProcess: '' };
        default: return { type };
    }
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
export function isLoopNode(type: string): boolean { return type === 'loop'; }
export function isEndNode(type: string): boolean { return type === 'end'; }
