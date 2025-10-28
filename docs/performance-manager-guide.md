# 进程管理器 (ProcessManager) 使用说明

## 概述

进程管理器是 AutoDaily 项目中用于管理子进程和 CPU 核心亲和性的模块，提供了强大的多进程管理功能。与线程管理不同，进程管理器可以启动独立的子进程并为它们设置 CPU 核心绑定和优先级。

## 主要功能

### 1. 系统信息获取
- 获取系统 CPU 核心数
- 查看可用的 CPU 核心 ID 列表
- 监控活跃进程数量

### 2. 进程管理
- 启动子进程并设置 CPU 核心亲和性
- 设置进程优先级
- 终止指定进程
- 获取进程输出信息
- 监控进程状态
- 清理已完成的进程

### 3. 性能优化
- 将计算密集型进程绑定到特定 CPU 核心
- 支持并行工作进程
- 提供不同优先级的进程管理
- 跨平台支持（Windows、Linux、macOS）

## Rust API 使用

### 基本使用

```rust
use crate::infrastructure::performance::{
    ProcessManager, 
    create_process_config, 
    create_process_config_with_core
};

// 创建进程管理器
let manager = ProcessManager::new();

// 获取系统信息
println!("CPU核心数: {}", manager.get_cpu_count());
println!("可用核心: {:?}", manager.get_available_core_ids());

// 创建基本进程
let config = create_process_config(
    "worker_process",
    "echo",
    vec!["Hello from subprocess!".to_string()]
);
let process_id = manager.spawn_process_with_affinity(config).unwrap();

// 等待一段时间让进程完成
std::thread::sleep(std::time::Duration::from_millis(100));

// 清理已完成的进程
manager.cleanup_finished_processes();
```

### 设置 CPU 核心亲和性

```rust
// 将进程绑定到特定的 CPU 核心
let config = create_process_config_with_core(
    "high_performance_process",
    "python",
    vec!["-c".to_string(), "import time; [i*i for i in range(1000000)]; time.sleep(1)".to_string()],
    0  // 绑定到核心 0
);
let process_id = manager.spawn_process_with_affinity(config).unwrap();

// 进程将在 CPU 核心 0 上执行
// 适合 CPU 密集型任务
```

### 并行处理示例

```rust
let manager = ProcessManager::new();
let cpu_count = manager.get_cpu_count();
let mut process_ids = Vec::new();

// 为每个 CPU 核心创建一个工作进程
for core_id in 0..cpu_count {
    let config = create_process_config_with_core(
        &format!("worker_{}", core_id),
        "python",
        vec![
            "-c".to_string(),
            format!("print('Worker {} running on core {}')", core_id, core_id)
        ],
        core_id
    );
    
    let process_id = manager.spawn_process_with_affinity(config).unwrap();
    process_ids.push(process_id);
}

// 等待所有进程完成
std::thread::sleep(std::time::Duration::from_secs(2));
manager.cleanup_finished_processes();
```

## Tauri 命令 API

### 前端调用示例

```javascript
import { invoke } from '@tauri-apps/api/core'

// 获取系统性能信息
const systemInfo = await invoke('get_system_performance_info')
console.log(JSON.parse(systemInfo))

// 启动测试进程
const processId = await invoke('start_test_process', {
    processName: 'my_test_process',
    program: 'python',
    args: ['-c', 'print("Hello from Python subprocess!")'],
    coreId: 0,  // 绑定到核心 0，设置为 null 则不绑定
    workingDir: null
})

// 终止进程
await invoke('terminate_process', { processId })

// 获取活跃进程信息
const activeProcesses = await invoke('get_active_processes_info')
console.log(JSON.parse(activeProcesses))

// 清理已完成的进程
await invoke('cleanup_finished_processes')

// 获取进程输出
const [stdout, stderr] = await invoke('get_process_output', { processId })
console.log('输出:', stdout)
console.log('错误:', stderr)
```

### 快速测试命令

```javascript
// 启动简单测试进程（跨平台兼容）
const simpleProcessId = await invoke('start_simple_test_process', {
    coreId: null  // 不绑定核心
})

// 启动CPU密集型进程
const cpuProcessId = await invoke('start_cpu_intensive_process', {
    coreId: 1,           // 绑定到核心 1
    durationSeconds: 5   // 运行 5 秒
})

// 启动多个并行进程
const processIds = await invoke('start_parallel_processes', {
    processCount: 4,           // 启动 4 个进程
    taskDurationSeconds: 3     // 每个进程运行 3 秒
})

console.log(`启动了 ${processIds.length} 个并行进程`)
```

## 最佳实践

### 1. CPU 密集型任务优化
- 将 CPU 密集型任务（如图像处理、机器学习推理）绑定到专用核心
- 避免在主线程执行长时间的计算任务
- 使用性能监控来确定最优的核心分配策略

### 2. 任务分配策略
```rust
// 示例：为不同类型的任务分配不同的核心
let cpu_count = manager.get_cpu_count();

// UI 线程使用核心 0
// OCR 处理使用核心 1
// YOLO 检测使用核心 2
// 其他任务使用剩余核心

if cpu_count >= 4 {
    // 多核系统：专门分配
    let ocr_core = 1;
    let yolo_core = 2;
    let general_core = 3;
} else {
    // 少核系统：共享使用
    let shared_core = cpu_count - 1;
}
```

### 3. 内存管理
- 在长时间运行的线程中及时清理内存
- 使用 `cleanup_finished_threads()` 定期清理已完成的线程
- 监控活跃线程数量，避免创建过多线程

### 4. 错误处理
```rust
// 始终处理可能的错误
match manager.spawn_thread_with_affinity(config, task) {
    Ok(thread_id) => {
        println!("线程启动成功: {}", thread_id);
        // 可以选择立即等待或者后续等待
    }
    Err(e) => {
        eprintln!("线程启动失败: {}", e);
        // 实施回退策略
    }
}
```

## 性能测试类型

### 1. CPU 密集型测试 (`cpu_intensive`)
- 执行大量数学计算
- 适合测试 CPU 核心绑定效果
- 可以观察不同核心的性能差异

### 2. 内存测试 (`memory_test`)  
- 频繁的内存分配和释放
- 测试内存带宽和延迟
- 适合验证内存访问模式

### 3. 睡眠测试 (`sleep_test`)
- 模拟 I/O 等待场景
- 测试线程调度开销
- 验证线程管理的正确性

## 注意事项

1. **核心 ID 验证**: 确保指定的核心 ID 在系统范围内（0 到 cpu_count-1）
2. **线程生命周期**: 记得等待或清理创建的线程，避免资源泄漏  
3. **性能监控**: 在生产环境中监控线程数量和 CPU 使用率
4. **平台兼容性**: CPU 核心亲和性在不同操作系统上的行为可能有差异
5. **权限要求**: 某些系统可能需要特殊权限才能设置 CPU 亲和性

## 故障排除

### 常见问题

1. **无法设置 CPU 亲和性**
   - 检查系统权限
   - 确认核心 ID 有效
   - 查看系统是否支持 CPU 亲和性

2. **线程创建失败**
   - 检查系统资源限制
   - 验证线程名称和配置参数
   - 查看错误日志获取详细信息

3. **性能没有提升**
   - 确认任务是 CPU 密集型的
   - 检查是否有其他进程争用相同核心
   - 尝试不同的核心分配策略

通过合理使用性能管理器，你可以显著提升 AutoDaily 应用在多核系统上的性能表现！