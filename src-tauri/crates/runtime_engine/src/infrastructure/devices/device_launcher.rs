// 设备启动器模块
// 负责启动模拟器进程 + 延时/重试连接设备

use crate::domain::devices::device_conf::{DeviceConfig, DeviceTransportKind};
use crate::infrastructure::adb_cli_local::adb_config::{
    ADBConnectConfig, AdbServeByIdentifier, AdbServerConfig,
};
use crate::infrastructure::ipc::message::ConnectionStatusKind;
use crate::infrastructure::logging::log_trait::Log;

use adb_client::server::ADBServer;
use adb_client::tcp::ADBTcpDevice;
use adb_client::ADBDeviceExt;
use std::net::{SocketAddr, SocketAddrV4};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;

/// 启动后的固定探测间隔
const PROBE_INTERVAL: Duration = Duration::from_secs(1);
/// 启动后的最长连接等待时间
const POST_LAUNCH_CONNECT_TIMEOUT: Duration = Duration::from_secs(20);
/// 非启动路径下的最长连接等待时间
const DIRECT_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
/// 单次 shell 探测的最长等待时间
const SINGLE_PROBE_TIMEOUT: Duration = Duration::from_secs(3);

/// 启动设备（模拟器）并等待连接就绪
///
/// 流程：
/// 1. 如有 exe_path → 启动模拟器进程
/// 2. 等待 startup_delay_secs
/// 3. 根据 transportKind 生成运行时连接配置并重试连接
pub async fn launch_device(config: &DeviceConfig) -> Result<ADBConnectConfig, String> {
    launch_device_with_progress(config, |_, _| {}).await
}

pub async fn launch_device_with_progress(
    config: &DeviceConfig,
    on_status: impl Fn(ConnectionStatusKind, String),
) -> Result<ADBConnectConfig, String> {
    start_device_process_with_progress(config, &on_status).await?;
    wait_for_connection_ready_with_progress(
        config,
        POST_LAUNCH_CONNECT_TIMEOUT,
        PROBE_INTERVAL,
        "模拟器启动后",
        on_status,
    )
    .await
}

pub async fn start_device_process(config: &DeviceConfig) -> Result<(), String> {
    start_device_process_with_progress(config, |_, _| {}).await
}

async fn start_device_process_with_progress(
    config: &DeviceConfig,
    on_status: impl Fn(ConnectionStatusKind, String),
) -> Result<(), String> {
    if let Some(exe_path) = &config.exe_path {
        Log::info(&format!("[ launcher ] 正在启动模拟器: {}", exe_path));
        on_status(
            ConnectionStatusKind::EmulatorStarting,
            "正在启动模拟器".to_string(),
        );

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

        let startup_delay = Duration::from_secs(u64::from(config.startup_delay_secs));
        Log::info(&format!(
            "[ launcher ] 模拟器进程已启动，等待 {} 秒后开始探测连接...",
            config.startup_delay_secs
        ));
        on_status(
            ConnectionStatusKind::EmulatorWaiting,
            format!(
                "模拟器启动中，等待 {} 秒后连接探测",
                config.startup_delay_secs
            ),
        );
        sleep(startup_delay).await;
    } else {
        Log::info("[ launcher ] 无 exe_path 配置，跳过模拟器启动，直接尝试连接...");
    }

    Ok(())
}

pub async fn wait_for_device_connection(config: &DeviceConfig) -> Result<ADBConnectConfig, String> {
    wait_for_device_connection_with_progress(config, |_, _| {}).await
}

pub async fn wait_for_device_connection_with_progress(
    config: &DeviceConfig,
    on_status: impl Fn(ConnectionStatusKind, String),
) -> Result<ADBConnectConfig, String> {
    let timeout = if config.uses_emulator_transport() {
        POST_LAUNCH_CONNECT_TIMEOUT
    } else {
        DIRECT_CONNECT_TIMEOUT
    };
    wait_for_connection_ready_with_progress(
        config,
        timeout,
        PROBE_INTERVAL,
        "直接连接",
        on_status,
    )
    .await
}

