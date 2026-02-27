<template>
  <div class="w-80 border-l border-base-300 flex flex-col bg-base-100 shadow-md z-1 h-full">
    <div class="p-3 font-bold text-sm bg-base-200 flex justify-between items-center">
      属性
      <div class="badge badge-sm" v-if="selectedNode">{{ nodeTypeDisplay }}</div>
      <div class="badge badge-sm badge-ghost" v-else>未选择</div>
    </div>

    <!-- Node Selected -->
    <div class="flex-1 p-4 overflow-y-auto" v-if="selectedNode">
      <div class="divider text-xs opacity-50 mt-0">Configuration</div>

      <!-- Step Mode: Use unified StepItemEditor -->
      <div v-if="!['start', 'end'].includes(selectedNode.data?.type)" class="-mx-3">
        <StepItemEditor :step="localData as any" :isPropertiesPanel="true" @update="onStepUpdate" />
      </div>

      <!-- Delete Button -->
      <div class="mt-8" v-if="!['start', 'input'].includes(selectedNode.data?.type)">
        <button class="btn btn-error btn-sm w-full btn-outline" @click="$emit('delete-node')">Delete Node</button>
      </div>
    </div>

    <!-- No Selection -->
    <div class="flex-1 p-10 flex flex-col items-center justify-center text-base-content/30" v-else>
      <svg
        xmlns="http://www.w3.org/2000/svg"
        width="48"
        height="48"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="1"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <rect x="3" y="3" width="18" height="18" rx="2" ry="2" />
        <line x1="9" y1="9" x2="15" y2="15" />
        <line x1="15" y1="9" x2="9" y2="15" />
      </svg>
      <span class="mt-2 text-sm">选择一个节点后编辑</span>
      <span class="mt-1 text-xs opacity-50">点击画布中的任意节点</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, defineAsyncComponent } from 'vue';
import type { Node } from '@vue-flow/core';
import { getNodeDisplay } from './config';

const StepItemEditor = defineAsyncComponent(() => import('./components/StepItemEditor.vue'));

const props = defineProps<{
  selectedNode: Node | null;
}>();

const emit = defineEmits<{
  (e: 'delete-node'): void;
  (e: 'update-node', id: string, data: any): void;
}>();

const localData = ref<any>({});

const nodeTypeDisplay = computed(() => {
  if (!props.selectedNode) return 'Unknown';
  const type = props.selectedNode.data?.type;
  return getNodeDisplay(type) || type || 'Unknown';
});

watch(
  () => props.selectedNode,
  (newNode) => {
    if (newNode) {
      localData.value = { ...newNode.data };
    }
  },
  { immediate: true, deep: true }
);

const onStepUpdate = (newStepConfig: any) => {
  localData.value = newStepConfig;
};

watch(
  localData,
  (newData) => {
    if (props.selectedNode) {
      emit('update-node', props.selectedNode.id, newData);
    }
  },
  { deep: true }
);
</script>
