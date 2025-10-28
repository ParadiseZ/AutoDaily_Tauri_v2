use std::time::Duration;
use std::thread;

// 注意：这是一个示例文件，展示如何使用 ProcessManager
// 实际使用时，请将这些代码集成到你的主项目中

use crate::infrastructure::performance::{
    ProcessManager, 
    create_process_config, 
    create_process_config_with_core,
    ProcessPriority
};

/// 进程管理器使用示例
fn main() {
    // 初始化日志（可选）
    tracing_subscriber::fmt::init();
    
    // 创建进程管理器
    let process_manager = ProcessManager::new();
    
    println!("系统信息：");
    println!("CPU核心数: {}", process_manager.get_cpu_count());
    println!("可用核心ID: {:?}", process_manager.get_available_core_ids());
    
    // 示例1：创建基本子进程
    example_basic_process(&process_manager);
    
    // 示例2：创建绑定到特定CPU核心的进程
    example_core_affinity_process(&process_manager);
    
    // 示例3：创建多个工作进程
    example_multiple_worker_processes(&process_manager);
    
    // 示例4：模拟图像处理任务进程
    example_image_processing_process(&process_manager);
}

/// 示例1：基本进程创建
fn example_basic_process(manager: &ProcessManager) {
    println!("\n=== 示例1：基本进程创建 ===");
    
    #[cfg(target_os = "windows")]
    let config = create_process_config(
        "basic_worker",
        "cmd",
        vec!["/C".to_string(), "echo".to_string(), "Hello from Windows subprocess!".to_string()]
    );
    
    #[cfg(not(target_os = "windows"))]
    let config = create_process_config(
        "basic_worker",
        "echo",
        vec!["Hello from Unix subprocess!".to_string()]
    );
    
    let process_id = manager
        .spawn_process_with_affinity(config)
        .expect("创建基本进程失败");
    
    println!("创建进程，ID: {}", process_id);
    
    // 等待进程完成
    thread::sleep(Duration::from_millis(500));
    manager.cleanup_finished_processes();
}

/// 示例2：CPU核心亲和性进程
fn example_core_affinity_process(manager: &ProcessManager) {
    println!("\n=== 示例2：CPU核心亲和性进程 ===");
    
    // 绑定到第一个CPU核心的计算密集型任务
    #[cfg(target_os = "windows")]
    let config = create_process_config_with_core(
        "core_bound_worker",
        "powershell",
        vec![
            "-Command".to_string(),
            "$sum = 0; for ($i = 0; $i -lt 1000000; $i++) { $sum += $i * $i }; Write-Host \"计算完成，结果: $sum\"".to_string()
        ],
        0
    );
    
    #[cfg(not(target_os = "windows"))]
    let config = create_process_config_with_core(
        "core_bound_worker",
        "bash",
        vec![
            "-c".to_string(),
            "sum=0; for i in {1..1000000}; do sum=$((sum + i * i)); done; echo \"计算完成，结果: $sum\"".to_string()
        ],
        0
    );
    
    let process_id = manager
        .spawn_process_with_affinity(config)
        .expect("创建核心绑定进程失败");
    
    println!("创建核心绑定进程，ID: {}", process_id);
    
    // 等待进程完成
    thread::sleep(Duration::from_secs(2));
    manager.cleanup_finished_processes();
}

/// 示例3：多个工作进程
fn example_multiple_worker_processes(manager: &ProcessManager) {
    println!("\n=== 示例3：多个工作进程 ===");
    
    let mut process_ids = Vec::new();
    let cpu_count = manager.get_cpu_count();
    
    // 创建多个工作进程，每个绑定到不同的CPU核心
    for i in 0..std::cmp::min(3, cpu_count) {
        #[cfg(target_os = "windows")]
        let config = create_process_config_with_core(
            &format!("worker_{}", i),
            "powershell",
            vec![
                "-Command".to_string(),
                format!(
                    "Write-Host '工作进程 {} 开始执行（CPU核心 {}）'; Start-Sleep -Seconds 1; Write-Host '工作进程 {} 完成'",
                    i, i, i
                )
            ],
            i
        );
        
        #[cfg(not(target_os = "windows"))]
        let config = create_process_config_with_core(
            &format!("worker_{}", i),
            "bash",
            vec![
                "-c".to_string(),
                format!(
                    "echo '工作进程 {} 开始执行（CPU核心 {}）'; sleep 1; echo '工作进程 {} 完成'",
                    i, i, i
                )
            ],
            i
        );
        
        let process_id = manager
            .spawn_process_with_affinity(config)
            .expect("创建工作进程失败");
        
        process_ids.push(process_id);
        println!("创建工作进程 {}，ID: {}", i, process_ids.last().unwrap());
    }
    
    // 等待所有进程完成
    thread::sleep(Duration::from_secs(3));
    
    // 显示进程状态
    let active_processes = manager.get_active_process_status();
    println!("当前活跃进程数: {}", active_processes.len());
    for status in &active_processes {
        println!("  进程 '{}' (PID: {}, 核心: {:?})", status.name, status.pid, status.core_id);
    }
    
    manager.cleanup_finished_processes();
    println!("所有工作进程已完成");
}

