<template>
  <div class="developer">
    <div class="dev-header">
      <h2>开发者工具</h2>
      <div class="dev-nav">
        <el-radio-group v-model="activeTab" size="small">
          <el-radio-button 
            v-for="tab in devTabs" 
            :key="tab.route" 
            :value="tab.route">
            {{ tab.name }}
          </el-radio-button>
        </el-radio-group>
      </div>
    </div>
    
    <div class="dev-content">
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue';
import { useRouter, useRoute } from 'vue-router';

const router = useRouter();
const route = useRoute();

const devTabs = [
  { name: '截图测试', route: 'capture-test' },
  { name: 'OCR测试', route: 'ocr-test' },
  { name: 'Yolo测试', route: 'yolo-test' },
  { name: 'Onnx测试', route: 'onnx-test' },
  { name: '性能测试', route: 'performance-test' }
];

// 根据当前路由设置活动标签
const activeTab = ref(route.path.split('/').pop() || 'capture-test');

// 监听标签变化，切换路由
watch(activeTab, (newTab) => {
  router.push(`/developer/${newTab}`);
});

// 监听路由变化，更新活动标签
watch(
  () => route.path,
  (newPath) => {
    const tab = newPath.split('/').pop();
    if (tab && tab !== activeTab.value) {
      activeTab.value = tab;
    }
  }
);
</script>

<style lang="scss" scoped>
.developer {
  display: flex;
  flex-direction: column;
  height: 100%;
  
  .dev-header {
    margin-bottom: 20px;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 16px;
    
    h2 {
      margin: 0;
      background: var(--primary-gradient);
      -webkit-background-clip: text;
      -webkit-text-fill-color: transparent;
    }
    
    .dev-nav {
      flex-grow: 1;
      
      .el-radio-group {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;
      }
    }
  }
  
  .dev-content {
    flex: 1;
    background-color: var(--bg-color-soft);
    border-radius: 8px;
    padding: 10px;
    box-shadow: var(--box-shadow);
    min-height: 100%;
  }
}
</style> 