# Rust 工作区重构执行计划

本文记录本轮工作区重构的目标、已确认问题、迁移顺序和验收条件。它取代《当前项目认知模型与边界审查》中“无需再进行整体架构重做”的结论；产品运行模型仍以该文档为参考。

## 目标

让每个公开结构体、枚举和函数按其业务语义与生命周期拥有明确归属，而不是按文件来源、名称关键词或是否直接使用第三方库归类。重构完成后，维护者应能按概念自然找到模型，并能从依赖方向判断运行边界。

## 已确认的约束

1. 采用逐类型的三步法：审查归属；复用已有包或新建必要包并移动；排查旧链和编译错误。
2. 包的粒度以业务概念、生命周期、复用与维护可定位性决定；包小不是拒绝拆分的理由。
3. 可在迁移前重命名包、模块、结构体、函数；名称必须反映真正拥有者。
4. Rust 模块统一使用 `foo.rs + foo/`，不新增 `mod.rs`。
5. API 必须先区分 `api/local` 与 `api/server`，再按领域划分模块。
6. 已被替代的旧包和旧链应删除；验证只覆盖当前新包及组合根，不验证已废弃包。
7. Windows 验证中若出现 `rustc.exe` 的 `0xc0000017`，先排查 Cargo 构建布局与环境，不把它误判为源码错误。

## 归属判定规则

| 所属层 | 判断方式 | 例子 |
| --- | --- | --- |
| `domain-*` | 更换 Tauri、ADB、SQLite、子进程实现后仍成立的业务概念、规则或稳定能力 | 设备配置、设备能力、脚本定义、时间窗口 |
| `api/local`、`api/server` | 本地 UI 或远程服务的输入、输出、事件契约与用例协调 | Tauri command、主进程发出的 UI 事件 |
| `child-runner` | 仅在子进程启动、会话或执行循环中存在的状态和组合流程 | 子进程初始化错误、任务/策略运行状态 |
| `runner-protocol` | 实际跨主进程与 child 进程传输的协议类型 | IPC message、channel error、跨进程运行状态 |
| `infra-*` | 连接外部技术、持久化、进程、窗口捕获、ADB、模型运行时的适配细节 | ADB 连接配置、SQLite repository、截图实现 |

第三方依赖不是划分依据：一个不依赖第三方库的 ADB 连接配置仍是基础设施；一个使用 `serde` 的设备能力模型仍可属于领域层。

## 当前已确认的待办

### P1：消除错误的执行类收纳包（已完成）

1. 审查并拆解 `application-execution`。
   - `Device*EventPayload` 与协议转换是主进程本地 API 事件契约，迁入 `api/local/execution`。
   - 经调用链确认，`RunningStatus` 未跨进程传输，已迁入 `child-runner`；主进程中未读取的同名缓存字段已删除。
   - `ChildRuntimeInitError` 与 `ChildRuntimeInitResult` 迁入 `child-runner`。
   - 移除 `infra-device-runtime -> application-execution` 的反向依赖：设备运行时返回自身错误，child 启动流程负责映射。
   - 包清空后删除 `application-execution`。
2. 审查并拆解 `domain-execution`。
   - `TaskCycle` 归入脚本任务模型。
   - `RunStatus` 归入调度运行记录，并改为能表明对象的名称。
   - `PolicyState`、`ActionState`、`TaskState` 和策略绑定操作归入 `child-runner`。
   - 包清空后删除 `domain-execution`。

### P2：校正设备运行时与 ADB 边界（已完成）

1. 将与 ADB、窗口捕获无关的 `DeviceOperation` 迁入 `domain-device`。其通用坐标 `Point<T>` 已迁入 `ad-kernel`，避免 `domain-device -> domain-script` 的反向依赖。
2. 保留 `infra-adb` 作为 ADB 适配器：`ADBCommand`、`ADBExecutor`、`ADBCtx`、`ADBConnectConfig`、ADB 错误和 shell 命令构造不创建 `domain-adb`。
3. 审查 `infra-device-runtime`：运行时实现、设备上下文、桌面/Android 实现保留；已删除无消费者的 `AdbInfo`/`AdbConnectStatus`；child 启动编排已从适配器错误中解耦。
4. 对 `device_launcher` 按函数判断：外部进程/ADB 探测保留基础设施，child 启动流程协调留在 `child-runner`。

### P3：全量逐类型审查（已完成；以最终复扫结果为准）

依次审查 `domain-*`、`app-*`、`infra-*`、`child-runner` 和组合根的公开类型及其调用者。每项记录为：当前定义、调用者、目标归属、是否改名、旧链删除情况。重点排查：

1. 领域包是否混入运行时状态或技术配置。
2. 基础设施包是否暴露应由领域拥有的稳定能力模型。
3. 应用/API 包是否承载本应在领域或 child 内部的模型。
4. `runner-protocol` 是否仅包含真实跨进程合同。
5. 空包、孤儿文件、重复类型和已无消费者的兼容层。

本阶段的硬门槛：未列出一个公开类型的定义处、直接调用者、目标所有者和旧链处理方式前，不得标记该包或该阶段完成；`cargo check` 只能验证迁移结果，不能作为归属依据。

为避免把边界问题留给人工复查，本轮交付采用下列闭环，而非“迁移后等待审查”的方式：

1. **全量台账先行**：对每个 `pub` 类型和导出函数记录定义、创建者、直接消费者、持久化/IPC/UI 合同、目标所有者和旧链处置；没有记录即不迁移、更不宣布完成。
2. **按所有权批量迁移**：同一概念的类型、构造函数、转换和调用者在同一批完成；若名称掩盖生命周期，先改名再移动。不得为了局部编译临时复制模型或新增兼容 re-export。
3. **迁移后反向审计**：搜索旧包名、旧模块名、同名定义、无消费者导出和反向依赖；逐项删除旧链、空包、空 re-export 与过期 TS 生成入口。
4. **双重验收**：先用台账确认语义所有权，再以受影响的新包、组合根、格式化、TS 类型检查验证；编译通过但台账、反向搜索或依赖方向任一项不通过，均视为未完成。

因此，最终目标不是“目录被拆开”，而是：**我能够在交付时给出完整的公开类型所有权清单、每项迁移证据和零旧链搜索结果，使你只需抽查结论，而不是替我发现遗漏。**

本轮已对全部 workspace crate 的 `pub fn`、`pub struct`、`pub enum`、`pub trait`、`pub type` 做过“crate 外文本消费者”候选扫描。删除并收窄无消费者项后，扫描仅余下下列 **间接合同**：`DeviceExecutionPolicy`、`EmailProviderPreset`、`ResolvedSmtpServer`、`TimeOfDayError`、`ScriptAccessError`、`ScriptPlatform`、`ScriptRuntimeSettings`、`IdleAction`、`StablePoint`、`RelativeValueType`、`RelativeCompareOp`、`SearchScope`、`LoggerInitError`、`LoggerInitResult`、`TextRecCacheError`、`TextRecCacheResult`、`ImageResult`。它们均在本计划对应台账中以公开字段、serde/TS 合同或公开函数签名说明保留理由；扫描结果不存在未解释的公开函数。该扫描用于发现候选，不以“无具名引用”替代所有权判断。

