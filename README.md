# AutoDaily - 智能自动化工具

一个基于Tauri的跨平台自动化工具，专注于提供高性能的视觉识别和设备自动化功能。

## 🚀 项目状态

**当前版本**: v0.1.0  
**重构阶段**: 第一阶段进行中 (多进程架构基础设施)  
**完成进度**: CPU核心分配器 ✅ | 进程管理器 🔄 | IPC通信 📋 | 配置管理 🔄

## 📋 重构进展

项目正在进行第一阶段重构，建立多进程架构的技术基础。详细进展请查看 [重构进度文档](docs/REFACTOR_PROGRESS.md)。

### 已完成模块
- ✅ **CPU核心分配器**: 支持Windows混合架构(P-Core/E-Core)检测，多种分配策略
- ✅ **Rayon线程池集成**: 线程与CPU核心精确绑定，动态重建支持
- ✅ **分配策略系统**: 节能、高性能、负载均衡等多种策略

### 进行中模块
- 🔄 **进程管理器**: 多进程架构和生命周期管理
- 🔄 **配置管理**: 改进的配置读取和错误处理
- 🔄 **日志系统**: 结构化日志和批量处理

## 🛠️ 技术栈

**前端**: Vue 3 + Vite + Element Plus  
**后端**: Rust + Tauri  
**AI/视觉**: ONNX Runtime + OpenCV  
**并发**: Rayon + Tokio  
**架构**: 多进程 + IPC通信

## 🔧 开发环境

### 推荐IDE设置
- [VS Code](https://code.visualstudio.com/) + [Volar](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

### 开发要求
- Node.js 18+
- Rust 1.70+
- pnpm (推荐) 或 npm

## 🚀 快速开始

```bash
# 克隆项目
git clone <repository-url>
cd AutoDaily

# 安装前端依赖
pnpm install

# 开发模式运行
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

## 📖 示例和测试

```bash
# 运行CPU核心分配器示例
cd src-tauri
cargo run --example cpu_core_allocator_example

# 检查代码编译
cargo check

# 运行测试
cargo test
```

## 🏗️ 架构特点

### 第一阶段重构目标
1. **多进程架构**: 设备进程独立运行，故障隔离
2. **CPU核心精确分配**: 支持P-Core/E-Core，避免线程竞争
3. **高性能IPC**: 异步消息传递和路由管理
4. **共享内存模型**: 跨进程模型物理内存共享

### 性能优化
- **ORT推理优化**: CPU核心绑定避免线程竞争
- **物理隔离**: 进程间CPU资源完全隔离
- **负载均衡**: 智能的核心分配策略
- **资源利用率**: 优化的核心使用效率

## 📚 文档

- [第一阶段详细开发计划](./doc/第一阶段详细开发计划.md)
- [重构进度文档](docs/REFACTOR_PROGRESS.md)
- [API文档](./docs/) (即将提供)

## 🤝 贡献

欢迎提交Issue和Pull Request。请确保：
1. 遵循项目的代码规范
2. 添加适当的测试
3. 更新相关文档

## 📄 许可证

本项目采用 [MIT 许可证](./LICENSE)。
