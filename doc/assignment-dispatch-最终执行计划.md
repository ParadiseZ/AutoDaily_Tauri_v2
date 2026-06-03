# Assignment Dispatch 最终执行计划

更新日期：2026-06-03

本文记录“设备队列自动运行”改造的最终口径。旧的 `DeviceQueue` 全量队列思路只保留为运行模式语义，不再作为主进程长期 pending 队列。

## 一、目标

- 主进程负责：
  - 判断当前时间窗口下哪些 `DeviceScriptAssignment` 应进入运行流程
  - 生成和维护 `AssignmentSchedule` 批次账本
  - 防止同一天同一当前窗口重复生成 planner 批次
  - 在 child 空闲并请求下一个 dispatch 时派发下一项
  - 让 debug 内存队列在 request-next 时拥有最高优先级
- 子进程负责：
  - 连接探测与连接准备
  - 执行当前 dispatch
  - 保留 task 级过滤、周期跳过、调度记录
  - 执行结束后向主进程请求下一个 dispatch

## 二、核心判断

- `DeviceScriptAssignment` 继续是静态配置表，只表达“设备队列定义”。
- `AssignmentSchedule` 是运行态账本，planner/user 会持久化，debug 不持久化。
- `RunTarget::DeviceQueue` 不删除。planner 派发单个 assignment 时仍使用它，让子进程保留 task 级调度记录、周期跳过和队列模式行为。
- 手动“运行队列”不是 user 批次，只是唤醒 planner 检查/生成当前批次并派发下一项。
- 任务管理页“临时运行”是 `user` 批次，优先级低于 debug、高于 planner。
- 脚本编辑器调试是 `debug`，只进入内存队列；如果设备空闲可立即派发，如果设备正在执行，则等待当前 dispatch 结束后的 request-next 优先派发。

## 三、数据模型

### AssignmentSchedule

- `id`
- `batch_id`
- `device_id`
- `assignment_id`
  - planner 记录为真实 assignment id
  - user 临时运行可以为空
- `script_id`
- `time_template_id`
- `window_start_at`
- `scope_hash`
- `dispatch_id`
- `order_index`
- `created_at`
- `run_target_json`
  - user 临时运行用它重建目标
  - planner 记录为空
- `status`
- `trigger_source`
- `started_at`
- `completed_at`
- `message`

### status

- `planned`
- `dispatched`
- `running`
- `success`
- `failed`
- `skipped`
- `cancelled`

### trigger_source

- `planner`
- `user`
- `debug`

## 四、去重规则

planner 不靠数据库唯一索引做去重。每次触发调度时：

- 加载当前时间窗口命中的 assignment 子集
- 每个 queue item 计算 `scope_hash`
- 用 `batch_id` 将同一次生成的记录组成批次
- 检查今天是否已有一个有效 planner 批次，其集合等于当前：
  - `assignment_id`
  - `window_start_at`
  - `scope_hash`
  - `order_index`
- 有完整批次则不重复生成；没有则追加新批次

有效状态包括：

- `planned`
- `dispatched`
- `running`
- `success`
- `skipped`

`dispatch_id` 只表达单次派发实例，不参与自动调度去重。

## 五、dispatch 优先级

request-next 的派发顺序：

1. debug 内存队列
2. user 持久化账本
3. planner 持久化账本

debug 的最高优先级不是中断当前脚本，而是在当前 dispatch 结束后由 request-next 优先取得 debug 内容。只有设备空闲时，debug 才会直接派发。

## 六、职责边界

主进程负责的筛选：

- 设备是否启用
- `autoStart` 是否开启
- child 是否在线且 IPC ready
- 当前是否已有 active dispatch
- 当前时间模板窗口是否命中
- 当前 planner 批次是否已经生成
- `DeviceScriptAssignment.index` 排序

子进程保留的筛选：

- 模板覆盖后的 task enabled
- `task_cycle`
- `record_schedule`
- `ScheduleJournal` 周期跳过

## 七、调度触发时机

主动触发：

- 应用启动完成后
- 启用设备的 child bootstrap 完成后
- child IPC ready 后
- child 重启完成后
- child 完成当前 dispatch 并 request-next 后

被动触发：

- assignment 新增、删除、改顺序、改时间模板
- 时间模板新增、修改、删除
- 脚本时间模板变量值变更
- 设备配置改动：`enable`、`autoStart`、`cores`、连接配置
- 手动运行队列
- 任务管理页临时运行
- 脚本编辑器调试运行

## 八、计时机制

- 不做 30/60 秒设备轮询
- 主进程维护全局下一次到点唤醒
- 时间模板、assignment、模板变量值、设备配置变更时用通知打断 sleep 并重新计算
- 到点唤醒一次，处理完再计算下一次

## 九、前端展示

任务管理页的调度记录展示以 `AssignmentSchedule` 为主：

- 按日期分 section
- 单条记录表示一个脚本级 dispatch
- 单条可展开显示子进程写入的 `DeviceScriptSchedule`
- 清空运行记录时同时清空 assignment 账本和子进程调度记录

## 十、当前明确不做

- 不在设备编辑/assignment 保存时直接插入 `AssignmentSchedule`
- 不在主进程复制 child 的 task 级过滤和周期跳过逻辑
- 不引入 `auto_queue` 这类第二套设备队列
- 不在 debug 运行时强制中断当前正在执行的脚本
