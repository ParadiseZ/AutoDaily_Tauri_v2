<template>
  <div class="adb-test">
    <h2>ADB 设备测试</h2>
    
    <div class="device-list">
      <el-button @click="refreshDevices">刷新设备列表</el-button>
      <el-table :data="devices" style="width: 100%">
        <el-table-column prop="id" label="设备ID"></el-table-column>
        <el-table-column label="操作">
          <template #default="scope">
            <el-button @click="captureScreenshot(scope.row.id)">截图</el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <div v-if="screenshot" class="screenshot-preview">
      <img :src="screenshot" alt="设备截图" />
    </div>
  </div>
</template>

<script setup>
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/tauri';

const devices = ref([]);
const screenshot = ref(null);

const refreshDevices = async () => {
  try {
    devices.value = await invoke('get_adb_devices');
  } catch (error) {
    console.error('获取设备列表失败:', error);
  }
};

const captureScreenshot = async (deviceId) => {
  try {
    const imageData = await invoke('capture_adb_screenshot', { deviceId });
    screenshot.value = `data:image/png;base64,${imageData}`;
  } catch (error) {
    console.error('截图失败:', error);
  }
};
</script>

<style scoped>
.adb-test {
  padding: 20px;
}

.screenshot-preview img {
  max-width: 100%;
  border: 1px solid #ddd;
  margin-top: 20px;
}
</style>
