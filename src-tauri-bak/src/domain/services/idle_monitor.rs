use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::config::sys_conf::IdleAction;
use crate::domain::manager::conf_mgr::ConfigManager;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub trait IdleMonitorTrait{
    async fn start_monitoring(&self, config_manager: ConfigManager) -> AppResult<()>;

    async fn stop_monitoring(&self);

    /// 返回是否应该继续监控（false表示重试次数达到上限，应停止监控）
    async fn handle_idle_action(
        idle_action: &IdleAction,
        shutdown_timer: &Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
        retry_num: &Arc<Mutex<u8>>,
        max_retry_num: u8
    ) -> bool;
    /// 处理操作失败，检查重试次数
    async fn handle_action_failure(
        action_name: &str,
        error: &AppError,
        retry_num: &Arc<Mutex<u8>>,
        max_retry_num: u8
    ) -> bool;
    /// 安排关机（5分钟倒计时）
    async fn schedule_shutdown(
        shutdown_timer: &Arc<Mutex<Option<JoinHandle<()>>>>
    );
    /// 取消关机
    async fn cancel_shutdown(&self) -> AppResult<()>;
    /// 发送关机通知
    async fn send_shutdown_notification();
    /// 关机系统
    fn shutdown_system() -> AppResult<()>;
    /// 睡眠系统
    fn sleep_system() -> AppResult<()>;
    /// 休眠系统
    fn hibernate_system() -> AppResult<()>;
    async fn check_running_scripts() ->bool;
    /// 更新活动时间
    async fn update_activity(&self);
}