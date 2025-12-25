<template>
  <div 
    class="card bg-base-200 shadow-sm p-3 cursor-pointer hover:bg-base-300 border border-transparent hover:border-primary transition-all mb-2 group"
    draggable="true"
    @click="onClick"
  >
    <div class="font-medium flex items-center gap-2 text-sm relative">
      <div 
        class="w-8 h-8 rounded-lg shrink-0 flex items-center justify-center text-white shadow-sm" 
        :class="color"
      >
        <IconRenderer :icon="icon" class="w-4 h-4" />
      </div>
      <span class="flex-1 truncate">{{ label }}</span>
      <!-- Quick add button (visible on hover) -->
      <button 
        class="btn btn-xs btn-circle btn-ghost opacity-0 group-hover:opacity-100 transition-opacity"
        @click.stop="onClick"
        title="添加"
      >
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="12" y1="5" x2="12" y2="19"></line>
          <line x1="5" y1="12" x2="19" y2="12"></line>
        </svg>
      </button>
    </div>
    <div class="text-[10px] opacity-40 mt-1 pl-10 leading-tight">{{ description }}</div>
</div>
</template>

<script setup>
import IconRenderer from './IconRenderer.vue';

const props = defineProps({
  type: {
    type: String,
    required: true
  },
  label: {
    type: String,
    required: true
  },
  icon: {
    type: String,
    default: 'box'
  },
  color: {
    type: String,
    default: 'bg-neutral'
  },
  description: {
    type: String,
    default: ''
  }
});

const emit = defineEmits(['add-node']);

// Drag start handler for drag-and-drop
/*const onDragStart = (event) => {
  if (event.dataTransfer) {
    event.dataTransfer.setData('application/vueflow', props.type);
    event.dataTransfer.effectAllowed = 'copy';
  }
};*/

// Click handler for quick add
const onClick = () => {
  emit('add-node', props.type);
};
</script>
