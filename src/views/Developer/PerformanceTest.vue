<template>
  <div class="performance-test">
    <el-card class="system-info-card">
      <template #header>
        <div class="card-header">
          <span>系统性能信息</span>
          <el-button type="primary" @click="refreshSystemInfo">刷新</el-button>
        </div>
      </template>
      
      <div class="system-info" v-if="systemInfo">
        <el-row :gutter="20">
          <el-col :span="8">
            <el-statistic title="CPU核心数" :value="systemInfo.cpuCount" />
          </el-col>
          <el-col :span="8">
            <el-statistic title="活跃进程数" :value="systemInfo.activeProcessCount" />
          </el-col>
          <el-col :span="8">
            <div>
              <div class="statistic-title">可用核心ID</div>
              <div class="statistic-content">{{ systemInfo.availableCoreIds.join(', ') }}</div>
            </div>
          </el-col>
        </el-row>
      </div>
    </el-card>

    <el-card class="process-management-card">
      <template #header>
        <div class="card-header">
          <span>进程管理</span>
          <div>
            <el-button type="success" @click="refreshActiveProcesses">刷新进程列表</el-button>
            <el-button type="warning" @click="cleanupFinishedProcesses">清理已完成进程</el-button>
          </div>
        </div>
      </template>

      <div class="process-controls">
        <el-row :gutter="20">
          <el-col :span="12">
            <h4>创建测试进程</h4>
            <el-form :model="processForm" label-width="120px">
              <el-form-item label="进程名称">
                <el-input v-model="processForm.name" placeholder="输入进程名称" />
              </el-form-item>
              
              <el-form-item label="程序路径">
                <el-input v-model="processForm.program" placeholder="输入程序路径" />
              </el-form-item>
              
              <el-form-item label="命令参数">
                <el-input v-model="processForm.argsText" placeholder="输入命令参数（空格分隔）" />
              </el-form-item>
              
              <el-form-item label="CPU核心">
                <el-select v-model="processForm.coreId" placeholder="选择CPU核心（可选）" clearable>
                  <el-option 
                    v-for="coreId in availableCores" 
                    :key="coreId" 
                    :label="`核心 ${coreId}`" 
                    :value="coreId"
                  />
                </el-select>
              </el-form-item>
              
              <el-form-item label="工作目录">
                <el-input v-model="processForm.workingDir" placeholder="输入工作目录（可选）" />
              </el-form-item>
              
              <el-form-item>
                <el-button type="primary" @click="startTestProcess" :loading="isStarting">
                  启动进程
                </el-button>
              </el-form-item>
            </el-form>
          </el-col>

          <el-col :span="12">
            <h4>快速操作</h4>
            <div class="quick-actions">
              <el-button type="info" @click="startSimpleTestProcess">
                启动简单测试进程
              </el-button>
              
              <el-button type="warning" @click="startCpuIntensiveProcess">
                启动CPU密集型进程
              </el-button>
              
              <el-button type="success" @click="startParallelProcesses">
                启动并行进程
              </el-button>
              
              <el-input-number 
                v-model="parallelProcessCount" 
                :min="1" 
                :max="systemInfo?.cpuCount || 4"
                placeholder="并行进程数"
                style="margin-left: 10px; width: 120px;"
              />
            </div>
          </el-col>
        </el-row>
      </div>
    </el-card>

    <el-card class="active-processes-card">
      <template #header>
        <span>活跃进程列表</span>
      </template>
      
      <el-table :data="activeProcesses" stripe>
        <el-table-column prop="name" label="进程名称" />
        <el-table-column prop="pid" label="PID" />
        <el-table-column prop="command" label="命令" show-overflow-tooltip />
        <el-table-column label="CPU核心">
          <template #default="{ row }">
            <el-tag v-if="row.core_id !== null" type="success">
              核心 {{ row.core_id }}
            </el-tag>
            <el-tag v-else type="info">未绑定</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="状态">
          <template #default="{ row }">
            <el-tag v-if="row.is_running" type="success">运行中</el-tag>
            <el-tag v-else type="danger">已结束</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作">
          <template #default="{ row }">
            <el-button 
              type="danger" 
              size="small" 
              @click="terminateProcess(row.id)"
              :loading="terminatingProcesses.includes(row.id)"
            >
              终止
            </el-button>
            <el-button 
              type="info" 
              size="small" 
              @click="getProcessOutput(row.id)"
            >
              查看输出
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-card class="log-card">
      <template #header>
        <div class="card-header">
          <span>操作日志</span>
          <el-button type="danger" size="small" @click="clearLogs">清空日志</el-button>
        </div>
      </template>
      
      <div class="log-container">
        <div 
          v-for="(log, index) in logs" 
          :key="index" 
          :class="['log-entry', `log-${log.level}`]"
        >
          <span class="log-time">{{ formatTime(log.time) }}</span>
          <span class="log-level">[{{ log.level.toUpperCase() }}]</span>
          <span class="log-message">{{ log.message }}</span>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup>
