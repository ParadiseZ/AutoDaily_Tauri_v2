<template>
  <div class="window-settings">
    
    <el-card shadow="hover" class="setting-card">
      <template #header>
        <h3>基本设置</h3>
      </template>
      <el-form label-position="top" :model="windowsStore" label-width="120px">
        <el-form-item label="应用启动模式">
          <el-radio-group v-model="windowsStore.startMode">
            <el-radio value="normal">普通窗口</el-radio>
            <el-radio value="minimized">最小化启动</el-radio>
            <el-radio value="tray">仅托盘图标</el-radio>
          </el-radio-group>
        </el-form-item>
        
        <el-form-item label="关闭按钮行为">
          <el-radio-group v-model="windowsStore.closeExit">
            <el-radio :value="true">完全退出</el-radio>
            <el-radio :value="false">最小化到托盘</el-radio>
          </el-radio-group>
        </el-form-item>
        
        <el-form-item label="窗口置顶">
          <el-checkbox v-model="windowsStore.alwaysOnTop">保持置顶</el-checkbox>
        </el-form-item>
        

        
        <el-form-item label="记住窗口状态">
          <el-checkbox v-model="windowsStore.remSizePosition">记住窗口大小和位置</el-checkbox>
        </el-form-item>
      </el-form>
    </el-card>
    
    <el-card shadow="hover" class="setting-card">
      <template #header>
        <h3>高级设置</h3>
      </template>
      
      <el-form label-position="top" :model="windowsStore" label-width="120px">
        <el-form-item label="开机自启动">
          <el-checkbox v-model="windowsStore.autoStart">开机时自动启动</el-checkbox>
        </el-form-item>
        
        <el-form-item label="空闲时行为（无脚本运行时）">
          <el-radio-group v-model="windowsStore.idleAction">
            <el-radio value="none">无操作</el-radio>
            <el-radio value="shutdown">关机</el-radio>
            <el-radio value="sleep">睡眠</el-radio>
            <el-radio value="hibernate">休眠</el-radio>
          </el-radio-group>
        </el-form-item>
      </el-form>
    </el-card>
    
    <el-card shadow="hover" class="setting-card">
      <template #header>
        <h3>快捷键设置</h3>
      </template>
      
      <el-form label-position="left" :model="windowsStore" label-width="180px">
        <el-form-item label="显示/隐藏主窗口">
          <el-input
            :value="getShortcutDisplay('toggleWindow')"
            :placeholder="getInputPlaceholder('toggleWindow')"
            :class="{ 'recording': isRecording('toggleWindow') }"
            @focus="startRecording('toggleWindow')"
            @blur="stopRecording"
            @keydown="onKeyDown"
            @keyup="onKeyUp"
            @input="onInput('toggleWindow', $event)"
          />
        </el-form-item>
        <el-form-item label="运行/停止所有脚本">
          <el-input
            :value="getShortcutDisplay('toggleAllScripts')"
            :placeholder="getInputPlaceholder('toggleAllScripts')"
            :class="{ 'recording': isRecording('toggleAllScripts') }"
            @focus="startRecording('toggleAllScripts')"
            @blur="stopRecording"
            @keydown="onKeyDown"
            @keyup="onKeyUp"
            @input="onInput('toggleAllScripts', $event)"
          />
        </el-form-item>
        <el-form-item label="截图">
          <el-input
            :value="getShortcutDisplay('capture')"
            :placeholder="getInputPlaceholder('capture')"
            :class="{ 'recording': isRecording('capture') }"
            @focus="startRecording('capture')"
            @blur="stopRecording"
            @keydown="onKeyDown"
            @keyup="onKeyUp"
            @input="onInput('capture', $event)"
          />
        </el-form-item>
      </el-form>
    </el-card>
    
    <div class="action-buttons">
      <el-button type="primary" @click="saveConfig">保存设置</el-button>
      <el-button @click="resetConfig">重置设置</el-button>
    </div>
  </div>
</template>

<script setup>
import { onMounted, ref, reactive } from 'vue';
import { useWindowsStore } from '../../stores/windows';
import { ElMessage, ElMessageBox } from 'element-plus';
import { register } from '@tauri-apps/plugin-global-shortcut';

// Use store
const windowsStore = useWindowsStore();

// 快捷键录制状态
const recordingState = reactive({
  isRecording: false,
  currentKey: null,
  pressedKeys: new Set(),
  tempDisplay: ''
});

onMounted(async () => {
  await windowsStore.loadConfig();
});

// 获取快捷键显示值
function getShortcutDisplay(key) {
  if (recordingState.isRecording && recordingState.currentKey === key) {
    return recordingState.tempDisplay || '空';
  }
  return windowsStore.shortcut[key] || '空';
}

// 获取输入框占位符
function getInputPlaceholder(key) {
  if (recordingState.isRecording && recordingState.currentKey === key) {
    return '请按下快捷键组合...';
  }
  return '点击设置快捷键';
}

// 检查是否正在录制
function isRecording(key) {
  return recordingState.isRecording && recordingState.currentKey === key;
}

// 开始录制快捷键
function startRecording(key) {
  recordingState.isRecording = true;
  recordingState.currentKey = key;
  recordingState.pressedKeys.clear();
  // 不清空显示内容，保持原有值
  recordingState.tempDisplay = windowsStore.shortcut[key] || '';
}

// 停止录制快捷键
function stopRecording() {
  recordingState.isRecording = false;
  recordingState.currentKey = null;
  recordingState.pressedKeys.clear();
  recordingState.tempDisplay = '';
}

