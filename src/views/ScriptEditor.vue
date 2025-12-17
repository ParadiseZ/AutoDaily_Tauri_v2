<template>
  <div class="h-screen w-screen flex flex-col bg-base-100 overflow-hidden">
    <!-- 1. Header (Toolbar) -->
    <div class="h-14 border-b border-base-300 flex items-center px-4 justify-between bg-base-200">
      <div class="flex items-center gap-4">
        <h1 class="text-lg font-bold">Script Editor</h1>
        <div class="badge badge-primary badge-outline">Script: Daily Login</div>
      </div>
      
      <div class="flex items-center gap-2">
        <select class="select select-sm select-bordered w-full max-w-xs">
          <option disabled selected>Select Device</option>
          <option>MuMu12 (127.0.0.1:7555)</option>
        </select>
        <button class="btn btn-sm btn-success gap-2">
          <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="5 3 19 12 5 21 5 3"></polygon></svg>
          Run
        </button>
        <button class="btn btn-sm btn-primary">Save</button>
      </div>
    </div>

    <!-- Main Content Grid -->
    <div class="flex-1 flex overflow-hidden">
      <!-- 2. Left Panel (Toolbox) -->
      <div class="w-64 border-r border-base-300 flex flex-col bg-base-100">
        <div class="p-2 font-bold text-sm bg-base-200">Toolbox</div>
        <div class="flex-1 p-2 overflow-y-auto space-y-2">
            <!-- Draggable Items Shell -->
            <div class="card bg-base-200 shadow-sm p-2 cursor-move hover:bg-base-300">
                <div class="font-medium">Click</div>
            </div>
            <div class="card bg-base-200 shadow-sm p-2 cursor-move hover:bg-base-300">
                <div class="font-medium">Wait</div>
            </div>
             <div class="card bg-base-200 shadow-sm p-2 cursor-move hover:bg-base-300">
                <div class="font-medium">IF Condition</div>
            </div>
             <div class="card bg-base-200 shadow-sm p-2 cursor-move hover:bg-base-300">
                <div class="font-medium">OCR</div>
            </div>
        </div>
      </div>

      <!-- 3. Center (Canvas) -->
      <div class="flex-1 relative bg-base-100" id="container">
        <!-- AntV X6 Container -->
        <div class="absolute inset-0 flex items-center justify-center text-base-content/30 pointer-events-none">
            Canvas Area
        </div>
      </div>

      <!-- 4. Right Panel (Properties) -->
      <div class="w-72 border-l border-base-300 flex flex-col bg-base-100">
        <div class="p-2 font-bold text-sm bg-base-200">Properties</div>
        <div class="flex-1 p-4">
            <div class="form-control w-full">
                <label class="label"><span class="label-text">Node Name</span></label>
                <input type="text" placeholder="Type here" class="input input-bordered w-full input-sm" />
            </div>
             <div class="form-control w-full mt-4">
                <label class="label"><span class="label-text">Variables</span></label>
                <textarea class="textarea textarea-bordered h-24" placeholder="JSON Config"></textarea>
            </div>
        </div>
      </div>
    </div>

    <!-- 5. Bottom Panel (Console) -->
    <div class="h-48 border-t border-base-300 flex flex-col bg-neutral text-neutral-content">
      <div class="p-1 px-4 text-xs font-bold bg-neutral-focus flex justify-between">
          <span>Console Output</span>
          <button class="btn btn-xs btn-ghost">Clear</button>
      </div>
      <div class="flex-1 p-2 font-mono text-sm overflow-y-auto">
        <div class="text-success">[10:00:01] Script started.</div>
        <div>[10:00:02] Taking screenshot... (var: img_1)</div>
        <div>[10:00:03] OCR Processing... Found "Login"</div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { onMounted, ref } from 'vue';
import { Graph } from '@antv/x6';

const graph = ref(null);

onMounted(() => {
    // Initialize X6 Graph
    graph.value = new Graph({
        container: document.getElementById('container'),
        autoResize: true,
        background: {
            color: 'var(--b1)', // DaisyUI base-100
        },
        grid: {
            size: 10,
            visible: true,
        },
        panning: true,
        mousewheel: true,
    });
    
    // Add demo node
    graph.value.addNode({
        x: 100,
        y: 100,
        width: 100,
        height: 40,
        label: 'Start',
        attrs: {
            body: {
                fill: '#2da44e',
                stroke: '#2da44e',
                rx: 4,
                ry: 4,
            },
            label: {
                fill: '#fff',
            }
        }
    })
});
</script>

<style scoped>
/* Ensure X6 container takes full height */
#container {
    width: 100%;
    height: 100%;
}
</style>