pub async fn ensure_device_connection(config: &DeviceConfig) -> Result<ADBConnectConfig, String> {
    ensure_device_connection_with_progress(config, |_, _| {}).await
}

pub async fn ensure_device_connection_with_progress(
    config: &DeviceConfig,
    on_status: impl Fn(ConnectionStatusKind, String),
) -> Result<ADBConnectConfig, String> {
    Log::info(&format!(
        "[ launcher ] 开始检查设备连接 transport={:?} connect_address={:?} connect_identifier={:?} exe_path={:?}",
        config.transport_kind,
        config.connect_address,
        config.connect_identifier,
        config.exe_path
    ));
    on_status(
        ConnectionStatusKind::ShellProbeChecking,
        "尝试连接设备...）".to_string(),
    );
    let initial_error = match probe_device_config_connection_with_timeout(
        config,
        SINGLE_PROBE_TIMEOUT,
    )
    .await
    {
        Ok(runtime_connect) => {
            Log::info(&format!(
                "[ launcher ] 现有设备连接可直接使用，无需启动模拟器: {}",
                runtime_connect
            ));
            return Ok(runtime_connect);
        }
        Err(error) => error,
    };
    Log::warn(&format!(
        "[ launcher ] 现有设备连接不可用，初次 shell 探测失败: {}",
        initial_error
    ));
    on_status(
        ConnectionStatusKind::ShellProbeChecking,
        format!(
            "连接设备失败：{}",
            initial_error
        ),
    );

    if config.uses_emulator_transport() {
        if config
            .exe_path
            .as_deref()
            .is_some_and(|path| !path.trim().is_empty())
        {
            Log::info("[ launcher ] 当前为模拟器 TCP，且已配置启动路径，进入启动模拟器分支");
            on_status(
                ConnectionStatusKind::EmulatorStarting,
                "现有连接不可用，准备启动模拟器".to_string(),
            );
            return launch_device_with_progress(config, on_status).await;
        }
        Log::warn("[ launcher ] 当前为模拟器 TCP，但未配置启动路径，无法自动启动模拟器");
        return Err(format!(
            "连接设备失败，且未配置模拟器启动程序: {}",
            initial_error
        ));
    }

    Log::info("[ launcher ] 当前不是模拟器 TCP，进入纯连接等待分支，不启动模拟器");
    wait_for_device_connection_with_progress(config, on_status).await
}

pub fn resolve_runtime_connect_config(config: &DeviceConfig) -> Result<ADBConnectConfig, String> {
    build_connection_candidates(config)
        .into_iter()
        .next()
        .ok_or_else(|| "未配置设备连接信息".to_string())
}

pub fn probe_device_config_connection(config: &DeviceConfig) -> Result<ADBConnectConfig, String> {
    let mut last_error = "未配置设备连接信息".to_string();
    for runtime_connect in build_connection_candidates(config) {
        match probe_device_connection(&runtime_connect) {
            Ok(()) => return Ok(runtime_connect),
            Err(error) => last_error = error,
        }
    }
    Err(last_error)
}

pub async fn probe_device_config_connection_with_timeout(
    config: &DeviceConfig,
    timeout: Duration,
) -> Result<ADBConnectConfig, String> {
    let mut last_error = "未配置设备连接信息".to_string();
    for runtime_connect in build_connection_candidates(config) {
        match probe_device_connection_with_timeout(runtime_connect.clone(), timeout).await {
            Ok(()) => return Ok(runtime_connect),
            Err(error) => last_error = error,
        }
    }
    Err(last_error)
}

async fn probe_device_connection_with_timeout(
    runtime_connect: ADBConnectConfig,
    timeout: Duration,
) -> Result<(), String> {
    match tokio::time::timeout(timeout, tokio::task::spawn_blocking(move || {
        probe_device_connection(&runtime_connect)
    }))
    .await
    {
        Ok(Ok(result)) => result,
        Ok(Err(error)) => Err(format!("ADB shell 探测任务异常退出: {}", error)),
        Err(_) => Err(format!("ADB shell 探测超时（{} 秒）", timeout.as_secs())),
    }
}