// 获取按键标签
function getKeyLabel(key) {
  const labels = {
    'toggleWindow': '显示/隐藏主窗口',
    'toggleAllScripts': '运行/停止所有脚本',
    'capture': '截图'
  };
  return labels[key] || key;
}

// 处理输入事件（删除操作）
async function onInput(key, value) {
  if (recordingState.isRecording) {
    // 在录制状态下处理删除
    if (value === '' || value === '空') {
      // 清空快捷键并保存
      recordingState.tempDisplay = '';
      await saveShortcut(key, '');
      stopRecording();
    } else {
      // 更新临时显示
      recordingState.tempDisplay = value;
    }
  } else {
    // 非录制状态下的删除操作
    if (value === '' || value === '空') {
      // 清空快捷键并保存
      windowsStore.$patch({
        shortcut: {
          ...windowsStore.shortcut,
          [key]: ''
        }
      });
      await windowsStore.saveConfig();
      ElMessage.success('快捷键已清空');
    }
  }
}

// 处理按键按下事件
function onKeyDown(event) {
  if (!recordingState.isRecording) return;
  
  event.preventDefault();
  event.stopPropagation();
  
  const key = event.key;
  const code = event.code;
  
  // 添加到已按下的键集合
  recordingState.pressedKeys.add(key);
  
  // 收集修饰键
  const mods = [];
  if (event.ctrlKey) mods.push('Ctrl');
  if (event.altKey) mods.push('Alt');
  if (event.shiftKey) mods.push('Shift');
  if (event.metaKey) mods.push('Meta');
  
  // 检查是否是修饰键本身
  const isModifierKey = ['Control', 'Alt', 'Shift', 'Meta'].includes(key);
  
  if (isModifierKey && mods.length > 0) {
    // 只按下修饰键时显示提示
    recordingState.tempDisplay = mods.join('+') + '+';
  } else if (!isModifierKey) {
    // 按下非修饰键时
    let finalKey = key;
    
    // 处理特殊键名
    if (key.length === 1) {
      finalKey = key.toUpperCase();
    } else {
      // 处理功能键等
      const keyMap = {
        ' ': 'Space',
        'Enter': 'Enter',
        'Escape': 'Esc',
        'Tab': 'Tab',
        'Backspace': 'Backspace',
        'Delete': 'Delete',
        'ArrowUp': 'Up',
        'ArrowDown': 'Down',
        'ArrowLeft': 'Left',
        'ArrowRight': 'Right'
      };
      finalKey = keyMap[key] || key;
    }
    
    // 组合完整的快捷键
    const combo = mods.length > 0 ? mods.join('+') + '+' + finalKey : finalKey;
    recordingState.tempDisplay = combo;
    
    // 立即保存（延迟一点让用户看到效果）
    setTimeout(async () => {
      await saveShortcut(recordingState.currentKey, combo);
      stopRecording();
    }, 100);
  }
}

// 处理按键释放事件
async function onKeyUp(event) {
  if (!recordingState.isRecording) return;
  
  event.preventDefault();
  event.stopPropagation();
  
  const key = event.key;
  recordingState.pressedKeys.delete(key);
  
  // 如果释放的是修饰键且没有其他键按下，保存空值并停止录制
  const isModifierKey = ['Control', 'Alt', 'Shift', 'Meta'].includes(key);
  if (isModifierKey && recordingState.pressedKeys.size === 0) {
    // 松开修饰键时，如果没有完整组合键，就保存空值
    recordingState.tempDisplay = '';
    await saveShortcut(recordingState.currentKey, '');
    stopRecording();
  }
}

// 保存快捷键
async function saveShortcut(key, combo) {
  try {
    windowsStore.$patch({
      shortcut: {
        ...windowsStore.shortcut,
        [key]: combo
      }
    });
    await windowsStore.saveConfig();
    
    if (combo === '' || combo === '空') {
      ElMessage.success('快捷键已清空');
    } else {
      ElMessage.success(`快捷键设置成功: ${combo}`);
    }
  } catch (error) {
    ElMessage.error('保存快捷键失败');
    console.error(error);
  }
}

// 保存配置
async function saveConfig() {
  await windowsStore.saveConfig();
  ElMessage.success('设置已保存');
}

// 重置配置
function resetConfig() {
  ElMessageBox.confirm('确认要重置所有设置吗？', '提示', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(() => {
    windowsStore.$reset(); // If using default, or set to defaults
    ElMessage.success('所有设置已重置为默认值');
  });
}
</script>

<style lang="scss" scoped>
.window-settings {
  max-width: 800px;
  margin: 0 auto;
  
  h1 {
    margin-top: 0;
    margin-bottom: 20px;
    font-size: 24px;
    background: var(--primary-gradient);
    background-clip: text;
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  
  .setting-card {
    margin-bottom: 20px;
    
    h3 {
      margin: 0;
      font-size: 16px;
    }
  }
  
  .action-buttons {
    display: flex;
    justify-content: center;
    gap: 12px;
    margin-top: 20px;
    padding: 20px 0;
  }
  
  // 快捷键录制状态样式
  :deep(.recording .el-input__wrapper) {
    border-color: var(--el-color-primary) !important;
    box-shadow: 0 0 0 1px var(--el-color-primary) inset, 
                0 0 0 1px var(--el-color-primary), 
                0 0 8px 0 rgba(var(--el-color-primary-rgb), 0.2) !important;
    background-color: var(--el-color-primary-light-9) !important;
  }
  
  :deep(.recording .el-input__inner) {
    color: var(--el-color-primary) !important;
    font-weight: 500;
  }
}
</style> 