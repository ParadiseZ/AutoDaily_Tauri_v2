<template>
  <div class="h-screen w-screen flex flex-col bg-base-100 overflow-hidden" :data-theme="currentTheme">
    <!-- 1. Header (Toolbar) -->
    <div class="h-14 border-b border-base-300 flex items-center px-4 justify-between bg-base-200 shadow-sm z-10">
      <div class="flex items-center gap-4">
        <h1 class="text-lg font-bold flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path  fill="none" d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/></svg>
            Script Editor
        </h1>
        <!-- Script Name Badge -->
        <div class="badge badge-primary badge-lg gap-1">
          <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" fill="none"></path></svg>
          {{ scriptName }}
        </div>
        <span class="text-sm opacity-50">›</span>
        <!-- Current Task Badge -->
        <div class="badge badge-secondary badge-outline" v-if="currentTask">
          {{ currentTask.name }}
        </div>
      </div>
      
      <div class="flex items-center gap-2">
        <button class="btn btn-sm btn-ghost btn-circle" @click="toggleTheme" title="Switch Theme">
            <svg v-if="currentTheme === 'light'" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" fill="none"></path></svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5" fill="none"/><line x1="12" y1="1" x2="12" y2="3" fill="none"/><line x1="12" y1="21" x2="12" y2="23" fill="none"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64" fill="none"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78" fill="none"/><line x1="1" y1="12" x2="3" y2="12" fill="none"/><line x1="21" y1="12" x2="23" y2="12" fill="none"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36" fill="none"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22" fill="none"/></svg>
        </button>

        <select class="select select-sm select-bordered max-w-xs">
          <option disabled selected>Select Device</option>
          <option>MuMu12 (127.0.0.1:7555)</option>
        </select>
        <button class="btn btn-sm btn-success gap-2 text-white">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="5 3 19 12 5 21 5 3" fill="none"></polygon></svg>
          Run
        </button>
        <button class="btn btn-sm btn-primary" @click="saveScript">Save</button>
      </div>
    </div>

    <!-- Main Content Grid -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 2. Left Panel (Sidebar) -->
      <div class="w-72 border-r border-base-300 flex flex-col bg-base-100 shadow-md z-1">
        <!-- Sidebar Tabs -->
        <div class="tabs tabs-boxed p-2 bg-base-100">
            <a class="tab flex-1" :class="{'tab-active': activeTab === 'toolbox'}" @click="activeTab = 'toolbox'">Toolbox</a>
            <a class="tab flex-1" :class="{'tab-active': activeTab === 'tasks'}" @click="activeTab = 'tasks'">Tasks</a>
        </div>

        <div class="flex-1 overflow-y-auto p-2 h-full">
            <!-- TAB: TOOLBOX -->
            <div v-show="activeTab === 'toolbox'" class="h-full">
                <Toolbox @add-node="addNodeToCanvas" />
            </div>

            <!-- TAB: TASKS (Current Script's Task List) -->
            <div v-show="activeTab === 'tasks'">
                <div class="flex justify-between items-center mb-2 px-1">
                    <span class="text-xs font-bold opacity-50 uppercase">Task List</span>
                    <button class="btn btn-xs btn-circle btn-primary" @click="createNewTask" title="New Task">
                         <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19" fill="none"></line><line x1="5" y1="12" x2="19" y2="12" fill="none"></line></svg>
                    </button>
                </div>

                <div class="form-control w-full mb-2">
                    <input type="text" v-model="taskSearch" placeholder="Search tasks..." class="input input-bordered input-sm w-full" />
                </div>
                
                <!-- Task List -->
                <div v-for="task in filteredTasks" :key="task.id" 
                     class="p-2 rounded hover:bg-base-200 cursor-pointer flex items-center justify-between group text-sm mb-1"
                     :class="{'bg-primary/10 text-primary font-bold': currentTask?.id === task.id}"
                     @click="selectTask(task)">
                    <div class="flex items-center gap-2 truncate">
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 11 12 14 22 4" fill="none"></polyline><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"></path></svg>
                        {{ task.name }}
                    </div>
                    <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100">
                        <button class="btn btn-xs btn-ghost btn-circle" 
                                @click.stop="editTaskName(task)" title="Rename">
                            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2"><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7" fill="none"></path><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z" fill="none"></path></svg>
                        </button>
                        <button class="btn btn-xs btn-ghost btn-circle text-error" 
                                @click.stop="deleteTask(task.id)" title="Delete">
                            <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" fill="none"></path></svg>
                        </button>
                    </div>
                </div>
                
                <div v-if="filteredTasks.length === 0" class="text-center text-sm opacity-50 py-8">
                  No tasks found
                </div>
            </div>
        </div>
      </div>

      <!-- 3. Center (Canvas - Vue Flow) -->
      <div class="flex-1 relative bg-base-100 flex flex-col h-full" 
           @dragover.prevent 
           @drop="onDrop">
        
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
        >
            <Background pattern-color="#aaa" :gap="16" />
            <Controls />
            <MiniMap v-if="showMiniMap" />
        </VueFlow>
        
        <!-- Floating Action Buttons -->
        <div class="absolute top-4 right-4 flex gap-2">
          <button class="btn btn-sm btn-circle btn-ghost" @click="showMiniMap = !showMiniMap" title="Toggle MiniMap">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><rect x="7" y="7" width="3" height="9"/><rect x="14" y="7" width="3" height="5"/></svg>
          </button>
          <button class="btn btn-sm btn-circle btn-ghost" @click="fitView" title="Fit View">
            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3"/></svg>
          </button>
        </div>
        
        <!-- Bottom Panel (Console) -->
        <div class="h-40 border-t border-base-300 flex flex-col bg-neutral text-neutral-content z-10 shrink-0">
            <div class="p-1 px-4 text-xs font-bold bg-neutral-focus flex justify-between items-center h-8">
                <span>Console Output</span>
                <button class="btn btn-xs btn-ghost text-xs" @click="clearConsole">Clear</button>
            </div>
            <div class="flex-1 p-2 font-mono text-xs overflow-y-auto">
                <div v-for="(log, idx) in consoleLogs" :key="idx" :class="logClass(log.level)">
                  [{{ log.time }}] {{ log.message }}
                </div>
                <div v-if="consoleLogs.length === 0" class="opacity-50">No logs yet...</div>
            </div>
        </div>
      </div>

      <!-- 4. Right Panel (Properties) -->
      <PropertiesPanel 
          :selectedNode="selectedNode" 
          @delete-node="requestDeleteSelected"
          @update-node="updateNodeData"
      />
    </div>

    <!-- Delete Confirmation Modal -->
    <dialog class="modal" :class="{ 'modal-open': showDeleteConfirm }">
      <div class="modal-box">
        <h3 class="font-bold text-lg">Confirm Delete</h3>
        <p class="py-4">Are you sure you want to delete {{ nodesToDelete.length }} selected node(s)?</p>
        <div class="modal-action">
          <button class="btn" @click="cancelDelete">Cancel</button>
          <button class="btn btn-error" @click="confirmDelete">Delete</button>
        </div>
      </div>
    </dialog>
    
    <!-- Task Rename Modal -->
    <dialog class="modal" :class="{ 'modal-open': showRenameModal }">
      <div class="modal-box">
        <h3 class="font-bold text-lg">Rename Task</h3>
        <div class="form-control w-full py-4">
          <input type="text" v-model="renameValue" class="input input-bordered w-full" placeholder="Enter task name..." @keyup.enter="confirmRename" />
        </div>
        <div class="modal-action">
          <button class="btn" @click="cancelRename">Cancel</button>
          <button class="btn btn-primary" @click="confirmRename">Save</button>
        </div>
      </div>
    </dialog>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, markRaw, onUnmounted } from 'vue';
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

