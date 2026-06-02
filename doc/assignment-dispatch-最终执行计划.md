# Assignment Dispatch 最终执行计划

编写日期：2026-06-02

本文记录“设备队列自动运行”这轮改造的最终目标，不再保留中间态方案口径。

## 一、目标

- 主进程负责：
  - 判断当前时间窗口下哪些 assignment 应进入运行流程
  - 防止重复派发
  - 维护每台设备的待执行 dispatch 队列
  - 在 child 空闲时派发下一个 dispatch
- 子进程负责：
  - 连接探测与连接准备
  - 执行当前 dispatch
  - 保留 task 级过滤、周期跳过、调度记录
  - 执行结束后向主进程请求下一个 dispatch

## 二、核心判断

- `DeviceScriptAssignment` 继续是静态配置表，只表达“设备队列定义”。
- 新增 `AssignmentSchedule`，它是运行态账本，不在保存 assignment 时直接插入。
- `RunTarget::DeviceQueue` 不删除，但语义改为“队列模式下的一次 assignment dispatch”，不再代表整台设备全量队列快照。
- 手动开始队列与自动调度命中的 assignment，统一走 `queue_assignment`。
- 临时运行与调试运行进入同一条 dispatch 主链，但不参与自动调度去重。

## 三、数据模型

### 1. AssignmentSchedule

至少包含以下字段：

- `id`
- `device_id`
- `assignment_id`
- `time_template_id`
- `window_start_at`
- `dispatch_id`
- `status`
- `trigger_source`
- `started_at`
- `completed_at`
- `message`

### 2. AssignmentSchedule.status

- `planned`
- `dispatched`
- `running`
- `success`
- `failed`
- `skipped`
- `cancelled`

### 3. AssignmentSchedule.trigger_source

- `planner`
- `user`
- `debug`

### 4. 去重规则

自动调度场景下，不看“assignment 有没有历史记录”，而看：

- `assignment_id`
- `window_start_at`
- `trigger_source=planner`

如果当前窗口下已经存在 `planned / dispatched / running / success / skipped` 记录，则不重复派发。

## 四、dispatch 模型

### 1. dispatch_kind

- `queue_assignment`
- `temporary_full_script`
- `temporary_task`
- `debug_policy`
- `debug_group`
- `debug_set`

### 2. dispatch_id

- 只表达“一次派发实例”
- 不承担自动调度去重语义

## 五、职责边界

### 1. 主进程负责的筛选

- 设备是否启用
- `autoStart` 是否开启
- child 是否在线且 IPC ready
- 当前是否已有活动 dispatch
- 当前时间模板窗口是否命中
- 当前窗口下是否已存在有效 `AssignmentSchedule`
- `DeviceScriptAssignment.index` 排序

### 2. 子进程保留的筛选

- 模板覆盖后的 task enabled
- `task_cycle`
- `record_schedule`
- `ScheduleJournal` 周期跳过

## 六、调度触发时机

### 1. 主动触发

- 应用启动完成后
- 启用设备的 child bootstrap 完成后
- child IPC ready 后
- child 重启完成后
- child 完成当前 dispatch 后

### 2. 被动触发

- assignment 新增、删除、改顺序、改时间模板
- 时间模板新增、修改、删除
- 设备配置改动：
  - `enable`
  - `autoStart`
  - `cores`
  - 连接配置
- 手动开始队列
- 临时运行
- 调试运行

## 七、计时机制

- 不做 30/60 秒设备轮询
- 主进程维护全局 `next_due_at`
- 每次 reevaluate 后重新计算下一次最早命中窗口
- 到点唤醒一次，处理完再计算下一次

## 八、最终改造步骤

1. 新增 `AssignmentSchedule` 表、状态枚举、触发来源枚举与索引。
2. 给 dispatch 载荷补齐基础字段：`dispatch_id / dispatch_kind / dispatch_source / window_start_at`。
3. 新增主进程 `DispatchPlanner` 骨架，作为唯一调度入口。
4. 把当前“按 device_id 加载整机全量队列”的 session 构建路径改成“按单个 dispatch 构建运行载荷”。
5. 改 child 为“一次只执行一个 dispatch”，执行结束后请求下一个。
6. 把手动开始队列改成“当前时刻命中的 assignment 子集”逐条派发，不再装全量队列。
7. 把自动调度接入 planner，按 `window_start_at` 写入 `AssignmentSchedule`。
8. 最后再补前端对 assignment 级运行状态的展示。

## 九、当前明确不做

- 不在设备编辑/assignment 保存时直接插入 `AssignmentSchedule`
- 不在主进程复制 child 的 task 级过滤和周期跳过逻辑
- 不引入 `auto_queue` 这类第二套设备队列