最近一次复扫的可复核结果为：各 crate 的 `pub fn` crate 外候选为 **0**；根 `src` 的 293 个 `pub fn/struct/enum/trait/type` 无源码消费者候选为 **0**；crate 外类型候选仅为上述 17 项。该轮删除了未接入的 `Rect`、颜色分析模块、ADB loop/config 修改 helper、IPC response helper、窗口截图 option wrapper、设备捕获方式切换整条转发链，以及根组合层的 10 个无调用方法；仍有 crate 内调用的实现方法统一收窄为 `pub(crate)`，而不是以伪公共 API 留存。

首批已审计台账（`app_schedule` / `app_script`）：

| 公开类型 | 直接调用者 | 目标所有者 | 处理 |
| --- | --- | --- | --- |
| `AssignmentProfile`、`TimeTemplateProfile`、`TemplateValueProfile` | 本地调度 API、运行队列装载、SQLite repository | `domain-schedule` | 迁移，保留现有序列化名称以避免无业务收益的前端改名 |
| `ExecutionScheduleProfile`、`AssignmentScheduleProfile`、`PlannerQueueItem` | child 执行日志、调度 API、planner、SQLite repository | `domain-schedule` | 迁移；它们是调度账本和调度计划，不是用例协调对象 |
| `ScriptProfile`、`PolicyProfile`、`PolicyGroupProfile`、`PolicySetProfile`、`ScriptTaskProfile`、关联类型和任务枚举 | 本地/服务端脚本 API、child 脚本执行、SQLite repository | `domain-script` | 迁移；它们组成可持久化、可装载的脚本图 |
| `ScriptTransferRecord` | 本地传输记录 API、服务端市场传输、SQLite repository | `domain-script` | 迁移；它是脚本发布/下载历史的持久模型 |
| `CreateScriptTransferRecordInput`、`FinishScriptTransferRecordInput` | 本地传输辅助函数与服务端市场流程构造，SQLite transfer repository 消费 | `infra-sqlite/scripts/transfer_repository` | 保留为持久化写入投影；已删除 `api/local` 的再导出，API 不拥有或转卖 SQLite 输入类型。 |

由此得出的当前结论：`app_schedule`、`app_script` 没有任何应用服务或协调能力，只有上述模型，必须在本批迁移后删除。此前将它们保留为“应用层数据模型”的判断错误，不能作为后续设计依据。

已覆盖的后续审计范围：

1. `domain-notification`、`domain-system` 的每个公开配置类型及使用方。
2. `infra-vision` 中序列化模型配置与 ONNX session 的混合边界；不得以复制 JSON 类型替代拆分。
3. `infra-adb`、`infra-device-runtime`、`runner-protocol`、`child-runner` 和组合根的所有公开类型，特别是是否混入领域模型或 API 契约。
4. `infra_sqlite` 的全部导出函数：确认它们仅依赖领域模型和基础设施输入，不反向依赖 API / application。

第二批已审计台账（设备、ADB、运行时与协议）：

| 公开类型组 | 直接调用者 / 证据 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `DeviceConfig`、`DeviceProfile`、`DeviceExecutionPolicy`、设备/连接/捕获枚举 | 本地设备 API、SQLite、运行时重建；配置由设备标识拥有 | `domain-device` | 保留。它们是可持久化的设备配置；其中 ADB、窗口捕获字段是设备能力的选择，不是 ADB executor 或截图实现。 |
| `DeviceOperation` | child 脚本执行与设备运行时 | `domain-device` | 保留。操作语义与 ADB、桌面实现无关。 |
| `ADBCommand`、`ADBCtx`、`ADBExecutor`、`ADBConnectConfig`、`AdbServerConfig`、`AdbError` | `infra-device-runtime`、child、主进程设备连接流程 | `infra-adb` | 保留。它们直接表达 ADB server、连接方式、shell 命令和 client 状态；不创建形式化的 `domain-adb`。已移除无源码使用的 `domain-script` 依赖。 |
| `DeviceCtx`、`DeviceRuntime`、`AndroidDeviceRuntime`、`DesktopDeviceRuntime`、设备启动/探测函数 | child 初始化、主进程运行时重建 | `infra-device-runtime` | 保留。它们包含 executor、图像与外部进程操作，是运行时适配器而非设备领域模型。已移除无源码使用的 `domain-script` 依赖。 |
| `ChildProcessInitData`、`IpcMessage`、全部 `*Message` / `*Event` / `Runtime*` wire 类型、`ChannelTrait` / `ChannelError` | 主进程发送会话快照，child 接收；child 回传 runtime event | `runner-protocol` | 保留。均实现 IPC 编解码或处于 `MessagePayload` 传输链；已删除 `child-runner` 对 `ChildProcessInitData` 的无必要再导出，入口直接使用协议类型。 |

`domain_device` 公开符号逐项台账：

| 公开符号 | 直接调用者 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `DeviceId`、`DeviceConfig`、`DeviceProfile` | 本地设备/执行 API、SQLite、主进程状态、child bootstrap IPC | `domain-device` | 保留。它们是设备可持久化身份与配置根；不持有 ADB client、窗口或 child handle。 |
| `DeviceExecutionPolicy`、`TimeoutAction`、`TimeoutNotifyChannel` | session builder、`RuntimeExecutionPolicy` 快照、child timeout 行为 | `domain-device` | 保留。它们是用户配置的设备执行策略；领域枚举实现 bincode 编解码后由协议直接引用，已删除协议同名副本及主进程逐项映射。 |
| `CapMethod`、`WindowCaptureInterface`、`DevicePlatform`、`DeviceTransportKind`、`EmulatorConnectMode` 与 `DeviceConfig` 的能力判断方法 | 本地设备设置、child 配置更新、device runtime 选择 ADB/窗口实现 | `domain-device` | 保留。它们是设备拥有的能力与用户选择；`infra-window-capture` 的实现接口/窗口资源仍独立，二者不合并。 |
| `DeviceOperation` | 脚本 child executor、`infra-device-runtime` 映射为具体 ADB/desktop 动作 | `domain-device` | 保留。它表达“设备要做什么”，不表达“怎样调用 ADB”。 |

`domain_schedule` 公开符号逐项台账：

