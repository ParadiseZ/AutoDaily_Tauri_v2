<template>
  <div class="capture-test">
    <div class="control-panel">
      <el-card shadow="hover">
        <template #header>
          <div class="card-header">
            <h3>截图测试工具</h3>
            <div>
              <el-tooltip content="截图结果会保存在capture-test文件夹中">
                <el-icon><QuestionFilled /></el-icon>
              </el-tooltip>
            </div>
          </div>
        </template>
        
        <div class="form-items">
          <div class="form-item">
            <span class="label">截图方式:</span>
            <el-select v-model="captureMethod" placeholder="请选择">
              <el-option label="ADB截图" value="adb" />
              <el-option label="窗口截图" value="window" />
            </el-select>
          </div>
          
          <template v-if="captureMethod === 'adb'">
            <div class="form-item">
              <span class="label">设备选择:</span>
              <el-select v-model="selectedDevice" placeholder="请选择设备">
                <el-option
                  v-for="device in deviceList"
                  :key="device.id"
                  :label="device.name"
                  :value="device.id"
                />
              </el-select>
            </div>
            <div class="form-item">
              <span class="label">ADB路径:</span>
              <div class="path-input">
                <el-input v-model="adbPath" placeholder="请输入ADB路径" />
                <el-button @click="selectAdbPath">浏览</el-button>
              </div>
            </div>
            <div class="form-item">
              <span class="label">加速器:</span>
              <el-select v-model="adbAccelerator" placeholder="请选择">
                <el-option label="无" value="none" />
                <el-option label="minicap" value="minicap" />
                <el-option label="screencapRaw" value="screencapRaw" />
              </el-select>
            </div>
          </template>
          
          <template v-if="captureMethod === 'window'">
            <div class="form-item">
              <span class="label">窗口名称:</span>
              <el-input v-model="windowName" placeholder="请输入窗口名称" />
              <el-button class="ml-2" @click="refreshWindowList">刷新窗口列表</el-button>
            </div>
            <div class="form-item window-list">
              <el-table :data="windowList" height="150" style="width: 100%" @row-click="selectWindow">
                <el-table-column prop="title" label="窗口标题" />
                <el-table-column prop="class" label="窗口类" width="120" />
              </el-table>
            </div>
          </template>
        </div>
        
        <div class="button-row">
          <el-button type="primary" @click="capture">开始截图</el-button>
          <el-button type="success" @click="saveSettings">保存设置</el-button>
        </div>
      </el-card>
    </div>
    
    <div class="preview-panel">
      <el-card shadow="hover">
        <template #header>
          <div class="card-header">
            <h3>预览区域</h3>
            <div>
              <el-tag v-if="lastCaptureTime" size="small">
                {{ formatDate(lastCaptureTime) }}
              </el-tag>
            </div>
          </div>
        </template>
        
        <div class="capture-preview" ref="previewArea">
          <img v-if="captureResult" :src="captureResult" alt="截图结果" />
          <el-empty v-else description="暂无截图" />
        </div>
        
        <div class="button-row" v-if="captureResult">
          <el-button @click="saveCapture">保存截图</el-button>
          <el-button @click="copyCapture">复制截图</el-button>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { QuestionFilled } from '@element-plus/icons-vue';
import { ElMessage } from 'element-plus';

// 截图设置
const captureMethod = ref('window');
const adbPath = ref('');
const adbAccelerator = ref('none');
const selectedDevice = ref('');
const windowName = ref('');

// 设备与窗口列表
const deviceList = ref([]);
const windowList = ref([]);

// 截图结果
const captureResult = ref(null);
const lastCaptureTime = ref(null);
const previewArea = ref(null);

// 初始化
onMounted(() => {
  loadSettings();
  if (captureMethod.value === 'adb') {
    loadDeviceList();
  } else {
    refreshWindowList();
  }
});

// 加载已保存设置
async function loadSettings() {
  try {
    // TODO: 实际项目中这里应该从Tauri API调用后端
    // const settings = await invoke('get_capture_settings');
    // 模拟数据
    const settings = {
      captureMethod: 'window',
      adbPath: 'C:\\Android\\platform-tools\\adb.exe',
      adbAccelerator: 'none',
      selectedDevice: '',
      windowName: 'AutoDaily'
    };
    
    captureMethod.value = settings.captureMethod;
    adbPath.value = settings.adbPath;
    adbAccelerator.value = settings.adbAccelerator;
    selectedDevice.value = settings.selectedDevice;
    windowName.value = settings.windowName;
  } catch (error) {
    ElMessage.error('加载设置失败：' + error);
  }
}

// 加载设备列表
async function loadDeviceList() {
  try {
    // TODO: 实际项目中这里应该从Tauri API调用后端
    // const devices = await invoke('list_adb_devices', { adbPath: adbPath.value });
    // 模拟数据
    deviceList.value = [
      { id: 'emulator-5554', name: '模拟器 - Pixel 4' },
      { id: 'R58M35EQWDJ', name: '华为 P40' }
    ];
  } catch (error) {
    ElMessage.error('加载设备列表失败：' + error);
  }
}

