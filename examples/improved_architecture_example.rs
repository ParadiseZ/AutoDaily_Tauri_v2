/// AutoDaily 架构改进示例
/// 展示了子进程、分页管理、JSON加载和数据存储策略的最佳实践

use std::path::PathBuf;

// 假设这些是我们改进后的模块导入
use crate::infrastructure::context::main_process::MainProcessCtx;
use crate::infrastructure::scripts::script_info::{
    ScriptManager, ScriptPageRequest, ScriptInfo, SortField, SortOrder
};

/// 主函数示例 - 展示完整的初始化和使用流程
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 AutoDaily 架构改进示例");
    
    // 1. 初始化主进程上下文
    println!("\n📋 初始化主进程上下文...");
    let scripts_dir = PathBuf::from("./scripts");
    let cache_size = 100; // 缓存最多100个脚本
    
    let mut main_ctx = MainProcessCtx::initialize(scripts_dir, cache_size).await?;
    println!("✅ 主进程上下文初始化完成");

    // 2. 演示脚本分页管理
    println!("\n📄 演示脚本分页管理...");
    demonstrate_script_pagination(&mut main_ctx).await?;

    // 3. 演示JSON文件操作
    println!("\n💾 演示JSON文件操作...");
    demonstrate_json_operations(&mut main_ctx).await?;

    // 4. 演示排序功能
    println!("\n🔢 演示排序功能...");
    demonstrate_sorting(&mut main_ctx).await?;

    // 5. 数据存储策略展示
    println!("\n🗄️ 数据存储策略展示...");
    demonstrate_storage_strategy(&main_ctx);

    println!("\n🎉 示例运行完成!");
    Ok(())
}

/// 演示脚本分页管理
async fn demonstrate_script_pagination(
    main_ctx: &mut MainProcessCtx
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建分页请求
    let page_request = ScriptPageRequest {
        page: 0,
        page_size: 10,
        sort_by: SortField::CreateTime,
        sort_order: SortOrder::Desc,
        filter: None,
    };

    // 获取分页结果
    let response = main_ctx.get_scripts_page(page_request).await?;
    
    println!("📊 分页结果:");
    println!("  - 当前页: {}/{}", response.page + 1, response.total_pages);
    println!("  - 每页大小: {}", response.page_size);
    println!("  - 总记录数: {}", response.total_count);
    println!("  - 本页记录: {}", response.scripts.len());

    // 显示脚本列表
    for (idx, script) in response.scripts.iter().enumerate() {
        println!("  {}. {} (类型: {}, 优先级: {})", 
                idx + 1, script.script_name, script.script_type, script.priority);
    }

    Ok(())
}

/// 演示JSON文件操作
async fn demonstrate_json_operations(
    main_ctx: &mut MainProcessCtx
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建一个示例脚本
    let sample_script = ScriptInfo {
        script_id: 12345,
        script_name: "示例自动化脚本".to_string(),
        script_type: "automation".to_string(),
        script_path: "./scripts/sample.js".to_string(),
        script_args: vec!["--mode".to_string(), "production".to_string()],
        script_env: std::collections::HashMap::new(),
        script_log: "./logs/sample.log".to_string(),
        script_status: "ready".to_string(),
        script_create_time: chrono::Utc::now().to_rfc3339(),
        priority: 5,
        last_modified: chrono::Utc::now().to_rfc3339(),
        execution_count: 0,
    };

    // 保存脚本到JSON文件
    main_ctx.script_manager.save_script(&sample_script).await?;
    println!("💾 脚本已保存到JSON文件: {}.json", sample_script.script_id);

    // 重新加载索引以包含新脚本
    main_ctx.script_manager.load_from_directory().await?;
    println!("🔄 脚本索引已重新加载");

    // 获取统计信息
    let stats = main_ctx.script_manager.get_statistics();
    println!("📈 脚本统计:");
    println!("  - 总脚本数: {}", stats.total_count);
    println!("  - 总执行次数: {}", stats.total_executions);
    for (script_type, count) in &stats.by_type {
        println!("  - {}: {} 个", script_type, count);
    }

    Ok(())
}

