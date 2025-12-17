<template>
  <div class="h-screen w-screen flex flex-col bg-base-100 overflow-hidden" :data-theme="currentTheme">
    <!-- 1. Header (Toolbar) -->
    <div class="h-14 border-b border-base-300 flex items-center px-4 justify-between bg-base-200 shadow-sm z-10">
      <div class="flex items-center gap-4">
        <h1 class="text-lg font-bold flex items-center gap-2">
            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14.5 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7.5L14.5 2z"/><polyline points="14 2 14 8 20 8"/></svg>
            Script Editor
        </h1>
        <div class="badge badge-primary badge-outline">{{ currentScript }}</div>
      </div>
      
      <div class="flex items-center gap-2">
        <!-- Theme Toggle -->
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

        <div class="flex-1 overflow-y-auto p-2 space-y-2">
            <!-- TAB: TOOLBOX -->
            <div v-show="activeTab === 'toolbox'">
                <div class="text-xs font-bold opacity-50 mb-2 uppercase tracking-wide">Basic</div>
                <div class="card bg-base-200 shadow-sm p-3 cursor-move hover:bg-base-300 border border-transparent hover:border-primary transition-all mb-2" 
                     draggable="true" @dragstart="onDragStart($event, 'Click')">
                    <div class="font-medium flex items-center gap-2">
                        <span class="w-2 h-2 rounded-full bg-blue-500"></span> Click
                    </div>
                </div>
                <div class="card bg-base-200 shadow-sm p-3 cursor-move hover:bg-base-300 border border-transparent hover:border-primary transition-all mb-2"
                     draggable="true" @dragstart="onDragStart($event, 'WaitMs')">
                    <div class="font-medium flex items-center gap-2">
                        <span class="w-2 h-2 rounded-full bg-gray-500"></span> Wait
                    </div>
                </div>
                <div class="card bg-base-200 shadow-sm p-3 cursor-move hover:bg-base-300 border border-transparent hover:border-primary transition-all mb-2"
                     draggable="true" @dragstart="onDragStart($event, 'If')">
                    <div class="font-medium flex items-center gap-2">
                        <span class="w-2 h-2 rounded-full bg-yellow-500"></span> IF Condition
                    </div>
                </div>

                <div class="text-xs font-bold opacity-50 mb-2 mt-4 uppercase tracking-wide">Vision</div>
                 <div class="card bg-base-200 shadow-sm p-3 cursor-move hover:bg-base-300 border border-transparent hover:border-primary transition-all mb-2"
                      draggable="true" @dragstart="onDragStart($event, 'Ocr')">
                    <div class="font-medium flex items-center gap-2">
                        <span class="w-2 h-2 rounded-full bg-purple-500"></span> OCR
                    </div>
                </div>
                
                 <div class="text-xs font-bold opacity-50 mb-2 mt-4 uppercase tracking-wide">Advanced</div>
                 <div class="card bg-base-200 shadow-sm p-3 cursor-move hover:bg-base-300 border border-transparent hover:border-primary transition-all mb-2"
                      draggable="true" @dragstart="onDragStart($event, 'CallScript')">
                    <div class="font-medium flex items-center gap-2">
                        <span class="w-2 h-2 rounded-full bg-red-500"></span> Sub-Flow
                    </div>
                </div>
            </div>

            <!-- TAB: SCRIPTS (Task List) -->
            <div v-show="activeTab === 'scripts'">
                <div class="form-control w-full mb-2">
                    <input type="text" placeholder="Search scripts..." class="input input-bordered input-sm w-full" />
                </div>
                <div v-for="script in scriptList" :key="script.id" 
                     class="p-2 rounded hover:bg-base-200 cursor-pointer flex items-center justify-between group"
                     :class="{'bg-primary/10 text-primary font-bold': currentScript === script.name}"
                     @click="currentScript = script.name">
                    <div class="flex items-center gap-2">
                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
                        {{ script.name }}
                    </div>
                </div>
            </div>
        </div>
      </div>

      <!-- 3. Center (Canvas) -->
      <div class="flex-1 relative bg-base-100 flex flex-col" 
           @dragover.prevent 
           @drop="onDrop">
        
        <!-- Canvas Container -->
        <div id="container" class="flex-1 bg-base-100 relative"></div>
        
        <!-- 5. Bottom Panel (Console) - collapsible? -->
        <div class="h-48 border-t border-base-300 flex flex-col bg-neutral text-neutral-content z-10">
            <div class="p-1 px-4 text-xs font-bold bg-neutral-focus flex justify-between items-center h-8">
                <span>Console Output</span>
                <button class="btn btn-xs btn-ghost text-xs">Clear</button>
            </div>
            <div class="flex-1 p-2 font-mono text-xs overflow-y-auto">
                <div class="text-success">[10:00:01] Script started.</div>
                <div>[10:00:02] Value 'img_1' updated.</div>
                <div class="text-warning">[10:00:03] Waiting for match... (Attempt 1/3)</div>
            </div>
        </div>
      </div>

      <!-- 4. Right Panel (Properties) -->
      <div class="w-80 border-l border-base-300 flex flex-col bg-base-100 shadow-md z-1">
        <div class="p-3 font-bold text-sm bg-base-200 flex justify-between items-center">
            PROPERTIES
            <div class="badge badge-sm">Node: Click</div>
        </div>
        <div class="flex-1 p-4 overflow-y-auto">
            <!-- Dynamic Form based on Selection -->
            <div class="form-control w-full">
                <label class="label"><span class="label-text font-bold">Step Name</span></label>
                <input type="text" value="Click Login" class="input input-bordered w-full input-sm" />
            </div>

            <div class="divider text-xs opacity-50">Configuration</div>

            <div class="form-control w-full">
                 <label class="label"><span class="label-text">Target (Coords or Var)</span></label>
                 <div class="join">
                    <input type="text" placeholder="x,y" class="input input-bordered input-sm join-item w-full" />
                    <button class="btn btn-sm btn-square join-item">
                        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="22" y1="12" x2="18" y2="12"/><line x1="6" y1="12" x2="2" y2="12"/><line x1="12" y1="6" x2="12" y2="2"/><line x1="12" y1="22" x2="12" y2="18"/></svg>
                    </button>
                 </div>
            </div>

             <div class="form-control w-full mt-4">
                <label class="label"><span class="label-text">Verify Condition</span></label>
                <input type="text" placeholder="rhai expression" class="input input-bordered w-full input-sm font-mono text-xs" />
                <label class="label"><span class="label-text-alt opacity-50">Example: text.contains("Welcome")</span></label>
            </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { onMounted, ref, reactive } from 'vue';
