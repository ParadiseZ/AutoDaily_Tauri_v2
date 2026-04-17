# 子进程管理与日志系统 — 工作记录（已更新）

> 首轮实现日期: 2026-03-03 ~ 2026-03-05
> 本次更新目的: 补充分包拆解后的实际归属，修正文档中过时路径与“前端未接入”描述
> 当前状态: 子进程基础设施、日志链路、前端接入已完成主链路；执行器闭环仍未完成

---

## 1. 当前结论

这部分能力已经不是单体 `src-tauri/src/*` 结构了，当前真实状态是：

- `src-tauri/` 主 crate 负责：
  - Tauri 应用入口
  - `invoke_handler` 命令注册
  - 将部分 `domain/infrastructure` 重新导出给旧路径使用
- `src-tauri/crates/runtime_engine/` 负责：
  - 主进程运行时基础设施
  - SQLite
  - HTTP client
  - 主进程 IPC 处理
  - 设备/脚本/调度等核心 domain
- `src-tauri/crates/child_support/` 负责：
  - 子进程运行时
  - 子进程 IPC 处理
  - 子进程日志
  - 子进程调度器 / 执行器
  - 子进程上下文初始化
- `src-tauri/crates/runtime_common/` 负责：
  - core / ipc / logging 通用定义
- `src-tauri/crates/vision_core/` 负责：
  - 视觉/OCR/ORT 相关能力

也就是说，这份文档里凡是提到旧的 `src-tauri/src/infrastructure/...` 路径，都应优先理解为：

- 主进程实现多半在 `runtime_engine`
- 子进程实现多半在 `child_support`
- `src-tauri/src/*` 很多只是 re-export 或命令入口

---

## 2. 分包后的实际结构

### 2.1 Workspace

`src-tauri/Cargo.toml` 当前 workspace 成员：

- `.`
- `crates/child_support`
- `crates/runtime_common`
- `crates/vision_core`
- `crates/runtime_engine`

并且保留了独立子进程二进制：

- `[[bin]] name = "child"`
- `path = "src/main_child.rs"`
- `required-features = ["child-bin"]`

### 2.2 主 crate 与 re-export 关系

主 crate 真实职责现在偏薄：

- `src-tauri/src/lib.rs`
  - 注册 Tauri 插件
  - 注册全部 `invoke_handler`
  - 启动时调用 `init_at_start`
- `src-tauri/src/main.rs`
  - 启动主应用
- `src-tauri/src/main_child.rs`
  - 启动子进程
  - 直接依赖 `child_support`

几个关键 re-export：

- `src-tauri/src/infrastructure.rs`
  - 导出 `runtime_engine::infrastructure::*`
- `src-tauri/src/infrastructure/context.rs`
  - 导出 `runtime_engine::infrastructure::context::*`
- `src-tauri/src/domain.rs`
  - 导出主 domain 模块

因此：

- “从旧路径还能访问”不代表“实现还在旧路径”
- 查问题时，优先去 `runtime_engine` / `child_support` 看真实代码

---

## 3. 已完成能力

## 3.1 第一阶段：日志系统优化

已完成：

- 主进程日志级别动态更新
- 子进程日志级别动态更新
- 日志目录可配置
- 日志保留天数可配置
- 启动时 + 定时自动清理日志
- `log_to_file` 支持
- 前端 console/unhandled error 可桥接到 Rust 侧

当前代码归属：

- 主进程日志：
  - `src-tauri/crates/runtime_engine/src/infrastructure/logging/`
- 子进程日志：
  - `src-tauri/crates/child_support/src/infrastructure/logging/`
- 命令入口：
  - `src-tauri/src/api/infrastructure/config/log_api.rs`
- 前端页面：
  - `src/views/Settings.vue`
  - `src/views/Logs.vue`

## 3.2 第二阶段：子进程基础设施

已完成：

- IPC 消息模型收敛
- 主进程 IPC server
- 子进程 IPC client
- 子进程管理器
- 子进程启动/关闭/状态同步
- 主进程向前端转发状态和日志事件
- 前端任务页和日志页已接主链路

### IPC 消息类型

当前核心消息负载仍然是这几类：

| Payload | 用途 |
| --- | --- |
| `ProcessControl` | `Start / Stop / Pause / Shutdown` |
| `ScriptTask` | `Add / Remove / Execute` |
| `ConfigUpdate` | 日志级别、ADB 相关配置更新 |
| `StatusReport` | 子进程状态上报 |
| `Logger` | 子进程日志回传 |
| `Heartbeat` | 心跳 |
| `Error` | 异常上报 |

