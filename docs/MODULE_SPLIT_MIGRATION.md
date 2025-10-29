# 模块拆分迁移指南

## 概述

本指南帮助你将现有的单体Tauri项目拆分为多个独立的crate，以解决rust-analyzer内存占用过大的问题。

## 迁移步骤

### 1. 代码迁移顺序

建议按以下顺序迁移代码：

1. **core模块** - 最基础的模块，无其他依赖
2. **domain模块** - 只依赖基础库
3. **infrastructure模块** - 依赖core和domain
4. **app模块** - 依赖以上所有模块
5. **api模块** - 依赖以上所有模块和Tauri
6. **main模块** - 整合所有模块

### 2. 具体迁移映射

#### Core模块映射
```
src/core/*          → crates/core/src/cpu/*
src/process/*       → crates/core/src/process/*
src/memory/*        → crates/core/src/memory/*
```

#### Domain模块映射
```
src/domain/config/* → crates/domain/src/config/*
src/domain/scripts/* → crates/domain/src/scripts/*
src/domain/vision/* → crates/domain/src/vision/*
```

#### Infrastructure模块映射
```
src/infrastructure/config/* → crates/infrastructure/src/config/*
src/infrastructure/logging/* → crates/infrastructure/src/logging/*
src/infrastructure/ipc/* → crates/infrastructure/src/ipc/*
src/infrastructure/shared/* → crates/infrastructure/src/shared/*
src/infrastructure/vision/* → crates/infrastructure/src/vision/*
src/infrastructure/devices/* → crates/infrastructure/src/devices/*
src/infrastructure/capture/* → crates/infrastructure/src/capture/*
```

#### App模块映射
```
src/app/* → crates/app/src/*
```

#### API模块映射
```
src/api/* → crates/api/src/
src/command.rs → crates/api/src/commands.rs
```

### 3. 依赖关系调整

#### 修改导入路径
```rust
// 之前
use crate::infrastructure::config::ConfigManager;

// 之后
use auto_daily_infrastructure::config::ConfigManager;
```

#### 处理循环依赖
- 确保依赖关系是单向的：core → domain → infrastructure → app → api → main
- 如果出现循环依赖，考虑将共享类型提取到core或domain模块

### 4. 重量级依赖隔离

将重量级依赖集中在infrastructure crate中：

```rust
// 在infrastructure/Cargo.toml中
ort = { workspace = true }  # ONNX Runtime
image = { workspace = true }  # 图像处理
```

### 5. rust-analyzer配置优化

在项目根目录创建`.rust-analyzer.toml`：

```toml
[cargo]
features = "all"

[procMacro]
enable = true

[server]
path = "rust-analyzer"

[diagnostics]
enable = true

[hover]
enable = true

[inlayHints]
enable = true

[lens]
enable = true

# 限制分析范围，减少内存占用
files.excludeDirs = ["target", "node_modules", "dist"]
```

### 6. VS Code设置优化

在`.vscode/settings.json`中添加：

```json
{
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.imports.granularity.group": "module",
  "rust-analyzer.imports.prefix": "crate",
  "rust-analyzer.completion.addCallParentheses": true,
  "rust-analyzer.completion.addCallArgumentSnippets": true,
  "rust-analyzer.inlayHints.typeHints.enable": true,
  "rust-analyzer.inlayHints.parameterHints.enable": true,
  "rust-analyzer.lens.enable": true,
  "rust-analyzer.lens.run.enable": true,
  "rust-analyzer.lens.debug.enable": true,
  "rust-analyzer.lens.implementations.enable": true,
  "rust-analyzer.lens.references.enable": true,
  "rust-analyzer.lens.methodReferences.enable": true,
  "rust-analyzer.lens.enumVariantReferences.enable": true
}
```

## 优势

### 1. rust-analyzer内存占用降低
- 每个crate独立分析，不需要一次性加载所有代码
- 重量级依赖隔离在特定crate中，减少分析负担

### 2. 编译速度提升
- 增量编译更有效，只重新编译修改的crate
- 并行编译多个crate

### 3. 代码组织更清晰
- 明确的模块边界和依赖关系
- 更好的代码复用性

### 4. 团队协作更高效
- 不同开发者可以专注于不同模块
- 减少代码冲突

## 注意事项

1. **公共API设计**：精心设计每个crate的公共API，避免频繁跨边界修改
2. **版本兼容性**：内部crate版本保持同步，避免兼容性问题
3. **测试策略**：为每个crate编写独立测试，确保模块边界正确
4. **性能监控**：迁移后监控rust-analyzer内存使用情况，确保优化效果

## 故障排除

### 编译错误
1. 检查依赖路径是否正确
2. 确保公共API正确导出
3. 验证Cargo.toml配置正确

### rust-analyzer问题
1. 重启rust-analyzer服务器
2. 清理target目录：`cargo clean`
3. 重新生成Cargo.lock：`rm Cargo.lock && cargo check`

### 运行时错误
1. 确保所有必要的资源文件路径正确
2. 检查初始化顺序是否正确
3. 验证依赖注入是否正确配置