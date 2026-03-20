<template>
  <div class="h-screen w-screen flex bg-base-100 overflow-hidden" :data-theme="currentEditorTheme">
    <!-- 0. Vertical Side Navigation -->
    <SideNavBar v-model="activeNavTab" />

    <div class="flex-1 flex flex-col overflow-hidden">
      <!-- 1. Header (Toolbar) -->
      <div class="h-14 border-b border-base-300 flex items-center px-4 justify-between bg-base-200 shadow-sm z-10">
        <div class="flex items-center gap-4">
          <h1 class="text-lg font-bold flex items-center gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path fill="none" d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z" />
              <polyline points="14 2 14 8 20 8" />
            </svg>
            {{ scriptName }}
          </h1>
          <span class="text-sm opacity-50">></span>
          <!-- Current Task Badge -->
          <div class="badge badge-secondary badge-outline" v-if="currentTask">
            {{ currentTask.name }}
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button class="btn btn-sm btn-ghost btn-circle" @click="toggleTheme(editorThemeKey)" title="Switch Theme">
            <svg
              v-if="currentEditorTheme === 'light'"
              xmlns="http://www.w3.org/2000/svg"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" fill="none"></path>
            </svg>
            <svg
              v-else
              xmlns="http://www.w3.org/2000/svg"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <circle cx="12" cy="12" r="5" fill="none" />
              <line x1="12" y1="1" x2="12" y2="3" fill="none" />
              <line x1="12" y1="21" x2="12" y2="23" fill="none" />
              <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" fill="none" />
              <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" fill="none" />
              <line x1="1" y1="12" x2="3" y2="12" fill="none" />
              <line x1="21" y1="12" x2="23" y2="12" fill="none" />
              <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" fill="none" />
              <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" fill="none" />
            </svg>
          </button>

          <!-- 设备选择器 -->
          <select
            class="select select-sm select-bordered max-w-xs"
            v-model="currentDevice"
            @change="selectDevice(currentDevice!)"
          >
            <option :value="null" disabled>Select Device</option>
            <option v-for="device in devices" :key="device.id" :value="device.id">
              {{ device.data.deviceName }}
            </option>
          </select>
          <button class="btn btn-sm btn-success gap-2 text-white">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <polygon points="5 3 19 12 5 21 5 3" fill="none"></polygon>
            </svg>
            Run
          </button>
          <button class="btn btn-sm btn-primary" @click="saveScript">Save</button>
        </div>
      </div>

      <!-- Main Content Area -->
      <div class="flex-1 flex overflow-hidden">
        <!-- TAB: TASK EDITOR -->
        <div v-show="activeNavTab === 'task'" class="flex-1 flex overflow-hidden">
          <!-- 2. Left Panel (Sidebar - Task List Only) -->
          <div class="w-72 border-r border-base-300 flex flex-col bg-base-100 shadow-md z-1">
            <div class="p-3 bg-base-100 border-b border-base-300">
              <span class="text-xs font-bold opacity-50 uppercase">任务列表</span>
            </div>

            <div class="flex-1 overflow-y-auto p-2 h-full">
              <div class="flex justify-between items-center mb-2 px-1">
                <div class="form-control w-full">
                  <input
                    type="text"
                    v-model="taskSearch"
                    placeholder="搜索任务..."
                    class="input input-bordered input-sm w-full"
                  />
                </div>
                <button class="btn btn-xs btn-circle btn-primary ml-2 shrink-0" @click="createNewTask" title="创建任务">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="14"
                    height="14"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                  >
                    <line x1="12" y1="5" x2="12" y2="19" fill="none"></line>
                    <line x1="5" y1="12" x2="19" y2="12" fill="none"></line>
                  </svg>
                </button>
              </div>

              <!-- Task List -->
              <div
                v-for="task in filteredTasks"
                :key="task.id"
                class="group p-2 rounded-lg cursor-pointer flex items-center justify-between text-sm mb-1.5 transition-all duration-200 relative overflow-hidden active:scale-[0.98]"
                :class="[
                  currentTask?.id === task.id
                    ? 'bg-primary text-white shadow-md'
                    : 'hover:bg-primary/10 hover:text-primary bg-base-200/50 text-base-content/70',
                ]"
                @click="selectTask(task)"
              >
                <!-- Selected indicator bar -->
                <div v-if="currentTask?.id === task.id" class="absolute left-0 top-0 bottom-0 w-1 bg-white/40"></div>

                <div class="flex items-center gap-2.5 truncate flex-1 z-10">
                  <!-- Task Visibility Icon with background -->
                  <div
                    class="w-8 h-8 rounded-lg flex items-center justify-center shrink-0 shadow-sm transition-all duration-200"
                    :class="[
                      currentTask?.id === task.id
                        ? 'bg-white/20 group-hover:bg-white/30 text-white'
                        : 'bg-base-300 group-hover:bg-primary/20 text-base-content/60 group-hover:text-primary',
                      task.isHidden ? 'opacity-40' : 'opacity-100',
                    ]"
                    @click.stop="toggleTaskVisibility(task)"
                    :title="task.isHidden ? 'Show Task' : 'Hide Task'"
                  >
                    <IconRenderer :icon="task.isHidden ? 'eye-off' : 'eye'" class="w-4.5 h-4.5" />
                  </div>
                  <span
                    class="truncate font-semibold tracking-tight transition-colors"
                    :class="[
                      task.isHidden ? 'opacity-40 italic font-normal' : '',
                      currentTask?.id === task.id ? 'text-white' : '',
                    ]"
                  >
                    {{ task.name }}
                  </span>
                </div>

                <div
                  class="flex items-center gap-1 ml-2 translate-x-1 opacity-0 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-200 z-10"
                >
                  <button
                    class="btn btn-xs btn-circle btn-ghost transition-colors"
                    :class="
                      currentTask?.id === task.id
                        ? 'hover:bg-white/20 text-white'
                        : 'hover:bg-primary/20 text-primary'
                    "
                    @click.stop="editTaskName(task)"
                    title="Rename"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="12"
                      height="12"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2.5"
                    >
                      <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"></path>
                      <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"></path>
                    </svg>
                  </button>
                  <button
                    class="btn btn-xs btn-circle btn-ghost transition-colors"
                    :class="
                      currentTask?.id === task.id ? 'hover:bg-white/20 text-white' : 'hover:bg-error/20 text-error'
                    "
                    @click.stop="deleteTask(task.id)"
                    title="Delete"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="12"
                      height="12"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      stroke-width="2.5"
                      stroke-linecap="round"
                      stroke-linejoin="round"
                    >
                      <polyline points="3 6 5 6 21 6"></polyline>
                      <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
                    </svg>
                  </button>
                </div>
              </div>

              <div v-if="filteredTasks.length === 0" class="text-center text-sm opacity-50 py-8">No tasks found</div>
            </div>
          </div>

          <!-- 3. Center (Action Sequence Editor) -->
          <div class="flex-1 relative bg-base-100 flex flex-col h-full overflow-y-auto custom-scrollbar">
            <ActionSequenceEditor
              :steps="steps"
              @update:steps="steps = $event"
            />
          </div>
        </div>

        <div v-show="activeNavTab !== 'task'" class="flex-1 flex flex-col bg-base-100 overflow-hidden">
          <PolicyManager
            ref="policyManagerRef"
            :active-tab="activeNavTab"
            :script-id="scriptId"
            :add-log="addLog"
            :log-levels="LOG_LEVELS"
            :get-uuid-v7="getUuidV7"
          />
        </div>
      </div>

      <!-- 5. Global Resizable Console -->
      <div
        class="border-t border-base-300 flex flex-col bg-neutral text-neutral-content z-10 shrink-0 relative"
        :style="{ height: `${consoleHeight}px` }"
      >
        <!-- Resize Handle -->
        <div
          class="absolute top-0 left-0 right-0 h-1 cursor-ns-resize hover:bg-primary/50 transition-colors z-20"
          @mousedown="startResize"
        ></div>

        <div class="p-1 px-4 text-xs font-bold bg-neutral-focus flex justify-between items-center h-8 shrink-0">
          <span class="flex items-center gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
            >
              <path d="M4 17l6-6-6-6M12 19h8" />
            </svg>
            Console Output
          </span>
          <button class="btn btn-xs btn-ghost text-xs" @click="clearConsole">Clear</button>
        </div>
        <div ref="consoleRef" class="flex-1 p-2 font-mono text-xs overflow-y-auto">
          <div v-for="(log, idx) in consoleLogs" :key="idx" :class="logClass(log.level)">
            [{{ log.time }}] {{ log.message }}
          </div>
          <div v-if="consoleLogs.length === 0" class="opacity-50 text-center py-4">No logs yet...</div>
        </div>
      </div>

      <!-- Task Rename Modal -->
      <dialog class="modal" :class="{ 'modal-open': editTaskModal }">
        <div class="modal-box">
          <h3 class="font-bold text-lg">编辑</h3>
          <div class="form-control w-full py-4">
            <input
              type="text"
              v-model="renameValue"
              class="input input-bordered w-full"
              placeholder="Enter task name..."
              @keyup.enter="confirmRename"
            />
          </div>
          <div class="modal-action">
            <button class="btn" @click="cancelRename">取消</button>
            <button class="btn btn-primary" @click="confirmRename">保存</button>
          </div>
        </div>
      </dialog>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, provide } from 'vue';

