// 设备启动器模块
// 负责启动模拟器进程 + 延时/重试连接设备

use crate::domain::devices::device_conf::DeviceConfig;
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;
use crate::infrastructure::logging::log_trait::Log;

use std::net::SocketAddr;
use std::time::Duration;
use adb_client::server::ADBServer;
use adb_client::tcp::ADBTcpDevice;
use tokio::process::Command;
use tokio::time::sleep;

/// 默认启动延迟（等模拟器完全启动）
const DEFAULT_STARTUP_DELAY: Duration = Duration::from_secs(15);
/// 最大重试次数
const MAX_RETRIES: u32 = 5;
/// 重试间隔
const RETRY_INTERVAL: Duration = Duration::from_secs(3);

/// 启动设备（模拟器）并等待连接就绪
///
/// 流程：
/// 1. 如有 exe_path → 启动模拟器进程
/// 2. 等待 startup_delay
/// 3. 根据 adb_connect 重试连接
pub async fn launch_device(config: &DeviceConfig) -> Result<(), String> {
    // 1. 启动模拟器进程
    if let Some(exe_path) = &config.exe_path {
        Log::info(&format!(
            "[ launcher ] 正在启动模拟器: {}",
            exe_path
        ));

        let mut cmd = Command::new(exe_path);

        // 添加启动参数
        if let Some(args) = &config.exe_args {
            for arg in args.split_whitespace() {
                cmd.arg(arg);
            }
        }

        // 非阻塞启动，因为模拟器是长期运行的进程
        match cmd.spawn() {
            Ok(_child) => {
                Log::info("[ launcher ] 模拟器进程已启动，等待启动完成...");
            }
            Err(e) => {
                return Err(format!("启动模拟器失败: {}", e));
            }
        }

        // 2. 等待模拟器启动
        sleep(DEFAULT_STARTUP_DELAY).await;
    } else {
        Log::info("[ launcher ] 无 exe_path 配置，跳过模拟器启动，直接尝试连接...");
    }

    // 3. 重试连接设备
    if let Some(adb_connect) = &config.adb_connect {
        try_connect_with_retry(adb_connect, MAX_RETRIES, RETRY_INTERVAL).await?;
    } else {
        Log::warn("[ launcher ] 无 adb_connect 配置，跳过连接验证");
    }

    Ok(())
}

/// 重试连接设备
async fn try_connect_with_retry(
    adb_connect: &ADBConnectConfig,
    max_retries: u32,
    retry_interval: Duration,
) -> Result<(), String> {
    for attempt in 1..=max_retries {
        Log::info(&format!(
            "[ launcher ] 连接尝试 {}/{}...",
            attempt, max_retries
        ));

        match probe_connection(adb_connect) {
            Ok(_) => {
                Log::info("[ launcher ] 设备连接成功！");
                return Ok(());
            }
            Err(e) => {
                Log::warn(&format!(
                    "[ launcher ] 连接失败 ({}/{}): {}",
                    attempt, max_retries, e
                ));
                if attempt < max_retries {
                    sleep(retry_interval).await;
                }
            }
        }
    }

    Err(format!(
        "设备连接失败：已重试 {} 次，请检查设备是否正常运行",
        max_retries
    ))
}

/// 探测设备连接（同步，一次性尝试连接并验证）
fn probe_connection(adb_connect: &ADBConnectConfig) -> Result<(), String> {
    match adb_connect {
        ADBConnectConfig::ServerConnectByName(dev) => {
            if !dev.valid() {
                return Err("ServerConnectByName 配置无效（缺少 adb_path / server_connect / device_name）".into());
            }
            let mut server = ADBServer::new_from_path(
                dev.adb_config.server_connect.unwrap(),
                dev.adb_config.adb_path.clone(),
            );
            let _device = server
                .get_device_by_name(dev.device_name.as_ref().unwrap().as_str())
                .map_err(|e| format!("ServerConnectByName 获取设备失败: {}", e))?;
            Ok(())
        }
        ADBConnectConfig::ServerConnectByIp(dev) => {
            if !dev.valid() {
                return Err("ServerConnectByIp 配置无效（缺少 adb_path / server_connect / client_connect）".into());
            }
            let mut adb_server = ADBServer::new_from_path(
                dev.adb_config.server_connect.unwrap(),
                dev.adb_config.adb_path.clone(),
            );
            adb_server
                .connect_device(dev.client_connect.unwrap())
                .map_err(|e| format!("ServerConnectByIp connect_device 失败: {}", e))?;
            let _device = adb_server
                .get_device_by_name(&dev.client_connect.unwrap().to_string())
                .map_err(|e| format!("ServerConnectByIp 获取设备失败: {}", e))?;
            Ok(())
        }
        ADBConnectConfig::DirectTcp(addr) => {
            let addr = addr.ok_or("DirectTcp 配置无效：未设置连接地址")?;
            let _device = ADBTcpDevice::new(SocketAddr::V4(addr))
                .map_err(|e| format!("DirectTcp 连接失败 ({}): {}", addr, e))?;
            Ok(())
        }
        ADBConnectConfig::DirectUsb(_) => {
            Err("DirectUsb 暂不支持".into())
        }
    }
}
