<template>
  <div class="flex-1 overflow-y-auto p-2 space-y-2">
    <div class="text-xs font-bold opacity-50 mb-2 uppercase tracking-wide">Basic</div>
    
    <div 
      class="card bg-base-200 shadow-sm p-3 cursor-move hover:bg-base-300 border border-transparent hover:border-primary transition-all mb-2" 
      draggable="true" 
      @dragstart="onDragStart($event, 'click')"
    >
      <div class="font-medium flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-blue-500"></span> Click
      </div>
    </div>

    <div 
      class="card bg-base-200 shadow-sm p-3 cursor-move hover:bg-base-300 border border-transparent hover:border-primary transition-all mb-2"
      draggable="true" 
      @dragstart="onDragStart($event, 'wait')"
    >
      <div class="font-medium flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-gray-500"></span> Wait
      </div>
    </div>

    <div 
      class="card bg-base-200 shadow-sm p-3 cursor-move hover:bg-base-300 border border-transparent hover:border-primary transition-all mb-2"
      draggable="true" 
      @dragstart="onWaitDragStart($event, 'if')"
    >
      <div class="font-medium flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-yellow-500"></span> IF Condition
      </div>
    </div>

    <div class="text-xs font-bold opacity-50 mb-2 mt-4 uppercase tracking-wide">Vision</div>
    <div 
      class="card bg-base-200 shadow-sm p-3 cursor-move hover:bg-base-300 border border-transparent hover:border-primary transition-all mb-2"
      draggable="true" 
      @dragstart="onDragStart($event, 'ocr')"
    >
      <div class="font-medium flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-purple-500"></span> OCR
      </div>
    </div>
    
    <div class="text-xs font-bold opacity-50 mb-2 mt-4 uppercase tracking-wide">Advanced</div>
    <div 
      class="card bg-base-200 shadow-sm p-3 cursor-move hover:bg-base-300 border border-transparent hover:border-primary transition-all mb-2"
      draggable="true" 
      @dragstart="onDragStart($event, 'subflow')"
    >
      <div class="font-medium flex items-center gap-2">
        <span class="w-2 h-2 rounded-full bg-red-500"></span> Sub-Flow
      </div>
    </div>
  </div>
</template>

<script setup>
const onDragStart = (event, type) => {
  if (event.dataTransfer) {
    event.dataTransfer.setData('application/vueflow', type);
    event.dataTransfer.effectAllowed = 'copy';
  }
};

// Fixed the typo in the IF condition div if present, but I'll mapped it to onDragStart as well
const onWaitDragStart = (event, type) => {
    onDragStart(event, type);
}
</script>