// Components
import IconRenderer from './script-editor/IconRenderer.vue';
import SideNavBar from './script-editor/components/SideNavBar.vue';
import PolicyManager from './script-editor/PolicyManagement.vue';
import ActionSequenceEditor from './script-editor/components/ActionSequenceEditor.vue';

// Composables
import {
  useConsoleLog,
  LOG_LEVELS,
  useTaskManager,
  useThemeManager,
  useEditorDevice,
} from './script-editor/composables';

// store
import { editorThemeKey, deviceKey, setToStore, getFromStore } from '@/store/store';

// data
import { useDevices } from '@/assets/js/useDevices';
import type { ScriptTable, ScriptInfo, ScriptTaskTable, Step } from '@/types/bindings';

// ============================================
// 核心状态
// ============================================
import { invoke } from '@tauri-apps/api/core';

const activeNavTab = ref('task');

const params = new URLSearchParams(window.location.search);
const scriptId = ref(params.get('id') || '');
const scriptName = ref('加载中...');
const scriptInfo = ref<ScriptInfo | null>(null);
provide('scriptInfo', scriptInfo);

const consoleHeight = ref(160);
const isResizing = ref(false);

const startResize = (_: MouseEvent) => {
  isResizing.value = true;
  document.addEventListener('mousemove', onResize);
  document.addEventListener('mouseup', stopResize);
  document.body.style.cursor = 'ns-resize';
};