async fn wait_for_connection_ready_with_progress(
    config: &DeviceConfig,
    timeout: Duration,
    retry_interval: Duration,
    probe_context: &str,
    on_status: impl Fn(ConnectionStatusKind, String),
) -> Result<ADBConnectConfig, String> {
    let started_at = tokio::time::Instant::now();
    let mut attempt = 0u32;
    let mut last_error = "未配置设备连接信息".to_string();

    loop {
        attempt += 1;
        let elapsed_secs = started_at.elapsed().as_secs();
        Log::info(&format!(
            "[ launcher ] 连接尝试第 {} 次，已等待 {} 秒...",
            attempt, elapsed_secs
        ));
        on_status(
            ConnectionStatusKind::ShellProbeChecking,
            format!(
                "正在尝试连接设备（{}，第 {} 次，已等待 {} 秒）",
                probe_context, attempt, elapsed_secs
            ),
        );

        let candidates = build_connection_candidates(config);
        if candidates.is_empty() {
            return Err(last_error);
        }

        for runtime_connect in candidates {
            match probe_device_connection_with_timeout(runtime_connect.clone(), SINGLE_PROBE_TIMEOUT)
                .await
            {
                Ok(_) => {
                    Log::info(&format!("[ launcher ] 设备连接成功：{}", runtime_connect));
                    return Ok(runtime_connect);
                }
                Err(e) => {
                    last_error = e;
                }
            }
        }

        Log::warn(&format!(
            "[ launcher ] 连接失败（第 {} 次）: {}",
            attempt, last_error
        ));
        on_status(
            ConnectionStatusKind::ShellProbeChecking,
            format!(
                "连接设备失败（{}，第 {} 次，已等待 {} 秒）：{}",
                probe_context, attempt, elapsed_secs, last_error
            ),
        );
        if started_at.elapsed() >= timeout {
            return Err(format!(
                "设备连接失败：等待 {} 秒后仍未就绪，最后一次错误: {}",
                timeout.as_secs(),
                last_error
            ));
        }
        sleep(retry_interval).await;
    }
}

fn build_connection_candidates(config: &DeviceConfig) -> Vec<ADBConnectConfig> {
    match config.transport_kind {
        DeviceTransportKind::EmulatorTcp => {
            vec![ADBConnectConfig::DirectTcp(config.connect_address)]
        }
        DeviceTransportKind::AdbUsb => vec![serve_by_identifier_config(config)],
        DeviceTransportKind::AdbWireless => {
            let mut configs = Vec::new();
            match resolve_wireless_mdns_direct_tcp(config) {
                Ok(Some(config)) => configs.push(config),
                Ok(None) => Log::warn(
                    "[ launcher ] mDNS 未找到匹配的无线调试设备，回退到 ServeByIdentifier",
                ),
                Err(error) => Log::warn(&format!(
                    "[ launcher ] mDNS 查询失败，回退到 ServeByIdentifier: {}",
                    error
                )),
            }
            configs.push(serve_by_identifier_config(config));
            configs
        }
    }
}

fn serve_by_identifier_config(config: &DeviceConfig) -> ADBConnectConfig {
    ADBConnectConfig::ServeByIdentifier(AdbServeByIdentifier {
        adb_config: AdbServerConfig {
            adb_path: config.adb_path.clone(),
            server_connect: config.adb_server_connect,
        },
        identifier: config.connect_identifier.clone(),
    })
}