实际实现分布：

- 公共消息定义：
  - `src-tauri/crates/runtime_common/src/ipc/`
  - 或经 `runtime_engine / child_support` 再导出

### 主进程消息处理

实际文件：

- `src-tauri/crates/runtime_engine/src/infrastructure/ipc/msg_handler_main.rs`

当前行为：

- `Logger`
  - 写主进程接收器
  - emit 前端事件 `child-log`
- `StatusReport`
  - emit 前端事件 `device-status`
- `Error`
  - emit 前端事件 `device-error`

注意：

- 不是旧文档里写的 `child-log-{deviceId}`
- 当前前端统一监听的是 `child-log`

### 子进程消息处理

实际文件：

- `src-tauri/crates/child_support/src/infrastructure/ipc/msg_handler_child.rs`

当前行为：

- `ProcessControl`
  - `Start` -> 状态切到 `Running`
  - `Stop` -> 状态切到 `Idle`
  - `Pause` -> 状态切到 `Paused`
  - `Shutdown` -> 状态切到 `Stopping` 并触发取消
- `SessionControl`
  - `LoadSession` -> 替换当前 `RuntimeSessionSnapshot`，同步 scheduler 队列，并发 `Loaded / Idle`
  - `ReloadSession` -> 热更新当前 session 和 scheduler 队列
  - `PrepareCheckpoint` -> 在安全点保存 checkpoint，并发 `CheckpointReady / RestartReady`
  - `ClearSession` -> 清空当前 session 与 scheduler 队列
- `ConfigUpdate`
  - 已接日志级别
  - 已接 ADB 路径 / 服务地址热更新

### 子进程管理器

实际文件：

- `src-tauri/crates/runtime_engine/src/infrastructure/context/child_process_manager.rs`

当前功能：

- `spawn_child(init_data)`
- `stop_child(device_id)`
- `restart_child`
- `is_running`
- `get_running_device_ids`
- `stop_all`

当前实现方式：

- 使用 `std::env::current_exe()` 拉起当前二进制
- 通过 `--child` + 环境变量 `CHILD_CONTEXT_DATA` 区分子进程模式

### 子进程入口

实际文件：

- `src-tauri/src/main_child.rs`

当前逻辑：

1. 读取 `CHILD_CONTEXT_DATA`
2. 反序列化 `ChildProcessInitData`
3. 调 `child_support::...::init_environment`
4. 初始化 `CancellationToken`
5. 初始化 `ScriptScheduler`
6. 进入主循环

这部分已经不再是旧文档里的“单体 crate 内部逻辑”，而是显式依赖 `child_support`。

### 子进程调度器

实际文件：

- `src-tauri/crates/child_support/src/infrastructure/scripts/scheduler.rs`

当前已完成：

- 队列管理
  - `load_session`
  - `clear_session`
- 运行时查询
  - `current_script`
  - `queue_len`
- 主循环调度
  - `tick`
- 正式执行入口
  - `execute_script`
  - 从当前 session bundle 加载脚本定义
  - 通过 `ExecutionPlanAssembler::assemble(...)` 直接装配 `ExecutionPlan`
  - `ExecutionPlan` 当前区分：
    - `DeviceQueue(TaskSelection)`
    - `FullScript(TaskSelection)`
    - `Task(TaskSelection)`
    - `PolicyDebug`
  - `TaskSelection` 内部继续携带 root/linkable/skipped task
  - `ExecutionPlanSummary` 与 `PlannedTask / SkippedTask.record_schedule` 也在装配期生成
  - 逐 task 调用 `ScriptExecutor`
  - `scheduler` 直接消费 plan summary，并使用计划结果里的 `record_schedule`
  - 仅正式 `DeviceQueue` 且 `record_schedule = true` 时写 `ScheduleJournal`
  - `FullScript / Task` 调试运行也走同一条主链，不再依赖独立 `debug_execute`

当前未完成：

- `PolicyGroup / PolicySet / Policy` 已进入 child 的调试运行主链，但仍不属于 `DeviceQueue` 正式执行计划
- 非 `DeviceQueue` 运行目标仍使用临时 `RuntimeQueueItem`，作用域语义弱于正式调度

补充记录：

- 任务管理页开始区分：
  - `运行队列`：正式 `DeviceQueue`
  - `临时运行脚本 / 临时运行任务`：临时 `FullScript / Task`
