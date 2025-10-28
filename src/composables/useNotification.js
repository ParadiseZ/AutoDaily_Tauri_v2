import { ref, reactive } from 'vue'
import { 
  isPermissionGranted, 
  requestPermission, 
  sendNotification 
} from '@tauri-apps/plugin-notification'
import { ElMessage } from 'element-plus'

const notificationPermission = ref(false)
const notificationQueue = reactive([])

// 检查和请求通知权限
export async function checkNotificationPermission() {
  try {
    if (await isPermissionGranted()) {
      notificationPermission.value = true
      return true
    }
    
    const permission = await requestPermission()
    notificationPermission.value = permission === 'granted'
    
    if (!notificationPermission.value) {
      ElMessage.warning('需要通知权限才能接收系统提醒')
    }
    
    return notificationPermission.value
  } catch (error) {
    console.error('检查通知权限失败:', error)
    return false
  }
}

// 发送系统通知
export async function showNotification(title, body, options = {}) {
  try {
    // 确保有权限
    if (!notificationPermission.value) {
      const hasPermission = await checkNotificationPermission()
      if (!hasPermission) {
        console.warn('没有通知权限，无法发送通知')
        return false
      }
    }
    
    await sendNotification({
      title,
      body,
      icon: options.icon || 'AutoDaily',
      ...options
    })
    
    return true
  } catch (error) {
    console.error('发送通知失败:', error)
    ElMessage.error('发送通知失败')
    return false
  }
}

// 发送关机倒计时通知
export async function showShutdownNotification(countdown = 300) {
  const minutes = Math.floor(countdown / 60)
  const seconds = countdown % 60
  
  const body = `系统将在 ${minutes}:${seconds.toString().padStart(2, '0')} 后自动关机`
  
  return await showNotification(
    'AutoDaily - 自动关机提醒',
    body,
    {
      icon: 'warning',
      tag: 'shutdown-warning' // 用于替换之前的通知
    }
  )
}

// 发送脚本完成通知
export async function showScriptCompleteNotification(scriptName, success = true) {
  const title = success ? '脚本执行完成' : '脚本执行失败'
  const body = `脚本 "${scriptName}" ${success ? '已成功完成' : '执行失败'}`
  
  return await showNotification(
    `AutoDaily - ${title}`,
    body,
    {
      icon: success ? 'success' : 'error',
      tag: `script-${scriptName}`
    }
  )
}

// 发送通用系统通知
export async function showSystemNotification(title, message, type = 'info') {
  return await showNotification(
    `AutoDaily - ${title}`,
    message,
    {
      icon: type,
      tag: 'system-notification'
    }
  )
}

// 导出状态
export function useNotification() {
  return {
    notificationPermission,
    notificationQueue,
    checkNotificationPermission,
    showNotification,
    showShutdownNotification,
    showScriptCompleteNotification,
    showSystemNotification
  }
}
