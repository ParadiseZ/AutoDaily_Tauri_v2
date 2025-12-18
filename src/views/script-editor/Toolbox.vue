<template>
  <div class="flex-1 overflow-y-auto p-2 space-y-2">
    <!-- Basic Nodes -->
    <div class="text-xs font-bold opacity-50 mb-2 uppercase tracking-wide">Basic</div>
    
    <ToolboxItem 
      type="click" 
      label="Click" 
      color="bg-blue-500" 
      description="Click on a target"
      @add-node="emitAddNode"
    />
    
    <ToolboxItem 
      type="wait" 
      label="Wait" 
      color="bg-gray-500" 
      description="Wait for duration"
      @add-node="emitAddNode"
    />
    
    <ToolboxItem 
      type="swipe" 
      label="Swipe" 
      color="bg-cyan-500" 
      description="Swipe gesture"
      @add-node="emitAddNode"
    />

    <!-- Condition Nodes -->
    <div class="text-xs font-bold opacity-50 mb-2 mt-4 uppercase tracking-wide">Conditions</div>
    
    <ToolboxItem 
      type="if_found" 
      label="IF Found" 
      color="bg-yellow-500" 
      description="If image/text found, then..."
      @add-node="emitAddNode"
    />
    
    <ToolboxItem 
      type="if_not_found" 
      label="IF Not Found" 
      color="bg-orange-500" 
      description="If image/text NOT found, then..."
      @add-node="emitAddNode"
    />

    <!-- Vision Nodes -->
    <div class="text-xs font-bold opacity-50 mb-2 mt-4 uppercase tracking-wide">Vision</div>
    
    <ToolboxItem 
      type="find_image" 
      label="Find Image" 
      color="bg-purple-500" 
      description="Locate image on screen"
      @add-node="emitAddNode"
    />
    
    <ToolboxItem 
      type="ocr" 
      label="OCR Text" 
      color="bg-violet-500" 
      description="Recognize text"
      @add-node="emitAddNode"
    />

    <!-- Control Flow -->
    <div class="text-xs font-bold opacity-50 mb-2 mt-4 uppercase tracking-wide">Control Flow</div>
    
    <ToolboxItem 
      type="loop" 
      label="Loop" 
      color="bg-green-500" 
      description="Repeat N times"
      @add-node="emitAddNode"
    />
    
    <ToolboxItem 
      type="fallback" 
      label="Fallback" 
      color="bg-red-500" 
      description="Retry actions when all conditions fail"
      @add-node="emitAddNode"
    />
    
    <ToolboxItem 
      type="subflow" 
      label="Sub-Flow" 
      color="bg-pink-500" 
      description="Call another task's flow"
      @add-node="emitAddNode"
    />
  </div>
</template>

<script setup>
import ToolboxItem from './ToolboxItem.vue';

const emit = defineEmits(['add-node']);

const emitAddNode = (type) => {
  emit('add-node', type);
};
</script>
