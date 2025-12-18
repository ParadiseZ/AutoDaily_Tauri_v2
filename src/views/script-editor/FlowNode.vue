<template>
  <div 
    class="shadow-md rounded-lg bg-base-100 border-2 border-transparent transition-all min-w-[160px] overflow-hidden"
    :class="{ 
      'border-primary! shadow-xl scale-105': selected,
      'border-base-300 hover:border-base-content/30': !selected
    }"
  >
    <!-- Input Handle (Top) -->
    <Handle 
      v-if="!isStartNode" 
      type="target" 
      position="top" 
      class="w-3! h-3! bg-primary! ring-2 ring-white" 
    />
    
    <!-- Header: Type Indicator -->
    <div 
      class="px-3 py-1.5 text-xs font-bold uppercase text-white flex items-center gap-2"
      :class="headerColorClass"
    >
      <!-- Icon based on type -->
      <component :is="nodeIcon" class="w-3 h-3" />
      {{ displayType }}
    </div>

    <!-- Body: Label/Remark -->
    <div class="p-3 text-sm bg-base-100 text-base-content">
      <div class="font-medium text-center" v-if="label">{{ label }}</div>
      <div class="text-xs opacity-50 text-center" v-else>{{ placeholderText }}</div>
      
      <!-- Special content for certain node types -->
      <div v-if="data.type === 'fallback'" class="mt-2 text-xs opacity-70">
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
      type="source" 
      position="bottom" 
      class="w-3! h-3! bg-primary! ring-2 ring-white" 
    />
    
    <!-- Conditional Output Handle (Right) for condition nodes -->
    <Handle 
      v-if="isConditionNode"
      type="source" 
      position="right" 
      id="condition-true"
      class="w-3! h-3! bg-success! ring-2 ring-white" 
    />
  </div>
</template>

<script setup>
import { computed, h } from 'vue';
import { Handle } from '@vue-flow/core';

const props = defineProps({
  id: String,
  label: String,
  data: {
    type: Object,
    default: () => ({})
  },
  selected: Boolean,
});

// Default fallback strategies
const defaultStrategies = [
  { target: 'Return/Back button', label: '尝试点击返回' },
  { target: 'Close button', label: '尝试点击关闭' },
  { target: 'Confirm button', label: '尝试点击确认' },
];

// Color mapping for node types
const nodeTypeConfig = {
  // Basic
  click: { color: 'bg-blue-500', icon: 'cursor', display: 'Click' },
  wait: { color: 'bg-gray-500', icon: 'clock', display: 'Wait' },
  swipe: { color: 'bg-cyan-500', icon: 'move', display: 'Swipe' },
  
  // Conditions
  if_found: { color: 'bg-yellow-500', icon: 'search', display: 'IF Found' },
  if_not_found: { color: 'bg-orange-500', icon: 'search-x', display: 'IF Not Found' },
  
  // Vision
  find_image: { color: 'bg-purple-500', icon: 'image', display: 'Find Image' },
  ocr: { color: 'bg-violet-500', icon: 'type', display: 'OCR' },
  
  // Control
  loop: { color: 'bg-green-500', icon: 'repeat', display: 'Loop' },
  fallback: { color: 'bg-red-500', icon: 'alert-triangle', display: 'Fallback' },
  subflow: { color: 'bg-pink-500', icon: 'git-branch', display: 'Sub-Flow' },
  
  // Special
  start: { color: 'bg-emerald-600', icon: 'play', display: 'Start' },
  end: { color: 'bg-rose-600', icon: 'square', display: 'End' },
  input: { color: 'bg-emerald-600', icon: 'play', display: 'Start' },
};

const isStartNode = computed(() => ['start', 'input'].includes(props.data?.type));
const isConditionNode = computed(() => ['if_found', 'if_not_found'].includes(props.data?.type));

const headerColorClass = computed(() => {
  const config = nodeTypeConfig[props.data?.type];
  return config?.color || 'bg-neutral';
});

const displayType = computed(() => {
  const config = nodeTypeConfig[props.data?.type];
  return config?.display || props.data?.type || 'Node';
});

const placeholderText = computed(() => {
  switch (props.data?.type) {
    case 'click': return 'Set click target...';
    case 'wait': return 'Set wait duration...';
    case 'if_found': return 'Set search target...';
    case 'if_not_found': return 'Set search target...';
    case 'find_image': return 'Select image...';
    case 'ocr': return 'Set OCR region...';
    case 'loop': return 'Configure loop...';
    case 'fallback': return 'Fallback actions';
    case 'subflow': return 'Select sub-flow...';
    default: return 'No description';
  }
});

