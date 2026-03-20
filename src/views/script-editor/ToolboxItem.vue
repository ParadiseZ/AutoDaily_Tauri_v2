<template>
  <div
    class="card bg-base-200 shadow-sm p-3 cursor-pointer hover:bg-base-300 border border-transparent hover:border-primary transition-all group"
    :draggable="true"
    @dragstart="handleDragStart"
    @click="onClick"
  >
    <div class="font-medium flex items-center gap-2 text-sm relative">
      <div class="w-6 h-6 rounded-lg shrink-0 flex items-center justify-center text-white shadow-sm" :class="color">
        <IconRenderer :icon="icon" class="w-4 h-4" />
      </div>
      <span class="flex-1 truncate">{{ label }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import IconRenderer from './IconRenderer.vue';

const props = defineProps<{
  type: string;
  label: string;
  icon: string;
  color?: string;
  description?: string;
}>();

const emit = defineEmits<{
  (e: 'add-node', type: string): void;
}>();

const handleDragStart = (event: DragEvent) => {
  if (event.dataTransfer) {
    event.dataTransfer.setData('application/vueflow', props.type);
    event.dataTransfer.effectAllowed = 'move';
  }
};

const onClick = () => {
  emit('add-node', props.type);
};
</script>
