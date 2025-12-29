/**
 * Console Log Composable
 * 
 * 管理脚本编辑器的控制台日志
 * - 添加日志
 * - 清除日志
 * - 自动滚动
 * - 日志数量限制
 */

import { ref, nextTick } from 'vue';

// 日志级别常量
export const LOG_LEVELS = {
    INFO: 'info',
    WARN: 'warn',
    ERROR: 'error',
    SUCCESS: 'success',
};

/**
 * Console Log Composable
 * 
 * @param {Object} options - 配置选项
 * @param {number} options.maxLogs - 最大日志数量，默认 500
 * @returns {Object} 日志相关的状态和方法
 */
export function useConsoleLog(options = {}) {
    const { maxLogs = 500 } = options;

    // 日志列表
    const consoleLogs = ref([
        { time: new Date().toTimeString().slice(0, 8), level: LOG_LEVELS.INFO, message: 'Script Editor initialized.' },
    ]);

    // 控制台 DOM 引用
    const consoleRef = ref(null);

    /**
     * 获取日志的 CSS 类
     * @param {string} level 
     * @returns {string}
     */
    function logClass(level) {
        switch (level) {
            case 'success': return 'text-success';
            case 'error': return 'text-error';
            case 'warn': return 'text-warning';
            default: return 'text-info';
        }
    }

    /**
     * 添加日志
     * @param {string} message - 日志消息
     * @param {string} level - 日志级别
     */
    function addLog(message, level = LOG_LEVELS.INFO) {
        const now = new Date();
        const time = now.toTimeString().slice(0, 8);

        // 1. 判断是否在最底部 (允许 5px 误差)
        let isAtBottom = true;
        if (consoleRef.value) {
            const { scrollTop, scrollHeight, clientHeight } = consoleRef.value;
            isAtBottom = Math.abs(scrollHeight - scrollTop - clientHeight) < 5;
        }

        // 2. 添加日志
        consoleLogs.value.push({ time, level, message });

        // 3. 限制日志数量，保留最新的
        if (consoleLogs.value.length > maxLogs) {
            consoleLogs.value = consoleLogs.value.slice(-maxLogs);
        }

        // 4. 如果之前在最底部，则更新 DOM 后自动滚动到底部
        if (isAtBottom) {
            nextTick(() => {
                if (consoleRef.value) {
                    consoleRef.value.scrollTop = consoleRef.value.scrollHeight;
                }
            });
        }
    }

    /**
     * 清除所有日志
     */
    function clearConsole() {
        consoleLogs.value = [];
    }

    return {
        // 状态
        consoleLogs,
        consoleRef,

        // 常量
        LOG_LEVELS,

        // 方法
        logClass,
        addLog,
        clearConsole,
    };
}

export default useConsoleLog;
