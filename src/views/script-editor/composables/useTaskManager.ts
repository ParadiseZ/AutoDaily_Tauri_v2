import { ref, computed } from 'vue';
import type { Ref } from 'vue';
import type { ScriptTaskTable } from '@/types/bindings';
import type { Step } from '@/types/bindings';
import type { JsonValue } from '@/types/bindings/serde_json/JsonValue';

interface TaskOptions {
    steps: Ref<Step[]>;
    addLog?: (message: string, level: any) => void;
    LOG_LEVELS?: any;
    getUuidV7?: () => Promise<string>;
}

export function useTaskManager(options: TaskOptions) {
    const { steps, addLog = () => { }, LOG_LEVELS = {}, getUuidV7 = async () => '' } = options;

    // 任务列表
    const taskList = ref<ScriptTaskTable[]>([]);

    // 当前任务
    const currentTask = ref<ScriptTaskTable | null>(null);

    // 搜索关键词
    const taskSearch = ref('');

    // 过滤后的任务列表
    const filteredTasks = computed(() => {
        if (!taskSearch.value) return taskList.value;
        const search = taskSearch.value.toLowerCase();
        return taskList.value.filter((t: any) => t.name.toLowerCase().includes(search));
    });

    // 重命名相关
    const editTaskModal = ref(false);
    const renameValue = ref('');
    const renameTarget = ref<ScriptTaskTable | null>(null);

    // ============================================
    // 任务选择
    // ============================================

    function selectTask(task: ScriptTaskTable) {
        // 保存当前任务的 steps 到 nodes 字段 (兼容后端存储)
        if (currentTask.value && steps.value) {
            (currentTask.value as any).data.steps = [...steps.value];
            (currentTask.value as any).data.steps = [];
        }

        currentTask.value = task;

        // 加载任务的步骤
        if (steps.value !== undefined) {
            // 兼容: 如果 nodes 里存的是 Step[] 则直接用, 否则清空
            const taskNodes = (task as any).data.steps;
            if (Array.isArray(taskNodes)) {
                steps.value = [...taskNodes];
            } else {
                steps.value = [];
            }
        }

        addLog(`切换任务： ${task.name}`, LOG_LEVELS.INFO);
    }

    // ============================================
    // 任务创建/删除
    // ============================================

    async function createNewTask() {
        const newId = await getUuidV7();
        const newTaskCount = taskList.value.length + 1;
        
        const newTask: ScriptTaskTable = {
            id: newId,
            scriptId: '', // Will be set by parent
            name: `新任务 ${newTaskCount}`,
            isHidden: false,
            taskType: 'main',
            index: newTaskCount,
            data: {
                uiData: {} as JsonValue,
                variables: {} as JsonValue,
                steps: [] as Step[],
            },
            createdAt: new Date().toISOString(),
            updatedAt: new Date().toISOString(),
            deletedAt: null,
            isDeleted: false,
        };

        // @ts-ignore
        taskList.value.push(newTask);
        selectTask(newTask);
    }

    function deleteTask(id: string) {
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

    function toggleTaskVisibility(task: ScriptTaskTable) {
        task.isHidden = !task.isHidden;
        addLog(`任务 "${task.name}" 已${task.isHidden ? '隐藏' : '显示'}`, LOG_LEVELS.INFO);
    }

    // ============================================
    // 任务重命名
    // ============================================

    function editTaskName(task: ScriptTaskTable) {
        renameTarget.value = task;
        renameValue.value = task.name;
        editTaskModal.value = true;
    }

    function confirmRename() {
        if (renameTarget.value && renameValue.value.trim()) {
            renameTarget.value.name = renameValue.value.trim();
            addLog(`重命名任务: ${renameValue.value}`, LOG_LEVELS.INFO);
        }
        cancelRename();
    }

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