- 临时运行目标不改 assignment 队列定义。
- 设备运行中切到临时目标时，前端会先确认，再发 `Pause + PrepareCheckpoint(manual)`。
- checkpoint 的 `updated_at` 视为恢复点设置时间；超过 1 天默认失效并清理。

---

## 4. 前端接入现状

旧文档这里已经过时，当前前端不是“尚未调用”，而是已经接入主流程了。

### 4.1 已接入页面

#### `src/views/TaskManagement.vue`

已接：

- `cmd_spawn_device`
- `cmd_device_start`
- `cmd_device_pause`
- `cmd_device_stop`
- `cmd_add_script_to_device`
- `cmd_remove_script_from_device`
- `cmd_get_running_devices`

通过 `deviceStore` 和 `taskStore` 完成：

- 设备启动/暂停/停止
- 设备队列增删
- 在线状态同步

#### `src/views/Logs.vue`

已接：

- `child-log` 实时日志事件
- `update_child_log_level_cmd`

#### `src/App.vue`

已接：

- `deviceStore.initIpcListeners()`
- `logsStore.initListener()`

#### `src/store/device.ts`

已监听：

- `device-status`
- `device-error`

#### `src/store/logs.ts`

已监听：

- `child-log`

### 4.2 仍未接完的前端部分

- `DeviceList.vue` 没有直接暴露“启动/关闭子进程”按钮
- assignment 重排 UI 没接
- 时间模板管理 UI 没接
- 编辑器页没接真实任务图编辑和保存

---

## 5. 本次分包拆解带来的实际变化

这次拆包之后，最大的收益和影响如下。

### 5.1 收益

- 主应用 crate 变薄，Tauri 入口和运行时实现分离更清楚
- 子进程逻辑集中在 `child_support`，不再与主进程逻辑混杂
- 公共定义下沉到 `runtime_common`
- 视觉/OCR 能力独立到 `vision_core`

### 5.2 对阅读代码的影响

后续排查时建议按这个顺序找：

1. 前端页面 / store / service
2. `src-tauri/src/api/*` 命令入口
3. `runtime_engine` 看主进程真实实现
4. `child_support` 看子进程真实实现
5. `runtime_common` 看共享结构

不要再只盯着 `src-tauri/src/infrastructure/*`，很多地方只是转发。

### 5.3 当前模块职责

#### `runtime_engine`

更偏主进程和共享主逻辑：

- app 启动初始化
- SQLite
- HTTP client
- 主进程日志
- 主进程 IPC
- 子进程管理器
- 设备/脚本/调度 domain

#### `child_support`

更偏子进程执行期：

- 子进程上下文
- 子进程日志
- 子进程 IPC 处理
- ADB 执行上下文
- 调度器
- 执行器

#### `runtime_common`

更偏通用基础结构：

- core
- ipc
- logging

#### `vision_core`

更偏视觉与模型：

- 图片处理
- OCR
- detection
- ORT 封装

---

## 6. 仍未完成的工作（当前真实版本）

## 6.1 脚本执行主链剩余边界

位置：

- `src-tauri/crates/child_support/src/infrastructure/scripts/scheduler.rs`
- `src-tauri/crates/child_support/src/infrastructure/scripts/execution_plan.rs`
- `src-tauri/crates/child_support/src/infrastructure/scripts/executor.rs`

当前状态：

- `execute_script()` 已经从 session bundle 读取脚本定义
- `ExecutionPlanAssembler` 已经收口为执行计划装配层：
  - 直接返回 `ExecutionPlan`
  - 统一提供 `ExecutionPlanSummary`
  - 在装配期确定 `record_schedule`
- `ScriptExecutor` 已真实执行动作、流程节点、策略节点的主链能力
- runtime progress / schedule event 已进入正式执行链路
- timeout 前进证据已不再只挂在动作后：
  - 已覆盖 `WaitMs / While / ForEach / HandlePolicySet / HandlePolicy`
  - 策略调试候选扫描也会进入同一条 detector 链

当前仍未收口：

- child 加载 `ResumeCheckpoint` 后，已经能按 `task / step` 做安全点恢复；当前剩余的是更细粒度作用域与恢复态展示继续补齐
- `PolicyGroup / PolicySet / Policy` 现已进入调试运行主链，但仍不属于 `DeviceQueue` 正式任务计划
- `ColorCompare` 等剩余条件/数据能力还没有接入真实执行
- `ExecutionPlanAssembler` 的剩余工作已经不是“把过滤器变成计划器”，而是继续补：
  - 恢复任务注入
  - 更复杂的补跑/跳过规则
  - 调试目标与正式运行目标更完整的语义统一

