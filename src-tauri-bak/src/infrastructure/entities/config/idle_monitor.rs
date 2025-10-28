use std::sync::Arc;
use std::time::Duration;
use tauri::{Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{sleep, Instant};
use crate::constant::project::MAIN_WINDOW;
use crate::constant::sys_conf_path::SYSTEM_SETTINGS_PATH;
use crate::domain::app_handle::get_app_handle;
use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::config::log_conf::{Log, Logger};
use crate::domain::entities::config::sys_conf::{IdleAction, SystemConfig};
use crate::domain::manager::conf_mgr::{ConfMgr, ConfigManager};
use crate::domain::services::idle_monitor::IdleMonitorTrait;

pub struct IdleMonitor {
    last_activity: Arc<Mutex<Instant>>,
    monitoring: Arc<Mutex<bool>>,
    shutdown_timer: Arc<Mutex<Option<JoinHandle<()>>>>,
    retry_num : Arc<Mutex<u8>>
}

impl IdleMonitor {
    pub(crate) fn new() -> Self{
        Self{
            last_activity : Arc::new(Mutex::new(Instant::now())),
            monitoring: Arc::new(Mutex::new(false)),
            shutdown_timer: Arc::new(Mutex::new(None)),
            retry_num: Arc::new(Mutex::new(3))
        }
    }
}

impl IdleMonitorTrait for IdleMonitor{
    /// 开始监控空闲状态
    async fn start_monitoring(&self, config_manager: ConfigManager) -> AppResult<()> {
        let mut monitoring = self.monitoring.lock().await;
        if *monitoring {
            return Ok(());
        }
        *monitoring = true;
        drop(monitoring);

        let last_activity = self.last_activity.clone();
        let monitoring_flag = self.monitoring.clone();
        let shutdown_timer = self.shutdown_timer.clone();
        let retry_num = self.retry_num.clone();

        tokio::spawn(async move {
            while *monitoring_flag.lock().await {
                sleep(Duration::from_secs(60)).await; // 每分钟检查一次

                let settings = match config_manager.get_conf::<SystemConfig>(SYSTEM_SETTINGS_PATH).await {
                    Ok(settings) => settings,
                    Err(e) => {
                        Log::error(&format!("获取系统设置失败: {}，将停止监控空闲状态", e));
                        // 停止监控：设置flag为false并取消关机定时器
                        let mut monitoring = monitoring_flag.lock().await;
                        *monitoring = false;

                        // 取消任何进行中的关机定时器
                        let mut timer = shutdown_timer.lock().await;
                        if let Some(handle) = timer.take() {
                            handle.abort();
                            Log::info("已取消进行中的关机定时器");
                        }
                        break;
                    }
                };

                // 检查是否有脚本在运行（这里需要实现脚本状态检查）
                let has_running_scripts = Self::check_running_scripts().await;

                if !has_running_scripts && !matches!(settings.idle_action, IdleAction::None) {
                    let last_time = *last_activity.lock().await;
                    let idle_duration = last_time.elapsed();

                    // 如果空闲超过5分钟，执行相应操作
                    if idle_duration > Duration::from_secs(300) {
                        let should_continue = Self::handle_idle_action(&settings.idle_action, &shutdown_timer, &retry_num, settings.max_idle_retry_num).await;

                        if should_continue {
                            // 重置活动时间，避免重复触发
                            *last_activity.lock().await = Instant::now();
                        } else {
                            // 重试次数达到上限，停止监控
                            let mut monitoring = monitoring_flag.lock().await;
                            *monitoring = false;

                            // 取消任何进行中的关机定时器
                            let mut timer = shutdown_timer.lock().await;
                            if let Some(handle) = timer.take() {
                                handle.abort();
                                Log::info("已取消进行中的关机定时器（重试次数达到上限）");
                            }
                            break;
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// 停止监控
    async fn stop_monitoring(&self) {
        let mut monitoring = self.monitoring.lock().await;
        *monitoring = false;

        // 取消任何进行中的关机定时器
        let mut timer = self.shutdown_timer.lock().await;
        if let Some(handle) = timer.take() {
            handle.abort();
        }
    }

    /// 处理空闲时的操作,返回是否应该继续监控（false表示重试次数达到上限，应停止监控）
    async fn handle_idle_action(
        idle_action: &IdleAction,
        shutdown_timer: &Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
        retry_num: &Arc<Mutex<u8>>,
        max_retry_num: u8
    ) -> bool {
        match idle_action {
            IdleAction::None => {
                // 重置重试次数
                let mut retry_count = retry_num.lock().await;
                *retry_count = 0;
                true
            },
            IdleAction::Shutdown => {
                // 关机操作通常不会失败，且有自己的取消机制
                Self::schedule_shutdown(shutdown_timer).await;
                // 重置重试次数
                let mut retry_count = retry_num.lock().await;
                *retry_count = 0;
                true
            },
            IdleAction::Sleep => {
                match Self::sleep_system() {
                    Ok(_) => {
                        Log::info("系统已进入睡眠状态");
                        // 成功时重置重试次数
                        let mut retry_count = retry_num.lock().await;
                        *retry_count = 0;
                        true
                    },
                    Err(e) => {
                        Self::handle_action_failure("睡眠", &e, retry_num, max_retry_num).await
                    }
                }
            },
            IdleAction::Hibernate => {
                match Self::hibernate_system() {
                    Ok(_) => {
                        Log::info("系统已进入休眠状态");
                        // 成功时重置重试次数
                        let mut retry_count = retry_num.lock().await;
                        *retry_count = 0;
                        true
                    },
                    Err(e) => {
                        Self::handle_action_failure("休眠", &e, retry_num, max_retry_num).await
                    }
                }
            },
        }
    }

    /// 处理操作失败，检查重试次数
    async fn handle_action_failure(
        action_name: &str,
        error: &AppError,
        retry_num: &Arc<Mutex<u8>>,
        max_retry_num: u8
    ) -> bool {
        let mut retry_count = retry_num.lock().await;
        *retry_count += 1;

        if max_retry_num == 0 {
            // 不允许重试
            Log::error(&format!("系统{}失败，重试已禁用: {}", action_name, error));
            false
        } else if *retry_count >= max_retry_num {
            // 达到最大重试次数
            Log::error(&format!("系统{}失败次数达到最大值({})，停止监控: {}", action_name, max_retry_num, error));
            false
        } else {
            // 还可以重试
            Log::warn(&format!("系统{}失败(第{}次/{}): {}，将在下个周期重试", action_name, *retry_count, max_retry_num, error));
            true
        }
    }

    /// 安排关机（5分钟倒计时）
    async fn schedule_shutdown(
        shutdown_timer: &Arc<Mutex<Option<JoinHandle<()>>>>
    ) {
        // 发送通知
        Self::send_shutdown_notification().await;

        let timer_handle = tokio::spawn(async move {
            // 等待5分钟
            sleep(Duration::from_secs(300)).await;

            // 执行关机
            if let Err(e) = Self::shutdown_system() {
                Log::error(&format!("系统关机失败: {}", e));
            }
        });

        let mut timer = shutdown_timer.lock().await;
        *timer = Some(timer_handle);
    }

    /// 取消关机
    async fn cancel_shutdown(&self) -> AppResult<()> {
        let mut timer = self.shutdown_timer.lock().await;
        if let Some(handle) = timer.take() {
            handle.abort();
            Log::info("已取消自动关机");
            Ok(())
        } else {
            Err(AppError::ConfigError("没有进行中的关机任务".to_string()))
        }
    }

    /// 发送关机通知
    async fn send_shutdown_notification() {
        // 发送Tauri原生系统通知
        if let Err(e) = get_app_handle().await.notification()
            .builder()
            .title("AutoDaily - 自动关机提醒")
            .body("系统将在5分钟后自动关机\n您可以在应用中取消此操作")
            .icon("AutoDaily")
            .show()
        {
            Log::error(&format!("发送系统通知失败: {}", e));
        } else {
            Log::info("关机倒计时通知已发送");
        }

        // 如果有主窗口，发送事件到前端进行更详细的处理
        if let Some(window) = get_app_handle().await.get_webview_window(MAIN_WINDOW) {
            let _ = window.emit("shutdown_scheduled", serde_json::json!({
                "message": "5分钟后将自动关机",
                "countdown": 300,
                "type": "shutdown_warning",
                "timestamp": chrono::Utc::now().timestamp()
            }));
        }
    }

    /// 关机系统
    fn shutdown_system() -> AppResult<()> {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("shutdown")
                .arg("/s")
                .arg("/t")
                .arg("0")
                .output()
                .map_err(|e| AppError::ConfigError(format!("关机命令执行失败: {}", e)))?;
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            Command::new("shutdown")
                .arg("-h")
                .arg("now")
                .output()
                .map_err(|e| AppError::ConfigError(format!("关机命令执行失败: {}", e)))?;
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("shutdown")
                .arg("-h")
                .arg("now")
                .output()
                .map_err(|e| AppError::ConfigError(format!("关机命令执行失败: {}", e)))?;
        }

        Log::info("系统关机命令已执行");
        Ok(())
    }

    /// 睡眠系统
    fn sleep_system() -> AppResult<()> {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("rundll32.exe")
                .arg("powrprof.dll,SetSuspendState")
                .arg("0,1,0")
                .output()
                .map_err(|e| AppError::ConfigError(format!("睡眠命令执行失败: {}", e)))?;
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            Command::new("systemctl")
                .arg("suspend")
                .output()
                .map_err(|e| AppError::ConfigError(format!("睡眠命令执行失败: {}", e)))?;
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            Command::new("pmset")
                .arg("sleepnow")
                .output()
                .map_err(|e| AppError::ConfigError(format!("睡眠命令执行失败: {}", e)))?;
        }

        Log::info("系统睡眠命令已执行");
        Ok(())
    }

    /// 休眠系统
    fn hibernate_system() -> AppResult<()> {
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            Command::new("shutdown")
                .arg("/h")
                .output()
                .map_err(|e| AppError::ConfigError(format!("休眠命令执行失败: {}", e)))?;
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            Command::new("systemctl")
                .arg("hibernate")
                .output()
                .map_err(|e| AppError::ConfigError(format!("休眠命令执行失败: {}", e)))?;
        }

        #[cfg(target_os = "macos")]
        {
            // macOS 通常不支持休眠，使用睡眠代替
            return Self::sleep_system();
        }

        Log::info("系统休眠命令已执行");
        Ok(())
    }

    async fn check_running_scripts()->bool {
        false
    }

    /// 更新活动时间
    async fn update_activity(&self) {
        let mut last_activity = self.last_activity.lock().await;
        *last_activity = Instant::now();
    }
}