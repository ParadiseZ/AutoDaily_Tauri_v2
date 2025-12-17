<template>
  <div 
    class="shadow-md rounded-md bg-base-100 border-2 border-transparent transition-all min-w-[150px] overflow-hidden"
    :class="{ 
      'border-primary! shadow-lg scale-105': selected,
      'border-base-300': !selected
    }"
  >
    <!-- Handle for connecting nodes (Inputs) -->
    <Handle type="target" position="top" class="w-3! h-3! bg-primary!" />
    
    <!-- Header: Type Indicator -->
    <div 
        class="px-2 py-1 text-xs font-bold uppercase text-white flex items-center gap-2"
        :class="headerColorClass"
    >
         <!-- Icon based on type could go here -->
         {{ data.type }}
    </div>

    <!-- Body: Label/Remark -->
    <div class="p-2 text-sm text-center font-medium bg-base-100 text-base-content">
        {{ label || 'No Remark' }}
    </div>

    <!-- Handle for connecting nodes (Outputs) -->
    <Handle type="source" position="bottom" class="w-3! h-3! bg-primary!" />
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { Handle } from '@vue-flow/core';

const props = defineProps({
  id: String,
  label: String,
  data: Object,
  selected: Boolean,
});

const headerColorClass = computed(() => {
    switch (props.data?.type) {
        case 'click': return 'bg-blue-500';
        case 'wait': return 'bg-gray-500';
        case 'if': return 'bg-yellow-500';
        case 'ocr': return 'bg-purple-500';
        case 'subflow': return 'bg-red-500';
        default: return 'bg-neutral';
    }
});
</script>

<style scoped>
/* Ensure handles are visible */
.vue-flow__handle {
    border: 2px solid white;
}
</style>
