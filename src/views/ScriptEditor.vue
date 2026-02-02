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
            @change="selectDevice(currentDevice)"
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
          <!-- 2. Left Panel (Sidebar) -->
          <div class="w-72 border-r border-base-300 flex flex-col bg-base-100 shadow-md z-1">
            <!-- Sidebar Tabs -->
            <div class="tabs tabs-boxed p-2 bg-base-100">
              <a class="tab flex-1" :class="{ 'tab-active': activeTab === 'toolbox' }" @click="activeTab = 'toolbox'"
                >工具</a
              >
              <a class="tab flex-1" :class="{ 'tab-active': activeTab === 'tasks' }" @click="activeTab = 'tasks'"
                >任务</a
              >
            </div>

            <div class="flex-1 overflow-y-auto p-2 h-full">
              <!-- TAB: TOOLBOX -->
              <div v-show="activeTab === 'toolbox'" class="h-full">
                <Toolbox @add-node="addNodeToCanvas" />
              </div>

              <!-- TAB: TASKS (Current Script's Task List) -->
              <div v-show="activeTab === 'tasks'">
                <div class="flex justify-between items-center mb-2 px-1">
                  <span class="text-xs font-bold opacity-50 uppercase">任务列表</span>
                  <button class="btn btn-xs btn-circle btn-primary" @click="createNewTask" title="创建任务">
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

                <div class="form-control w-full mb-2">
                  <input
                    type="text"
                    v-model="taskSearch"
                    placeholder="Search tasks..."
                    class="input input-bordered input-sm w-full"
                  />
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
                        task.hidden ? 'opacity-40' : 'opacity-100',
                      ]"
                      @click.stop="toggleTaskVisibility(task)"
                      :title="task.hidden ? 'Show Task' : 'Hide Task'"
                    >
                      <IconRenderer :icon="task.hidden ? 'eye-off' : 'eye'" class="w-4.5 h-4.5" />
                    </div>
                    <span
                      class="truncate font-semibold tracking-tight transition-colors"
                      :class="[
                        task.hidden ? 'opacity-40 italic font-normal' : '',
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
          </div>

          <!-- 3. Center (Canvas - Vue Flow) -->
          <div class="flex-1 relative bg-base-100 flex flex-col h-full" @dragover.prevent @drop="onDrop">
            <VueFlow
              ref="vueFlowRef"
              v-model:nodes="nodes"
              v-model:edges="edges"
              :node-types="nodeTypes"
              :delete-key-code="null"
              :default-viewport="{ zoom: 1 }"
              :min-zoom="0.2"
              :max-zoom="4"
              fit-view-on-init
              class="flex-1 h-full"
              @pane-click="onPaneClick"
              @connect="onConnect"
              @dragover="onDragOver"
              @dragleave="onDragLeave"
            >
              <Background
                pattern-color="#aaa"
                :gap="16"
                :style="{
                  backgroundColor: isDragOver ? '#e7f3ff' : 'transparent',
                  transition: 'background-color 0.2s ease',
                  opacity: isDragOver ? 0.5 : 1,
                }"
              />
              <Controls />
              <MiniMap v-if="showMiniMap" />
            </VueFlow>

            <!-- Floating Action Buttons -->
            <div class="absolute top-4 right-4 flex gap-2">
              <button
                class="btn btn-sm btn-circle btn-ghost"
                @click="showMiniMap = !showMiniMap"
                title="Toggle MiniMap"
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
                  <rect x="7" y="7" width="3" height="9" />
                  <rect x="14" y="7" width="3" height="5" />
                </svg>
              </button>
              <button class="btn btn-sm btn-circle btn-ghost" @click="fitView" title="Fit View">
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="16"
                  height="16"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path
                    d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"
                  />
                </svg>
              </button>
            </div>

            <!-- Console moved to parent level -->
          </div>

          <!-- 4. Right Panel (Properties) -->
          <PropertiesPanel
            :selectedNode="selectedNode"
            @delete-node="requestDeleteSelected"
            @update-node="updateNodeData"
          />
        </div>

        <div v-if="activeNavTab !== 'task'" class="flex-1 flex flex-col bg-base-100 overflow-hidden">
          <PolicyManager :active-tab="activeNavTab" :script-id="scriptId" :add-log="addLog" :log-levels="LOG_LEVELS" :get-uuid-v7="getUuidV7" />
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

      <!-- Delete Confirmation Modal -->
      <dialog class="modal" :class="{ 'modal-open': showDeleteConfirm }">
        <div class="modal-box">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="36"
            height="36"
            viewBox="0 0 24 24"
            fill="none"
            stroke="red"
            stroke-width="2"
          >
            <path
              d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"
              fill="none"
            />
            <line x1="12" y1="9" x2="12" y2="13" fill="none" />
            <line x1="12" y1="17" x2="12.01" y2="17" fill="none" />
          </svg>
          <p class="py-4">确认删除选择的 {{ nodesToDelete.length }} 个节点?</p>
          <div class="modal-action">
            <button class="btn" @click="cancelDelete">取消</button>
            <button class="btn btn-error" @click="confirmDelete">删除</button>
          </div>
        </div>
      </dialog>

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

<script setup>
import { ref, onMounted, markRaw, onUnmounted } from 'vue';
import { VueFlow, useVueFlow } from '@vue-flow/core';
import { Background } from '@vue-flow/background';
import { Controls } from '@vue-flow/controls';
import { MiniMap } from '@vue-flow/minimap';
import '@vue-flow/core/dist/style.css';
import '@vue-flow/core/dist/theme-default.css';
import '@vue-flow/controls/dist/style.css';
import '@vue-flow/minimap/dist/style.css';