fn resolve_wireless_mdns_direct_tcp(
    config: &DeviceConfig,
) -> Result<Option<ADBConnectConfig>, String> {
    let identifier = config
        .connect_identifier
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "未设置无线调试设备标识".to_string())?;
    let adb_path = config
        .adb_path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "未设置 adb 程序路径".to_string())?;
    let server_connect = config
        .adb_server_connect
        .ok_or_else(|| "未设置 ADB server 地址".to_string())?;

    let mut server = ADBServer::new_from_path(server_connect, Some(adb_path.to_string()));
    let services = server
        .mdns_services()
        .map_err(|e| format!("读取 mDNS services 失败: {}", e))?;

    let direct_addr = services
        .iter()
        .find(|service| {
            service.reg_type.contains("_adb-tls-connect")
                && mdns_service_matches(identifier, &service.service_name, service.socket_v4)
        })
        .map(|service| service.socket_v4);

    Ok(direct_addr.map(|addr| ADBConnectConfig::DirectTcp(Some(addr))))
}

fn mdns_service_matches(identifier: &str, service_name: &str, socket_v4: SocketAddrV4) -> bool {
    service_name.eq_ignore_ascii_case(identifier)
        || service_name
            .to_ascii_lowercase()
            .contains(&identifier.to_ascii_lowercase())
        || socket_v4.to_string() == identifier
}

/// 探测设备连接（同步，一次性尝试连接并验证）
pub fn probe_device_connection(runtime_connect: &ADBConnectConfig) -> Result<(), String> {
    match runtime_connect {
        ADBConnectConfig::ServeByIdentifier(dev) => {
            if !dev.valid() {
                return Err(
                    "ServeByIdentifier 配置无效（缺少 adb_path / server_connect / identifier）"
                        .into(),
                );
            }
            let mut server = ADBServer::new_from_path(
                dev.adb_config.server_connect.unwrap(),
                dev.adb_config.adb_path.clone(),
            );
            let mut device = server
                .get_device_by_name(dev.identifier.as_ref().unwrap().as_str())
                .map_err(|e| format!("ServeByIdentifier 获取设备失败: {}", e))?;
            let mut stdout = Vec::new();
            device
                .shell_command(&"echo autodaily-probe", Some(&mut stdout), None)
                .map_err(|e| {
                    format!(
                        "ServeByIdentifier shell 探测失败 ({}): {}",
                        dev.identifier.as_deref().unwrap_or_default(),
                        e
                    )
                })?;
            let output = String::from_utf8_lossy(&stdout);
            if !output.contains("autodaily-probe") {
                return Err(format!(
                    "ServeByIdentifier shell 探测响应异常 ({}): {}",
                    dev.identifier.as_deref().unwrap_or_default(),
                    output.trim()
                ));
            }
            Ok(())
        }
        ADBConnectConfig::DirectTcp(addr) => {
            let addr = addr.ok_or("DirectTcp 配置无效：未设置连接地址")?;
            let mut device = ADBTcpDevice::new(SocketAddr::V4(addr))
                .map_err(|e| format!("DirectTcp 连接失败 ({}): {}", addr, e))?;
            let mut stdout = Vec::new();
            device
                .shell_command(&"echo autodaily-probe", Some(&mut stdout), None)
                .map_err(|e| format!("DirectTcp shell 探测失败 ({}): {}", addr, e))?;
            let output = String::from_utf8_lossy(&stdout);
            if !output.contains("autodaily-probe") {
                return Err(format!(
                    "DirectTcp shell 探测响应异常 ({}): {}",
                    addr,
                    output.trim()
                ));
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[tokio::test]
    async fn emulator_without_exe_fails_after_initial_shell_probe() {
        let config = DeviceConfig::default();
        let statuses = Mutex::new(Vec::new());

        let result = ensure_device_connection_with_progress(&config, |status, message| {
            statuses.lock().unwrap().push((status, message));
        })
        .await;

        let error = result.expect_err("missing emulator executable must fail");
        assert!(error.contains("未配置模拟器启动程序"));

        let statuses = statuses.into_inner().unwrap();
        assert!(statuses
            .iter()
            .all(|(status, _)| *status == ConnectionStatusKind::ShellProbeChecking));
        assert!(!statuses.iter().any(|(status, _)| matches!(
            status,
            ConnectionStatusKind::EmulatorStarting | ConnectionStatusKind::EmulatorWaiting
        )));
    }
}
