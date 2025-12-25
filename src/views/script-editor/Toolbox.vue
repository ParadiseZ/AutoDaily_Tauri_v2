<template>
  <div class="flex-1 overflow-y-auto p-2 space-y-2">
    <!-- 动态生成分类和节点 -->
    <template v-for="category in nodeCategories" :key="category.key">
      <div class="text-xs font-bold opacity-50 mb-2 uppercase tracking-wide" :class="{ 'mt-4': category.key !== 'basic' }">
        {{ category.label }}
      </div>
      
      <ToolboxItem 
        v-for="nodeType in category.types" 
        :key="nodeType"
        :type="nodeType"
        :label="getToolboxLabel(nodeType)"
        :color="getNodeColor(nodeType)"
        :description="getToolboxDescription(nodeType)"
        @add-node="emitAddNode"
      />
    </template>
  </div>
</template>

<script setup>
import ToolboxItem from './ToolboxItem.vue';
import { NODE_CATEGORIES, NODE_TYPES, getNodeColor, getNodeDescription } from './config.js';

const emit = defineEmits(['add-node']);

// 使用统一配置的分类
const nodeCategories = NODE_CATEGORIES;

// 获取工具箱显示标签
const getToolboxLabel = (type) => {
  const config = NODE_TYPES[type];
  if (!config) return type;
  return config.displayCn ? `${config.displayCn}|${config.display}` : config.display;
};

// 获取工具箱描述 - 使用配置函数
const getToolboxDescription = (type) => getNodeDescription(type);

const emitAddNode = (type) => {
  emit('add-node', type);
};
</script>

