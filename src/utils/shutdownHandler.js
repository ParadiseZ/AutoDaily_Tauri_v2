import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useNotification } from '../composables/useNotification'

const { showShutdownNotification } = useNotification()

// 关机倒计时状态
const shutdownCountdown = ref(0)
const isShutdownScheduled = ref(false)
const shutdownTimer = ref(null)

// 初始化关机事件监听
export async function initShutdownListener() {
  // 监听来自后端的关机事件
  await listen('shutdown_scheduled', (event) => {
    const { countdown, message } = event.payload
    handleShutdownScheduled(countdown, message)
  })
}

// 处理关机计划事件
function handleShutdownScheduled(countdown, message) {
  shutdownCountdown.value = countdown
  isShutdownScheduled.value = true
  
  // 发送系统通知
  showShutdownNotification(countdown)
  
  // 显示确认对话框
  showShutdownConfirmDialog(countdown, message)
  
  // 开始倒计时
  startCountdown()
}

// 显示关机确认对话框
function showShutdownConfirmDialog(countdown, message) {
  const minutes = Math.floor(countdown / 60)
  
  ElMessageBox.confirm(
    `${message}\n\n系统将在 ${minutes} 分钟后自动关机。您可以选择取消此操作。`,
    '自动关机提醒',
    {
      confirmButtonText: '取消关机',
      cancelButtonText: '允许关机',
      type: 'warning',
      distinguishCancelAndClose: true,
      beforeClose: async (action, instance, done) => {
        if (action === 'confirm') {
          // 用户选择取消关机
          try {
            await invoke('cancel_shutdown_cmd')
            ElMessage.success('已取消自动关机')
            stopCountdown()
            done()
          } catch (error) {
            console.error('取消关机失败:', error)
            ElMessage.error('取消关机失败')
            done()
          }
        } else {
          // 用户选择允许关机或关闭对话框
          done()
        }
      }
    }
  ).catch(() => {
    // 用户关闭对话框，不做特殊处理
  })
}

// 开始倒计时
function startCountdown() {
  if (shutdownTimer.value) {
    clearInterval(shutdownTimer.value)
  }
  
  shutdownTimer.value = setInterval(() => {
    if (shutdownCountdown.value > 0) {
      shutdownCountdown.value--
      
      // 每分钟更新一次通知
      if (shutdownCountdown.value % 60 === 0 && shutdownCountdown.value > 0) {
        showShutdownNotification(shutdownCountdown.value)
      }
    } else {
      // 倒计时结束
      stopCountdown()
      ElMessage.warning('系统正在关机...')
    }
  }, 1000)
}

// 停止倒计时
function stopCountdown() {
  if (shutdownTimer.value) {
    clearInterval(shutdownTimer.value)
    shutdownTimer.value = null
  }
  isShutdownScheduled.value = false
  shutdownCountdown.value = 0
}

// 手动取消关机
export async function cancelShutdown() {
  try {
    await invoke('cancel_shutdown_cmd')
    stopCountdown()
    ElMessage.success('已取消自动关机')
    return true
  } catch (error) {
    console.error('取消关机失败:', error)
    ElMessage.error('取消关机失败')
    return false
  }
}

// 格式化倒计时显示
export function formatCountdown(seconds) {
  const minutes = Math.floor(seconds / 60)
  const remainingSeconds = seconds % 60
  return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`
}

// 导出状态和方法
export function useShutdownHandler() {
  return {
    shutdownCountdown,
    isShutdownScheduled,
    initShutdownListener,
    cancelShutdown,
    formatCountdown
  }
}
