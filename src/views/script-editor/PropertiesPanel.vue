<template>
  <div class="w-80 border-l border-base-300 flex flex-col bg-base-100 shadow-md z-1 h-full">
    <div class="p-3 font-bold text-sm bg-base-200 flex justify-between items-center">
      PROPERTIES
      <div class="badge badge-sm" v-if="selectedNode">Node: {{ selectedNode.data?.type || 'Unknown' }}</div>
      <div class="badge badge-sm badge-ghost" v-else>No Selection</div>
    </div>
    
    <div class="flex-1 p-4 overflow-y-auto" v-if="selectedNode">
      <!-- Common Attributes -->
      <div class="form-control w-full">
        <label class="label"><span class="label-text font-bold">Remark (Label)</span></label>
        <input 
          type="text" 
          v-model="selectedNode.label" 
          class="input input-bordered w-full input-sm" 
          placeholder="Enter a description..."
        />
        <label class="label"><span class="label-text-alt opacity-60">Displayed on the node</span></label>
      </div>

      <div class="divider text-xs opacity-50">Configuration</div>
      
      <!-- Type Specific: Click -->
      <div class="form-control w-full" v-if="selectedNode.data?.type === 'click'">
         <label class="label"><span class="label-text">Target Coordinates (x,y)</span></label>
         <div class="join">
            <input type="text" placeholder="100,200" class="input input-bordered input-sm join-item w-full" />
         </div>
      </div>
      
      <!-- Type Specific: Wait -->
       <div class="form-control w-full" v-if="selectedNode.data?.type === 'wait'">
         <label class="label"><span class="label-text">Duration (ms)</span></label>
         <input type="number" value="1000" class="input input-bordered input-sm w-full" />
      </div>

       <div class="form-control w-full mt-4">
        <label class="label"><span class="label-text">Verify Condition</span></label>
        <input type="text" placeholder="rhai expression" class="input input-bordered w-full input-sm font-mono text-xs" />
      </div>

      <div class="mt-8">
        <button class="btn btn-error btn-sm w-full btn-outline" @click="$emit('delete-node')">
            Delete Node
        </button>
      </div>
    </div>
    
    <div class="flex-1 p-10 flex flex-col items-center justify-center text-base-content/30" v-else>
      <svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><line x1="9" y1="9" x2="15" y2="15"/><line x1="15" y1="9" x2="9" y2="15"/></svg>
      <span class="mt-2 text-sm">Select a node to edit</span>
    </div>
  </div>
</template>

<script setup>
//import { defineProps, defineEmits } from 'vue';

const props = defineProps({
  selectedNode: {
    type: Object,
    default: null
  }
});

const emit = defineEmits(['delete-node']);
</script>
