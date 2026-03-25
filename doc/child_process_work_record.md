# 子进程管理与日志系统 — 工作记录

> 会话日期: 2026-03-03 ~ 2026-03-05
> 状态: 第二阶段主体完成，编译通过

---

## 第一阶段：日志系统优化 ✅（已完成）

### 完成内容

- **动态日志级别**：主进程（Settings页设置）和子进程（DeviceList/Logs页设置）均支持运行时切换
- **日志文件管理**：按 `{app_name}_{YYMMDD}.log`（主进程）/`{设备名}_{YYMMDD}.log`（子进程）命名
- **日志目录可配置**：Settings页设置，持久化到 `tauri-plugin-store`
- **自动清理**：启动 + 每6h扫描超期 `.log` 文件，保留天数可在 Logs 页设置
- **logToFile**：`DeviceConfig` 新增 `log_to_file: bool` 字段，禁用时仅 emit 到前端
- **Log::init_logger 修复**：确保自定义 `Log::info()` 等方法正确输出
- **设置持久化**：日志级别/目录/保留天数写入 `tauri-plugin-store`，子进程日志级别写入数据库

### 涉及文件

- `src-tauri/src/infrastructure/logging/` — config.rs, log_trait.rs, child_log.rs, main_process_log_handler.rs
- `src-tauri/src/api/infrastructure/config/log_api.rs`
- `src/views/Settings.vue`, `src/views/Logs.vue`, `src/views/DeviceList.vue`

---

## 第二阶段：子进程功能实现 ✅（主体完成）

### 1. IPC 消息精简

**文件**: `src-tauri/src/infrastructure/ipc/message.rs`

从 ~278 行精简至 ~200 行，移除未使用类型。保留 6 个核心 MessagePayload：

| Payload          | 用途                      |
| ---------------- | ------------------------- |
| `ProcessControl` | Start/Stop/Pause/Shutdown |
| `ScriptTask`     | Add/Remove/Execute        |
| `ConfigUpdate`   | LogLevel/LogToFile        |
| `StatusReport`   | 子进程状态上报            |
| `Logger`         | 子进程日志转发            |
| `Heartbeat`      | 心跳                      |

新增类型：`ConfigUpdateType`（LogLevel+LogToFile）、`ChildProcessStatus`枚举、`ScriptTaskAction::Execute`（开发者调试）

### 2. 消息处理器

| 文件                       | 功能                                                                        |
| -------------------------- | --------------------------------------------------------------------------- |
| `ipc/msg_handler_child.rs` | 子进程端：分发 ProcessControl/ScriptTask/ConfigUpdate，连接 ScriptScheduler |
| `ipc/msg_handler_main.rs`  | 主进程端：Logger→文件+前端事件、StatusReport→前端事件、Error→前端事件       |
| `ipc/chanel_server.rs`     | `handle_msg` 委托 `msg_handler_main`，`send_to_client` 公开                 |
| `ipc/chanel_client.rs`     | `handle_msg` 委托 `msg_handler_child`                                       |

### 3. 子进程管理器（主进程端）

**文件**: `src-tauri/src/infrastructure/context/child_process_manager.rs`

- `spawn_child(init_data)` — 通过 `tokio::process::Command` 启动子进程，env 传 `CHILD_CONTEXT_DATA`
- `stop_child(device_id)` — IPC 发 Shutdown → 5s超时 kill
- `restart_child`, `is_running`, `get_running_device_ids`, `stop_all`
- 全局单例 `OnceLock`，在 `init_at_start` 中初始化

### 4. 子进程主循环 + CancellationToken 优雅停止

**文件**: `src-tauri/src/main_child.rs`

```
启动流程:
env CHILD_CONTEXT_DATA → 反序列化 ChildProcessInitData
→ init_environment() (CPU亲和性/日志/数据库/IPC/ADB/运行时上下文)
→ init_scheduler(cancel_token)
→ set_running_status(Idle)
→ run_main_loop()
```

状态机:

- `Idle/Paused` → 500ms 轮询等待命令
- `Running` → `scheduler.tick()` 执行脚本队列，队列空回到 Idle
- `Stopping/Stopped/Error` → 退出主循环
- Ctrl+C / `ProcessAction::Shutdown` → `trigger_cancel()` 立即退出

**全局 CancellationToken**: `src-tauri/src/infrastructure/context/child_process_sec.rs`

- `init_cancel_token()` / `get_cancel_token()` / `trigger_cancel()`

### 5. 脚本调度器

**文件**: `src-tauri/src/infrastructure/scripts/scheduler.rs`

- `add_script(id)` / `remove_script(id)` / `clear_queue()` — 队列管理
- `tick()` — 取出下一个脚本执行，返回 false 表示队列空
- `debug_execute(script_id, target)` — 开发者调试，不走队列直接执行
- `current_script()` / `queue_len()` — 查询
- 全局单例 `OnceLock`

### 6. 前端 API

**文件**: `src-tauri/src/api/infrastructure/process_api.rs`