| 公开符号 | 直接调用者 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `AssignmentId`、`DeviceId`、`ScriptId`、`TemplateId` | 调度 API、SQLite、planner、child journal | `domain-schedule`（ID 定义复用 `ad-kernel`） | 保留。它们是调度记录间的稳定关联键。 |
| `TimeOfDay`、`TimeOfDayError`、`TimeWindow` | 本地调度 API、queue loader、主进程 planner | `domain-schedule/time_window` | 保留。它们定义跨午夜和无边界的时间窗口规则，不含 clock/job 实现。 |
| `AssignmentProfile`、`TimeTemplateProfile`、`TemplateValueProfile` | 本地调度 API、SQLite、queue loader | `domain-schedule` | 保留。它们是用户保存的设备-脚本分配与时间模板模型。 |
| `ExecutionScheduleProfile`、`AssignmentScheduleProfile`、`PlannerQueueItem` | SQLite planner ledger、主进程 scheduler、child schedule journal | `domain-schedule` | 保留。它们是调度计划/执行账本值；实际 queue、timer 和数据库操作不在领域包。 |
| `AssignmentScheduleStatus`、`AssignmentTriggerSource`、`TaskRunStatus` | planner 状态迁移、SQLite 账本编码、child task journal | `domain-schedule` | 保留。它们表达调度结果语义；协议/UI 的映射在 runner/root API。 |

`infra_adb` 公开符号逐项台账：

| 公开符号 | 直接调用者 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `ADBCommand` | `infra-device-runtime` 将设备操作映射为 ADB action | `infra-adb/adb_command` | 保留。它是 adb client 命令语义；通用坐标仍在 `ad-kernel`。 |
| `AdbServerConfig`、`AdbServeByIdentifier`、`ADBConnectConfig` 及其校验/更新方法 | `infra-device-runtime::device_launcher`、根视觉调试 | `infra-adb/adb_config` | 保留。它们描述 adb server、path、identifier 与 TCP 连接细节，不是设备领域配置。 |
| `ADBCtx`、`ADBCtx::new`、`try_get_adb_ctx` 及发送/捕获/循环命令方法 | child IPC 初始化、`infra-device-runtime`、根视觉调试 | `infra-adb/adb_context` | 保留。它持有 client、queue 与连接状态，是运行时适配器。 |
| `adb_command`、`adb_config`、`adb_context` 模块路径 | crate 外只需要 `ADBCommand`、三项连接配置类型与 `ADBCtx`/`try_get_adb_ctx` | `infra-adb` 显式根门面 | 模块已收回私有；六项实际跨 crate ADB 合同从 crate 根显式导出。全仓旧深层路径已清零。 |
| 命令字符串 helper、`ADBCmdConv`、`ADBCommandResult`、`ADBExecutor`、`AdbError` / `AdbResult` | 仅 adb context/executor 内部互调 | `infra-adb` crate 内部 | 已收窄为 `pub(crate)`，不构成跨 crate 合同。 |
| `get_adb_ctx`、未构造的 command result 分支、未调用常量/helper、未调用 executor 校验 | 全仓无消费者 | 无所有者 | 已删除。`cargo check -p infra-adb` 通过且无上述 dead-code/private-interface warning。 |

`infra_device_runtime` 公开符号逐项台账：

| 公开符号 | 直接调用者 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `DeviceCtx`、其截图、配置应用与设备操作方法，以及 `init/get/try_get_device_ctx` | child 初始化、child IPC、脚本 executor、根视觉调试 | `infra-device-runtime/device_ctx` | 保留。它持有 `DeviceRuntime`、截图资源与 ADB 调用，是 child 内运行时 façade，不是设备持久化模型。 |
| `ensure_device_connection_with_progress`、`probe_device_config_connection_with_timeout`、`resolve_runtime_connect_config` | child IPC 的连接控制与配置更新 | `infra-device-runtime/device_launcher` | 保留。它们执行 emulator process、ADB/TCP 探测与进度回报。 |
| `device_ctx`、`device_launcher` 模块路径 | crate 外仅使用 `DeviceCtx`、全局 context 与三项连接函数；不存在必须依赖实现目录的调用 | `infra-device-runtime` 显式根门面 | 模块已收回私有；`DeviceCtx`、`init/get/try_get_device_ctx` 和三项连接函数从 crate 根显式导出。全仓旧深层路径已清零。 |
| `DeviceRuntime`、Android/Desktop runtime、`DeviceCtx.runtime` 字段及无进度 launcher wrapper | 仅 device context / launcher 内部；全仓无外部消费者 | `infra-device-runtime` crate 内部或无所有者 | runtime 实现与字段已收窄为 `pub(crate)`；5 个无消费者 wrapper 和未使用 platform 方法已删除。`cargo check -p infra-device-runtime` 通过且无 warning。 |

`runner_protocol` 公开符号逐项台账：

| 公开符号组 | 直接调用者 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `ChildProcessInitData` | 根 `session_builder` / `child_process_manager` 写入启动数据，`child_runner::bootstrap::process` 反序列化 | `runner-protocol` | 保留。它是跨进程 bootstrap 数据。 |
| `codec::{config, encode_to_vec, decode_from_slice}`、`MAX_MESSAGE_SIZE`、`ChannelTrait`、`ChannelError`、`ChannelResult` | 根 IPC server 与 child IPC client | `runner-protocol` | 保留。它们共同定义 framing、编码和通道错误。 |
| `IpcMessage`、`MessageType`、`MessagePayload`、`Process*`、`Connection*`、`CaptureControlMessage`、`SessionControlMessage`、`ConfigUpdateMessage`、`LogMessage` | 根发送/处理与 child 接收/发送 | `runner-protocol` | 保留。全部是 `MessagePayload` 的真实 wire 合同。未构造的 `HeartbeatMessage` / `ErrorMessage` 及主进程的 TODO/no-op 处理分支已删除。 |
| `Dispatch*`、`RunTarget`、`RuntimeExecutionPolicy`、`RuntimeQueueItem`、`ScriptBundleSnapshot`、`RuntimeSessionSnapshot` | 根 session builder，child session/scheduler | `runner-protocol` | 保留。它们是主进程构造、child 消费的会话快照。未消费的 `RuntimeVisionTextCachePolicy` 已删除。 |
| `RuntimeLifecycle*`、`RuntimeProgress*`、`RuntimeSchedule*`、`ConnectionStatus*`、`CaptureResultEvent`、`RuntimeDispatchEvent`、`RuntimeEventMessage` | child reporter 发送，根 IPC handler 映射为 local API event | `runner-protocol` | 保留。它们是 child 回传的 wire event。 |
| 领域 `TimeoutAction` / `TimeoutNotifyChannel` | `RuntimeExecutionPolicy`、child observer、根 UI timeout event | `domain-device` | 协议只传输领域值，不再定义同名 wire 副本；两枚举补充 `bincode::Encode/Decode`，session builder 直接 clone 配置。 |
| `AssignmentScheduleStatus` / `AssignmentTriggerSource` 与 `RuntimeScheduleStatus` / `DispatchSource` | 前者由 SQLite planner 读写，后者仅随 child 事件或会话快照传输；根 scheduler 显式映射 | 分别为 `domain-schedule` 与 `runner-protocol` | 保留两组。名称和值有重叠，但前者含持久化排程语义（如 Planned/Dispatched/Cancelled），后者表达 child 当前执行/传输状态；不存在可安全删除的同型副本。 |

