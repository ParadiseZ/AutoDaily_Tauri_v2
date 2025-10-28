# AutoDaily 第一阶段重构 - 新建文件结构文档

## 概述

本文档记录了在AutoDaily第一阶段重构过程中创建的所有新文件，包括其目录结构、作用和主要功能。

---

## 📁 核心模块文件结构

### 1. 核心模块 (`src-tauri/src/core/`)

这是第一阶段重构的核心模块，实现CPU核心分配和管理功能。

#### 📄 `src-tauri/src/core/mod.rs`
**作用**: 核心模块的入口文件和重新导出
**主要功能**:
- 定义模块结构和子模块
- 重新导出主要类型供外部使用
- 定义全局类型别名（DeviceId, ScriptId, ProcessId）
- 引入必要的依赖（AHash, UUID, SystemTime等）

#### 📄 `src-tauri/src/core/detection.rs`
**作用**: CPU核心检测模块
**主要功能**:
- 跨平台CPU拓扑结构检测（Windows/Linux/macOS）
- Windows混合架构支持（P-Core/E-Core识别）
- 物理核心和逻辑核心信息收集
- 核心频率检测和核心类型判断
- 超线程检测和核心映射关系

**核心结构体**:
- `PhysicalCore`: 物理核心信息
- `LogicalCore`: 逻辑核心信息
- `CoreType`: 核心类型枚举（Performance/Efficiency/Unknown）
- `HybridArchitectureInfo`: 混合架构信息
- `CpuTopology`: CPU拓扑结构
- `CpuTopologyDetector`: 拓扑检测器

#### 📄 `src-tauri/src/core/strategies.rs`
**作用**: CPU核心分配策略模块
**主要功能**:
- 多种分配策略实现（节能、高性能、负载均衡等）
- 策略评估和推荐系统
- 优先级管理和分配类型定义
- 工作负载类型识别和策略匹配

**核心枚举和结构体**:
- `AllocationPolicy`: 分配策略枚举
- `LoadBalanceStrategy`: 负载均衡策略
- `Priority`: 进程优先级
- `AllocationType`: 分配类型
- `StrategyEvaluator`: 策略评估器
- `PolicyApplicator`: 策略应用器

#### 📄 `src-tauri/src/core/allocator.rs`
**作用**: CPU核心分配器主模块
**主要功能**:
- 精确的CPU核心分配和管理
- 分配冲突检测和解决
- 动态重新分配支持
- 分配统计和性能监控
- 完整的错误处理机制

**核心结构体**:
- `CpuCoreAllocator`: 主分配器
- `CoreAllocation`: 核心分配记录
- `AllocationStats`: 分配统计信息
- `AllocationError`: 分配错误类型

#### 📄 `src-tauri/src/core/rayon_pool.rs`
**作用**: Rayon线程池集成和动态重建模块
**主要功能**:
- 与分配的CPU核心精确绑定的线程池
- 线程池动态重建支持
- CPU亲和性设置
- 线程池生命周期管理
- 配置验证和错误处理

**核心结构体**:
- `RayonPoolManager`: 线程池管理器
- `RayonConfig`: 线程池配置
- `PoolInfo`: 线程池信息
- `PoolManagerStats`: 管理器统计信息
- `PoolExecutor`: 便捷执行器

### 2. 进程管理模块 (`src-tauri/src/process/`)

#### 📄 `src-tauri/src/process/mod.rs`
**作用**: 进程管理模块入口
**主要功能**:
- 定义进程管理相关的模块结构
- 重新导出进程管理类型
- 引入必要的依赖

**子模块**:
- `manager`: 主进程管理器
- `handle`: 进程句柄管理
- `config`: 进程配置
- `monitor`: 性能监控
- `lifecycle`: 生命周期管理

#### 📄 `src-tauri/src/process/handle.rs`
**作用**: 进程句柄管理模块
**主要功能**:
- 进程句柄、状态和优先级管理
- 进程类型定义（设备、Node.js、服务、工作进程）
- 进程健康检查和超时检测
- 进程生命周期状态跟踪

**核心结构体**:
- `ProcessHandle`: 通用进程句柄
- `DeviceProcessHandle`: 设备进程句柄
- `NodeProcessHandle`: Node.js进程句柄
- `ProcessState`: 进程状态枚举
- `ProcessPriority`: 进程优先级枚举
- `ProcessType`: 进程类型枚举

#### 📄 `src-tauri/src/process/config.rs`
**作用**: 进程配置模块
**主要功能**:
- 进程配置、资源约束和重启策略定义
- 为不同类型进程创建专门的配置
- 配置验证和启动命令生成
- 健康检查配置和日志配置

**核心结构体**:
- `ProcessConfig`: 进程配置主结构
- `RestartPolicy`: 重启策略枚举
- `ResourceConstraints`: 资源约束配置
- `HealthCheckConfig`: 健康检查配置
- `ProcessLogConfig`: 进程日志配置

