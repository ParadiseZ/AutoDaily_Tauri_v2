<template>
  <div class="local-scripts">
    <div class="content-wrapper">
      <div class="content-sidebar">
        <div class="sidebar-header">
          <h3>本地列表</h3>
          <el-button type="primary" size="small" circle>
            <el-icon><plus /></el-icon>
          </el-button>
        </div>
        
        <div class="script-list">
          <div
            v-for="(script, index) in scripts"
            :key="index"
            class="script-card"
            :class="{ 'active': selectedScriptIndex === index }"
            @click="selectScript(index)"
          >
            <el-checkbox v-model="script.selected" @click.stop />
            <div class="script-info">
              <div class="script-name">{{ script.name }}</div>
              <div class="script-desc">{{ script.description }}</div>
            </div>
            <div class="script-actions">
              <el-button 
                size="small" 
                circle
                :type="script.running ? 'danger' : 'primary'"
                :icon="script.running ? 'VideoPause' : 'VideoPlay'"
                @click.stop="toggleScript(index)"
              />
            </div>
          </div>
        </div>
        
        <div class="sidebar-footer">
          <el-button type="primary" round>
            <el-icon><video-play /></el-icon>
            全部运行
          </el-button>
        </div>
      </div>
      
      <div class="content-main">
        <template v-if="selectedScriptIndex !== null">
          <div class="script-details">
            <h2>{{ selectedScript.name }}</h2>
            <el-descriptions border :column="1">
              <el-descriptions-item label="作者">{{ selectedScript.author }}</el-descriptions-item>
              <el-descriptions-item label="版本">{{ selectedScript.version }}</el-descriptions-item>
              <el-descriptions-item label="路径">{{ selectedScript.path }}</el-descriptions-item>
            </el-descriptions>
            
            <div class="script-description">
              <h3>脚本简介</h3>
              <p>{{ selectedScript.description }}</p>
            </div>
            
            <div class="script-models">
              <h3>模型信息</h3>
              <el-tag v-for="model in selectedScript.models" :key="model" class="mr-2">{{ model }}</el-tag>
            </div>
          </div>
        </template>
        <el-empty v-else description="请选择脚本" />
      </div>
      
      <div class="content-config" v-if="selectedScriptIndex !== null">
        <div class="config-header">
          <h3>配置</h3>
          <div>
            <el-select v-model="selectedTemplate" placeholder="选择模板" size="small">
              <el-option v-for="template in templates" :key="template" :label="template" :value="template" />
            </el-select>
            <el-button type="text" size="small">
              <el-icon><plus /></el-icon>
            </el-button>
          </div>
        </div>
        
        <el-divider />
        
        <div class="config-form">
          <el-form label-position="top">
            <el-form-item label="设备配置">
              <el-select v-model="deviceConfig.type" placeholder="截图方式">
                <el-option label="ADB截图" value="adb" />
                <el-option label="窗口截图" value="window" />
              </el-select>
            </el-form-item>
            
            <el-form-item label="关联设备">
              <el-select v-model="deviceConfig.device" placeholder="选择设备">
                <el-option label="模拟器1" value="emulator-1" />
                <el-option label="模拟器2" value="emulator-2" />
                <el-option label="真机" value="device-1" />
              </el-select>
            </el-form-item>
            
            <el-form-item label="日志类型">
              <el-radio-group v-model="deviceConfig.logLevel">
                <el-radio label="none">关闭</el-radio>
                <el-radio label="basic">基本</el-radio>
                <el-radio label="detailed">详细</el-radio>
                <el-radio label="all">所有</el-radio>
              </el-radio-group>
            </el-form-item>
            
            <el-form-item>
              <el-checkbox v-model="deviceConfig.showOverlay">遮罩显示日志</el-checkbox>
            </el-form-item>
            
            <el-form-item label="操作后延迟 (秒)">
              <el-slider v-model="deviceConfig.delay" :min="0" :max="10" :step="0.1" />
            </el-form-item>
          </el-form>
          
          <h4>任务列表</h4>
          <el-tree
            :data="taskTree"
            :props="{ label: 'name', children: 'children' }"
            node-key="id"
            default-expand-all
            show-checkbox
            draggable
          />
          
          <div class="button-row">
            <el-button type="primary">保存配置</el-button>
            <el-button type="success">应用配置</el-button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue';
import { 
  Plus, 
  VideoPlay, 
  VideoPause 
} from '@element-plus/icons-vue';

