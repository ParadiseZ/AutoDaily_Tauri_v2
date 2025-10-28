<template>
  <div class="sidebar">
    <div class="sidebar-header">
      <h2 class="app-title">AutoDaily</h2>
    </div>
    <el-menu
      :default-active="activeIndex"
      class="sidebar-menu"
      router
      :collapse="isCollapse">
      <el-menu-item index="/">
        <el-icon><el-icon-monitor /></el-icon>
        <template #title>性能监控</template>
      </el-menu-item>
      <el-menu-item index="/local-scripts">
        <el-icon><el-icon-files /></el-icon>
        <template #title>本地列表</template>
      </el-menu-item>
      <el-menu-item index="/marketplace">
        <el-icon><el-icon-search /></el-icon>
        <template #title>搜索</template>
      </el-menu-item>
      <el-menu-item index="/settings">
        <el-icon><el-icon-setting /></el-icon>
        <template #title>设置</template>
      </el-menu-item>
      <el-menu-item index="/developer">
        <el-icon><el-icon-cpu /></el-icon>
        <template #title>开发者</template>
      </el-menu-item>
      <el-menu-item index="/about">
        <el-icon><el-icon-info-filled /></el-icon>
        <template #title>关于</template>
      </el-menu-item>
    </el-menu>
  </div>
</template>

<script setup>
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { 
  Monitor as ElIconMonitor,
  Files as ElIconFiles,
  Search as ElIconSearch,
  Setting as ElIconSetting,
  InfoFilled as ElIconInfoFilled,
  Cpu as ElIconCpu
} from '@element-plus/icons-vue';

const route = useRoute();
const isCollapse = false;

// 计算当前活动的菜单项
const activeIndex = computed(() => {
  const path = route.path;
  // 处理一级路由
  if (path === '/') return '/';
  // 处理二级路由
  const mainPath = '/' + path.split('/')[1];
  return mainPath;
});
</script>

<style lang="scss" scoped>
.sidebar {
  display: flex;
  flex-direction: column;
  height: 100%;
  
  &-header {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 60px;
    padding: 0 16px;
    background: var(--primary-gradient);
    color: white;
  }
  
  &-menu {
    flex: 1;
    border-right: none !important;
    
    :deep(.el-menu-item) {
      margin: 4px 8px;
      border-radius: 4px;
      
      &.is-active {
        background: var(--primary-gradient) !important;
        color: white !important;
      }
    }
  }
}

.app-title {
  margin: 0;
  font-size: 20px;
  font-weight: bold;
  background: linear-gradient(45deg, #fff, #e0e0e0);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}
</style> 