建议继续看：

- `src-tauri/crates/runtime_engine/src/domain/scripts/`
- `src-tauri/crates/child_support/src/infrastructure/scripts/executor.rs`

## 6.2 调试运行作用域仍弱于正式调度

位置：

- `src-tauri/src/api/infrastructure/process_api.rs`
- `build_runtime_session_snapshot()`
- `build_debug_template_values_json()`

当前状态：

- 编辑器调试运行 `FullScript / Task` 已走 `LoadSession -> Start -> scheduler.execute_script`
- 运行时会为当前脚本任务强制注入 `everyRun` 的 task-cycle 覆盖
- 调试运行不写 `device_script_schedules`，但保留运行日志与 runtime event
- `DeviceQueue` 正式运行已经真实消费 `RuntimeQueueItem.template_values_json`：
  - `ExecutionPlanAssembler` 会读取 `taskSettings.enabled / taskSettings.taskCycle`
  - `ScriptExecutor` 会读取 `variables` 并装入 input scope

当前仍未收口：

- 非 `DeviceQueue` 目标仍使用临时 `RuntimeQueueItem`
- `time_template_id / account_id / account_data_json` 仍未补成正式调度语义
- `build_debug_template_values_json()` 当前只补了最小 `everyRun` task-cycle 覆盖，不等于完整模板/账户作用域

## 6.3 运行时持久化还没做

位置：

- `src-tauri/crates/child_support/src/infrastructure/ipc/msg_handler_child.rs`

当前问题：

- `Stop`
- `Shutdown`

这两个分支还只有：

- 改状态
- 触发取消

但没有：

- 保存当前运行进度
- 保存运行时变量
- 补完整调度结果

## 6.4 子进程打包策略仍需最终验证

当前现状：

- workspace 中有独立 `child` bin
- `ChildProcessManager` 实际仍使用 `current_exe()` + `--child`

这代表当前模式是“逻辑上保留独立 child bin，运行上先沿用同一可执行入口区分子模式”。

需要继续确认：

- 打包产物里是否真的需要独立 child 可执行文件
- `--child` 模式是否已经在主二进制入口完整支持
- `child-bin` feature 在开发/打包链路中的真实使用方式

---

## 7. 当前相关文件速查

### 7.1 主入口

- `src-tauri/src/lib.rs`
- `src-tauri/src/main.rs`
- `src-tauri/src/main_child.rs`

### 7.2 命令入口

- `src-tauri/src/api/infrastructure/process_api.rs`
- `src-tauri/src/api/infrastructure/config/log_api.rs`

### 7.3 主进程真实实现

- `src-tauri/crates/runtime_engine/src/infrastructure/context/child_process_manager.rs`
- `src-tauri/crates/runtime_engine/src/infrastructure/ipc/msg_handler_main.rs`
- `src-tauri/crates/runtime_engine/src/infrastructure/db.rs`
- `src-tauri/crates/runtime_engine/src/infrastructure/http_client.rs`

### 7.4 子进程真实实现

- `src-tauri/crates/child_support/src/infrastructure/context/child_process.rs`
- `src-tauri/crates/child_support/src/infrastructure/context/child_process_sec.rs`
- `src-tauri/crates/child_support/src/infrastructure/ipc/msg_handler_child.rs`
- `src-tauri/crates/child_support/src/infrastructure/scripts/scheduler.rs`
- `src-tauri/crates/child_support/src/infrastructure/scripts/executor.rs`

### 7.5 前端接入

- `src/store/device.ts`
- `src/store/logs.ts`
- `src/views/TaskManagement.vue`
- `src/views/Logs.vue`

---

## 8. 对下个 AI 的提醒

- 先接受一个事实：这块代码已经拆包了，不能再按旧文档把所有实现都理解成在 `src-tauri/src/*`。
- 如果要继续做子进程/执行链路，优先看 `child_support`。
- 如果要继续做命令、DB、主进程事件转发，优先看 `runtime_engine`。
- 旧文档里“前端未接入”的说法已经不成立，任务页和日志页都已接主链路。
- 现在最值得继续投入的点，不是再整理框架，而是把 `scheduler -> script_tasks -> executor` 真正打通。
