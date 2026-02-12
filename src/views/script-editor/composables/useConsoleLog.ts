import { ref, nextTick } from 'vue';
import type { Ref } from 'vue';

export const LOG_LEVELS = {
    INFO: 'info',
    WARN: 'warn',
    ERROR: 'error',
    SUCCESS: 'success',
} as const;

export type LogLevel = typeof LOG_LEVELS[keyof typeof LOG_LEVELS];

interface LogEntry {
    time: string;
    level: LogLevel;
    message: string;
}

interface ConsoleOptions {
    maxLogs?: number;
}

export function useConsoleLog(options: ConsoleOptions = {}) {
    const { maxLogs = 300 } = options;

    const consoleLogs = ref<LogEntry[]>([
        { time: new Date().toTimeString().slice(0, 8), level: LOG_LEVELS.INFO, message: 'Script Editor initialized.' },
    ]);

    const consoleRef = ref<HTMLElement | null>(null);

    function logClass(level: LogLevel) {
        switch (level) {
            case LOG_LEVELS.INFO: return 'text-success';
            case LOG_LEVELS.ERROR: return 'text-error';
            case LOG_LEVELS.WARN: return 'text-warning';
            case LOG_LEVELS.SUCCESS: return 'text-success';
            default: return 'text-info';
        }
    }

    function addLog(message: string, level: LogLevel = LOG_LEVELS.INFO) {
        const now = new Date();
        const time = now.toTimeString().slice(0, 8);

        let isAtBottom = true;
        if (consoleRef.value) {
            const { scrollTop, scrollHeight, clientHeight } = consoleRef.value;
            isAtBottom = Math.abs(scrollHeight - scrollTop - clientHeight) < 5;
        }

        consoleLogs.value.push({ time, level, message });

        if (consoleLogs.value.length > maxLogs) {
            consoleLogs.value = consoleLogs.value.slice(-maxLogs);
        }

        if (isAtBottom) {
            nextTick(() => {
                if (consoleRef.value) {
                    consoleRef.value.scrollTop = consoleRef.value.scrollHeight;
                }
            }).then();
        }
    }

    function clearConsole() {
        consoleLogs.value = [];
    }

    return {
        consoleLogs,
        consoleRef,
        LOG_LEVELS,
        logClass,
        addLog,
        clearConsole,
    };
}

export default useConsoleLog;
