<template>
  <div class="h-screen w-screen flex flex-col bg-base-100 overflow-hidden" :data-theme="currentTheme">
    <!-- 1. Header (Toolbar) -->
    <div class="h-14 border-b border-base-300 flex items-center px-4 justify-between bg-base-200 shadow-sm z-10">
      <div class="flex items-center gap-4">
        <h1 class="text-lg font-bold flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/></svg>
            Script Editor
        </h1>
        <div class="badge badge-primary badge-outline">{{ currentScript?.name || 'No Script Selected' }}</div>
      </div>
      
      <div class="flex items-center gap-2">
        <button class="btn btn-sm btn-ghost btn-circle" @click="toggleTheme" title="Switch Theme">
            <svg v-if="currentTheme === 'light'" xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path></svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="5"/><line x1="12" y1="1" x2="12" y2="3"/><line x1="12" y1="21" x2="12" y2="23"/><line x1="4.22" y1="4.22" x2="5.64" y2="5.64"/><line x1="18.36" y1="18.36" x2="19.78" y2="19.78"/><line x1="1" y1="12" x2="3" y2="12"/><line x1="21" y1="12" x2="23" y2="12"/><line x1="4.22" y1="19.78" x2="5.64" y2="18.36"/><line x1="18.36" y1="5.64" x2="19.78" y2="4.22"/></svg>
        </button>

        <select class="select select-sm select-bordered w-full max-w-xs">
          <option disabled selected>Select Device</option>
          <option>MuMu12 (127.0.0.1:7555)</option>
        </select>
        <button class="btn btn-sm btn-success gap-2 text-white">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
          Run
        </button>
        <button class="btn btn-sm btn-primary">Save</button>
      </div>
    </div>

    <!-- Main Content Grid -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 2. Left Panel (Sidebar) -->
      <div class="w-72 border-r border-base-300 flex flex-col bg-base-100 shadow-md z-1">
        <!-- Sidebar Tabs -->
        <div class="tabs tabs-boxed p-2 bg-base-100">
            <a class="tab flex-1" :class="{'tab-active': activeTab === 'toolbox'}" @click="activeTab = 'toolbox'">Toolbox</a>
            <a class="tab flex-1" :class="{'tab-active': activeTab === 'scripts'}" @click="activeTab = 'scripts'">Scripts</a>
        </div>

        <div class="flex-1 overflow-y-auto p-2 h-full">
            <!-- TAB: TOOLBOX -->
            <div v-show="activeTab === 'toolbox'" class="h-full">
                <Toolbox />
            </div>

            <!-- TAB: SCRIPTS (List) -->
            <div v-show="activeTab === 'scripts'">
                <div class="flex justify-between items-center mb-2 px-1">
                    <span class="text-xs font-bold opacity-50 uppercase">Local Scripts</span>
                    <button class="btn btn-xs btn-circle btn-primary" @click="createNewScript" title="New Script">
                         <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><line x1="12" y1="5" x2="12" y2="19"></line><line x1="5" y1="12" x2="19" y2="12"></line></svg>
                    </button>
                </div>

                <div class="form-control w-full mb-2">
                    <input type="text" placeholder="Search..." class="input input-bordered input-sm w-full" />
                </div>
                
                <div v-for="script in scriptList" :key="script.id" 
                     class="p-2 rounded hover:bg-base-200 cursor-pointer flex items-center justify-between group text-sm mb-1"
                     :class="{'bg-primary/10 text-primary font-bold': currentScript?.id === script.id}"
                     @click="selectScript(script)">
                    <div class="flex items-center gap-2 truncate">
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
                        {{ script.name }}
                    </div>
                    <button class="btn btn-xs btn-ghost btn-circle opacity-0 group-hover:opacity-100 text-error" 
                            @click.stop="deleteScript(script.id)" title="Delete">
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path></svg>
                    </button>
                </div>
            </div>
        </div>
      </div>

      <!-- 3. Center (Canvas - Vue Flow) -->
      <div class="flex-1 relative bg-base-100 flex flex-col h-full" 
           @dragover.prevent 
           @drop="onDrop">
        
        <VueFlow 
            v-model:nodes="nodes" 
            v-model:edges="edges" 
            :node-types="nodeTypes"
            :delete-key-code="null"
            :default-viewport="{ zoom: 1 }"
            :min-zoom="0.2"
            :max-zoom="4"
            fit-view-on-init
            class="flex-1 h-full"
        >
            <Background pattern-color="#aaa" :gap="16" />
            <Controls />
        </VueFlow>
        
        <!-- Bottom Panel (Console) -->
        <div class="h-48 border-t border-base-300 flex flex-col bg-neutral text-neutral-content z-10 shrink-0">
            <div class="p-1 px-4 text-xs font-bold bg-neutral-focus flex justify-between items-center h-8">
                <span>Console Output</span>
                <button class="btn btn-xs btn-ghost text-xs">Clear</button>
            </div>
            <div class="flex-1 p-2 font-mono text-xs overflow-y-auto">
                <div class="text-success">[10:00:01] Script started.</div>
            </div>
        </div>
      </div>

      <!-- 4. Right Panel (Properties) -->
      <PropertiesPanel 
          :selectedNode="selectedNode" 
          @delete-node="requestDeleteSelected"
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
  </div>
</template>

<script setup>
import { ref, onMounted, markRaw, onUnmounted } from 'vue';
import { VueFlow, useVueFlow } from '@vue-flow/core';
import { Background } from '@vue-flow/background';
import { Controls } from '@vue-flow/controls';
import '@vue-flow/core/dist/style.css';
import '@vue-flow/core/dist/theme-default.css';
import '@vue-flow/controls/dist/style.css';