//store
import { getFromStore,setToStore,defaultEditorThemeKey } from '../store/store.js';

//log
const INFO = "info";
const WARN = "warn";
const ERROR = "error";
const SUCCESS = "success";

// --- State ---
const nodes = ref([]);
const edges = ref([]);
const vueFlowRef = ref(null);
const showMiniMap = ref(false);

// Register Custom Node Type
const nodeTypes = {
  custom: markRaw(FlowNode),
};

const {onNodeClick, onConnect, addEdges, removeNodes, getSelectedNodes, fitView: flowFitView } = useVueFlow();
const activeTab = ref('tasks'); // Start on tasks tab
const currentTheme = ref('light');

// --- Script Data (Simulated - in reality, passed as props or loaded from backend) ---
const scriptName = ref('崩坏三');
const scriptId = ref(1);

// --- Task Management ---
const taskList = ref([
  { 
    id: 1, 
    name: 'Login',
    nodes: [
      { id: '1', type: 'custom', label: 'Start', position: { x: 200, y: 50 }, data: { type: 'start' } },
      { id: '2', type: 'custom', label: 'Find Login', position: { x: 200, y: 150 }, data: { type: 'find_image', target: 'login_btn.png' } },
      { id: '3', type: 'custom', label: 'Click Login', position: { x: 200, y: 250 }, data: { type: 'click' } },
    ], 
    edges: [
      { id: 'e1-2', source: '1', target: '2' },
      { id: 'e2-3', source: '2', target: '3' },
    ] 
  },
  { 
    id: 2, 
    name: 'Sign In',
    nodes: [
      { id: 'start-1', type: 'custom', label: 'Start', position: { x: 200, y: 50 }, data: { type: 'start' } },
    ], 
    edges: [] 
  },
  { 
    id: 3, 
    name: 'Claim Rewards',
    nodes: [
      { id: 'start-1', type: 'custom', label: 'Start', position: { x: 200, y: 50 }, data: { type: 'start' } },
    ], 
    edges: [] 
  },
  { 
    id: 4, 
    name: 'Daily Sweep',
    nodes: [
      { id: 'start-1', type: 'custom', label: 'Start', position: { x: 200, y: 50 }, data: { type: 'start' } },
    ], 
    edges: [] 
  },
]);

