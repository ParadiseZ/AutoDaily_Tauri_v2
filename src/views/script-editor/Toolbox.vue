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
      
      <div 
        class="grid gap-2"
        :class="category.key === 'composite' ? 'grid-cols-1' : 'grid-cols-2'"
      >
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

<script setup>
import ToolboxItem from './ToolboxItem.vue';
import { NODE_CATEGORIES, NODE_TYPES, getNodeColor, getNodeDescription,getNodeIcon } from './config.js';

const emit = defineEmits(['add-node']);

// 使用统一配置的分类
const nodeCategories = NODE_CATEGORIES;

// 获取工具箱显示标签
const getToolboxLabel = (type) => {
  const config = NODE_TYPES[type];
  if (!config) return type;
  //return config.displayCn ? `${config.displayCn}|${config.display}` : config.display;
  return config.displayCn;
};

// 获取工具箱描述 - 使用配置函数
const getToolboxDescription = (type) => getNodeDescription(type);

const emitAddNode = (type) => {
  emit('add-node', type);
};
</script>