// 刷新窗口列表
async function refreshWindowList() {
  try {
    // TODO: 实际项目中这里应该从Tauri API调用后端
    // const windows = await invoke('list_windows');
    // 模拟数据
    windowList.value = [
      { title: 'AutoDaily', class: 'tauri-app' },
      { title: '计算器', class: 'calc' },
      { title: '记事本', class: 'notepad' }
    ];
  } catch (error) {
    ElMessage.error('刷新窗口列表失败：' + error);
  }
}

// 选择窗口
function selectWindow(row) {
  windowName.value = row.title;
}

// 选择ADB路径
async function selectAdbPath() {
  try {
    // TODO: 实际项目中这里应该从Tauri API调用文件选择器
    // const path = await invoke('select_file', { filter: '*.exe' });
    // if (path) adbPath.value = path;
    ElMessage.info('请选择ADB可执行文件路径');
  } catch (error) {
    ElMessage.error('选择文件失败：' + error);
  }
}

// 执行截图
async function capture() {
  try {
    // 验证输入
    if (captureMethod.value === 'adb' && !selectedDevice.value) {
      return ElMessage.error('请选择一个设备');
    }
    if (captureMethod.value === 'window' && !windowName.value) {
      return ElMessage.error('请输入或选择一个窗口名称');
    }
    
    ElMessage.info('开始截图，请稍候...');
    
    let result;
    if(captureMethod.value === 'adb'){
      result = await invoke('adb_capture_test', {
        method: captureMethod.value,
        device: selectedDevice.value,
        adbPath: adbPath.value,
        accelerator: adbAccelerator.value,
        winName: windowName.value
      });
    } else {
      result = await invoke('window_capture_test', {
        method: captureMethod.value,
        device: selectedDevice.value,
        winName: windowName.value
      });
    }
    
    // 处理截图结果
    if (result.startsWith('ok|')) {
      const base64Data = result.substring(3);
      if (base64Data === 'no_image') {
        ElMessage.warning('找到窗口但无法获取图像');
      } else {
        // 设置截图结果
        captureResult.value = `data:image/png;base64,${base64Data}`;
        lastCaptureTime.value = new Date();
        ElMessage.success('截图成功');
      }
    } else if (result === 'not found') {
      ElMessage.error('未找到指定窗口');
    } else {
      ElMessage.error(`截图失败: ${result}`);
    }
  } catch (error) {
    ElMessage.error(`截图失败: ${error}`);
  }
}

// 保存设置
async function saveSettings() {
  try {
    // TODO: 实际项目中这里应该从Tauri API调用后端保存设置
    // await invoke('save_capture_settings', {
    //   captureMethod: captureMethod.value,
    //   adbPath: adbPath.value,
    //   adbAccelerator: adbAccelerator.value,
    //   selectedDevice: selectedDevice.value,
    //   windowName: windowName.value
    // });
    
    ElMessage.success('设置已保存');
  } catch (error) {
    ElMessage.error('保存设置失败：' + error);
  }
}

// 保存截图
async function saveCapture() {
  try {
    if (!captureResult.value) {
      return ElMessage.warning('没有可保存的截图');
    }
    
    // 从data:image/png;base64,前缀中提取base64数据
    const base64Data = captureResult.value.split(',')[1];
    
    // 调用后端保存图像
    const savedPath = await invoke('save_captured_image', {
      imageData: base64Data,
      deviceName: captureMethod.value === 'window' ? windowName.value : selectedDevice.value,
      imageType: captureMethod.value
    });
    console.log('保存路径:', savedPath);
    console.log('保存路径json:', JSON.parse(savedPath));
    ElMessage.success(`截图已保存到: ${savedPath}`);
  } catch (error) {
    ElMessage.error(`保存截图失败: ${error}`);
  }
}

// 复制截图
async function copyCapture() {
  try {
    // TODO: 实际项目中这里应该从Tauri API调用后端复制截图
    // await invoke('copy_image_to_clipboard', {
    //   imageSrc: captureResult.value
    // });
    
    ElMessage.success('已复制截图到剪贴板');
  } catch (error) {
    ElMessage.error('复制截图失败：' + error);
  }
}

// 格式化日期
function formatDate(date) {
  return date.toLocaleTimeString();
}
</script>

<style lang="scss" scoped>
.capture-test {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  height: 100%;
  
  @media (max-width: 1200px) {
    grid-template-columns: 1fr;
  }
  
  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    
    h3 {
      margin: 0;
      font-size: 16px;
    }
  }
  
  .form-items {
    display: flex;
    flex-direction: column;
    gap: 16px;
    margin-bottom: 16px;
  }
  
  .form-item {
    display: flex;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;
    
    .label {
      min-width: 80px;
      color: var(--text-color-regular);
    }
    
    .el-select, .el-input, .path-input {
      flex: 1;
    }
    
    .path-input {
      display: flex;
      gap: 8px;
    }
    
    &.window-list {
      flex-direction: column;
      align-items: stretch;
    }
  }
  
  .button-row {
    display: flex;
    justify-content: center;
    gap: 16px;
    margin-top: 16px;
  }
  
  .capture-preview {
    height: 400px;
    background-color: var(--bg-color-mute);
    display: flex;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    
    img {
      max-width: 100%;
      max-height: 100%;
    }
  }
  
  .ml-2 {
    margin-left: 8px;
  }
}
</style> 