// Components
import Toolbox from './script-editor/Toolbox.vue';
import PropertiesPanel from './script-editor/PropertiesPanel.vue';
import FlowNode from './script-editor/FlowNode.vue';
import IconRenderer from './script-editor/IconRenderer.vue';
import SideNavBar from './script-editor/components/SideNavBar.vue';
import PolicyManager from './script-editor/PolicyManagement.vue';

// Composables
import { useDragAndDrop } from './script-editor/composables/useDragAndDrop.js';
import { useConsoleLog, LOG_LEVELS } from './script-editor/composables/useConsoleLog.js';
import { useTaskManager } from './script-editor/composables/useTaskManager.js';
import { useThemeManager, useFlowEditor, useEditorDevice } from './script-editor/composables';

//store
import { editorThemeKey, deviceKey, setToStore, getFromStore } from '../store/store.js';

//data
import { useDevices } from '../assets/js/useDevices.js';

// ============================================
// 核心状态
// ============================================
import { invoke } from '@tauri-apps/api/core';

const vueFlowRef = ref(null);
const showMiniMap = ref(false);
const activeTab = ref('tasks');
const activeNavTab = ref('task');

// 从 URL 解析脚本 ID
const params = new URLSearchParams(window.location.search);
const scriptId = ref(params.get('id') || '');
const scriptName = ref('加载中...');
const scriptInfo = ref(null);

// Console Resizing
const consoleHeight = ref(160);
const isResizing = ref(false);

const startResize = (_) => {
  isResizing.value = true;
  document.addEventListener('mousemove', onResize);
  document.addEventListener('mouseup', stopResize);
  document.body.style.cursor = 'ns-resize';
};

const onResize = (e) => {
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

// 1. Console Log
const { consoleLogs, logClass, addLog, clearConsole } = useConsoleLog();

// 2. Theme Manager
const { currentEditorTheme, toggleTheme, initTheme } = useThemeManager();

// 5. 设备管理
const { getAllDevices,getUuidV7 } = useDevices();
const { devices, currentDevice, loadDevices, selectDevice } = useEditorDevice({
  getAllDevices,
  getFromStore,
  setToStore,
  deviceKey,
});

// 3.VueFlow 设置
const nodeTypes = { custom: markRaw(FlowNode) };

const { screenToFlowCoordinate } = useVueFlow();

const {
  nodes,
  edges,
  selectedNode,
  showDeleteConfirm,
  nodesToDelete,

  // 节点操作
  addNodeToCanvas,
  updateNodeData,

  //vue-flow 内容
  onPaneClick,
  fitView,

  // 连接
  onConnect,

  // 删除
  requestDeleteSelected,
  confirmDelete,
  cancelDelete,
} = useFlowEditor({ addLog, LOG_LEVELS, getUuidV7 });



// 4. Task Manager
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
} = useTaskManager({ nodes, edges, addLog, LOG_LEVELS, getUuidV7 });

// ============================================
// 保存与加载
// ============================================
const loadScriptData = async () => {
  if (!scriptId.value) return;

  try {
    const table = await invoke('get_script_by_id_cmd', { scriptId: scriptId.value });
    if (table) {
      scriptInfo.value = table.data;
      scriptName.value = table.data.name;
      addLog(`加载脚本成功: ${scriptName.value}`, LOG_LEVELS.INFO);

      // 加载任务
      const tasks = await invoke('get_script_tasks_cmd', { scriptId: scriptId.value });
      if (tasks && tasks.length > 0) {
        taskList.value = tasks.map((t) => ({
          id: t.id,
          name: t.name,
          hidden: t.isHidden,
          nodes: t.nodes,
          edges: t.edges,
          uiData: t.data?.uiData || {},
          variables: t.data?.variables || {},
        }));
      } else {
        taskList.value = [];
        await createNewTask();
      }

      if (taskList.value.length > 0) {
        selectTask(taskList.value[0]);
      }
    }
  } catch (e) {
    addLog(`加载脚本失败: ${e}`, LOG_LEVELS.ERROR);
  }
};

const saveScript = async () => {
  try {
    // 1. 更新当前任务的数据
    if (currentTask.value) {
      currentTask.value.nodes = [...nodes.value];
      currentTask.value.edges = [...edges.value];
    }

    // 2. 将所有任务打包到 scriptInfo 中
    if (scriptInfo.value) {
      scriptInfo.value.updateTime = new Date().toISOString();
      scriptInfo.value.tasks = taskList.value.map((t) => ({
        id: t.id,
        name: t.name,
        isHidden: t.hidden,
        nodes: t.nodes,
        edges: t.edges,
        data: {
          uiData: t.uiData || {},
          variables: t.variables || {},
        },
      }));

      // 3. 保存全量脚本信息
      await invoke('save_script_cmd', {
        script: {
          id: scriptId.value,
          data: scriptInfo.value,
        },
      });

      addLog('保存脚本成功', LOG_LEVELS.SUCCESS);
    }
  } catch (e) {
    addLog(`保存失败: ${e}`, LOG_LEVELS.ERROR);
  }
};

// ============================================
// 拖放功能
// ============================================
const { onDragOver, onDrop, onDragLeave, isDragOver } = useDragAndDrop({
  onAddNode: addNodeToCanvas,
  screenToFlowCoordinate,
});

// ============================================
// 键盘事件
// ============================================
const handleKeyDown = (event) => {
  if (event.key === 'Delete' || event.key === 'Backspace') {
    const activeElement = document.activeElement;
    if (activeElement.tagName === 'INPUT' || activeElement.tagName === 'TEXTAREA') {
      return;
    }
    event.preventDefault();
    requestDeleteSelected();
  }
};

// ============================================
// 生命周期
// ============================================
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
<!-- MyComponent.vue -->
<style>
@import '../assets/css/script-editor.css';
</style>
