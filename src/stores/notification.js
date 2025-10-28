import { defineStore } from 'pinia';
import { invoke } from '@tauri-apps/api/core';

export const useNotificationStore = defineStore('notification', {
  state: () => ({
    // 桌面通知
    desktopNotice: true,
    notificationTypes: ['script_complete', 'script_error'],
    displayDuration: 5000,
    
    // 邮件通知
    emailNotification: false,
    smtpServer: '',
    smtpPort: 465,
    username: '',
    password: '',
    recipient: '',
    emailTriggers: ['script_error', 'system_error'],
    
    // 高级设置
    enableQuietHours: false,
    quietStartTime: null,
    quietEndTime: null,
    quietHoursBehavior: 'disable',
    enableSound: true,
    saveHistory: true
  }),
  
  actions: {
    async loadConfig() {
      try {
        // 等后端实现后替换此处
        // const configJson = await invoke('get_notification_config_cmd');
        // const config = JSON.parse(configJson);
        // Object.assign(this.$state, config);
        console.log('加载通知配置...');
      } catch (error) {
        console.error('Failed to load notification config:', error);
      }
    },
    
    async saveConfig() {
      try {
        // 等后端实现后替换此处
        // await invoke('set_notification_config_cmd', { config: this.$state });
        console.log('保存通知配置:', this.$state);
      } catch (error) {
        console.error('Failed to save notification config:', error);
      }
    },
    
    async testEmailConfig() {
      try {
        // 等后端实现后替换此处
        // await invoke('test_email_send_cmd', { config: this.$state });
        console.log('测试邮件发送:', this.$state);
        return true;
      } catch (error) {
        console.error('Email test failed:', error);
        throw error;
      }
    },
    
    async clearNotificationHistory() {
      try {
        // 等后端实现后替换此处
        // await invoke('clear_notification_history_cmd');
        console.log('清空通知历史');
      } catch (error) {
        console.error('Failed to clear notification history:', error);
        throw error;
      }
    }
  }
});