第三批已审计结论（通用内核、配置领域与视觉边界）：

| 公开类型 / 内容 | 调用者 / 证据 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `UuidV7`、各领域 ID 别名、`Point<T>`、`LogLevel` | 多个领域、协议和基础设施 | `ad-kernel` | 保留。它们不包含产品业务或具体适配器知识。 |
| `ad_kernel::geometry`、`ad_kernel::logging` 模块路径 | `geometry` 无 crate 外深层路径；原有 4 个 `logging::LogLevel` 使用点均可改为根门面 | `ad-kernel` 显式根门面 | 已收回两条模块路径，保留 `ad_kernel::Point` 与新增显式 `ad_kernel::LogLevel` 重导出；`ids` 因仍有实际跨 crate 的领域 ID 路径而保持公开。全仓旧深层路径为零。 |
| `ad_kernel::ids` 对 `serde`、`thiserror`、`ahash::AHashMap` 的转售 | 根 API、child session、SQLite、ADB、视觉包曾经借此间接导入第三方类型 | 无领域所有权；这是依赖泄漏，不是共享内核合同 | 已删除转售。每个使用点直接导入 `serde` / `thiserror` / `std::collections::HashMap`；`GuardId`、`SubFlowId` 经全仓搜索无调用者，已删除。`cargo check -p ad_kernel -p infra-adb -p infra-sqlite -p infra-vision-cache` 通过。 |
| `ad_kernel::constants::table_name` | 当前无消费者 | 无所有者 | 已删除；不把死表名常量迁入 SQLite。 |
| `ad_kernel::constants::sys_conf_path`、`MAIN_WINDOW`、`SOCKET_NAME`、截图目录 | 主应用配置、主进程窗口/IPC、根基础设施 | 组合根 `src/app` / `src/infra` | 已迁入：配置键和窗口标签归 `src/app/constants.rs`，截图目录归 `infra/image.rs`，socket 名称归 `infra/ipc.rs`；旧内核模块已删除。 |
| `EmailProviderPreset`、`EmailSecurity`、`ResolvedSmtpServer`、`EmailConfig` | 本地邮件设置 API、根配置装载、`infra-mail::send_email`；`ResolvedSmtpServer` 仅由 `EmailConfig` 解析后交给发送器 | `domain-notification` | 保留。通知偏好、SMTP 参数和供应商预设是稳定通知概念；`infra-mail` 只负责将其转换为 lettre transport。 |
| `ShortCut`、`StartMode`、`IdleAction`、`SystemConfig` | 本地系统设置 API、启动/退出、窗口/托盘/快捷键协调 | `domain-system` | 保留。它们是持久化的本地系统行为配置；宿主动作仍由根 `app` 执行。`dispatch_schedule_retention_days` 只驱动启动期的本机 SQLite 清理，不参与排程或 child 执行；为一个 `u16` 不创建跨领域依赖或空壳 schedule 配置。 |
| `ImageCompression`、搜索规则、视觉结果、文字缓存配置 | 脚本领域、设备配置、视觉运行时 | `domain-vision` | 保留。它们描述视觉语义和可持久化配置，而非 ONNX 适配实现。 |
| `VisionTextCacheConfig`、`VisionTextCacheRuntimeConfig` 与旧 `RuntimeVisionTextCachePolicy` | 根设置转换为 runtime config，`ChildProcessInitData` 在 child bootstrap 初始化 `ScriptTextRecCacheRuntime`；session snapshot 中的同名投影无读取者 | `domain-vision` / child bootstrap | 已删除未消费的协议投影。持久化值、路径解析后的 runtime 值和实际 child 缓存初始化保持单链，避免同一 OCR 配置通过 bootstrap 与 session 双重传递。 |
| `ModelSource`、`ModelType`、`InferenceBackend`、`BaseModel` 的可序列化字段，`DetectorType` / `RecognizerType`，`YoloDet` / `PaddleDetDbNet` / `PaddleRecCrnn` 的可序列化字段，以及 `YoloPostprocessKind`、`RecResizeFilter`、`RecProcessingMode` | `ScriptInfo`、服务端脚本传输、视觉 API；类型内部同时含 ORT `Session`、字典/标签/预处理缓存和加载方法 | `domain-vision` | 已拆分为同名纯模型；`infra-vision` 内部改为 `Runtime*` 并仅由 `OcrService` 构造，外部仅保留模型/字典路径适配入口。`domain-script -> infra-vision` 依赖与旧 TS 生成依赖已删除。 |
| `VisionSnapshot`、`OcrSearcher` 与视觉结果排序/文本索引 | child 执行器、`SearchRule`；只依赖 `domain-vision` 的结果和规则 | `domain-vision` | 已迁入；`domain-vision`/`domain-script` 检查通过。它们是视觉结果解释规则，不依赖 ORT、截图或文件。 |
| `StablePoint`、`RelativeValueType`、`RelativeCompareOp`、`SearchScope` | 前者是 `DetResult`/`OcrResult`/`SearchHit` 的公开字段与 TS 合同；后三者是持久化 `SearchRule` 的字段并参与规则求值 | `domain-vision` | 保留。候选扫描未发现具名 crate 外导入，但字段、serde 和 TS 生成是实际合同，不能按文本引用删除或收窄。 |
| `Rect` | 全仓只剩定义与 `domain_vision` 根 re-export；不在结果字段、serde/TS 绑定或测试构造中出现 | 无所有者 | 已删除。它不是 `BoundingBox` 的别名或运行时转换所需类型，不能因位于视觉结果文件就保留。`cargo check -p domain-vision` 与 36 项测试通过。 |
| `OcrService`、图像裁剪/加载、模型路径解析、模型运行错误 | child OCR 服务、主进程视觉调试和错误映射；直接操作 ORT session、`image`、模型文件 | `infra-vision` | 保留。模型配置迁出后，这些运行时实现由配置构造实例。`ModelHandler` / `TextDetector` / `TextRecognizer`、ORT provider、色彩分析和运行时模型细节已收窄为 crate 内部。 |
| `PolicyActionKind` / `PolicyActionSource` / `PolicyActionTarget*` / `PolicyActionTrace` / `PolicyExecutionRound` / `PolicyExecutionResult` | 仅 child executor、Rhai bridge、child 测试构造或反序列化 | `child-runner` | 已迁入 child 执行结果模块；取消无消费者的 TS 导出。它们不是脚本定义或跨进程合同。 |
| `TaskControl`、`StateTarget`、`StateStatus` | `StepKind`、脚本编辑 JSON 与 child 执行器 | `domain-script` | 保留。它们是脚本可持久化步骤语义，child 仅解释执行。 |
| 脚本访问规则与 `ScriptAccessError` | 本地脚本 API、SQLite 编辑保护 | `domain-script` | 保留。它们定义发布脚本的稳定编辑/克隆规则。 |

