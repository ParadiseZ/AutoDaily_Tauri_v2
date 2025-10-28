use std::process;
use anyhow::Result;
use crate::infrastructure::performance::{ProcessManager, create_physical_core_process_config};
use crate::infrastructure::entities::vision::base_model::BaseModel;

/// 物理核心+ORT推理的完整示例
/// 
/// 演示如何正确分配物理核心给子进程，并确保ORT推理使用这些核心
#[tokio::main]
async fn main() -> Result<()> {
    println!("=== 物理核心+ORT推理示例 ===");
    
    let process_manager = ProcessManager::new();
    
    // 1. 检查系统信息
    let physical_cores = process_manager.get_physical_cpu_count();
    let logical_cores = process_manager.get_logical_cpu_count();
    
    println!("系统信息:");
    println!("  物理CPU核心: {} 个", physical_cores);
    println!("  逻辑CPU核心: {} 个", logical_cores);
    println!("  超线程支持: {}", logical_cores > physical_cores);
    
    // 2. 为ORT推理分配4个物理核心（基于配置cores_per_device=4）
    let physical_cores_needed = 4.min(physical_cores); // 不超过系统物理核心数
    
    println!("\n=== CPU核心分配 ===");
    let logical_mapping = process_manager.allocate_logical_cores_for_physical(physical_cores_needed);
    println!("为ORT分配{}个物理核心，映射到逻辑核心: {:?}", 
             physical_cores_needed, logical_mapping);
    
    // 3. 创建基于物理核心的进程配置
    let config = create_physical_core_process_config(
        "ort_yolo_inference",
        "target/debug/ort_inference_worker", // 假设的推理工作进程
        vec![
            "--model=yolo.onnx".to_string(),
            "--input-size=640".to_string(),
        ],
        physical_cores_needed, // 👈 关键：基于物理核心数配置
    );
    
    println!("\n=== 进程配置 ===");
    println!("进程名: {}", config.name);
    println!("绑定逻辑核心: {:?}", config.core_ids);
    if let Some(env_vars) = &config.env_vars {
        println!("环境变量:");
        for (key, value) in env_vars {
            println!("  {}={}", key, value);
        }
    }
    
    // 4. 启动绑定物理核心的子进程
    println!("\n=== 启动推理进程 ===");
    match process_manager.spawn_process_with_affinity(config) {
        Ok(process_id) => {
            println!("✅ 成功启动推理进程: {}", process_id);
            
            // 验证进程状态
            let processes = process_manager.get_active_process_status();
            for process in processes {
                if process.id == process_id {
                    println!("进程状态:");
                    println!("  PID: {}", process.pid);
                    println!("  主核心: {:?}", process.core_id);
                    println!("  运行状态: {}", process.is_running);
                }
            }
            
            // 5. 在子进程中，演示ORT模型创建
            demonstrate_ort_configuration();
            
            // 清理
            std::thread::sleep(std::time::Duration::from_secs(2));
            if let Err(e) = process_manager.terminate_process(&process_id) {
                println!("⚠️  终止进程失败: {}", e);
            } else {
                println!("✅ 进程已正常终止");
            }
        }
        Err(e) => {
            println!("❌ 启动进程失败: {}", e);
        }
    }
    
    Ok(())
}

/// 演示在子进程中如何配置ORT模型
fn demonstrate_ort_configuration() {
    println!("\n=== ORT配置演示 ===");
    println!("在子进程中，ORT模型将这样创建:");
    
    // 模拟在子进程中创建ORT模型
    let model_bytes = vec![0u8; 1024]; // 模拟模型数据
    
    // ✅ 推荐方式：使用物理核心优化配置
    println!("使用 BaseModel::new_with_physical_core_optimization():");
    let _optimized_model = BaseModel::new_with_physical_core_optimization(
        640, 640,
        model_bytes.clone(),
        "cpu".to_string(),
    );
    println!("  - 自动从环境变量读取物理核心分配");
    println!("  - ORT intra_threads = 分配的物理核心数");
    println!("  - ORT inter_threads = 1 (避免竞争)");
    
    // ❌ 旧方式对比：手动配置（容易出错）
    println!("\n对比旧方式 BaseModel::new() (不推荐):");
    let _manual_model = BaseModel::new(
        640, 640,
        model_bytes,
        "cpu".to_string(),
        4,     // 手动设置，可能与实际分配不符
        true,
        1,
        false,
    );
    println!("  - 需要手动猜测线程数");
    println!("  - 无法确保与CPU分配对应");
    
    println!("\n=== 性能验证要点 ===");
    println!("1. 进程CPU亲和性掩码应该匹配分配的逻辑核心");
    println!("2. ORT intra_threads数量应该等于物理核心数");
    println!("3. 推理任务应该只在分配的核心上运行");
    
    #[cfg(target_os = "windows")]
    println!("验证命令 (Windows): Get-Process -Id <PID> | Select ProcessorAffinity");
    
    #[cfg(target_os = "linux")]
    println!("验证命令 (Linux): taskset -cp <PID>");
}

/// CPU核心映射分析示例
#[allow(dead_code)]
fn analyze_cpu_mapping() {
    println!("\n=== CPU核心映射分析 ===");
    
    let process_manager = ProcessManager::new();
    let physical = process_manager.get_physical_cpu_count();
    let logical = process_manager.get_logical_cpu_count();
    
    println!("假设4核8线程CPU的典型映射:");
    println!("物理核心0 → 逻辑核心0, 4");
    println!("物理核心1 → 逻辑核心1, 5");
    println!("物理核心2 → 逻辑核心2, 6");
    println!("物理核心3 → 逻辑核心3, 7");
    println!();
    
    println!("推理任务优化策略:");
    println!("✅ 分配逻辑核心0,1,2,3 (4个物理核心的主线程)");
    println!("❌ 分配逻辑核心0,1,2,3,4,5,6,7 (包含超线程，性能可能下降)");
    println!();
    
    if logical > physical {
        println!("当前系统支持超线程，比率: {}:1", logical / physical);
        println!("优化配置已自动选择主线程核心");
    } else {
        println!("当前系统无超线程，物理=逻辑核心");
    }
}