// Components
import Toolbox from './script-editor/Toolbox.vue';
import PropertiesPanel from './script-editor/PropertiesPanel.vue';
import FlowNode from './script-editor/FlowNode.vue';

// --- State ---
const nodes = ref([]);
const edges = ref([]);

// Register Custom Node Type
const nodeTypes = {
  custom: markRaw(FlowNode),
};

const {project, onNodeClick, onConnect, addEdges, removeNodes, getSelectedNodes} = useVueFlow();
const activeTab = ref('scripts'); // Start on scripts tab
const currentTheme = ref('light');

// --- Scripts Management ---
const scriptList = ref([
    { id: 1, name: 'Daily Login', nodes: [
        { id: '1', type: 'custom', label: 'Start', position: { x: 100, y: 5 }, data: { type: 'input' } }
    ], edges: [] },
    { id: 2, name: 'Claim Rewards', nodes: [], edges: [] },
]);

const currentScript = ref(null);

// Select initial script
onMounted(() => {
    initTheme();
    if(scriptList.value.length > 0) {
        selectScript(scriptList.value[0]);
    }
    window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown);
});


const handleKeyDown = (event) => {
    if (event.key === 'Delete' || event.key === 'Backspace') {
        const activeElement = document.activeElement;
        // Avoid deleting if user is typing in an input
        if (activeElement.tagName === 'INPUT' || activeElement.tagName === 'TEXTAREA') {
            return;
        }
        
        event.preventDefault(); // Prevent default immediate deletion
        requestDeleteSelected();
    }
};

// -- Selection Logic --
const selectedNode = ref(null);
onNodeClick((event) => {
    selectedNode.value = event.node;
});

// -- Connection Logic --
onConnect((params) => {
    addEdges(params);
});

// --- Delete Confirmation Logic ---
const showDeleteConfirm = ref(false);
const nodesToDelete = ref([]);

const requestDeleteSelected = () => {
    const selected = getSelectedNodes.value;
    if (selected.length > 0) {
        nodesToDelete.value = selected;
        showDeleteConfirm.value = true;
    }
};

const confirmDelete = () => {
    removeNodes(nodesToDelete.value);
    selectedNode.value = null; // Clear selection
    showDeleteConfirm.value = false;
    nodesToDelete.value = [];
};

const cancelDelete = () => {
    showDeleteConfirm.value = false;
    nodesToDelete.value = [];
};

// --- Actions ---

const selectScript = (script) => {
    // Save current (mock save)
    if(currentScript.value) {
        currentScript.value.nodes = [...nodes.value];
        currentScript.value.edges = [...edges.value];
    }
    
    currentScript.value = script;
    // Deep copy to prevent reference issues, or just load
    // Ensure nodes use 'custom' type if not set
    nodes.value = script.nodes.map(n => ({ ...n, type: 'custom' }));
    edges.value = [...script.edges];
    selectedNode.value = null; // deselect
};

const createNewScript = () => {
    const newId = scriptList.value.length + 1;
    const newScript = {
        id: newId,
        name: `New Script ${newId}`,
        nodes: [{ id: '1', type: 'custom', label: 'Start', position: { x: 100, y: 50 }, data: { type: 'input' } }],
        edges: []
    };
    scriptList.value.push(newScript);
    selectScript(newScript);
};

const deleteScript = (id) => {
    const idx = scriptList.value.findIndex(s => s.id === id);
    if(idx !== -1) {
        scriptList.value.splice(idx, 1);
        if(currentScript.value?.id === id) {
            // Select another if deleted current
            if(scriptList.value.length > 0) selectScript(scriptList.value[0]);
            else {
                currentScript.value = null;
                nodes.value = [];
            }
        }
    }
};

const toggleTheme = () => {
    currentTheme.value = currentTheme.value === 'light' ? 'dark' : 'light';
    localStorage.setItem('theme', currentTheme.value);
    document.documentElement.setAttribute('data-theme', currentTheme.value);
};

const initTheme = () => {
    const saved = localStorage.getItem('theme') || 'light';
    currentTheme.value = saved;
    document.documentElement.setAttribute('data-theme', saved);
};

// --- Drag & Drop ---
// Now handled mostly by Toolbox, but we need the Drop handler here

const onDrop = (event) => {
    const type = event.dataTransfer?.getData('application/vueflow');
    if (type) {
        const { left, top } = document.querySelector('.vue-flow__renderer').getBoundingClientRect();
        const position = project({ 
            x: event.clientX - left, 
            y: event.clientY - top 
        });

        const newNode = {
            id: `node-${nodes.value.length + 1}`,
            type: 'custom', // Use our custom node component
            label: '', // User will enter remark
            position,
            data: { 
                type: type, // The internal type (click, wait, etc)
                remark: '' 
            },
        };

        nodes.value.push(newNode);
    }
};
</script>

<style>
/* Vue Flow overrides */
.vue-flow__node {
    box-shadow: 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1);
    border-radius: 0.375rem;
    border-width: 2px;
    border-color: transparent;
    transition: all 150ms cubic-bezier(0.4, 0, 0.2, 1);
    background-color: white; /* Default light theme bg */
}

/* DaisyUI variable usage for consistency */
.vue-flow__node:hover {
    border-color: oklch(var(--p) / 0.5);
}

.vue-flow__node.selected {
    border-color: oklch(var(--p));
    box-shadow: 0 0 0 2px oklch(var(--p) / 0.2), 0 10px 15px -3px rgb(0 0 0 / 0.1);
}

[data-theme='dark'] .vue-flow__node {
    background-color: oklch(var(--b2));
    color: oklch(var(--bc));
}
</style>
