<template>
  <div class="dashboard">
    <h1>控制面板</h1>
    <el-row :gutter="20">
      <el-col :span="8">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <h3>系统状态</h3>
            </div>
          </template>
          <div class="system-status">
            <div class="status-item">
              <div class="label">CPU 使用率</div>
              <el-progress :percentage="45" :color="progressColor" />
            </div>
            <div class="status-item">
              <div class="label">内存使用率</div>
              <el-progress :percentage="60" :color="progressColor" />
            </div>
            <div class="status-item">
              <div class="label">磁盘使用率</div>
              <el-progress :percentage="30" :color="progressColor" />
            </div>
          </div>
        </el-card>
      </el-col>
      
      <el-col :span="8">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <h3>运行中的任务</h3>
            </div>
          </template>
          <el-empty v-if="true" description="暂无运行任务" />
        </el-card>
      </el-col>
      
      <el-col :span="8">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <h3>设备状态</h3>
            </div>
          </template>
          <el-empty description="未配置设备" />
        </el-card>
      </el-col>
    </el-row>
    
    <el-row :gutter="20" class="mt-4">
      <el-col :span="24">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <h3>最近活动</h3>
            </div>
          </template>
          <div class="recent-logs">
            <el-empty description="暂无日志记录" />
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup>
import { computed } from 'vue';

// 生成渐变色进度条颜色
const progressColor = computed(() => {
  return (percentage) => {
    if (percentage < 30) {
      return '#67C23A';
    } else if (percentage < 70) {
      return '#E6A23C';
    } else {
      return '#F56C6C';
    }
  };
});
</script>

<style lang="scss" scoped>
.dashboard {
  padding: 20px 0;
  
  h1 {
    margin-top: 0;
    margin-bottom: 20px;
    font-size: 24px;
    background: var(--primary-gradient);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
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
  
  .system-status {
    .status-item {
      margin-bottom: 16px;
      
      &:last-child {
        margin-bottom: 0;
      }
      
      .label {
        margin-bottom: 8px;
        font-size: 14px;
        color: var(--text-color-regular);
      }
    }
  }
  
  .recent-logs {
    min-height: 200px;
  }
  
  .mt-4 {
    margin-top: 20px;
  }
}
</style> 