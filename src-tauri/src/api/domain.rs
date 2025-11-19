use std::process::{Command, Stdio};
use tauri::command;
use tokio::process::Child;
use crate::infrastructure::context::child_process::ChildProcessInitData;

#[command]
/// 启动子进程
pub async fn launch_child_process(
    executable_path:  String,
    init_data: ChildProcessInitData,
) -> Result<Child, Box<dyn std::error::Error>> {
    // 1. 序列化初始化数据
    let serialized_data = serde_json::to_string(&init_data)?;

    // 2. 创建子进程命令
    let mut cmd = Command::new(executable_path);

    // 3. 设置环境变量
    cmd.env("CHILD_CONTEXT_DATA", serialized_data);
    //cmd.env("RUST_LOG", &init_data.log_level);

    // 4. 设置标准输入输出
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // 5. 启动进程
    let child = tokio::process::Command::from(cmd)
        .spawn()
        .map_err(|e| format!("启动子进程失败: {}", e))?;

    tracing::info!(
            "子进程已启动: device_id={},device_name={}, pid={}",
            init_data.device_id,
            init_data.device_config.device_name,
            //init_data.process_id,
            child.id().unwrap_or(0)
        );

    Ok(child)
}