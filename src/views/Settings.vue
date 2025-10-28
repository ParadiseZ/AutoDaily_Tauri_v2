<template>
  <div class="settings-container">
    <div class="settings-sidebar">
      <el-menu
        :default-active="activeTab"
        class="settings-menu"
        @select="handleTabChange"
      >
        <el-menu-item index="window">
          <el-icon><monitor /></el-icon>
          <span>窗口设置</span>
        </el-menu-item>
        <el-menu-item index="scriptGlobalConfig">
          <el-icon><setting /></el-icon>
          <span>脚本全局设置</span>
        </el-menu-item>
        <el-menu-item index="devices">
          <el-icon><Cellphone /></el-icon>
          <span>设备管理</span>
        </el-menu-item>
        <el-menu-item index="resources">
          <el-icon><files /></el-icon>
          <span>资源管理</span>
        </el-menu-item>
        <el-menu-item index="theme">
          <el-icon><brush /></el-icon>
          <span>主题设置</span>
        </el-menu-item>
      </el-menu>
    </div>
    
    <div class="settings-content">
      <router-view />
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { 
  Monitor, 
  Setting, 
  Cellphone, 
  Files, 
  Brush 
} from '@element-plus/icons-vue';

const router = useRouter();
const route = useRoute();

const activeTab = computed(() => {
  const name = route.name;
  return name ? name.split('-')[1] : 'window';
});

function handleTabChange(tab) {
  if (tab !== activeTab.value) {
    router.push({ name: `settings-${tab}` });
  }
}

onMounted(() => {
  // 如果当前路径是settings根路径，自动跳转到窗口设置页面
  if (route.name === 'settings') {
    router.replace({ name: 'settings-window' });
  }
});
</script>

<style lang="scss" scoped>
.settings-container {
  height: 100%;
  display: flex;
}

.settings-sidebar {
  width: 220px;
  border-right: 1px solid var(--border-color-light);
  background-color: var(--bg-color-soft);
  
  .settings-menu {
    border-right: none;
  }
}

.settings-content {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

:deep(.el-menu-item) {
  height: 50px;
  line-height: 50px;
  display: flex;
  align-items: center;
}

:deep(.el-menu-item.is-active) {
  border-right: 3px solid var(--el-color-primary);
}

:deep(.el-icon) {
  margin-right: 10px;
  width: 24px;
  text-align: center;
  font-size: 18px;
  vertical-align: middle;
}
</style> 