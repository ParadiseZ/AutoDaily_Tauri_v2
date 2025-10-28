// CPU核心分配器使用示例
// 展示如何使用第一阶段重构的CPU核心分配器

use auto_daily_lib::core::{
    allocator::{CpuCoreAllocator, AllocationError},
    strategies::{AllocationPolicy, Priority, AllocationType, WorkloadType, StrategyEvaluator},
    detection::{CpuTopologyDetector, CoreType},
    rayon_pool::{RayonPoolManager, RayonConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();
    
    println!("=== AutoDaily CPU核心分配器示例 ===\n");
    
    // 1. 检测CPU拓扑结构
    println!("1. 检测CPU拓扑结构:");
    let topology = CpuTopologyDetector::detect();
    println!("   物理核心数: {}", topology.physical_core_count());
    println!("   逻辑核心数: {}", topology.logical_core_count());
    println!("   是否混合架构: {}", topology.is_hybrid_architecture());
    
    if topology.is_hybrid_architecture() {
        println!("   性能核心: {:?}", topology.performance_cores());
        println!("   效率核心: {:?}", topology.efficiency_cores());
    }
    
    for core in &topology.physical_cores {
        println!("   核心 {}: {} (逻辑核心: {:?})", 
                core.core_id, core.core_type, core.logical_cores);
    }
    println!();
    
    // 2. 创建CPU核心分配器
    println!("2. 创建CPU核心分配器:");
    let allocator = CpuCoreAllocator::new();
    println!("   可用核心: {:?}", allocator.get_available_cores());
    println!();
    
    // 3. 测试不同的分配策略
    println!("3. 测试分配策略:");
    test_allocation_policies(&allocator).await?;
    
    // 4. 测试Rayon线程池集成
    println!("4. 测试Rayon线程池集成:");
    test_rayon_integration(&allocator).await?;
    
    // 5. 性能基准测试
    println!("5. 性能基准测试:");
    benchmark_allocator(&allocator).await?;
    
    println!("示例执行完成!");
    Ok(())
}

async fn test_allocation_policies(allocator: &CpuCoreAllocator) -> Result<(), AllocationError> {
    let topology = allocator.get_topology();
    
    // 测试高性能策略
    println!("   测试高性能策略:");
    let hp_policy = AllocationPolicy::HighPerformance {
        prefer_performance_cores: true,
        avoid_hyperthreading: false,
    };
    
    let evaluation = StrategyEvaluator::evaluate_policy(&hp_policy, topology);
    println!("     策略评估: {:?}", evaluation);
    
    // 分配核心
    let process_id = "high_perf_process".to_string();
    let allocation = allocator.allocate_cores(
        process_id.clone(),
        2,
        Priority::High,
        AllocationType::Exclusive,
    )?;
    
    println!("     分配结果: {}", allocation);
    
    // 释放核心
    let deallocated = allocator.deallocate_cores(&process_id)?;
    println!("     释放核心: {:?}", deallocated);
    println!();
    
    // 测试节能策略
    println!("   测试节能策略:");
    let power_policy = AllocationPolicy::PowerSaving {
        prefer_efficiency_cores: true,
        max_frequency_mhz: Some(2000),
        min_core_mode: true,
    };
    
    let process_id_2 = "power_save_process".to_string();
    let allocation_2 = allocator.allocate_cores(
        process_id_2.clone(),
        1,
        Priority::Low,
        AllocationType::Shared,
    )?;
    
    println!("     分配结果: {}", allocation_2);
    allocator.deallocate_cores(&process_id_2)?;
    println!();
    
    Ok(())
}

async fn test_rayon_integration(allocator: &CpuCoreAllocator) -> Result<(), Box<dyn std::error::Error>> {
    // 创建Rayon线程池管理器
    let pool_manager = RayonPoolManager::new();
    
    // 分配CPU核心
    let process_id = "rayon_test_process".to_string();
    let allocation = allocator.allocate_cores(
        process_id.clone(),
        4,
        Priority::Normal,
        AllocationType::Exclusive,
    )?;
    
    println!("   为进程分配核心: {:?}", allocation.allocated_cores);
    
    // 创建Rayon配置
    let rayon_config = RayonConfig::from_allocated_cores(&process_id, &allocation.allocated_cores);
    println!("   Rayon配置: {} 线程, 亲和性 {:?}", 
             rayon_config.thread_count, rayon_config.core_affinity);
    
    // 创建线程池
    let pool = pool_manager.create_pool(process_id.clone(), rayon_config)?;
    println!("   线程池创建成功");
    
    // 在线程池中执行计算任务
    let result = pool.install(|| {
        (0..1000000).into_iter().map(|i| i * i).sum::<i32>()
    });
    
    println!("   计算结果: {}", result);
    
    // 清理
    pool_manager.destroy_pool(&process_id)?;
    allocator.deallocate_cores(&process_id)?;
    
    println!("   线程池清理完成");
    println!();
    
    Ok(())
}

async fn benchmark_allocator(allocator: &CpuCoreAllocator) -> Result<(), AllocationError> {
    use std::time::Instant;
    
    println!("   进行分配性能基准测试...");
    
    let start = Instant::now();
    let mut process_ids = Vec::new();
    
    // 批量分配
    for i in 0..10 {
        let process_id = format!("bench_process_{}", i);
        let _allocation = allocator.allocate_cores(
            process_id.clone(),
            1,
            Priority::Normal,
            AllocationType::Shared,
        )?;
        process_ids.push(process_id);
    }
    
    let allocation_time = start.elapsed();
    println!("     批量分配 10 个进程耗时: {:?}", allocation_time);
    
    let start = Instant::now();
    // 批量释放
    for process_id in process_ids {
        allocator.deallocate_cores(&process_id)?;
    }
    
    let deallocation_time = start.elapsed();
    println!("     批量释放 10 个进程耗时: {:?}", deallocation_time);
    
    // 显示统计信息
    let stats = allocator.get_stats();
    println!("     分配统计: 总计={}, 成功={}, 失败={}, 平均耗时={}ms", 
             stats.total_allocations,
             stats.successful_allocations,
             stats.failed_allocations,
             stats.average_allocation_time_ms);
    
    println!();
    Ok(())
}

// 工作负载类型推荐测试
fn test_workload_recommendations() {
    println!("   工作负载策略推荐:");
    let topology = CpuTopologyDetector::detect();
    
    let workloads = [
        WorkloadType::CpuIntensive,
        WorkloadType::LowPower,
        WorkloadType::Balanced,
        WorkloadType::RealTime,
    ];
    
    for workload in &workloads {
        let recommended = StrategyEvaluator::recommend_policy(&topology, *workload);
        println!("     {:?}: {}", workload, recommended);
    }
}