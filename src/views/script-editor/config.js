/**
 * Script Editor Configuration
 * 
 * 将所有配置集中于此文件，方便维护和修改。
 * 如需修改节点类型、颜色、图标等，只需在此处修改一次即可。
 */

// ============================================
// Node Type Configuration
// ============================================

/**
 * 节点类型配置
 * - type: 节点类型标识符
 * - color: 背景颜色 class (Tailwind)
 * - icon: 图标类型标识
 * - display: 显示名称
 * - category: 分类 (basic, condition, vision, control, special)
 * - placeholder: 未配置时的占位文本
 */
export const NODE_TYPES = {
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
        // ... (remaining definitions will be updated in chunks or via replacement)
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

    // Templates (For Toolbox display)
    macro_1: {
        color: 'bg-amber-600',
        icon: 'zap',
        display: 'Smart Click',
        displayCn: '宏点击（截图|检测|点击）',
        category: 'composite',
        placeholder: 'Unified action configuration',
        description: 'Unified: Capture -> Detect -> Click',
    },
    template_1: {
        color: 'bg-indigo-600',
        icon: 'layers',
        display: 'Vision Loop',
        displayCn: '宏模板（循环|截图|检测|点击）',
        category: 'composite',
        description: '自动生成: 循环 -> 截图 -> 检测 -> 点击',
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

// ============================================
// Node Categories (for Toolbox)
// ============================================

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
        types: ['capture', 'detect', 'ocr'],
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
    {
        key: 'composite',
        label: '复合模板',
        labelEn: 'Composite',
        types: ['macro_1', 'template_1'],
    },
];

// ============================================
// Node Templates (Fragmented)
// ============================================

export const NODE_TEMPLATES = {
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
            { sourceIdx: 0, targetIdx: 1, handle: 'loopStart' }, // 循环开始 -> 截图
            { sourceIdx: 1, targetIdx: 2 },                       // 截图 -> 检测
            { sourceIdx: 2, targetIdx: 3 },                       // 检测 -> 识别
            { sourceIdx: 3, targetIdx: 4, handle: 'ifTrue' },   // 识别成功 -> 点击
            { sourceIdx: 4, targetIdx: 0, targetHandle: 'loopEnd' } // 点击后 -> 回到循环结束点
        ]
    }
};

// ============================================
// 边handleID类型
// ============================================
export const SOURCE_HANDLE = {
    'ifTrue': { label: '是', animated: true },
    'ifFalse': { label: '否', animated: true },
    'loopStart': { label: '循环开始', animated: true },
    'output': { label: null, animated: false }
}

export const TARGET_HANDLE = {
    'loopEnd': { label: '循环结束', animated: true },
    'input': { label: null, animated: false }
}

// ============================================
// 拖动添加节点 - 已迁移到 composables/useDragAndDrop.js
// ============================================
// 请使用: import { useDragAndDrop } from './composables';

// ============================================
// Default Values
// ============================================

/**
 * 默认回退策略
 */
export const DEFAULT_FALLBACK_STRATEGIES = [
    { target: 'back_button', action: 'click', label: '尝试点击返回' },
    { target: 'close_button', action: 'click', label: '尝试点击关闭' },
    { target: 'confirm_button', action: 'click', label: '尝试点击确认' },
];

/**
 * 节点数据默认值
 */
export const NODE_DATA_DEFAULTS = {
    // Click
    targetType: 'coordinates',
    x: 0,
    y: 0,
    target: '',

    // Wait
    duration: 1000,
    randomize: false,

    // Condition
    searchType: 'image',
    confidence: 80,
    timeout: 5000,

    // Loop
    count: 3,
    loopType: 'count',
    breakCondition: '',

    // Fallback
    maxRetries: 3,

    // SubFlow
    targetTaskId: null,
    waitForComplete: true,

    // Advanced
    delayBefore: 0,
    delayAfter: 0,
    condition: '',
};

/**
 * 获取节点的默认数据
 * @param {string} type - 节点类型
 * @returns {Object} 默认数据对象
 */