import { ref, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'

// 响应式数据
const systemInfo = ref(null)
const activeProcesses = ref([])
const terminatingProcesses = ref([])
const isStarting = ref(false)
const parallelProcessCount = ref(2)

// 进程表单
const processForm = ref({
  name: 'test_process',
  program: 'echo',
  argsText: 'Hello from subprocess!',
  coreId: null,
  workingDir: ''
})

// 日志
const logs = ref([])

// 计算属性
const availableCores = computed(() => {
  return systemInfo.value?.availableCoreIds || []
})

// 添加日志
const addLog = (level, message) => {
  logs.value.unshift({
    level,
    message,
    time: new Date()
  })
  
  // 限制日志数量
  if (logs.value.length > 100) {
    logs.value = logs.value.slice(0, 100)
  }
}

// 格式化时间
const formatTime = (time) => {
  return time.toLocaleTimeString()
}

// 清空日志
const clearLogs = () => {
  logs.value = []
  addLog('info', '日志已清空')
}

// 获取系统性能信息
const refreshSystemInfo = async () => {
  try {
    const info = await invoke('get_system_performance_info')
    systemInfo.value = JSON.parse(info)
    addLog('info', `系统信息已刷新：CPU核心数 ${systemInfo.value.cpuCount}`)
  } catch (error) {
    addLog('error', `获取系统信息失败: ${error}`)
    ElMessage.error('获取系统信息失败')
  }
}

// 启动测试进程
const startTestProcess = async () => {
  if (!processForm.value.name.trim()) {
    ElMessage.warning('请输入进程名称')
    return
  }
  
  if (!processForm.value.program.trim()) {
    ElMessage.warning('请输入程序路径')
    return
  }
  
  isStarting.value = true
  
  try {
    const args = processForm.value.argsText.trim() ? 
      processForm.value.argsText.split(' ').filter(arg => arg.length > 0) : []
    
    const processId = await invoke('start_test_process', {
      processName: processForm.value.name,
      program: processForm.value.program,
      args: args,
      coreId: processForm.value.coreId,
      workingDir: processForm.value.workingDir || null
    })
    
    addLog('success', `进程已启动: ${processForm.value.name} (ID: ${processId})`)
    ElMessage.success('进程启动成功')
    
    // 刷新活跃进程列表
    refreshActiveProcesses()
    
  } catch (error) {
    addLog('error', `启动进程失败: ${error}`)
    ElMessage.error(`启动进程失败: ${error}`)
  } finally {
    isStarting.value = false
  }
}

// 终止进程
const terminateProcess = async (processId) => {
  try {
    await ElMessageBox.confirm('确定要终止这个进程吗？', '警告', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    })
    
    terminatingProcesses.value.push(processId)
    
    await invoke('terminate_process', { processId })
    addLog('success', `进程已终止: ${processId}`)
    ElMessage.success('进程终止成功')
    
    // 刷新进程列表和系统信息
    refreshActiveProcesses()
    refreshSystemInfo()
    
  } catch (error) {
    if (error !== 'cancel') {
      addLog('error', `终止进程失败: ${error}`)
      ElMessage.error(`终止进程失败: ${error}`)
    }
  } finally {
    terminatingProcesses.value = terminatingProcesses.value.filter(id => id !== processId)
  }
}

// 获取活跃进程信息
const refreshActiveProcesses = async () => {
  try {
    const processesInfo = await invoke('get_active_processes_info')
    activeProcesses.value = JSON.parse(processesInfo)
    addLog('info', `活跃进程列表已刷新：${activeProcesses.value.length} 个进程`)
  } catch (error) {
    addLog('error', `获取进程信息失败: ${error}`)
    ElMessage.error('获取进程信息失败')
  }
}