`domain_notification` / `domain_system` 公开符号逐项台账：

| 公开符号 | 直接调用者 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `EmailProviderPreset` 与 `preset_server` | `EmailConfig::resolved_server`；本地邮件设置序列化 | `domain-notification` | 保留。供应商预设是通知配置的稳定值规则，不是 lettre/SMTP transport。 |
| `EmailSecurity` | `EmailConfig`、`ResolvedSmtpServer`；`infra-mail` 映射为 lettre TLS 设置 | `domain-notification` | 保留。安全策略由通知配置决定，基础设施只解释它。 |
| `ResolvedSmtpServer` 与 `EmailConfig::resolved_server` | 根邮件配置规范化、`infra-mail::send_email` | `domain-notification` | 保留。它是配置规则解析的结果，不持有 SMTP client。 |
| `EmailConfig`、发件人/收件人/超时规范化方法 | 本地设置 API、配置 store、根邮件适配与 `infra-mail` | `domain-notification` | 保留。它定义通知偏好和可持久化参数；网络发送仍在 `infra-mail`。 |
| `ShortCut` | `SystemConfig`、根快捷键注册 | `domain-system` | 保留。它是持久化的用户系统偏好；Tauri shortcut handle 不在类型中。 |
| `StartMode` | `SystemConfig`、根启动窗口/托盘协调 | `domain-system` | 保留。它只表达启动策略，不包含窗口实现。 |
| `IdleAction` | `SystemConfig`、主进程空闲处理 | `domain-system` | 保留。它是用户选择的系统行为，不包含关机/睡眠平台调用。 |
| `SystemConfig` | 本地系统设置 API、配置 store、启动、自动启动、快捷键与调度记录清理 | `domain-system` | 保留。它是可持久化本地系统配置；使用该配置的 Tauri/plugin 副作用由根 `app` 执行。 |

`domain_script` 公开符号台账（模块名不再泄漏；`metadata` 全部根导出后已收回私有）：

| 公开符号组 | 直接调用者 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `ScriptId`、`ScriptProfile`、`ScriptInfo`、`ScriptRuntimeSettings`、`ScriptTransferRecord` | 本地/服务端脚本 API、SQLite、child session | `domain-script` | 保留。它们是脚本及发布/传输记录的可持久化标识和根模型。 |
| `ScriptType`、`RuntimeType`、`ScriptPlatform`、`SCRIPT_RUNTIME_SCHEMA`、`supported_script_features` | 脚本编辑/市场 API、服务端传输、运行时装载 | `domain-script` | 保留。它们定义脚本兼容性合同，不含 Tauri 或模型加载实现。 |
| `Policy*Profile`、`Policy*Info`、`PolicyGroupPolicyLink`、`PolicySetGroupLink` | 编辑器 API、SQLite 图读写、child scheduler/executor | `domain-script` | 保留。它们组成持久化策略图；执行状态仍在 `child-runner`。 |
| `PolicyInfo` / `PolicyGroupInfo` / `PolicySetInfo` 与对应 `*Profile` | SQLite 将前者作为 JSON 内容列保存，编辑器与 child 以 profile 的 ID、脚本归属、排序和 info 组合读取 | `domain-script` | 保留两层而不合并：`Info` 是可嵌入策略内容，`Profile` 是图节点。它们不是两份同一持久化模型。 |
| `ScriptTaskProfile`、`ScriptTask`、`TaskRowType`、`TaskTone`、`TaskTriggerMode`、`TaskCycle` | 编辑器、SQLite、child task planner | `domain-script` | 保留。它们描述任务定义和周期，不含调度执行状态。 |
| `ScriptTask` 与 `ScriptTaskProfile` | SQLite 的 task data JSON、编辑器、child execution plan | `domain-script` | 保留两层而不合并：前者是步骤/变量载荷，后者补充 ID、脚本归属、展示层级、周期和软删除元数据。 |
| `Step`、`StepKind`、`Action`、`ClickMode`、`SwipeMode`、`SwipeTarget`、`TaskControl`、`StateTarget`、`StateStatus` | 编辑器 JSON、child executor | `domain-script` | 保留。它们是脚本步骤语义；child 仅解释执行。 |
| `FlowControl`、`ConditionNode`、`CurrentTaskCondition`、比较运算与 policy-set-result 字段 | 编辑器条件表单、child 流程解释 | `domain-script` | 保留。它们是可序列化脚本规则。 |
| `DataHanding`、`VarValue`、颜色/区域/筛选模型、`Point*`、变量目录/定义/命名空间/来源/值类型、`VisionNode` | 编辑器、脚本存储与 child 执行 | `domain-script` | 保留。它们描述脚本数据和视觉步骤的引用，不拥有 OCR runtime。 |
| `ScriptAccessError`、`ensure_editable`、`ensure_clone_allowed`、`clone_cloud_id`、`ScriptError`、`ExecuteResult` | 本地脚本 API、SQLite 编辑保护、child 执行 | `domain-script` | 保留。前者是脚本访问规则，后者是脚本语义错误；SQL/IPC 错误不迁入此包。无消费者且与 `ExecuteResult` 完全同型的 `ScriptResult` 已删除。 |
| `PointI32` | 全仓 Rust、前端生成类型和实际脚本模型均无消费者 | 无所有者 | 已删除。脚本图仍使用有实际 JSON/运行时消费者的 `PointU16`、`PointF32`；不保留未被任何脚本字段引用的坐标壳。类型生成器不会自动删除历史文件，故同步删除 `src/types/bindings/PointI32.ts` 与 barrel export；重跑后绑定索引为 151 项。 |
| `ScriptPlatform`、`ScriptRuntimeSettings` | `ScriptInfo` 的持久化/传输字段、TS 合同与 child 脚本装载 | `domain-script` | 保留。它们没有具名 crate 外导入，但定义可持久化脚本的目标平台和恢复/点击偏好。 |
| `ScriptTransferRecord` | SQLite transfer repository、local transfer-record API | `domain-script` | 保留。它表达脚本下载/上传的业务记录；SQLite 的 `Create/Finish...Input` 仍是 adapter 写入投影，未反向迁入领域。 |

`infra_vision` 公开门面逐项台账（配置与 runtime 内部对象已排除在包外 API 之外）：