// 脚本列表数据
const scripts = ref([
  {
    name: '每日签到',
    description: '自动完成多个平台的每日签到任务',
    author: '开发者1',
    version: '1.0.0',
    path: '/scripts/daily-signin.json',
    running: false,
    selected: false,
    models: ['通用OCR', 'YOLO目标检测'],
  },
  {
    name: '自动刷游戏',
    description: '自动完成游戏每日任务',
    author: '开发者2',
    version: '1.2.3',
    path: '/scripts/auto-game.json',
    running: false,
    selected: false,
    models: ['游戏专用OCR', 'YOLO目标检测'],
  },
  {
    name: '自动浏览',
    description: '自动浏览指定网页内容',
    author: '开发者3',
    version: '0.9.5',
    path: '/scripts/auto-browse.json',
    running: true,
    selected: true,
    models: ['通用OCR'],
  }
]);

const selectedScriptIndex = ref(0);
const selectedScript = computed(() => {
  if (selectedScriptIndex.value === null) return null;
  return scripts.value[selectedScriptIndex.value];
});

// 选择脚本
function selectScript(index) {
  selectedScriptIndex.value = index;
}

// 切换脚本运行状态
function toggleScript(index) {
  scripts.value[index].running = !scripts.value[index].running;
}

// 模板数据
const templates = ['默认模板', '模板1', '模板2'];
const selectedTemplate = ref('默认模板');

// 设备配置
const deviceConfig = ref({
  type: 'adb',
  device: 'emulator-1',
  logLevel: 'basic',
  showOverlay: true,
  delay: 0.5
});

// 任务树
const taskTree = [
  {
    id: 1,
    name: '签到任务',
    children: [
      { id: 11, name: '第一平台签到' },
      { id: 12, name: '第二平台签到' }
    ]
  },
  {
    id: 2,
    name: '日常任务',
    children: [
      { id: 21, name: '每日浏览' },
      { id: 22, name: '每日分享' },
      { id: 23, name: '每日评论' }
    ]
  },
  {
    id: 3,
    name: '清理任务',
    children: [
      { id: 31, name: '关闭应用' },
      { id: 32, name: '清理缓存' }
    ]
  }
];
</script>

<style lang="scss" scoped>
.local-scripts {
  height: 100%;
}

.content-wrapper {
  display: flex;
  height: 100%;
}

.content-sidebar {
  width: 240px;
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: var(--bg-color-soft);
  border-right: 1px solid var(--border-color-light);
}

.sidebar-header {
  padding: 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--border-color-light);
  
  h3 {
    margin: 0;
    font-size: 16px;
  }
}

.script-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.script-card {
  display: flex;
  align-items: center;
  padding: 8px;
  border-radius: 8px;
  margin-bottom: 8px;
  background-color: var(--bg-color-soft);
  cursor: pointer;
  transition: background-color 0.2s;
  
  &:hover {
    background-color: var(--bg-color-mute);
  }
  
  &.active {
    background-color: var(--el-color-primary-light-9);
    border-left: 3px solid var(--el-color-primary);
  }
  
  .script-info {
    flex: 1;
    margin: 0 10px;
    
    .script-name {
      font-weight: 500;
      margin-bottom: 4px;
    }
    
    .script-desc {
      font-size: 12px;
      color: var(--text-color-secondary);
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
      max-width: 130px;
    }
  }
}

.sidebar-footer {
  padding: 12px;
  display: flex;
  justify-content: center;
  border-top: 1px solid var(--border-color-light);
}

.content-main {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
  
  h2 {
    margin-top: 0;
    margin-bottom: 20px;
    font-size: 20px;
    background: var(--primary-gradient);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  
  .script-description {
    margin-top: 20px;
  }
  
  .script-models {
    margin-top: 20px;
    
    .mr-2 {
      margin-right: 8px;
      margin-bottom: 8px;
    }
  }
}

.content-config {
  width: 280px;
  height: 100%;
  padding: 16px;
  background-color: var(--bg-color-soft);
  border-left: 1px solid var(--border-color-light);
  overflow-y: auto;
  
  .config-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    
    h3 {
      margin: 0;
      font-size: 16px;
    }
  }
  
  .config-form {
    margin-top: 10px;
  }
  
  .button-row {
    display: flex;
    justify-content: center;
    gap: 12px;
    margin-top: 20px;
  }
}
</style> 