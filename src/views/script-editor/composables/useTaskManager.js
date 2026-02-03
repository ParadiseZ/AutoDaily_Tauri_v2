/**
 * Task Manager Composable
 * 
 * 管理脚本中的任务列表
 * - 任务切换
 * - 创建/删除任务
 * - 任务重命名
 * - 任务可见性
 */

import { ref, computed } from 'vue';

/**
 * Task Manager Composable
 * 
 * @param {Object} options - 配置选项
 * @param {Ref} options.nodes - 节点数组的响应式引用
 * @param {Ref} options.edges - 边数组的响应式引用
 * @param {Function} options.addLog - 日志函数
 * @returns {Object} 任务管理相关的状态和方法
 */
export function useTaskManager(options = {}) {
    const { nodes, edges, addLog = () => { }, LOG_LEVELS={}, getUuidV7= async () => {}} = options;

    // 任务列表
    const taskList = ref([]);

    // 当前任务
    const currentTask = ref(null);

    // 搜索关键词
    const taskSearch = ref('');

    // 过滤后的任务列表
    const filteredTasks = computed(() => {
        if (!taskSearch.value) return taskList.value;
        const search = taskSearch.value.toLowerCase();
        return taskList.value.filter(t => t.name.toLowerCase().includes(search));
    });

    // 重命名相关
    const editTaskModal = ref(false);
    const renameValue = ref('');
    const renameTarget = ref(null);

    // ============================================
    // 任务选择
    // ============================================

    /**
     * 选择任务
     * @param {Object} task 
     */
    function selectTask(task) {
        // 保存当前任务状态
        if (currentTask.value && nodes && edges) {
            currentTask.value.nodes = [...nodes.value];
            currentTask.value.edges = [...edges.value];
        }

        currentTask.value = task;

        // 加载任务的节点和边
        if (nodes && edges) {
            //nodes.value = task.nodes.map(n => ({ ...n, type: 'custom' }));
            nodes.value = [...task.nodes];
            edges.value = [...task.edges];
        }

        addLog(`切换任务： ${task.name}`, LOG_LEVELS.INFO);
    }

    // ============================================
    // 任务创建/删除
    // ============================================

    /**
     * 创建新任务
     */
    async function createNewTask() {
        //const newId = crypto.randomUUID();
        const newId = await getUuidV7();
        const newTaskCount = taskList.value.length + 1;
        const newTask = {
            id: newId,
            name: `New Task ${newTaskCount}`,
            hidden: false,
            nodes: [
                { id: await getUuidV7(), type: 'custom', label: '开始', position: { x: 200, y: 50 }, data: { type: 'start' } },
                { id: await getUuidV7(), type: 'custom', label: '结束', position: { x: 200, y: 150 }, data: { type: 'end' } }
            ],
            edges: [],
            uiData: {},
            variables: {}
        };
        taskList.value.push(newTask);
        //addLog(`创建任务: ${newTask.name}`, LOG_LEVELS.SUCCESS);
        selectTask(newTask);
    }

    /**
     * 删除任务
     * @param {number} id 
     */
    function deleteTask(id) {
        if (taskList.value.length <= 1) {
            addLog('无法删除最后一个任务', LOG_LEVELS.ERROR);
            return;
        }

        const idx = taskList.value.findIndex(t => t.id === id);
        if (idx !== -1) {
            const taskName = taskList.value[idx].name;
            taskList.value.splice(idx, 1);

            addLog(`删除任务: ${taskName}`, LOG_LEVELS.WARN);
            if (currentTask.value?.id === id) {
                selectTask(taskList.value[0]);
            }

        }
    }

    // ============================================
    // 任务可见性
    // ============================================

    /**
     * 切换任务可见性
     * @param {Object} task 
     */
    function toggleTaskVisibility(task) {
        task.hidden = !task.hidden;
        addLog(`任务 "${task.name}" 已${task.hidden ? '隐藏' : '显示'}`, LOG_LEVELS.INFO);
    }

    // ============================================
    // 任务重命名
    // ============================================

    /**
     * 开始编辑任务名称
     * @param {Object} task 
     */
    function editTaskName(task) {
        renameTarget.value = task;
        renameValue.value = task.name;
        editTaskModal.value = true;
    }

    /**
     * 确认重命名
     */
    function confirmRename() {
        if (renameTarget.value && renameValue.value.trim()) {
            renameTarget.value.name = renameValue.value.trim();
            addLog(`重命名任务: ${renameValue.value}`, LOG_LEVELS.INFO);
        }
        cancelRename();
    }

    /**
     * 取消重命名
     */
    function cancelRename() {
        editTaskModal.value = false;
        renameValue.value = '';
        renameTarget.value = null;
    }

    return {
        // 状态
        taskList,
        currentTask,
        taskSearch,
        filteredTasks,

        // 重命名相关
        editTaskModal,
        renameValue,

        // 方法
        selectTask,
        createNewTask,
        deleteTask,
        toggleTaskVisibility,
        editTaskName,
        confirmRename,
        cancelRename,
    };
}

export default useTaskManager;