/// 示例4：模拟图像处理任务进程
fn example_image_processing_process(manager: &ProcessManager) {
    println!("\n=== 示例4：图像处理任务进程 ===");
    
    // 为图像处理任务创建专用进程，绑定到最后一个CPU核心
    let cpu_count = manager.get_cpu_count();
    let processing_core = if cpu_count > 1 { cpu_count - 1 } else { 0 };
    
    let mut config = create_process_config_with_core(
        "image_processor",
        if cfg!(target_os = "windows") { "powershell" } else { "bash" },
        if cfg!(target_os = "windows") {
            vec![
                "-Command".to_string(),
                format!(
                    "Write-Host '图像处理进程开始执行（CPU核心 {}）'; \
                     Write-Host '1. 加载图像...'; Start-Sleep -Milliseconds 200; \
                     Write-Host '2. 图像预处理...'; Start-Sleep -Milliseconds 300; \
                     Write-Host '3. 特征提取...'; Start-Sleep -Milliseconds 400; \
                     Write-Host '4. 结果保存...'; Start-Sleep -Milliseconds 100; \
                     Write-Host '图像处理完成'",
                    processing_core
                )
            ]
        } else {
            vec![
                "-c".to_string(),
                format!(
                    "echo '图像处理进程开始执行（CPU核心 {}）'; \
                     echo '1. 加载图像...'; sleep 0.2; \
                     echo '2. 图像预处理...'; sleep 0.3; \
                     echo '3. 特征提取...'; sleep 0.4; \
                     echo '4. 结果保存...'; sleep 0.1; \
                     echo '图像处理完成'",
                    processing_core
                )
            ]
        },
        processing_core
    );
    
    // 设置高优先级
    config.priority = Some(ProcessPriority::High);
    
    let process_id = manager
        .spawn_process_with_affinity(config)
        .expect("创建图像处理进程失败");
    
    println!("创建图像处理进程，ID: {}", process_id);
    
    // 显示活跃进程信息
    thread::sleep(Duration::from_millis(100));
    println!("当前活跃进程数: {}", manager.get_active_process_count());
    
    let active_processes = manager.get_active_process_status();
    for status in &active_processes {
        println!("  进程 '{}' (PID: {}, 核心: {:?}, 运行中: {})", 
                 status.name, status.pid, status.core_id, status.is_running);
    }
    
    // 等待图像处理完成
    thread::sleep(Duration::from_secs(2));
    manager.cleanup_finished_processes();
    
    println!("图像处理任务完成");
}

/// 演示进程输出获取
fn example_process_output(manager: &ProcessManager) {
    println!("\n=== 进程输出获取示例 ===");
    
    let config = create_process_config(
        "output_test",
        if cfg!(target_os = "windows") { "cmd" } else { "echo" },
        if cfg!(target_os = "windows") {
            vec!["/C".to_string(), "echo".to_string(), "这是测试输出".to_string()]
        } else {
            vec!["这是测试输出".to_string()]
        }
    );
    
    let process_id = manager
        .spawn_process_with_affinity(config)
        .expect("创建输出测试进程失败");
    
    // 等待进程完成
    thread::sleep(Duration::from_millis(100));
    
    // 获取进程输出
    match manager.get_process_output(&process_id) {
        Ok((stdout, stderr)) => {
            println!("进程输出:");
            if !stdout.is_empty() {
                println!("  标准输出: {}", stdout.trim());
            }
            if !stderr.is_empty() {
                println!("  错误输出: {}", stderr.trim());
            }
        }
        Err(e) => {
            println!("获取进程输出失败: {}", e);
        }
    }
    
    manager.cleanup_finished_processes();
}