const currentTask = ref(null);
const taskSearch = ref('');

const filteredTasks = computed(() => {
  if (!taskSearch.value) return taskList.value;
  const search = taskSearch.value.toLowerCase();
  return taskList.value.filter(t => t.name.toLowerCase().includes(search));
});

// --- Console Logs ---
const consoleLogs = ref([
  { time: '10:00:01', level: INFO, message: 'Script Editor initialized.' },
]);

const logClass = (level) => {
  switch (level) {
    case 'success': return 'text-success';
    case 'error': return 'text-error';
    case 'warn': return 'text-warning';
    default: return 'text-info';
  }
};

const addLog = (message, level = INFO) => {
  const now = new Date();
  const time = now.toTimeString().slice(0, 8);
  consoleLogs.value.push({ time, level, message });
};

const clearConsole = () => {
  consoleLogs.value = [];
};

// --- Lifecycle ---
onMounted(() => {
  document.documentElement.setAttribute('data-theme', currentTheme.value);
  if (taskList.value.length > 0) {
    selectTask(taskList.value[0]);
  }
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
});

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

// --- Selection Logic ---
const selectedNode = ref(null);

onNodeClick((event) => {
  selectedNode.value = event.node;
});

const onPaneClick = () => {
  selectedNode.value = null;
};

// --- Connection Logic ---
onConnect((params) => {
  addEdges([{ ...params, id: `${params.source}-${params.target}` }]);
  edges.value.forEach(edge => {
    addLog(`${edge.id}, ${edge.name}, ${edge.source} → ${edge.target}`, INFO);
  });
  //addLog(`edges: ${edges.value.keys()} → ${edges.value.length}`, INFO);
  addLog(`连接: ${params.source} → ${params.target}`, INFO);
});

// ---------------------------------------------- Delete Confirmation Logic --------------------------------------------
const showDeleteConfirm = ref(false);
const nodesToDelete = ref([]);

const requestDeleteSelected = () => {
  const selected = getSelectedNodes.value;
  // Filter out start nodes - they cannot be deleted
  const deletable = selected.filter(n => n.data?.type !== 'start' && n.data?.type !== 'input');
  if (deletable.length > 0) {
    nodesToDelete.value = deletable;
    showDeleteConfirm.value = true;
  }
};

const confirmDelete = () => {
  removeNodes(nodesToDelete.value);
  addLog(`删除 ${nodesToDelete.value.length} 个节点`, WARN);
  selectedNode.value = null;
  showDeleteConfirm.value = false;
  nodesToDelete.value = [];
};

const cancelDelete = () => {
  showDeleteConfirm.value = false;
  nodesToDelete.value = [];
};

// ------------------------------------------------- 任务相关 -----------------------------------------------------------
const selectTask = (task) => {
  // Save current task's state
  if (currentTask.value) {
    currentTask.value.nodes = [...nodes.value];
    currentTask.value.edges = [...edges.value];
  }
  
  currentTask.value = task;
  nodes.value = task.nodes.map(n => ({ ...n, type: 'custom' }));
  edges.value = [...task.edges];
  selectedNode.value = null;
  addLog(`切换任务： ${task.name}`, INFO);
};

const createNewTask = () => {
  const newId = Math.max(...taskList.value.map(t => t.id), 0) + 1;
  const newTask = {
    id: newId,
    name: `New Task ${newId}`,
    nodes: [{ id: 'start-1', type: 'custom', label: 'Start', position: { x: 200, y: 50 }, data: { type: 'start' } }],
    edges: []
  };
  taskList.value.push(newTask);
  selectTask(newTask);
  addLog(`Created new task: ${newTask.name}`, SUCCESS);
};

const deleteTask = (id) => {
  if (taskList.value.length <= 1) {
    addLog('Cannot delete the last task', ERROR);
    return;
  }
  
  const idx = taskList.value.findIndex(t => t.id === id);
  if (idx !== -1) {
    const taskName = taskList.value[idx].name;
    taskList.value.splice(idx, 1);
    
    if (currentTask.value?.id === id) {
      selectTask(taskList.value[0]);
    }
    addLog(`删除任务: ${taskName}`, 'warn');
  }
};

// ----------------重命名任务-------------
const showRenameModal = ref(false);
const renameValue = ref('');
const renameTarget = ref(null);

const editTaskName = (task) => {
  renameTarget.value = task;
  renameValue.value = task.name;
  showRenameModal.value = true;
};