import { Graph } from '@antv/x6';

const graph = ref(null);
const activeTab = ref('toolbox');
const currentScript = ref('Daily Login');
const currentTheme = ref('light');

const scriptList = ref([
    { id: 1, name: 'Daily Login' },
    { id: 2, name: 'Claim Rewards' },
    { id: 3, name: 'Dungeon Farm' },
    { id: 4, name: 'Event: Summer' },
]);

const toggleTheme = () => {
    currentTheme.value = currentTheme.value === 'light' ? 'dark' : 'light';
    // Optionally save to localStorage
    localStorage.setItem('theme', currentTheme.value);
    document.documentElement.setAttribute('data-theme', currentTheme.value);
};

// Initialize theme from localStorage
const initTheme = () => {
    const saved = localStorage.getItem('theme') || 'light';
    currentTheme.value = saved;
    document.documentElement.setAttribute('data-theme', saved);
};

onMounted(() => {
    initTheme();

    // Initialize X6 Graph
    graph.value = new Graph({
        container: document.getElementById('container'),
        autoResize: true,
        background: {
             color: 'transparent', // Let CSS bg handle it
        },
        grid: {
            size: 10,
            visible: true,
            type: 'doubleMesh',
            args: [
                { color: '#eee', thickness: 1 },
                { color: '#ddd', thickness: 1, factor: 4 },
            ],
        },
        panning: true,
        mousewheel: true,
        connecting: {
            router: 'manhattan',
            connector: {
                name: 'rounded',
                args: {
                    radius: 8,
                },
            },
            anchor: 'center',
            connectionPoint: 'anchor',
            allowBlank: false,
            snap: {
                radius: 20,
            },
            createEdge() {
                return new Shape.Edge({
                    attrs: {
                        line: {
                            stroke: '#A2B1C3',
                            strokeWidth: 2,
                            targetMarker: {
                                name: 'block',
                                width: 12,
                                height: 8,
                            },
                        },
                    },
                    zIndex: 0,
                })
            },
            validateConnection({ targetMagnet }) {
                return !!targetMagnet
            },
        },
    });
    
    // Initial Node
    addNodeAt('Start', 100, 100);
});

const addNodeAt = (type, x, y) => {
    const node = graph.value.addNode({
        x,
        y,
        width: 140,
        height: 40,
        label: type,
        ports: {
            groups: {
                in: {
                    position: 'top',
                    attrs: { circle: { r: 4, magnet: true, stroke: '#31d0c6', strokeWidth: 2, fill: '#fff' } },
                },
                out: {
                    position: 'bottom',
                    attrs: { circle: { r: 4, magnet: true, stroke: '#31d0c6', strokeWidth: 2, fill: '#fff' } },
                },
            },
            items: [
                { id: 'in', group: 'in' },
                { id: 'out', group: 'out' },
            ],
        },
        attrs: {
            body: {
                stroke: '#5F95FF',
                fill: 'var(--b2, #EFF4FF)',
                rx: 6,
                ry: 6,
                strokeWidth: 1,
            },
            label: {
                fill: 'currentColor', // Adapt to theme
                fontSize: 12,
                fontWeight: 'bold',
            }
        }
    });
}

// Drag & Drop Handlers
const onDragStart = (e, type) => {
    // Required to allow drag
    e.dataTransfer.effectAllowed = 'copy'; 
    e.dataTransfer.setData('node-type', type);
};

const onDrop = (e) => {
    // Handled by Vue @drop="onDrop" on the wrapper
    const type = e.dataTransfer.getData('node-type');
    if (type && graph.value) {
        // Calculate coordinates relative to the graph container
        // Note: clientToLocal converts from screen coordinates to graph coordinates
        // We need to account for valid drop target
        const { clientX, clientY } = e;
        const p = graph.value.clientToLocal(clientX, clientY);
        addNodeAt(type, p.x, p.y);
    }
};
</script>

<style scoped>
/* Ensure X6 container takes full height */
#container {
    width: 100%;
    height: 100%;
}
</style>

<style scoped>
/* Ensure X6 container takes full height */
#container {
    width: 100%;
    height: 100%;
}
</style>
