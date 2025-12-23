<template>
  <div class="flex-1 overflow-y-auto p-2 space-y-2">
    <!-- Basic Nodes -->
    <div class="text-xs font-bold opacity-50 mb-2 uppercase tracking-wide">基本</div>
    
    <ToolboxItem 
      type="click" 
      label="点击|Click"
      color="bg-blue-500" 
      description="Click on a target"
      @add-node="emitAddNode"
    />
    
    <ToolboxItem 
      type="wait" 
      label="等待|Wait"
      color="bg-gray-500" 
      description="Wait for duration"
      @add-node="emitAddNode"
    />
    
    <ToolboxItem 
      type="swipe" 
      label="滑动|Swipe"
      color="bg-cyan-500" 
      description="Swipe gesture"
      @add-node="emitAddNode"
    />

    <!-- Condition Nodes -->
    <div class="text-xs font-bold opacity-50 mb-2 mt-4 uppercase tracking-wide">条件逻辑</div>
    
    <ToolboxItem 
      type="if"
      label="如果|If"
      color="bg-yellow-500" 
      description="If image/text found, then..."
      @add-node="emitAddNode"
    />

    <!-- Vision Nodes -->
    <div class="text-xs font-bold opacity-50 mb-2 mt-4 uppercase tracking-wide">视觉</div>
    
    <ToolboxItem 
      type="detect"
      label="目标检测"
      color="bg-purple-500" 
      description="Locate image on screen"
      @add-node="emitAddNode"
    />
    
    <ToolboxItem 
      type="ocr" 
      label="文字识别"
      color="bg-violet-500" 
      description="Recognize text"
      @add-node="emitAddNode"
    />

    <!-- Control Flow -->
    <div class="text-xs font-bold opacity-50 mb-2 mt-4 uppercase tracking-wide">控制流</div>
    
    <ToolboxItem 
      type="loop" 
      label="循环|Loop"
      color="bg-green-500" 
      description="Repeat N times"
      @add-node="emitAddNode"
    />
    
    <ToolboxItem 
      type="fallback" 
      label="回调"
      color="bg-red-500" 
      description="Retry actions when all conditions fail"
      @add-node="emitAddNode"
    />
    
    <ToolboxItem 
      type="subflow" 
      label="子流程"
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
