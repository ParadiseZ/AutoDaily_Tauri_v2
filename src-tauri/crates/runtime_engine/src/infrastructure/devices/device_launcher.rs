// 设备启动器模块
// 负责启动模拟器进程 + 延时/重试连接设备

use crate::domain::devices::device_conf::DeviceConfig;
use crate::infrastructure::adb_cli_local::adb_config::ADBConnectConfig;
use crate::infrastructure::logging::log_trait::Log;

use adb_client::server::ADBServer;
use adb_client::tcp::ADBTcpDevice;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;

/// 启动后的固定探测间隔
const PROBE_INTERVAL: Duration = Duration::from_secs(1);
/// 启动后的最长连接等待时间
const POST_LAUNCH_CONNECT_TIMEOUT: Duration = Duration::from_secs(60);
/// 非启动路径下的最长连接等待时间
const DIRECT_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);

/// 启动设备（模拟器）并等待连接就绪
///
/// 流程：
/// 1. 如有 exe_path → 启动模拟器进程
/// 2. 等待 startup_delay_secs
/// 3. 根据 adb_connect 重试连接
pub async fn launch_device(config: &DeviceConfig) -> Result<(), String> {
    start_device_process(config).await?;

    // 3. 重试连接设备
    if let Some(adb_connect) = &config.adb_connect {
        wait_for_connection_ready(adb_connect, POST_LAUNCH_CONNECT_TIMEOUT, PROBE_INTERVAL).await?;
    } else {
        Log::warn("[ launcher ] 无 adb_connect 配置，跳过连接验证");
    }

    Ok(())
}

pub async fn start_device_process(config: &DeviceConfig) -> Result<(), String> {
    if let Some(exe_path) = &config.exe_path {
        Log::info(&format!("[ launcher ] 正在启动模拟器: {}", exe_path));

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

        let startup_delay = Duration::from_secs(config.startup_delay_secs);
        Log::info(&format!(
            "[ launcher ] 模拟器进程已启动，等待 {} 秒后开始探测连接...",
            config.startup_delay_secs
        ));
        sleep(startup_delay).await;
    } else {
        Log::info("[ launcher ] 无 exe_path 配置，跳过模拟器启动，直接尝试连接...");
    }

    Ok(())
}

pub async fn wait_for_device_connection(config: &DeviceConfig) -> Result<(), String> {
    if let Some(adb_connect) = &config.adb_connect {
        let timeout = if config.uses_emulator_transport() {
            POST_LAUNCH_CONNECT_TIMEOUT
        } else {
            DIRECT_CONNECT_TIMEOUT
        };
        wait_for_connection_ready(adb_connect, timeout, PROBE_INTERVAL).await
    } else {
        Err("未配置设备连接信息".to_string())
    }
}

pub async fn ensure_device_connection(config: &DeviceConfig) -> Result<(), String> {
    if let Some(adb_connect) = &config.adb_connect {
        if probe_device_connection(adb_connect).is_ok() {
            return Ok(());
        }
    }

    if config.uses_emulator_transport()
        && config
        .exe_path
        .as_deref()
        .is_some_and(|path| !path.trim().is_empty())
    {
        return launch_device(config).await;
    }

    wait_for_device_connection(config).await
}

/// 在指定时间预算内循环探测连接就绪
async fn wait_for_connection_ready(
    adb_connect: &ADBConnectConfig,
    timeout: Duration,
    retry_interval: Duration,
) -> Result<(), String> {
    let started_at = tokio::time::Instant::now();
    let mut attempt = 0u32;

    loop {
        attempt += 1;
        Log::info(&format!(
            "[ launcher ] 连接尝试第 {} 次，已等待 {} 秒...",
            attempt,
            started_at.elapsed().as_secs()
        ));

        match probe_device_connection(adb_connect) {
            Ok(_) => {
                Log::info("[ launcher ] 设备连接成功！");
                return Ok(());
            }
            Err(e) => {
                Log::warn(&format!(
                    "[ launcher ] 连接失败（第 {} 次）: {}",
                    attempt, e
                ));
                if started_at.elapsed() >= timeout {
                    return Err(format!(
                        "设备连接失败：等待 {} 秒后仍未就绪，最后一次错误: {}",
                        timeout.as_secs(),
                        e
                    ));
                }
                sleep(retry_interval).await;
            }
        }
    }
}

/// 探测设备连接（同步，一次性尝试连接并验证）
pub fn probe_device_connection(adb_connect: &ADBConnectConfig) -> Result<(), String> {
    match adb_connect {
        ADBConnectConfig::ServerConnectByName(dev) => {
            if !dev.valid() {
                return Err(
                    "ServerConnectByName 配置无效（缺少 adb_path / server_connect / device_name）"
                        .into(),
                );
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
                return Err(
                    "ServerConnectByIp 配置无效（缺少 adb_path / server_connect / client_connect）"
                        .into(),
                );
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
        ADBConnectConfig::DirectUsb(_) => Err("DirectUsb 暂不支持".into()),
    }
}