const confirmRename = () => {
  if (renameTarget.value && renameValue.value.trim()) {
    renameTarget.value.name = renameValue.value.trim();
    addLog(`重命名任务: ${renameValue.value}`, 'info');
  }
  cancelRename();
};

const cancelRename = () => {
  showRenameModal.value = false;
  renameValue.value = '';
  renameTarget.value = null;
};

// --- Node Data Update ---
const updateNodeData = (nodeId, updates) => {
  const node = nodes.value.find(n => n.id === nodeId);
  if (node) {
    Object.assign(node.data, updates);
    if (updates.label !== undefined) {
      node.label = updates.label;
    }
  }
};

// -------------------------------------------------- Theme ----------------------------------------------------------
const toggleTheme = () => {
  currentTheme.value = currentTheme.value === 'light' ? 'dark' : 'light';
  document.documentElement.setAttribute('data-theme', currentTheme.value);
  setToStore(defaultEditorThemeKey, currentTheme.value)
};

// --------------------------------------------------- Save ----------------------------------------------------------
const saveScript = () => {
  // Save current task first
  if (currentTask.value) {
    currentTask.value.nodes = [...nodes.value];
    currentTask.value.edges = [...edges.value];
  }
  
  const scriptData = {
    id: scriptId.value,
    name: scriptName.value,
    tasks: taskList.value
  };
  
  console.log('Saving script:', scriptData);
  addLog('Script saved successfully!', 'success');
  // TODO: Call backend API to save
};

// --- Fit View ---
const fitView = () => {
  flowFitView({ padding: 0.2 });
};

// --- Add Node (Click or Drop) ---
const addNodeToCanvas = (type) => {
  // Calculate position: center of viewport, or below the last selected node
  let position = { x: 200, y: 200 };
  
  if (selectedNode.value) {
    // Add below selected node
    position = {
      x: selectedNode.value.position.x,
      y: selectedNode.value.position.y + 120
    };
  } else if (nodes.value.length > 0) {
    // Add below the last node
    const lastNode = nodes.value[nodes.value.length - 1];
    position = {
      x: lastNode.position.x,
      y: lastNode.position.y + 120
    };
  }
  
  const newNode = createNode(type, position);
  
  // Auto-connect to selected node if exists
  if (selectedNode.value) {
    const newEdge = {
      id: `e-${selectedNode.value.id}-${newNode.id}`,
      source: selectedNode.value.id,
      target: newNode.id
    };
    edges.value.push(newEdge);
    addLog(`Auto-connected: ${selectedNode.value.id} → ${newNode.id}`, 'info');
  }
  
  // Select the new node
  selectedNode.value = newNode;
};

const createNode = (type, position) => {
  const nodeId = `node-${Date.now()}`;
  const newNode = {
    id: nodeId,
    type: 'custom',
    label: '',
    position,
    data: { 
      type: type,
      // Default data based on type
      ...(type === 'wait' && { duration: 1000 }),
      ...(type === 'loop' && { count: 3 }),
      ...(type === 'fallback' && { 
        strategies: [
          { target: 'back_button', action: 'click' },
          { target: 'close_button', action: 'click' },
          { target: 'confirm_button', action: 'click' },
        ],
        maxRetries: 3
      }),
    },
  };

  nodes.value.push(newNode);
  addLog(`Added node: ${type}`, 'success');
  return newNode;
};

// 异步加载后赋值
getFromStore(defaultEditorThemeKey).then(val => {
  if (val !== 'light'){
    currentTheme.value = val;
    document.documentElement.setAttribute('data-theme', currentTheme.value);
  }
})
</script>

<style>
/* Vue Flow overrides */
.vue-flow__node {
    box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
    border-radius: 0.5rem;
    border-width: 2px;
    border-color: transparent;
    /*transition: all 50ms cubic-bezier(0.4, 0, 0.2, 1);*/
    background-color: white;
}

.vue-flow__node:hover {
    border-color: oklch(var(--p));
}

.vue-flow__node.selected {
    border-color: oklch(var(--p));
    box-shadow: 0 0 0 2px oklch(var(--p) / 0.2), 0 10px 15px -3px rgb(0 0 0 / 0.1);
}

[data-theme='dark'] .vue-flow__node {
    background-color: oklch(var(--b2));
    color: oklch(var(--bc));
}

/* Edge styling */
.vue-flow__edge-path {
    stroke: oklch(var(--p));
    stroke-width: 2;
}

.vue-flow__edge.selected .vue-flow__edge-path {
    stroke: oklch(var(--s));
    stroke-width: 3;
}

/* MiniMap styling */
.vue-flow__minimap {
    background-color: oklch(var(--b2));
    border-radius: 0.5rem;
    border: 1px solid oklch(var(--b3));
}
</style>
