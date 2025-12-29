<template>
  <div 
    class="shadow-md rounded-lg bg-base-100 border-2 border-transparent transition-all min-w-[120px] overflow-hidden"
    :class="{ 
      'border-primary! shadow-sm scale-105': selected,
      'border-base-300': !selected
    }"
  >
    <!-- Input Handle (Top) -->
    <Handle 
      v-if="!isStartNode" 
      type="target"
      :position="Position.Top" 
      id="input"
      class="w-3! h-3! bg-primary! ring-2 ring-white"
    />
    
    <!-- Header: Type Indicator -->
    <div 
      class="px-2 py-1 text-xs font-bold text-white flex items-center gap-1"
      :class="headerColorClass"
    >
      <!-- Icon based on type -->
      <IconRenderer :icon="currentIcon" class="w-3 h-3" />
      {{ displayType }}
    </div>

    <!-- Body: Label/Remark -->
    <div class="p-3 text-sm bg-base-100 text-base-content">
      <div class="font-medium text-center" v-if="label">{{ label }}</div>
      <div class="text-xs opacity-50 text-center" v-else>{{ placeholderText }}</div>
      
      <!-- Special content for certain node types -->
      <div v-if="data.type === 'fallback'" class="mt-2 text-xs opacity-60">
        <div class="flex items-center gap-1" v-for="(strategy, idx) in (data.strategies || defaultStrategies)" :key="idx">
          <span class="badge badge-xs badge-ghost">{{ idx + 1 }}</span>
          {{ strategy.label || strategy.target }}
        </div>
      </div>
      
      <div v-if="data.type === 'loop'" class="text-center mt-1">
        <span class="badge badge-sm badge-primary">{{ data.count || 1 }}x</span>
      </div>
    </div>

    <!-- Output Handle (Bottom) -->
    <Handle
      v-if="!isConditionNode && !isEndNode"
      type="source"
      :position="Position.Bottom" 
      id="output"
      class="w-3! h-3! bg-primary! ring-2 ring-white"
    />

    <Handle
        v-if="isConditionNode"
        type="source"
        :position="Position.Bottom"
        id="ifFalse"
        class="w-3! h-3! bg-error! ring-2 ring-white"
    />
    
    <!-- Conditional Output Handle (Right) for condition nodes -->
    <Handle 
      v-if="isConditionNode"
      type="source"
      :position="Position.Right"
      id="ifTrue"
      class="w-3! h-3! bg-success! ring-2 ring-white" 
    />

    <Handle
        v-if="isLoopNode"
        type="source"
        :position="Position.Right"
        id="loopStart"
        class="w-3! h-3! bg-success! ring-2 ring-white"
        style="top: 34%;"
    />

    <Handle
        v-if="isLoopNode"
        type="target"
        :position="Position.Right"
        id="loopEnd"
        class="w-3! h-3! bg-error! ring-2 ring-white"
        style="top: 68%;"
    />
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { Handle, Position } from '@vue-flow/core';
import {
  DEFAULT_FALLBACK_STRATEGIES,
  getNodeColor,
  getNodeDisplay,
  getNodeIcon,
  getNodePlaceholder,
  isStartNode as checkIsStartNode,
  isConditionNode as checkIsConditionNode,
  isLoopNode as checkIsLoopNode,
  isEndNode as checkIsEndNode,
} from './config.js';

// Components
import IconRenderer from './IconRenderer.vue';

// ---------------------------
const props = defineProps({
  id: String,
  label: String,
  data: {
    type: Object,
    default: () => ({})
  },
  selected: Boolean,
});

// 使用统一配置
const defaultStrategies = DEFAULT_FALLBACK_STRATEGIES;

// 计算属性使用配置辅助函数
const isStartNode = computed(() => checkIsStartNode(props.data?.type));
const isConditionNode = computed(() => checkIsConditionNode(props.data?.type));
const isLoopNode = computed(() => checkIsLoopNode(props.data?.type));
const isEndNode = computed(() => checkIsEndNode(props.data?.type));

const headerColorClass = computed(() => getNodeColor(props.data?.type));
const displayType = computed(() => getNodeDisplay(props.data?.type));
const placeholderText = computed(() => getNodePlaceholder(props.data?.type));
const currentIcon = computed(() => getNodeIcon(props.data?.type));
</script>

<style scoped>
/* Ensure handles are visible and properly styled */
.vue-flow__handle {
  border: 1px solid white;
}
</style>
