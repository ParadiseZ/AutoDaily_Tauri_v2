import { ref, computed } from 'vue';

/**
 * Composable for managing tasks within a script
 */
export function useTasks() {
    const taskList = ref([
        {
            id: 1,
            name: '登录',
            nodes: [
                { id: 'start-1', type: 'custom', label: 'Start', position: { x: 150, y: 50 }, data: { type: 'start' } },
            ],
            edges: []
        },
        { id: 2, name: '奖励领取', nodes: [{ id: 'start-1', type: 'custom', label: 'Start', position: { x: 150, y: 50 }, data: { type: 'start' } }], edges: [] },
        { id: 3, name: '签到', nodes: [{ id: 'start-1', type: 'custom', label: 'Start', position: { x: 150, y: 50 }, data: { type: 'start' } }], edges: [] },
    ]);

    const currentTask = ref(null);
    const taskSearch = ref('');

    const filteredTasks = computed(() => {
        if (!taskSearch.value) return taskList.value;
        const search = taskSearch.value.toLowerCase();
        return taskList.value.filter(t => t.name.toLowerCase().includes(search));
    });

    const selectTask = (task, nodes, edges, addLog) => {
        if (currentTask.value) {
            currentTask.value.nodes = [...nodes.value];
            currentTask.value.edges = [...edges.value];
        }
        currentTask.value = task;
        nodes.value = task.nodes.map(n => ({ ...n, type: 'custom' }));
        edges.value = task.edges.map(e => ({ ...e, type: e.type || 'smoothstep' }));
        addLog?.(`Switched to task: ${task.name}`, 'info');
    };

    const createNewTask = (nodes, edges, addLog) => {
        const newId = Math.max(...taskList.value.map(t => t.id), 0) + 1;
        const newTask = {
            id: newId,
            name: `Task ${newId}`,
            nodes: [{ id: 'start-1', type: 'custom', label: 'Start', position: { x: 150, y: 50 }, data: { type: 'start' } }],
            edges: []
        };
        taskList.value.push(newTask);
        selectTask(newTask, nodes, edges, addLog);
        addLog?.(`Created: ${newTask.name}`, 'success');
    };

    const deleteTask = (id, nodes, edges, addLog) => {
        if (taskList.value.length <= 1) {
            addLog?.('Cannot delete the last task', 'error');
            return;
        }
        const idx = taskList.value.findIndex(t => t.id === id);
        if (idx !== -1) {
            const name = taskList.value[idx].name;
            taskList.value.splice(idx, 1);
            if (currentTask.value?.id === id) {
                selectTask(taskList.value[0], nodes, edges, addLog);
            }
            addLog?.(`Deleted: ${name}`, 'warn');
        }
    };

    return {
        taskList,
        currentTask,
        taskSearch,
        filteredTasks,
        selectTask,
        createNewTask,
        deleteTask,
    };
}

/**
 * Composable for console logging
 */
export function useConsole() {
    const consoleLogs = ref([]);

    const logClass = (level) => {
        const classes = { success: 'text-success', error: 'text-error', warn: 'text-warning' };
        return classes[level] || 'text-info';
    };

    const addLog = (message, level = 'info') => {
        const time = new Date().toTimeString().slice(0, 8);
        consoleLogs.value.push({ time, level, message });
        // Keep only last 100 logs
        if (consoleLogs.value.length > 100) consoleLogs.value.shift();
    };

    const clearConsole = () => { consoleLogs.value = []; };

    return { consoleLogs, logClass, addLog, clearConsole };
}

/**
 * Composable for theme management
 */
import { DEFAULT_THEME } from './config.js';

export function useTheme() {
    const currentTheme = ref(DEFAULT_THEME);

    const toggleTheme = () => {
        currentTheme.value = currentTheme.value === 'light' ? 'dark' : 'light';
        localStorage.setItem('theme', currentTheme.value);
        document.documentElement.setAttribute('data-theme', currentTheme.value);
    };

    const initTheme = () => {
        const saved = localStorage.getItem('theme') || DEFAULT_THEME;
        currentTheme.value = saved;
        document.documentElement.setAttribute('data-theme', saved);
    };

    return { currentTheme, toggleTheme, initTheme };
}
