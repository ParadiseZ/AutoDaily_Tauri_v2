<template>
  <div class="flex-1 overflow-y-auto p-2">
    <!-- 动态生成分类和节点 -->
    <div v-for="category in nodeCategories" :key="category.key" class="mb-4 last:mb-0">
      <div
        class="text-xs font-bold opacity-50 mb-3 uppercase tracking-wide"
        :class="{ 'mt-2': category.key !== 'basic' }"
      >
        {{ category.label }}
      </div>

      <div class="grid gap-2" :class="category.key === 'composite' ? 'grid-cols-1' : 'grid-cols-2'">
        <ToolboxItem
          v-for="nodeType in category.types"
          :key="nodeType"
          :type="nodeType"
          :label="getToolboxLabel(nodeType)"
          :color="getNodeColor(nodeType)"
          :icon="getNodeIcon(nodeType)"
          :description="getToolboxDescription(nodeType)"
          @add-node="emitAddNode"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import ToolboxItem from './ToolboxItem.vue';
import { NODE_CATEGORIES, NODE_TYPES, getNodeColor, getNodeDescription, getNodeIcon } from './config';

const emit = defineEmits<{
  (e: 'add-node', type: string): void;
}>();

const nodeCategories = NODE_CATEGORIES;

const getToolboxLabel = (type: string) => {
  const config = (NODE_TYPES as any)[type];
  if (!config) return type;
  return config.displayCn;
};

const getToolboxDescription = (type: string) => getNodeDescription(type);

const emitAddNode = (type: string) => {
  emit('add-node', type);
};
</script>