// Simple SVG icon component
const nodeIcon = computed(() => {
  const iconType = nodeTypeConfig[props.data?.type]?.icon || 'box';
  
  // Return a functional component that renders the appropriate icon
  return {
    render() {
      const icons = {
        cursor: h('svg', { xmlns: 'http://www.w3.org/2000/svg', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
          h('path', { d: 'M3 3l7.07 16.97 2.51-7.39 7.39-2.51L3 3z' }),
        ]),
        clock: h('svg', { xmlns: 'http://www.w3.org/2000/svg', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
          h('circle', { cx: '12', cy: '12', r: '10' }),
          h('polyline', { points: '12 6 12 12 16 14' }),
        ]),
        move: h('svg', { xmlns: 'http://www.w3.org/2000/svg', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
          h('polyline', { points: '5 9 2 12 5 15' }),
          h('polyline', { points: '9 5 12 2 15 5' }),
          h('polyline', { points: '15 19 12 22 9 19' }),
          h('polyline', { points: '19 9 22 12 19 15' }),
          h('line', { x1: '2', y1: '12', x2: '22', y2: '12' }),
          h('line', { x1: '12', y1: '2', x2: '12', y2: '22' }),
        ]),
        search: h('svg', { xmlns: 'http://www.w3.org/2000/svg', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
          h('circle', { cx: '11', cy: '11', r: '8' }),
          h('line', { x1: '21', y1: '21', x2: '16.65', y2: '16.65' }),
        ]),
        'search-x': h('svg', { xmlns: 'http://www.w3.org/2000/svg', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
          h('circle', { cx: '11', cy: '11', r: '8' }),
          h('line', { x1: '21', y1: '21', x2: '16.65', y2: '16.65' }),
          h('line', { x1: '8', y1: '8', x2: '14', y2: '14' }),
          h('line', { x1: '14', y1: '8', x2: '8', y2: '14' }),
        ]),
        image: h('svg', { xmlns: 'http://www.w3.org/2000/svg', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
          h('rect', { x: '3', y: '3', width: '18', height: '18', rx: '2', ry: '2' }),
          h('circle', { cx: '8.5', cy: '8.5', r: '1.5' }),
          h('polyline', { points: '21 15 16 10 5 21' }),
        ]),
        type: h('svg', { xmlns: 'http://www.w3.org/2000/svg', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
          h('polyline', { points: '4 7 4 4 20 4 20 7' }),
          h('line', { x1: '9', y1: '20', x2: '15', y2: '20' }),
          h('line', { x1: '12', y1: '4', x2: '12', y2: '20' }),
        ]),
        repeat: h('svg', { xmlns: 'http://www.w3.org/2000/svg', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
          h('polyline', { points: '17 1 21 5 17 9' }),
          h('path', { d: 'M3 11V9a4 4 0 0 1 4-4h14' }),
          h('polyline', { points: '7 23 3 19 7 15' }),
          h('path', { d: 'M21 13v2a4 4 0 0 1-4 4H3' }),
        ]),
        'alert-triangle': h('svg', { xmlns: 'http://www.w3.org/2000/svg', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
          h('path', { d: 'M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z' }),
          h('line', { x1: '12', y1: '9', x2: '12', y2: '13' }),
          h('line', { x1: '12', y1: '17', x2: '12.01', y2: '17' }),
        ]),
        'git-branch': h('svg', { xmlns: 'http://www.w3.org/2000/svg', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
          h('line', { x1: '6', y1: '3', x2: '6', y2: '15' }),
          h('circle', { cx: '18', cy: '6', r: '3' }),
          h('circle', { cx: '6', cy: '18', r: '3' }),
          h('path', { d: 'M18 9a9 9 0 0 1-9 9' }),
        ]),
        play: h('svg', { xmlns: 'http://www.w3.org/2000/svg', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
          h('polygon', { points: '5 3 19 12 5 21 5 3' }),
        ]),
        square: h('svg', { xmlns: 'http://www.w3.org/2000/svg', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
          h('rect', { x: '3', y: '3', width: '18', height: '18', rx: '2', ry: '2' }),
        ]),
        box: h('svg', { xmlns: 'http://www.w3.org/2000/svg', viewBox: '0 0 24 24', fill: 'none', stroke: 'currentColor', 'stroke-width': '2' }, [
          h('path', { d: 'M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z' }),
        ]),
      };
      return icons[iconType] || icons.box;
    }
  };
});
</script>

<style scoped>
/* Ensure handles are visible and properly styled */
.vue-flow__handle {
  border: 2px solid white;
}
</style>