| 命令                                                  | 功能                                 |
| ----------------------------------------------------- | ------------------------------------ |
| `cmd_spawn_device(app_handle, device_id)`             | 从DB加载配置→构造InitData→启动子进程 |
| `cmd_device_start(device_id)`                         | 发送 Start 命令                      |
| `cmd_device_stop(device_id)`                          | 发送 Stop（回到Idle）                |
| `cmd_device_pause(device_id)`                         | 发送 Pause                           |
| `cmd_device_shutdown(device_id)`                      | 关闭子进程                           |
| `cmd_add_script_to_device(device_id, script_id)`      | 添加脚本到队列                       |
| `cmd_remove_script_from_device(device_id, script_id)` | 移除脚本                             |
| `cmd_get_running_devices()`                           | 查询全部运行设备                     |
| `cmd_is_device_running(device_id)`                    | 查询单设备状态                       |

已在 `lib.rs` `invoke_handler` 中注册。

### 7. 其他关键修改

| 文件                           | 修改                                                                    |
| ------------------------------ | ----------------------------------------------------------------------- |
| `Cargo.toml`                   | 添加 `tokio-util` 依赖                                                  |
| `context/child_process.rs`     | 移除旧 `ChildProcessCtx`，仅保留 `ChildProcessInitData`                 |
| `context/child_process_sec.rs` | 添加 `Stopping` 状态、`get_ipc_client` 返回 `Option`、CancellationToken |
| `scripts.rs` (mod)             | 添加 `pub mod scheduler`，`script_runtime` 改为 `pub`                   |
| `app/init_start.rs`            | IPC Server 启动后初始化 `ProcessManager`                                |
| `logging/child_log.rs`         | 修复 `get_ipc_client()` 返回 `Option` 后的解包                          |

---

## 未完成的工作（TODO）

### 1. 脚本实际加载逻辑

**位置**: `scheduler.rs` 的 `execute_script()` 方法

当前状态：只有占位逻辑，需要实现：

- 从数据库 `script_tasks` 表加载 `ScriptTaskTable`（nodes+edges）
- 将 VueFlow 的 nodes/edges 解析为 `Vec<Step>` 执行序列
- 加载脚本参数（`ScriptTask.variables`）到 `ScriptRuntime`
- 调用 `ScriptExecutor.execute(steps)` 执行

相关数据结构：

- `domain/scripts/script_task.rs` — `ScriptTaskTable`（id, script_id, nodes, edges, data）
- `domain/scripts/script_decision.rs` — `Step`/`StepKind` 定义
- `infrastructure/scripts/executor.rs` — `ScriptExecutor.execute(steps)`
- `infrastructure/scripts/script_runtime.rs` — `ScriptRuntime`（decision, back_decision, global_decision）

### 2. 开发者调试执行

**位置**: `scheduler.rs` 的 `debug_execute()` 方法

当前状态：只有占位逻辑，需要根据 `ExecuteTarget` 加载对应的步骤：

- `FullScript` → 加载所有 Main 类型任务
- `Task(task_id)` → 加载指定任务
- `PolicyGroup(pg_id)` → 加载指定策略组
- `PolicySet(ps_id)` → 加载指定策略集

### 3. 运行时数据持久化

**位置**: `msg_handler_child.rs` 的 `ProcessAction::Stop` 和 `ProcessAction::Shutdown` 分支

当前状态：已标记 `// TODO: 持久化运行时数据`
需要实现：

- 保存当前脚本执行进度到数据库
- 保存 ScriptRuntime 中的变量状态
- 停止时通知主进程当前执行状态

### 4. 前端界面接入

目前 9 个 Tauri 命令已注册可用，但前端页面尚未调用：

- `TaskManagement.vue` — 需接入 spawn/start/stop/pause/shutdown
- `DeviceList.vue` — 需接入启动/停止按钮
- `Logs.vue` — 子进程日志通过事件 `child-log-{deviceId}` emit，需订阅

### 5. 子进程二进制分离（可选）

当前 `child_process_manager.rs` 中 `spawn_child` 使用 `std::env::current_exe()` 获取路径并传 `--child` 参数，但 `main_child.rs` 同时也是独立的 `[[bin]]` target（当前名为 `child`，默认不参与构建，需要显式启用 `child-bin` feature）。需要验证：

- Tauri 打包后是否正确包含 `child` 二进制
- 开发模式下 `current_exe()` 路径是否正确指向 child binary
- 可能需要改为显式指定 child binary 路径

---

## 前端调用示例（TypeScript）

```typescript
import { invoke } from '@tauri-apps/api/core';

// 启动设备子进程
await invoke('cmd_spawn_device', { deviceId: 'uuid...' });

// 添加脚本到队列
await invoke('cmd_add_script_to_device', { deviceId: 'uuid...', scriptId: 'uuid...' });

// 开始执行
await invoke('cmd_device_start', { deviceId: 'uuid...' });

// 停止执行
await invoke('cmd_device_stop', { deviceId: 'uuid...' });

// 关闭子进程
await invoke('cmd_device_shutdown', { deviceId: 'uuid...' });

// 查询运行中的设备
const devices = await invoke('cmd_get_running_devices');
```

---

## 编译验证

- ✅ `cargo build` 全部编译通过 (exit code 0, 53 warnings均为unused)
