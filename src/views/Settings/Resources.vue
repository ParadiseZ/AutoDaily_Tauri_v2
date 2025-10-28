<template>
  <div class="resources-container">
    <el-card class="resource-card">
      <template #header>
        <div class="card-header">
          <h2>系统资源配置</h2>
          <span class="subtitle">优化任务执行性能</span>
        </div>
      </template>
      
      <el-alert
        v-if="showWarning"
        title="警告：当前设置的资源总量超出系统最大可用资源"
        type="warning"
        :closable="false"
        show-icon
      >
        单设备核心数 × 设备数量 = {{ form.coresPerDevice * form.maxDevices }}，
        超过系统最大核心数 {{ maxCores }}。
      </el-alert>

      <el-form 
        :model="form" 
        label-width="180px" 
        class="resource-form"
        :disabled="loading"
      >
        <el-form-item label="系统最大可用核心数">
          <el-input 
            v-model="maxCores" 
            disabled 
            class="max-cores-display"
          >
            <template #append>核心</template>
          </el-input>
          <div class="hint">当前系统检测到的物理CPU核心数量</div>
        </el-form-item>
        
        <el-form-item label="单设备CPU分配核心数">
          <el-input-number 
            v-model="form.coresPerDevice"
            :min="1"
            :max="maxCores"
            @change="validateResources"
          />
          <div class="hint">每个设备将分配的CPU核心数量</div>
        </el-form-item>
        
        <el-form-item label="同时运行最大设备数">
          <el-input-number 
            v-model="form.maxDevices"
            :min="1"
            :max="maxCores"
            @change="validateResources"
          />
          <div class="hint">系统支持同时运行的最大设备数量</div>
        </el-form-item>
        
        <el-divider />
        
        <el-form-item>
          <el-button 
            type="primary" 
            :disabled="showWarning || loading" 
            @click="saveConfig"
            :loading="loading"
          >
            保存配置
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { invoke } from '@tauri-apps/api/core'

const loading = ref(false)
const maxCores = ref(0)
const form = ref({
  coresPerDevice: 4,
  maxDevices: 1
})

const showWarning = computed(() => {
  return form.value.coresPerDevice * form.value.maxDevices > maxCores.value
})

const validateResources = () => {
  // 防止输入负数或超过最大值
  if (form.value.coresPerDevice < 1) form.value.coresPerDevice = 1
  if (form.value.maxDevices < 1) form.value.maxDevices = 1
  
  if (form.value.coresPerDevice > maxCores.value) {
    form.value.coresPerDevice = maxCores.value
  }
}

const getSystemInfo = async () => {
  loading.value = true
  try {
    // 获取系统CPU核心数
    maxCores.value = await invoke('get_cpu_cores')
    
    // 获取当前资源配置
    const config = await invoke('get_resource_config')
    if (config) {
      form.value.coresPerDevice = config.cores_per_device || 4
      form.value.maxDevices = config.max_devices || 1
    }
  } catch (error) {
    console.error('获取系统信息失败:', error)
    ElMessage.error('获取系统信息失败')
  } finally {
    loading.value = false
  }
}

const saveConfig = async () => {
  if (showWarning.value) {
    ElMessage.warning('资源配置超出系统限制，无法保存')
    return
  }
  
  loading.value = true
  try {
    await invoke('save_resource_config', {
      config: {
        cores_per_device: form.value.coresPerDevice,
        max_devices: form.value.maxDevices
      }
    })
    ElMessage.success('资源配置已保存')
  } catch (error) {
    console.error('保存配置失败:', error)
    ElMessage.error('保存配置失败')
  } finally {
    loading.value = false
  }
}

onMounted(() => {
  getSystemInfo()
})
</script>

<style scoped>
.resources-container {
  padding: 20px;
}

.resource-card {
  max-width: 800px;
  margin: 0 auto;
}

.card-header {
  display: flex;
  flex-direction: column;
}

.subtitle {
  font-size: 14px;
  color: #909399;
  margin-top: 5px;
}

.resource-form {
  margin-top: 20px;
}

.hint {
  font-size: 12px;
  color: #909399;
  margin-top: 5px;
}

.max-cores-display {
  width: 120px;
}

.el-input-number {
  width: 120px;
}

.el-divider {
  margin: 32px 0;
}
</style>