| 公开符号 | 直接调用者 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `OcrService` | child scheduler、child runtime context/执行器、根开发测试 | `infra-vision` | 保留。它持有 trait object、加载字典和 ORT session。 |
| `resolve_model_path`、`resolve_recognizer_dict_path` | child OCR 缓存资产签名 | `infra-vision` | 保留。它们读取本机模型/字典文件布局，是运行时文件适配而非配置模型。 |
| `VisionError`、`VisionResult` | 根 `AppError`、child 资产签名与 OCR 调用链 | `infra-vision` | 保留。它们表达 ORT、模型和图像运行时失败。 |
| `ImageError`、`ImageResult`、图像加载函数与 `get_crop_image_rgba` | 根视觉调试 API、child 截图识别 | `infra-vision` | 保留。它们直接操作 `image` 和文件。`ImageResult` 是公开加载/裁剪函数的返回类型；`get_crop_image`、`get_crop_images` 与 `get_crop_images_rgba` 仅供视觉 crate 内部 trait/测试使用，已收窄为 `pub(crate)`。 |
| `infra_vision::infra` 及 image/ort/vision 的整棵内部模块树 | 外部调用者实际仅使用 OCR 服务、图像加载/裁剪、路径解析与错误类型 | `infra-vision` 内部实现 | 已收回为私有/`pub(crate)` 模块；crate 根显式导出 `OcrService`、两类错误/结果、两个路径解析函数及实际使用的图像工具。全仓 `infra_vision::infra::*` 调用已清零。 |
| `ort::{OrtError, backend_name, configure_or_switch_provider}` | `vision_error` 与 `base_model` 是同一 `infra` 内的 sibling；它们不能穿透 `ort_error` / `execution_provider_mgr` 的私有实现模块 | `infra-vision/infra/ort` 的内部门面 | 已在 `ort.rs` 以 `pub(super)` 重导出三项内部合同，调用方改为门面路径；深层调用已清零。组合根检查最初发现的三个 `E0603` 因此消除，重跑后才停在 ORT 下载环境。 |

第四批已审计结论（组合根、child 与 SQLite 边界）：

| 公开类型 / 内容 | 调用者 / 证据 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `auto_daily_lib::run` | 唯一外部调用者为 `src/main.rs` | 组合根 | 保留。它装配 Tauri、API command 与主进程资源；根 `api`、`app`、`infra` 模块均已收回私有，根 crate 不再伪装成可复用基础设施库。 |
| `api/local` 的 89 个 `#[command]`、`api/local/execution/events.rs` 的 UI event payload、`api/server/dto.rs` 的远端请求/响应 DTO | 本轮从全部 `#[command]` 定义重新扫描：89 个 local command 加 19 个 `backend_*` server command，合计 108 个；根 `generate_handler!` 去除注释后同为 108 个，双向比较均为零差异；前端 TS 合同、HTTP client | 根 `api/local` / `api/server` | 保留。它们是入口契约与用例协调；不再创建仅装 DTO 的 application crate。 |
| 空的 `api/backend_cmd`、`api/infra`、`api/scripts` 目录树 | 不被 `api.rs` 装载，无 tracked 文件或源码消费者 | 无所有者 | 已删除。它们是重构后遗留的空模块目录，不能作为“未来 API 分层”保留。 |
| 根 `app/config/*`、启动/退出和快捷键函数 | Tauri 生命周期、配置 store、infra 调用 | 根 `app` | 保留。它们协调配置与宿主副作用，符合 application 层职责。 |
| 根 `infra/context/main_process.rs`、`child_process_manager.rs`、IPC server、HTTP、日志和邮件适配 | 主进程运行期、child IPC、外部 HTTP/文件/窗口 | 根 `infra` | 保留。它们含 Tauri/进程/连接状态或技术实现，不是领域模型。 |
| 根组合层的 10 个无调用方法：`download_file`、`restart_child`、`replace_pending_dispatches`、`pop_next_dispatch`、`info_with_fields`、`fn_begin`、`fn_end`、`info_with_tag`、`get_retention_days`、`update_log_to_file` | 对根 `src` 的 303 个 `pub fn/struct/enum/trait/type` 做具名源码消费者扫描后，这 10 项仅在定义处出现；删除后 293 项公开定义的候选数为 0 | 无所有者 | 已删除。它们分别是未接入的整文件下载/重启/调度队列操作和历史日志 convenience helper；不以“根模块内部也许会用到”为理由保留。 |
| 根 `infra` 及其 context/logging/image/ipc 子模块的可见性 | 无 crate 外调用者；组合根只对外公开 `run` | 组合根内部实现 | 已收紧为 `pub(crate)`。这不改变主进程运行态的归属，只防止根内部模块被误读为 workspace 公共适配器。 |
| `child_runner` 的初始化、会话、执行计划、运行时状态和 reporter | `src/main.rs` 只调用 `bootstrap::run_child_process_entry`；其余均在 child 内部互调 | `child-runner` | child 主循环、错误、环境读取、信号处理、调度 tick 和收尾已从根 `main_child.rs` 整体迁入 `bootstrap/process.rs`。现仅公开一个启动门面，`infra/*` 与所有运行时状态均为 `pub(crate)`；删除无消费者的独立 `child` Cargo target。内部状态不可迁入 `runner-protocol`。 |
| `child_runner/src/domain.rs` | 根库未装载；全仓无调用者；仅声明不存在的 `vision` 子模块 | 无所有者 | 已删除。它是旧迁移残留，不能通过补空目录伪装为 child 领域层。 |

根 `api` / `app` / `infra` 公开符号台账（同 crate 内的 `pub` 仍逐组按可达生命周期审计）：

