<template>
  <header class="app-header">
    <div class="header-left">
      <h3 class="page-title">{{ currentRouteTitle }}</h3>
    </div>
    <div class="header-right">
      <el-button 
        size="small" 
        circle 
        @click="toggleAlwaysOnTop"
        :type="windowsStore.alwaysOnTop ? 'primary' : 'default'"
        :icon="windowsStore.alwaysOnTop ? 'Lock' : 'Unlock'"
        title="窗口置顶"
      />
      <el-button 
        size="small" 
        circle 
        @click="themeStore.toggleDarkMode()"
        :icon="themeStore.isDark ? 'Sunny' : 'Moon'"
      />
      <el-dropdown trigger="click">
        <el-button size="small" class="theme-button" type="primary">
          <el-icon><Brush /></el-icon>
          <span>主题色</span>
        </el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <div class="color-picker-dropdown">
              <el-color-picker
                v-model="themeStore.primaryColor"
                @change="themeStore.changePrimaryColor"
                show-alpha
                size="small"
              />
              <div class="preset-colors">
                <div
                  v-for="(color, index) in presetColors"
                  :key="index"
                  class="color-preset"
                  :style="{ backgroundColor: color }"
                  @click="themeStore.changePrimaryColor(color)"
                ></div>
              </div>
            </div>
          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </div>
  </header>
</template>

<script setup>
import { computed, onMounted } from 'vue';
import { useRoute } from 'vue-router';
import { useThemeStore } from '../../stores/theme';
import { useWindowsStore } from '../../stores/windows';
import { Sunny, Moon, Brush, Lock, Unlock } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';

const route = useRoute();
const themeStore = useThemeStore();
const windowsStore = useWindowsStore();

onMounted(async () => {
  await windowsStore.loadConfig();
});

// 预设颜色
const presetColors = [
  '#409EFF', // 默认蓝
  '#67C23A', // 绿色
  '#E6A23C', // 黄色
  '#F56C6C', // 红色
  '#909399', // 灰色
  '#8957e5', // 紫色
  '#13C2C2', // 青色
  '#722ED1'  // 深紫
];

// 获取当前路由标题
const currentRouteTitle = computed(() => {
  return route.meta.title || 'AutoDaily';
});

// 切换窗口置顶状态
async function toggleAlwaysOnTop() {
  try {
    windowsStore.alwaysOnTop = !windowsStore.alwaysOnTop;
    await windowsStore.saveConfig();
    ElMessage.success(windowsStore.alwaysOnTop ? '窗口置顶已启用' : '窗口置顶已禁用');
  } catch (error) {
    console.error('Failed to toggle always on top:', error);
    ElMessage.error('切换窗口置顶失败');
    // 回滚状态
    windowsStore.alwaysOnTop = !windowsStore.alwaysOnTop;
  }
}
</script>

<style lang="scss" scoped>
.app-header {
  height: 60px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  background-color: var(--bg-color-soft);
  box-shadow: var(--box-shadow);
  z-index: 100;
  
  .header-left {
    .page-title {
      font-size: 18px;
      font-weight: 600;
      color: var(--text-color-primary);
      margin: 0;
    }
  }
  
  .header-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }
}

.color-picker-dropdown {
  padding: 12px;
  
  .preset-colors {
    margin-top: 12px;
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 8px;
    
    .color-preset {
      width: 24px;
      height: 24px;
      border-radius: 4px;
      cursor: pointer;
      transition: transform 0.2s;
      
      &:hover {
        transform: scale(1.1);
      }
    }
  }
}

.theme-button {
  display: flex;
  align-items: center;
  gap: 5px;
}
</style> 