const onResize = (e: MouseEvent) => {
  if (!isResizing.value) return;
  const newHeight = window.innerHeight - e.clientY;
  if (newHeight > 60 && newHeight < window.innerHeight * 0.7) {
    consoleHeight.value = newHeight;
  }
};

const stopResize = () => {
  isResizing.value = false;
  document.removeEventListener('mousemove', onResize);
  document.removeEventListener('mouseup', stopResize);
  document.body.style.cursor = 'default';
};

// ============================================
// Composables 初始化
// ============================================

const { consoleLogs, consoleRef, logClass, addLog, clearConsole } = useConsoleLog();
const { currentEditorTheme, toggleTheme, initTheme } = useThemeManager();

const { getAllDevices, getUuidV7 } = useDevices();
const { devices, currentDevice, loadDevices, selectDevice } = useEditorDevice({
  getAllDevices,
  getFromStore: (key: string) => getFromStore<string>(key).then((r) => r ?? null),
  setToStore: (key: string, value: string) => setToStore(key, value),
  deviceKey,
});

const policyManagerRef = ref<any>(null);

// Steps for ActionSequenceEditor
const steps = ref<Step[]>([]);

const {
  taskList,
  currentTask,
  taskSearch,
  filteredTasks,
  editTaskModal,
  renameValue,
  selectTask,
  createNewTask,
  deleteTask,
  toggleTaskVisibility,
  editTaskName,
  confirmRename,
  cancelRename,
} = useTaskManager({ steps, addLog, LOG_LEVELS, getUuidV7 });

// ============================================
// 保存与加载
// ============================================
const loadScriptData = async () => {
  if (!scriptId.value) return;

  try {
    const table = await invoke<ScriptTable>('get_script_by_id_cmd', { scriptId: scriptId.value });
    if (table) {
      scriptInfo.value = table.data;
      scriptName.value = table.data.name;
      addLog(`加载脚本成功: ${scriptName.value}`, LOG_LEVELS.INFO);

      const tasks = await invoke<ScriptTaskTable[]>('get_script_tasks_cmd', { scriptId: scriptId.value });
      if (tasks && tasks.length > 0) {
        taskList.value = tasks;
      } else {
        taskList.value = [];
        await createNewTask();
      }

      if (taskList.value.length > 1) {
        // @ts-ignore
        selectTask(taskList.value[0]);
      }
    }
  } catch (e) {
    addLog(`加载脚本失败: ${e}`, LOG_LEVELS.ERROR);
  }
};

const saveScript = async () => {
  try {
    if (policyManagerRef.value?.saveCurrentPolicy) {
      await policyManagerRef.value.saveCurrentPolicy();
    }
    if (policyManagerRef.value?.saveComposers) {
      await policyManagerRef.value.saveComposers();
    }

    // 保存当前任务的 steps 到 nodes 字段
    if (currentTask.value) {
      (currentTask.value as any).data.steps = [...steps.value];
    }

    if (scriptInfo.value) {
      scriptInfo.value.updateTime = new Date().toISOString();
      const payloadTasks = taskList.value.map((t, index) => ({
        id: t.id,
        scriptId: t.scriptId || scriptId.value,
        name: t.name,
        isHidden: t.isHidden,
        taskType: t.taskType || 'main',
        index: t.index !== undefined ? t.index : index,
        createdAt: t.createdAt || new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        deletedAt: t.deletedAt || null,
        isDeleted: t.isDeleted || false,
        data: {
          uiData: t.data?.uiData || {},
          variables: t.data?.variables || {},
        },
      }));

      await invoke('save_script_cmd', {
        script: {
          id: scriptId.value,
          data: {
            ...scriptInfo.value,
            tasks: payloadTasks,
          },
        },
      });

      addLog('保存脚本成功', LOG_LEVELS.SUCCESS);
    }
  } catch (e) {
    addLog(`保存失败: ${e}`, LOG_LEVELS.ERROR);
  }
};

const handleKeyDown = (event: KeyboardEvent) => {
  if ((event.ctrlKey || event.metaKey) && event.key === 's') {
    event.preventDefault();
    saveScript();
    return;
  }
};

onMounted(async () => {
  await initTheme(editorThemeKey);
  await loadDevices();
  await loadScriptData();
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});
</script>

<style>
@import '../assets/css/script-editor.css';
</style>