export function getNodeDefaults(type) {
    const base = { ...NODE_DATA_DEFAULTS };

    switch (type) {
        case 'click':
            return {
                type,
                targetType: base.targetType,
                x: base.x,
                y: base.y,
                target: base.target,
            };
        case 'wait':
            return {
                type,
                duration: base.duration,
                randomize: base.randomize,
            };
        case 'swipe':
            return {
                type,
                startX: 0,
                startY: 0,
                endX: 0,
                endY: 0,
                duration: base.duration,
            };
        case 'if':
            return {
                type,
                searchType: base.searchType,
                target: base.target,
                confidence: base.confidence,
                timeout: base.timeout,
            };
        case 'detect':
            return {
                type,
                imagePath: '',
                confidence: base.confidence,
                resultVar: '',
            };
        case 'ocr':
            return {
                type,
                regionX: null,
                regionY: null,
                regionW: null,
                regionH: null,
                resultVar: '',
            };
        case 'loop':
            return {
                type,
                count: base.count,
                loopType: base.loopType,
                breakCondition: base.breakCondition,
            };
        case 'fallback':
            return {
                type,
                maxRetries: base.maxRetries,
                strategies: DEFAULT_FALLBACK_STRATEGIES.map(s => ({ ...s })),
            };
        case 'subflow':
            return {
                type,
                targetTaskId: base.targetTaskId,
                waitForComplete: base.waitForComplete,
            };
        case 'capture':
            return {
                type,
                outputVar: 'last_capture',
            };
        case 'variable':
            return {
                type,
                varName: '',
                opType: 'set', // set, math, string, regex
                expression: '',
            };
        case 'filter':
            return {
                type,
                sourceVar: '',
                targetVar: '',
                mode: 'filter', // filter, map
                logic: '',
            };
        case 'macro_1':
            return {
                type,
                screenshot: true,
                detectTarget: '',
                confidence: 80,
                clickType: 'coordinates',
                postProcess: '', // Optional logic
            };
        default:
            return { type };
    }
}

// ============================================
// Themes Configuration
// ============================================

/**
 * 可用主题列表
 * Core themes are 'dark' and 'light', designed specifically for AutoDaily.
 * Others are default DaisyUI themes.
 */
export const THEMES = [
    'dark', 'light', 'cupcake', 'bumblebee', 'emerald', 'corporate',
    'synthwave', 'retro', 'cyberpunk', 'valentine', 'halloween', 'garden',
    'forest', 'aqua', 'lofi', 'pastel', 'fantasy', 'wireframe',
    'black', 'luxury', 'dracula', 'cmyk', 'autumn', 'business',
    'acid', 'lemonade', 'night', 'coffee', 'winter', 'dim', 'nord', 'sunset',
];

/**
 * 默认显示的主题数量 (在设置页面初始化展示的数量)
 */
export const DEFAULT_VISIBLE_THEMES_COUNT = 8;

/**
 * 默认起始主题 - AutoDaily 推荐使用 dark 模式
 */
export const DEFAULT_THEME = 'light';

// ============================================
// Helper Functions
// ============================================

/**
 * 获取节点类型配置
 * @param {string} type - 节点类型
 * @returns {Object} 节点配置
 */
export function getNodeTypeConfig(type) {
    return NODE_TYPES[type] || {
        color: 'bg-neutral',
        icon: 'box',
        display: 'Node',
        displayCn: '节点',
        category: 'special',
        placeholder: '无描述',
    };
}

/**
 * 获取节点的背景颜色 class
 * @param {string} type - 节点类型
 * @returns {string} Tailwind color class
 */
export function getNodeColor(type) {
    return getNodeTypeConfig(type).color;
}

/**
 * 获取节点的显示名称
 * @param {string} type - 节点类型
 * @param {string} lang - 语言 ('en' | 'cn')
 * @returns {string} 显示名称
 */
export function getNodeDisplay(type, lang = 'en') {
    const config = getNodeTypeConfig(type);
    return lang === 'cn' ? config.displayCn : config.display;
}

/**
 * 获取节点的图标类型
 * @param {string} type - 节点类型
 * @returns {string} 图标类型标识
 */
export function getNodeIcon(type) {
    return getNodeTypeConfig(type).icon;
}

/**
 * 获取节点的占位文本
 * @param {string} type - 节点类型
 * @returns {string} 占位文本
 */
export function getNodePlaceholder(type) {
    return getNodeTypeConfig(type).placeholder;
}

/**
 * 获取节点的描述文本
 * @param {string} type - 节点类型
 * @returns {string} 描述文本
 */
export function getNodeDescription(type) {
    return getNodeTypeConfig(type).description || '';
}

/**
 * 判断是否为开始节点
 * @param {string} type - 节点类型
 * @returns {boolean}
 */
export function isStartNode(type) {
    return ['start'].includes(type);
}

/**
 * 判断是否为条件节点
 * @param {string} type - 节点类型
 * @returns {boolean}
 */
export function isConditionNode(type) {
    return ['if'].includes(type);
}

/**
 * 判断是否为循环节点
 * @param {string} type - 节点类型
 * @returns {boolean}
 */
export function isLoopNode(type) {
    return ['loop'].includes(type);
}

/**
 * 判断是否为结束节点
 * @param {string} type - 节点类型
 * @returns {boolean}
 */
export function isEndNode(type) {
    return ['end'].includes(type);
}