// 清理已完成的进程
const cleanupFinishedProcesses = async () => {
  try {
    await invoke('cleanup_finished_processes')
    addLog('success', '已清理完成的进程')
    ElMessage.success('已清理完成的进程')
    
    // 刷新进程列表和系统信息
    refreshActiveProcesses()
    refreshSystemInfo()
    
  } catch (error) {
    addLog('error', `清理进程失败: ${error}`)
    ElMessage.error('清理进程失败')
  }
}

// 获取进程输出
const getProcessOutput = async (processId) => {
  try {
    const [stdout, stderr] = await invoke('get_process_output', { processId })
    
    let message = `进程 ${processId} 输出：\n`
    if (stdout) {
      message += `标准输出：${stdout}\n`
    }
    if (stderr) {
      message += `错误输出：${stderr}\n`
    }
    if (!stdout && !stderr) {
      message += '无输出信息'
    }
    
    await ElMessageBox.alert(message, '进程输出', {
      confirmButtonText: '确定'
    })
    
  } catch (error) {
    addLog('error', `获取进程输出失败: ${error}`)
    ElMessage.error(`获取进程输出失败: ${error}`)
  }
}

// 启动简单测试进程
const startSimpleTestProcess = async () => {
  try {
    const processId = await invoke('start_simple_test_process', { coreId: null })
    addLog('success', `简单测试进程已启动 (ID: ${processId})`)
    ElMessage.success('简单测试进程启动成功')
    refreshActiveProcesses()
  } catch (error) {
    addLog('error', `启动简单测试进程失败: ${error}`)
    ElMessage.error(`启动简单测试进程失败: ${error}`)
  }
}

// 启动CPU密集型进程
const startCpuIntensiveProcess = async () => {
  try {
    const processId = await invoke('start_cpu_intensive_process', {
      coreId: null,
      durationSeconds: 5
    })
    addLog('success', `CPU密集型进程已启动 (ID: ${processId})`)
    ElMessage.success('CPU密集型进程启动成功')
    refreshActiveProcesses()
  } catch (error) {
    addLog('error', `启动CPU密集型进程失败: ${error}`)
    ElMessage.error(`启动CPU密集型进程失败: ${error}`)
  }
}

// 启动并行进程
const startParallelProcesses = async () => {
  try {
    const processIds = await invoke('start_parallel_processes', {
      processCount: parallelProcessCount.value,
      taskDurationSeconds: 3
    })
    
    addLog('success', `${processIds.length} 个并行进程已启动`)
    ElMessage.success(`${processIds.length} 个并行进程启动成功`)
    refreshActiveProcesses()
    
  } catch (error) {
    addLog('error', `启动并行进程失败: ${error}`)
    ElMessage.error(`启动并行进程失败: ${error}`)
  }
}

// 组件挂载时初始化
onMounted(() => {
  addLog('info', '进程管理测试页面已加载')
  refreshSystemInfo()
  refreshActiveProcesses()
})
</script>

<style scoped>
.performance-test {
  padding: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.system-info-card,
.process-management-card,
.active-processes-card,
.log-card {
  margin-bottom: 20px;
}

.system-info {
  margin-top: 20px;
}

.statistic-title {
  font-size: 14px;
  color: #666;
  margin-bottom: 8px;
}

.statistic-content {
  font-size: 20px;
  font-weight: bold;
  color: #303133;
}

.process-controls {
  margin-top: 20px;
}

.quick-actions {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.quick-actions .el-button {
  width: 100%;
}

.log-container {
  height: 300px;
  overflow-y: auto;
  background-color: #f5f5f5;
  padding: 10px;
  border-radius: 4px;
}

.log-entry {
  display: flex;
  margin-bottom: 5px;
  font-family: monospace;
  font-size: 12px;
}

.log-time {
  color: #909399;
  margin-right: 10px;
  min-width: 80px;
}

.log-level {
  margin-right: 10px;
  min-width: 60px;
  font-weight: bold;
}

.log-message {
  flex: 1;
}

.log-info .log-level {
  color: #409eff;
}

.log-success .log-level {
  color: #67c23a;
}

.log-error .log-level {
  color: #f56c6c;
}

.log-warning .log-level {
  color: #e6a23c;
}
</style>