| 公开符号组 | 直接调用者 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `api::response::ApiResponse`、`api/local/*` 全部 `#[command]` 函数、设备运行事件与 `DeviceRuntimeSnapshotPayload`、`VisionLabModelConfig`、脚本传输控制/进度 | Tauri command 注册、Vue 绑定、主进程 IPC event 映射 | `api/local` | 保留。它们是本地 UI 输入/输出、事件或用例协调状态；设备持久化模型仍在 `domain-device`。 |
| `api/server/dto` 的认证、用户、脚本搜索/上传/下载、分页、赞助 DTO 与 `CurrentAuthenticatedUser`、`ScriptVersionPreflight` | HTTP client、服务端脚本市场流程、认证缓存 | `api/server` | 保留。它们是远端请求/响应与远端版本协商合同，不是脚本领域模型。 |
| `api`、`api/local`、`api/server`、`api/response` 及 local 的 script/settings/vision/execution 子模块 | 无 crate 外调用者；根 `lib.rs`、主进程 context 和 HTTP adapter 均在同一组合根内直接使用 | 组合根内部 API 实现 | 已全部收紧为 `pub(crate)`；保留命令函数和 DTO 的 `pub`，因为它们仍须由 Tauri 宏、序列化和同 crate 调用者使用。这样不会把 UI/远端合同误读成 workspace 对外 Rust API。 |
| `AppError` / `AppResult`、`ScriptsConfig`、启动/退出/快捷键/配置 store 函数 | Tauri 生命周期、主窗口、应用数据目录和 plugin store | 根 `app` | 保留。它们含 Tauri handle、窗口标签或宿主配置文件；`ScriptsConfig` 的默认值直接解析 app data dir。 |
| `app` 与 `app/config` 的模块可见性 | 仅根 crate 的 API、启动和基础设施模块使用；根 crate 本身只公开 `run` | 组合根内部实现 | 已由 `pub` 收紧为 `pub(crate)`。`ScriptsConfig` 仍留在 app：其默认值依赖 `AppHandle::app_data_dir`，不是脚本领域的可持久化模型。 |
| `APP_STORE`、配置 key、`MAIN_WINDOW`、`SOCKET_NAME`、`SCREENSHOT_DIR`、用户 profile cache key 与根日志静态值 | 仅根 `app` / `api` / `infra` 模块使用；所有定义处所在模块均不对 crate 外开放 | 组合根内部常量和运行态 | 已由 `pub` 收窄为 `pub(crate)`。常量值的所有权不变，但不再形成看似可被 workspace 其他包依赖的 API。 |
| `DeviceConnectionState`、`DeviceLifecycleState`、`DeviceDispatchState`、`DeviceProgressState`、`DeviceRuntimeState`、`DeviceCaptureResult`、`DeviceDispatchSignal`、`RuntimeReconcileJob`、`MainProcessCtx` | 本地执行 API、child process manager、root IPC handler | 根 `infra/context` | 保留。它们是主进程内存态、锁和 child 运行期协调，不是可持久化设备领域模型。 |
| `ChildProcessHandle` / `ChildProcessManager`、`IpcClientState` / `IpcServer`、`HttpClient` / `FileTransferProgress`、app handle 函数 | 主进程子进程管理、local socket、远端 HTTP、Tauri 生命周期 | 根 `infra` | 保留。它们直接持有进程、socket、HTTP client 或 `AppHandle`。 |
| 根日志、邮件、图片模块的公开函数/错误/资源类型 | Tauri 配置、IPC handler、开发调试、文件保存和邮件发送 | 根 `infra` | 保留。它们是根宿主对 `infra-logging`、`infra-mail`、文件系统的适配；不引入新的 application crate。 |

`child_runner` 公开门面逐项台账（`infra` 已私有，以下是全部可从 crate 外访问的符号）：

| 公开符号 | 直接调用者 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `bootstrap::run_child_process_entry` | `src/main.rs` 在 `--child` 分支调用；主进程以当前可执行文件加该参数启动 | `child-runner` | 保留。这是唯一跨 crate 的 child 生命周期入口；`ChildProcessError`、环境读取、信号处理、状态机、调度器与 reporter 均已收回 crate 内部。 |
| `infra_sqlite` 的 bootstrap、device/script/schedule repository 函数及脚本传输写入输入 | 根 local/server API 和 child journal | `infra-sqlite` | 保留。逐文件反查与 manifest 均无 `app_schedule`、`app_script`、旧执行包或 `domain_execution` 残留；只使用领域模型或 SQLite 写入输入。细分台账如下。 |
| `DeviceDispatchEventPayload` | 当前全仓只有定义，无任何 emit 或消费 | 无所有者 | 已删除，类型生成入口无对应残留；`gen:types` 也已移除不再导出任何 TS 类型的 `infra-vision`。 |
| `child_runner::constant` 对 `ad_kernel::constants` 的再导出；`ad_kernel::constants::table_name` | 当前无消费者 | 无所有者 | 已删除，不迁移空壳或死常量。 |
| 根 `infra::ipc` 与 `child_runner::infra::ipc` 对 `runner-protocol::{message, channel_error, channel_trait}` 的转售 | 主/child 两侧调用者原先经各自的 infra 路径间接使用协议 | `runner-protocol` | 已删除两处转售，调用者全部改为直接导入 `runner_protocol`。根 `infra::ipc` 仅保留主进程 socket server/handler，child `infra::ipc` 仅保留 client/handler/reporter；拼写错误的 `chanel_trait` / `chanel_server` / `chanel_client` 文件和模块也已统一重命名为 `channel_*`，旧路径为零。 |

`infra_sqlite` 公开门面逐项台账（按 SQLite 适配的表/事务边界拆分，而非平铺在 crate 根目录）：

| 公开符号组 | 直接调用者 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `init_db_with_path`、`init_db_and_migrate_with_path`、`get_pool` | 根启动、child 环境初始化；脚本下载在“替换旧图 + 插入新图”中显式开启一个 SQLite transaction | `infra-sqlite/bootstrap` | 保留。`get_pool` 是唯一跨 repository 原子写入的窄事务桥，不包含领域概念；不能把 `SqlitePool` 迁入 API 或 domain。 |
| `get_device`、`get_all_devices`、`save_device`、`delete_device_with_assignments` | 本地设备 API、日志配置与 child session builder | `infra-sqlite/devices` | 保留。它们读写 `domain-device::DeviceProfile`，不拥有设备模型。 |
| 模板、分配、执行记录与 planner ledger 的 CRUD / queue 同步函数 | 本地调度 API、主进程 planner/reconcile、child schedule journal、脚本模板值动作 | `infra-sqlite/schedules` | 保留。代码已分别位于 template、assignment、execution-record 与 planner-ledger 模块；输入/输出均为 `domain-schedule` 的模型或 ID。 |
| `find_assignment_schedule_scope` | 全仓无调用者；仅有两层 re-export | 无所有者 | 已删除。它是未接入任何流程的 SQLite 查询，不以“未来可能按 scope 查询”为理由保留。 |
| `list_planner_batch_ids` | 仅 `planner_ledger_repository::load_sync_target_planner_batch_ids` 调用 | `infra-sqlite/schedules` 内部实现 | 收窄为 `pub(crate)` 并改为直接从 sibling repository 导入；不再作为 crate 根 SQLite 合同。 |
| 脚本图的读写、克隆、关系替换与编辑保护函数 | 本地脚本编辑 API、服务端脚本市场下载/覆盖 | `infra-sqlite/scripts` | 保留。它们只保存/读取 `domain-script` 图；下载流程之所以持有 transaction，是为了和本地目录替换保持原子顺序。 |
| `CreateScriptTransferRecordInput`、`FinishScriptTransferRecordInput` 与传输记录 CRUD | 本地传输辅助函数、服务端市场传输流程 | `infra-sqlite/scripts/transfer_repository` | 保留。这两种类型是 `script_transfer_records` 的写入投影；已删除 `api/local` 的再导出，API 仅构造并传入 adapter 合同。 |

第五批已审计结论（其余基础设施包）：

