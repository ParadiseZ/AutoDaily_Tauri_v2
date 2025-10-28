<template>
  <div class="notification-settings">
    
    <el-card shadow="hover" class="setting-card">
      <template #header>
        <h3>桌面通知</h3>
      </template>
      <el-form label-position="top" :model="notificationStore" label-width="150px">
        <el-form-item label="启用桌面通知">
          <el-checkbox v-model="notificationStore.desktopNotice">接收系统桌面通知</el-checkbox>
        </el-form-item>
        
        <el-form-item label="通知类型" v-if="notificationStore.desktopNotice">
          <el-checkbox-group v-model="notificationStore.notificationTypes">
            <el-checkbox value="script_complete">脚本执行完成</el-checkbox>
            <el-checkbox value="script_error">脚本执行错误</el-checkbox>
            <el-checkbox value="system_idle">系统空闲提醒</el-checkbox>
            <el-checkbox value="auto_shutdown">自动关机提醒</el-checkbox>
          </el-checkbox-group>
        </el-form-item>
        
        <el-form-item label="通知显示时长" v-if="notificationStore.desktopNotice">
          <el-select v-model="notificationStore.displayDuration" placeholder="选择显示时长">
            <el-option label="3秒" :value="3000" />
            <el-option label="5秒" :value="5000" />
            <el-option label="10秒" :value="10000" />
            <el-option label="直到手动关闭" :value="0" />
          </el-select>
        </el-form-item>
      </el-form>
    </el-card>
    
    <el-card shadow="hover" class="setting-card">
      <template #header>
        <h3>邮件通知</h3>
      </template>
      <el-form label-position="top" :model="notificationStore" label-width="150px">
        <el-form-item label="启用邮件通知">
          <el-checkbox v-model="notificationStore.emailNotification">发送邮件通知</el-checkbox>
        </el-form-item>
        
        <div v-if="notificationStore.emailNotification" class="email-config">
          <el-divider content-position="left">SMTP 服务器配置</el-divider>
          
          <el-row :gutter="20">
            <el-col :span="18">
              <el-form-item label="SMTP 服务器地址">
                <el-input v-model="notificationStore.smtpServer" placeholder="例如：smtp.gmail.com" />
              </el-form-item>
            </el-col>
            <el-col :span="6">
              <el-form-item label="端口">
                <el-input-number v-model="notificationStore.smtpPort" :min="1" :max="65535" />
              </el-form-item>
            </el-col>
          </el-row>
          
          <el-row :gutter="20">
            <el-col :span="12">
              <el-form-item label="用户名/邮箱">
                <el-input v-model="notificationStore.username" placeholder="发送方邮箱地址" />
              </el-form-item>
            </el-col>
            <el-col :span="12">
              <el-form-item label="密码/授权码">
                <el-input v-model="notificationStore.password" type="password" placeholder="邮箱密码或授权码" show-password />
              </el-form-item>
            </el-col>
          </el-row>
          
          <el-form-item label="收件人邮箱">
            <el-input v-model="notificationStore.recipient" placeholder="接收通知的邮箱地址" />
          </el-form-item>
          
          <el-divider content-position="left">邮件通知设置</el-divider>
          
          <el-form-item label="邮件通知触发条件">
            <el-checkbox-group v-model="notificationStore.emailTriggers">
              <el-checkbox value="script_error">脚本执行失败</el-checkbox>
              <el-checkbox value="system_error">系统错误</el-checkbox>
              <el-checkbox value="daily_summary">每日运行总结</el-checkbox>
              <el-checkbox value="auto_shutdown">自动关机前提醒</el-checkbox>
            </el-checkbox-group>
          </el-form-item>
          
          <el-form-item>
            <el-space>
              <el-button type="primary" @click="testEmailConfig" :loading="testing">测试邮件发送</el-button>
              <el-text type="info" size="small">发送测试邮件到收件人邮箱</el-text>
            </el-space>
          </el-form-item>
        </div>
      </el-form>
    </el-card>
    
    <el-card shadow="hover" class="setting-card">
      <template #header>
        <h3>高级通知设置</h3>
      </template>
      <el-form label-position="top" :model="notificationStore" label-width="150px">
        <el-form-item label="免打扰时间">
          <el-checkbox v-model="notificationStore.enableQuietHours">启用免打扰时间</el-checkbox>
        </el-form-item>
        
        <div v-if="notificationStore.enableQuietHours" class="quiet-hours-config">
          <el-form-item label="免打扰时间段">
            <el-time-picker
              v-model="notificationStore.quietStartTime"
              format="HH:mm"
              placeholder="开始时间"
              style="width: 120px; margin-right: 10px;"
            />
            <span>至</span>
            <el-time-picker
              v-model="notificationStore.quietEndTime"
              format="HH:mm"
              placeholder="结束时间"
              style="width: 120px; margin-left: 10px;"
            />
          </el-form-item>
          
          <el-form-item label="免打扰期间行为">
            <el-radio-group v-model="notificationStore.quietHoursBehavior">
              <el-radio value="disable">完全禁用通知</el-radio>
              <el-radio value="email_only">仅发送邮件通知</el-radio>
              <el-radio value="critical_only">仅紧急通知</el-radio>
            </el-radio-group>
          </el-form-item>
        </div>
        
        <el-form-item label="通知音效">
          <el-checkbox v-model="notificationStore.enableSound">播放通知音效</el-checkbox>
        </el-form-item>
        
        <el-form-item label="通知历史">
          <el-space>
            <el-checkbox v-model="notificationStore.saveHistory">保存通知历史记录</el-checkbox>
            <el-button size="small" @click="clearNotificationHistory">清空历史记录</el-button>
          </el-space>
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
import { ref, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { useNotificationStore } from '../../stores/notification';

// 使用通知store
const notificationStore = useNotificationStore();

// 测试状态
const testing = ref(false);

onMounted(async () => {
  await notificationStore.loadConfig();
});

// 保存配置
async function saveConfig() {
  try {
    await notificationStore.saveConfig();
    ElMessage.success('通知设置已保存');
  } catch (error) {
    console.error('Failed to save notification config:', error);
    ElMessage.error('保存通知设置失败');
  }
}

// 重置配置
function resetConfig() {
  ElMessageBox.confirm('确认要重置所有通知设置吗？', '提示', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(() => {
    // 重置为默认值
    notificationStore.$reset();
    ElMessage.success('通知设置已重置为默认值');
  });
}

// 测试邮件配置
async function testEmailConfig() {
  if (!notificationStore.smtpServer || !notificationStore.username || !notificationStore.recipient) {
    ElMessage.warning('请先完善邮件服务器配置');
    return;
  }
  
  testing.value = true;
  try {
    await notificationStore.testEmailConfig();
    ElMessage.success('测试邮件发送成功！请检查收件箱');
  } catch (error) {
    console.error('Email test failed:', error);
    ElMessage.error('测试邮件发送失败');
  } finally {
    testing.value = false;
  }
}

// 清空通知历史
function clearNotificationHistory() {
  ElMessageBox.confirm('确认要清空所有通知历史记录吗？', '提示', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(async () => {
    try {
      await notificationStore.clearNotificationHistory();
      ElMessage.success('通知历史记录已清空');
    } catch (error) {
      console.error('Failed to clear notification history:', error);
      ElMessage.error('清空通知历史失败');
    }
  });
}
</script>

<style lang="scss" scoped>
.notification-settings {
  max-width: 800px;
  margin: 0 auto;
  
  .setting-card {
    margin-bottom: 20px;
    
    h3 {
      margin: 0;
      font-size: 16px;
    }
  }
  
  .email-config {
    padding: 10px 0;
    background-color: var(--el-color-primary-light-9);
    border-radius: 8px;
    padding: 20px;
    margin-top: 15px;
  }
  
  .quiet-hours-config {
    padding: 15px;
    background-color: var(--el-color-info-light-9);
    border-radius: 8px;
    margin-top: 10px;
  }
  
  .action-buttons {
    display: flex;
    justify-content: center;
    gap: 12px;
    margin-top: 20px;
    padding: 20px 0;
  }
  
  :deep(.el-checkbox-group) {
    display: flex;
    flex-direction: column;
    gap: 8px;
    
    .el-checkbox {
      margin-right: 0;
    }
  }
  
  :deep(.el-form-item__label) {
    font-weight: 500;
    color: var(--text-color-primary);
  }
}
</style>
