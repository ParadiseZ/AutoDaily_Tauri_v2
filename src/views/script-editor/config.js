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
        icon: 'search',
        display: 'IF Found',
        displayCn: '如果',
        category: 'condition',
        placeholder: 'Set search target...',
        description: 'If image/text found, then...',
    },
    if_found: {
        color: 'bg-yellow-500',
        icon: 'search',
        display: 'IF Found',
        displayCn: '如果找到',
        category: 'condition',
        placeholder: 'Set search target...',
        description: 'If image/text found, then...',
    },
    if_not_found: {
        color: 'bg-orange-500',
        icon: 'search-x',
        display: 'IF Not Found',
        displayCn: '如果未找到',
        category: 'condition',
        placeholder: 'Set search target...',
        description: 'If image/text not found, then...',
    },

    // Vision Nodes
    detect: {
        color: 'bg-purple-500',
        icon: 'image',
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
    input: {
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
        types: ['detect', 'ocr'],
    },
    {
        key: 'control',
        label: '控制流',
        labelEn: 'Control Flow',
        types: ['loop', 'fallback', 'subflow'],
    },
];

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
        case 'if_found':
        case 'if_not_found':
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
export const DEFAULT_THEME = 'dark';

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
    return ['start', 'input'].includes(type);
}

/**
 * 判断是否为条件节点
 * @param {string} type - 节点类型
 * @returns {boolean}
 */
export function isConditionNode(type) {
    return ['if', 'if_found', 'if_not_found'].includes(type);
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