#### 📄 `src-tauri/src/process/monitor.rs`
**作用**: 性能监控模块
**主要功能**:
- 监控进程的CPU、内存、网络等资源使用情况
- ORT推理性能统计（检测、识别延迟等）
- 进程健康评分计算
- 性能历史记录和趋势分析

**核心结构体**:
- `ProcessMonitor`: 进程监控器
- `PerformanceMetrics`: 详细性能指标
- `InferenceMetrics`: 推理性能指标
- `PerformanceSummary`: 性能摘要
- `InferenceType`: 推理类型枚举

#### 📄 `src-tauri/src/process/lifecycle.rs`
**作用**: 生命周期管理模块
**主要功能**:
- 管理进程的启动、停止、重启和清理
- 进程状态转换验证
- 优雅关闭和强制终止
- 启动超时和健康检查

**核心结构体**:
- `ProcessLifecycleManager`: 生命周期管理器
- `LifecycleError`: 生命周期错误类型

#### 📄 `src-tauri/src/process/manager.rs`
**作用**: 主进程管理器
**主要功能**:
- 统一管理所有子进程的创建、监控和销毁
- 集成CPU核心分配器和Rayon线程池管理器
- 设备进程和Node.js进程的创建和管理
- 全局状态监控和资源协调

**核心结构体**:
- `MainProcessManager`: 主进程管理器
- `GlobalState`: 全局进程状态
- `MemoryManager`: 内存管理器

### 3. IPC通信模块 (`src-tauri/src/ipc/`)

#### 📄 `src-tauri/src/ipc/mod.rs`
**作用**: IPC通信框架模块入口
**主要功能**:
- 定义IPC通信相关的模块结构
- 重新导出IPC通信类型
- 引入异步和序列化依赖

**计划子模块**:
- `manager`: IPC通信管理器
- `channel`: IPC通道
- `message`: 消息定义
- `router`: 消息路由器
- `serializer`: 消息序列化

### 4. 共享内存模块 (`src-tauri/src/shared/`)

#### 📄 `src-tauri/src/shared/mod.rs`
**作用**: 共享内存模型管理器模块入口
**主要功能**:
- 定义共享内存相关的模块结构
- 重新导出共享内存管理类型
- 支持MD5模型管理和内存映射

**子模块**:
- `model_info`: 模型信息管理
- `memory_map`: 内存映射包装器
- `access_control`: 访问控制模块
- `store`: 共享模型存储管理器

#### 📄 `src-tauri/src/shared/model_info.rs`
**作用**: 模型信息管理模块
**主要功能**:
- MD5作为模型唯一标识的ModelInfo结构体
- 支持det/rec分离设计的ModelType枚举
- 模型配置验证和文件MD5计算
- ModelRegistry用于模型注册和管理
- 完整的异步模型操作支持

**核心结构体**:
- `ModelInfo`: 模型基本信息
- `ModelType`: 模型类型枚举（Detection/Recognition）
- `ModelRegistry`: 模型注册表
- `ModelConfig`: 模型配置信息

#### 📄 `src-tauri/src/shared/memory_map.rs`
**作用**: 内存映射包装器模块
**主要功能**:
- 提供跨进程安全的内存映射文件访问
- 支持只读和读写两种模式
- 进程引用计数和自动清理
- 内存映射统计和监控
- 内存映射管理器统一管理所有映射

**核心结构体**:
- `MemoryMap`: 内存映射包装器
- `MemoryMapManager`: 内存映射管理器
- `MemoryMapStats`: 内存映射统计信息

#### 📄 `src-tauri/src/shared/access_control.rs`
**作用**: 访问控制模块
**主要功能**:
- 多进程安全访问控制，防止并发冲突
- 支持只读、读写、管理员三种访问级别
- 并发限制和等待队列管理
- 访问权限过期和自动清理
- 管理员进程特权管理

**核心结构体**:
- `AccessController`: 访问控制管理器
- `AccessPermission`: 访问权限记录
- `AccessRequest`: 访问请求
- `AccessLevel`: 访问级别枚举
- `AccessStats`: 访问统计信息

#### 📄 `src-tauri/src/shared/store.rs`
**作用**: 共享模型存储管理器
**主要功能**:
- 完整的共享模型存储解决方案
- LRU缓存策略和自动内存管理
- MD5模型管理，支持模型版本检测
- 模型预加载和按需加载
- 存储统计和性能监控

**核心结构体**:
- `SharedModelStore`: 共享模型存储管理器
- `CachePolicy`: LRU缓存策略配置
- `StoreStats`: 存储统计信息
- `CleanupResult`: 清理结果
- `CacheEntry`: 缓存项（内部使用）
- `model_info`: 模型信息管理
- `access_control`: 访问控制模块

### 5. 配置管理模块 (`src-tauri/src/config/`)

重构后的配置管理系统，改进初始化、读取和错误处理。

#### 📄 `src-tauri/src/config/mod.rs`
**作用**: 配置管理模块入口
**主要功能**:
- 定义配置管理相关的模块结构
- 重新导出配置管理类型