/// 演示排序功能
async fn demonstrate_sorting(
    main_ctx: &mut MainProcessCtx
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 按优先级降序排序:");
    let high_priority_request = ScriptPageRequest {
        page: 0,
        page_size: 5,
        sort_by: SortField::Priority,
        sort_order: SortOrder::Desc,
        filter: None,
    };

    let response = main_ctx.get_scripts_page(high_priority_request).await?;
    for script in &response.scripts {
        println!("  - {} (优先级: {})", script.script_name, script.priority);
    }

    println!("\n🔍 按名称升序排序:");
    let name_sorted_request = ScriptPageRequest {
        page: 0,
        page_size: 5,
        sort_by: SortField::Name,
        sort_order: SortOrder::Asc,
        filter: None,
    };

    let response = main_ctx.get_scripts_page(name_sorted_request).await?;
    for script in &response.scripts {
        println!("  - {}", script.script_name);
    }

    // 演示复合排序
    println!("\n🔍 复合排序（优先级降序 + 名称升序）:");
    let complex_sort_rules = vec![
        (SortField::Priority, SortOrder::Desc),
        (SortField::Name, SortOrder::Asc),
    ];
    
    let response = main_ctx.script_manager.get_scripts_with_complex_sort(0, 5, complex_sort_rules)?;
    for script in &response.scripts {
        println!("  - {} (优先级: {})", script.script_name, script.priority);
    }

    Ok(())
}

/// 演示数据存储策略
fn demonstrate_storage_strategy(main_ctx: &MainProcessCtx) {
    println!("📚 数据存储策略说明:");
    println!("
📦 1. 大量数据（脚本信息）：
   - ✅ 使用索引 + 分页 + LRU缓存
   - ✅ 避免全量内存加载
   - ✅ 按需从文件加载
   - 🔍 当前缓存大小: {} 个脚本
   - 💡 适合处理数千个脚本文件

🔧 2. 少量配置数据（设备配置）：
   - ✅ 全量加载到内存
   - ✅ 快速访问，无磁盘IO延迟
   - 🔍 当前设备数: {}
   - 💡 适合处理几十个设备配置

🚀 3. 运行时数据（IPC通道）：
   - ✅ 必须在内存中维护
   - ✅ 高性能进程间通信
   - 🔍 当前IPC通道数: {}
   - 💡 实时通信，零延迟

💾 4. 持久化策略：
   - ✅ 关键数据定期写入文件
   - ✅ 防止数据丢失
   - ✅ 应用重启时可恢复状态
   - 💡 平衡性能与数据安全

⚡ 5. 临时计算数据：
   - ✅ 仅在内存中处理
   - ✅ 不占用磁盘空间
   - ✅ 处理完即释放
   - 💡 OCR结果、检测结果等
", 
        100, // 假设缓存大小
        main_ctx.device_config.len(),
        main_ctx.ipc_channel.len()
    );

    println!("\n💡 最佳实践建议:");
    println!("
1. 🎯 根据数据访问模式选择存储策略
2. 📊 监控内存使用情况，调整缓存大小
3. 🔄 定期清理过期数据
4. 📈 使用指标监控系统性能
5. 🛡️ 实现优雅的错误恢复机制
    ");
}

/// 模拟的子进程启动示例
async fn demonstrate_child_process_launch() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔧 子进程启动示例:");
    
    // 这通常会在主进程中调用，启动设备子进程
    println!("
📝 启动命令示例:
  ./autodaily-child 1 device-process-1
  
📋 参数说明:
  - 参数1: 设备ID (1)
  - 参数2: 进程ID (device-process-1)

🔄 子进程初始化流程:
  1. 解析命令行参数
  2. 创建子进程上下文
  3. 建立与主进程的IPC连接
  4. 初始化设备上下文和模型
  5. 向主进程报告就绪状态
  6. 进入主循环处理任务
  7. 监听关闭信号，优雅退出
    ");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_script_pagination() {
        // 创建临时目录用于测试
        let temp_dir = tempfile::tempdir().unwrap();
        let scripts_dir = temp_dir.path().to_path_buf();
        
        // 创建脚本管理器
        let mut manager = ScriptManager::new(scripts_dir, 10);
        
        // 创建测试脚本
        let test_script = ScriptInfo {
            script_id: 1,
            script_name: "测试脚本".to_string(),
            script_type: "test".to_string(),
            script_path: "./test.js".to_string(),
            script_args: vec![],
            script_env: std::collections::HashMap::new(),
            script_log: "./test.log".to_string(),
            script_status: "ready".to_string(),
            script_create_time: "2024-01-01T00:00:00Z".to_string(),
            priority: 1,
            last_modified: "2024-01-01T00:00:00Z".to_string(),
            execution_count: 0,
        };

        // 保存并加载
        manager.save_script(&test_script).await.unwrap();
        manager.load_from_directory().await.unwrap();

        // 测试分页
        let request = ScriptPageRequest::default();
        let response = manager.get_scripts_page(request).unwrap();
        
        assert_eq!(response.total_count, 1);
        assert_eq!(response.scripts.len(), 1);
        assert_eq!(response.scripts[0].script_name, "测试脚本");
    }

    #[tokio::test] 
    async fn test_sorting() {
        // 类似的排序测试...
    }
}