| 公开类型 / 内容 | 调用者 / 证据 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `CaptureMethod`、`WindowCaptureInterface`、`WindowCaptureConfig`、`WindowInfo`、Windows capture 函数 | `infra-device-runtime` 构造并持有 `xcap::Window` 与 DXGI/GDI 超时；本地视觉调试 API 只选择方式 | `infra-window-capture` | 保留。它们直接携带窗口句柄、DXGI/GDI、帧等待和平台函数；设备领域的捕获偏好会显式转换为该运行时配置，两个 `WindowCaptureInterface` 不合并。 |
| `TextRecCacheError`、`TextRecCacheResult`、`ScriptTextRecCacheRuntime` | child runtime context；内部直接读写按脚本切换的缓存文件 | `infra-vision-cache` | 保留。它是脚本执行期文件缓存和文件格式错误，不是可持久化领域配置；child 只通过启用判断、按脚本加载/刷新、写入和 OCR 结果快照使用它。 |
| `LogTrait`、`LoggerInitError`、`LoggerInitResult`、`Log` | 根日志实现、child 日志实现、各基础设施调用 | `infra-logging` | 保留。它是日志技术端口；`LogLevel` 仍由内核共享。 |
| `EmailMessagePayload`、`send_email` | 根通知/配置与 child runtime event 的邮件派发 | `infra-mail` | 保留。邮件内容是 lettre 发送适配器输入；通知偏好与 SMTP 配置仍由 `domain-notification` 拥有。 |

`infra-logging`、`infra-mail`、`infra-vision-cache`、`infra-window-capture` 公开符号台账：

| 公开符号组 | 直接调用者 | 所有者结论 | 处理 |
| --- | --- | --- | --- |
| `LogTrait`、`LoggerInitError`、`LoggerInitResult`、`Log::{init_logger, debug, info, warn, error}` | 根 `LogMain`、child `LogChild`，以及 child/vision runtime 的日志调用 | `infra-logging` | 保留。它定义日志实现的技术端口；日志级别在 `ad-kernel`，不形成产品领域模型。`log_trait` 模块路径已收回私有，四项合同从 crate 根显式导出；根与 child 的同 crate 适配模块不再向 crate 外转售路径。 |
| `EmailMessagePayload`、`send_email` | 根邮件配置/派发适配 | `infra-mail` | 保留。它直接构造 lettre transport/message；`EmailConfig` 与 SMTP 策略仍在 `domain-notification`。 |
| `ScriptTextRecCacheRuntime`、`TextRecCacheError` / `TextRecCacheResult` | child runtime context、OCR action 缓存命中与脚本切换刷新 | `infra-vision-cache` | 保留。它拥有按脚本文件缓存的运行期状态。`TextRecCacheEntry`、行解析/序列化保持 crate 内部；child 通过 `cached_ocr_results` 获得值快照，不再跨 crate 调用私有 entry 方法。未读取的 session stats、`find_entry`、动态配置与活动脚本入口已删除；包级检查与 5 项测试通过。 |
| `CaptureMethod`、窗口捕获配置/接口/窗口信息及结果捕获方法 | `infra-device-runtime`、本地视觉调试 | `infra-window-capture` | 保留。它们直接依赖 Windows window / DXGI / GDI；不得迁入设备领域。 |
| `capture_method`、`monitor_capture`、`window_cap` 模块路径，以及 `monitor_capture::{is_supported, capture_window_region}` | crate 外无 `infra_window_capture::<module>::*` 调用；区域捕获和 DXGI 支持探测仅由 `window_cap` 调用；外部只使用 re-export 的 `CaptureMethod`、`WindowCapture*` 和 `WindowInfo` | `infra-window-capture` 内部实现 | 已收回模块路径，两个平台实现函数也收窄为 `pub(crate)`，仅保留显式根门面。`cargo test -p infra-window-capture` 的纯裁剪测试通过；两个要求本地窗口标题的硬件手工测试保持 ignored。 |

### P4：文档与验证（已完成）

1. 已根据最终代码更新当前认知模型图和边界说明，并删除被取代的旧重构文档。
2. 已完成当前 workspace 的格式、编译、测试目标链接、领域/基础设施包测试、TS 绑定生成和前端类型检查。
3. Windows 运行时阻塞已与源码结果分开记录，不再把系统级进程启动错误当成 Rust 测试失败。

最终验收以本轮现行代码为准：`cargo fmt --all -- --check` 与普通 `cargo check --workspace` 均通过，后者覆盖 `infra-vision`、`child_runner` 和组合根且为零 warning；`cargo test --workspace --no-run` 证明 18 个现行包的全部测试目标能够完成编译和链接。排除会在进程启动时加载 Tauri/ORT 的 `auto_daily`、`child_runner`、`infra-vision` 后，其余 workspace 包的实际测试全部通过：146 项通过，2 项窗口硬件手工测试按设计 ignored。`pnpm gen:types` 生成 151 个绑定模块，`pnpm type-check` 通过。

ORT 下载链已从默认 `native-tls` 改为 `tls-rustls`，并通过仓库 `.cargo/config.toml` 将 `ORT_CACHE_DIR` 固定为 `src-tauri/target/ort-cache`；不再依赖用户级缓存或每次手工注入环境变量。`child_runner -- --list` 在禁用 Windows 错误弹窗后确认由系统在测试框架启动前返回 `0xc0000022 (STATUS_ACCESS_DENIED)`，因此未将该系统级运行限制伪装成测试通过，也未归咎于源码逻辑。

最终反向扫描结果：Cargo metadata 共 18 个现行包；108 个 Tauri command 与 108 个 handler 完全对应；旧包名、旧模块拼写 `chanel_*`、`mod.rs`、`src`/`crates` 空目录均为 0。公开模块只保留 `child_runner::bootstrap`、`ad_kernel::ids` 与 `runner-protocol` 的 codec/IPC 合同门面。

## 本轮执行顺序

1. P1：`application-execution` 与 `domain-execution`。
2. P2：`DeviceOperation`、子进程初始化错误、`infra-device-runtime` 和 `infra-adb`。
3. P3：按依赖方向继续逐类型审查，不为“对称”或“看起来整齐”创建包。
4. P4：更新认知模型与验证。

## 完成定义

1. 所有公开类型与导出函数均有“定义、创建/消费、合同边界、目标所有者、旧链结果”的台账条目；没有例外项。
2. 不再存在只按主题收纳、但内部类型生命周期不同的 workspace 包；任何暂不迁移项必须在台账中给出依赖与生命周期证据。
3. `application-execution`、`domain-execution` 已被拆除；旧包名、旧模块名、旧 re-export、重复定义和无消费者导出均经全仓搜索清零或有记录的保留理由。
4. `domain-device` 不依赖 ADB、窗口捕获或 child 进程实现；`infra-adb` 不伪装成领域包。
5. 子进程初始化和执行器内部状态由 `child-runner` 拥有；跨进程合同由 `runner-protocol` 拥有。
6. 当前 workspace 在可用的 Windows 构建环境中通过格式与编译检查，且只验证当前新包与组合根，不再验证已废弃包。