#### 📄 `src-tauri/src/config/manager.rs`
**作用**: 配置管理器（临时实现）
**主要功能**:
- 保持向后兼容的配置管理接口
- 内存缓存管理
- 配置加载和保存

#### 📄 `src-tauri/src/config/entities.rs`
**作用**: 配置实体定义
**主要功能**:
- 系统配置结构体定义
- 日志配置结构体定义
- 性能配置结构体定义
- 默认值实现

#### 📄 `src-tauri/src/config/storage.rs`
**作用**: 配置存储层（临时实现）
**主要功能**:
- 配置存储接口定义
- 配置类别trait定义

#### 📄 `src-tauri/src/config/validation.rs`
**作用**: 配置验证（临时实现）
**主要功能**:
- 配置有效性验证

### 6. 日志系统模块 (`src-tauri/src/logging/`)

优化的日志系统，支持结构化日志和批量处理。

#### 📄 `src-tauri/src/logging/mod.rs`
**作用**: 日志系统模块入口
**主要功能**:
- 定义日志系统相关的模块结构
- 重新导出日志类型

#### 📄 `src-tauri/src/logging/logger.rs`
**作用**: 日志记录器（临时实现）
**主要功能**:
- 基础的日志记录功能
- 日志级别定义
- 与tracing集成

#### 📄 `src-tauri/src/logging/config.rs`
**作用**: 日志配置
**主要功能**:
- 日志配置结构体定义
- 日志级别和参数配置

#### 📄 `src-tauri/src/logging/handler.rs`
**作用**: 日志处理器（临时实现）
**主要功能**:
- 文件日志处理器
- 控制台日志处理器

#### 📄 `src-tauri/src/logging/batch.rs`
**作用**: 批量日志处理（临时实现）
**主要功能**:
- 批量日志处理器

---

## 📁 示例和文档文件

### 1. 示例文件

#### 📄 `examples/cpu_core_allocator_example.rs`
**作用**: CPU核心分配器完整使用示例
**主要功能**:
- 展示CPU拓扑检测功能
- 演示不同分配策略的使用
- 展示Rayon线程池集成
- 性能基准测试示例
- 完整的错误处理示例

**示例内容**:
- CPU拓扑结构检测和显示
- 分配策略测试（高性能、节能策略）
- Rayon线程池创建和任务执行
- 分配性能基准测试
- 工作负载策略推荐

### 2. 文档文件

#### 📄 `REFACTOR_PROGRESS.md`
**作用**: 重构进度追踪文档
**主要内容**:
- 重构目标和背景说明
- 项目结构重构记录
- 各阶段完成情况追踪
- 技术亮点和性能优化效果
- 示例使用方法
- 下一步计划

#### 📄 `README.md` (更新)
**作用**: 项目主文档
**主要更新内容**:
- 项目重构状态说明
- 技术栈和架构特点
- 开发环境设置
- 快速开始指南
- 示例运行方法
- 性能优化特点

---

## 🔧 配置文件更新

### 📄 `src-tauri/Cargo.toml` (更新)
**更新内容**:
- 添加新的依赖项：
  - `memmap2`: 内存映射支持
  - `sysinfo`: 系统信息获取
  - `xxhash-rust`: 高性能哈希
  - `interprocess`: 进程间通信

---

## 📊 文件统计

### 新创建文件数量
- **核心模块文件**: 5个
- **进程管理文件**: 6个 (新增)
- **其他模块入口文件**: 4个
- **配置管理文件**: 4个  
- **日志系统文件**: 4个
- **示例文件**: 1个
- **文档文件**: 2个
- **总计**: 26个新文件

### 文件类型分布
- **Rust源码文件**: 23个
- **Markdown文档**: 2个
- **配置文件更新**: 1个

### 代码行数统计（估算）
- **核心模块总计**: ~2500行
- **进程管理模块**: ~2000行 (新增)
- **其他模块**: ~200行
- **示例代码**: ~300行
- **文档**: ~1200行
- **总计**: ~6200行

---

## 🎯 重构成果总结

### 完成的功能模块
1. ✅ **CPU核心分配器系统** - 完整实现
2. ✅ **Rayon线程池集成** - 完整实现
3. ✅ **分配策略系统** - 完整实现
4. ✅ **进程管理系统** - 完整实现 (新增)
5. ✅ **项目架构重组** - 完整实现
6. 🔄 **配置和日志系统** - 临时实现，保持兼容

### 技术特色
- **跨平台支持**: Windows/Linux/macOS
- **混合架构支持**: Intel 12代+ P-Core/E-Core
- **策略化分配**: 多种智能分配策略
- **动态重建**: 运行时核心重新分配
- **深度集成**: Rayon线程池与CPU核心精确绑定

### 为下一阶段铺路
- 建立了清晰的模块架构
- 提供了完整的示例和文档
- 保持了向后兼容性
- 为多进程架构奠定了基础

---

*文档创建时间: 2025-09-02*  
*对应重构阶段: 第一阶段 - 多进程架构基础设施*