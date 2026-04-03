# 脚本任务结构与预览设计

本文档用于明确脚本任务在“开发者编辑器”“普通用户预览”“设备运行时”“时间模板覆盖值”四个层面的职责划分，避免继续把执行语义、展示语义和模板值语义混在同一个字段里。

## 1. 设计目标

- 让开发者可以在脚本编辑器中定义：
  - 哪些是普通任务行
  - 哪些是标题行
  - 哪些任务可以直接进入一级循环执行
  - 哪些任务只能被 link 触发
  - 哪些任务默认启用、默认周期是什么
  - 哪些任务在普通用户侧需要强调风险
- 让普通用户看到更接近最终运行场景的任务预览：
  - 有块级标题
  - 有缩进层级
  - 有任务启用开关
  - 有周期信息
  - 有风险色提示
- 让时间模板只承担“覆盖默认值”的职责，而不是回写脚本定义。

## 2. 字段分层原则

### 2.1 顶层任务元数据

以下字段属于 `ScriptTaskTable` 顶层，而不是 `task.data.uiData`：

- `rowType`
- `triggerMode`
- `recordSchedule`
- `sectionId`
- `indentLevel`
- `defaultTaskCycle`
- `showEnabledToggle`
- `defaultEnabled`
- `taskTone`

原因：

- 这些字段决定的是“任务行本身如何显示 / 如何调度”
- 它们不是某个 UI 控件的内部配置
- 它们需要在任务排序、预览、调度、模板覆盖时统一访问

### 2.2 UI schema 只负责字段布局

`task.data.uiData` 继续只承担“该任务有哪些可配置 UI 字段、控件类型、绑定变量、如何排布”的职责，不负责：

- 标题分组
- 周期定义
- 默认启用
- 调度记录策略
- 任务风险等级

## 3. 最终字段方案

建议任务结构新增如下字段：

```ts
rowType: 'task' | 'title'
triggerMode: 'rootOnly' | 'linkOnly' | 'rootAndLink'
recordSchedule: boolean
sectionId?: string
indentLevel: number
defaultTaskCycle: TaskCycle
showEnabledToggle: boolean
defaultEnabled: boolean
taskTone: 'normal' | 'warning' | 'danger'
```

其中：

- `rowType`
  - `task` 表示可执行任务行
  - `title` 表示标题行，只用于预览分块和视觉结构
- `triggerMode`
  - `rootOnly`：只允许进入一级循环
  - `linkOnly`：只能通过 `link` 或其他步骤跳转进入
  - `rootAndLink`：既可在一级循环执行，也可被其他任务跳转进入
- `recordSchedule`
  - 为 `true` 时，任务执行或调度完成后记录调度记录
  - 为 `false` 时，不写调度记录，适合启动、登录、准备动作等任务
- `sectionId`
  - 指向某个 `rowType = title` 的任务 `id`
  - 表示当前任务应显示在该标题分组下
- `indentLevel`
  - 只负责 UI 缩进，不负责块级归属
- `defaultTaskCycle`
  - 开发者定义的任务默认周期
- `showEnabledToggle`
  - 普通用户预览中是否显示该任务的启用开关
- `defaultEnabled`
  - 该任务默认是否启用
- `taskTone`
  - 用于提醒普通用户该任务的重要程度或风险等级

## 4. 为什么不用旧的 taskType

旧的 `taskType = main | child` 实际上混合表达了“一级循环入口”和“被其他任务触发”两种运行语义。

继续在这上面叠加 `title`、`special` 会带来问题：

- `title` 本身不是任务类型，而是展示行类型
- `special` 本质上又同时包含了：
  - 是否进入一级循环
  - 是否记录调度记录
- 同一字段同时承载执行语义和展示语义，会让后续模型越来越难扩展

因此，当前方案用：

- `rowType` 负责显示角色
- `triggerMode` 负责进入方式
- `recordSchedule` 负责记录策略

三者各自只承担一个维度。

## 5. 标题分组与 sectionId

### 5.1 设计方式

不单独建立 block 表，第一阶段直接复用标题行本身作为分组锚点：

- `rowType = title` 的任务行自身就是一个“块标题”
- 普通任务通过 `sectionId = 某个标题行 id` 表示归属

示例：

```text
title_daily   -> 标题行：每日任务
title_weekly  -> 标题行：每周任务

task_sign     -> 所属分组: title_daily
task_energy   -> 所属分组: title_daily
task_weekly_a -> 所属分组: title_weekly
```

### 5.2 优点

- 不需要额外的 block 表和 block 排序逻辑
- 标题和普通任务统一走同一套排序体系
- 拖拽排序、预览渲染、脚本保存都更简单
- 开发者在编辑器里能直观看到标题行本身

## 6. 缩进与分组的关系

- `sectionId` 决定“属于哪个块”
- `indentLevel` 决定“显示时缩进多少”

两者不要混用。

示例：

```text
每周任务
  周常副本
    周奖励领取
```

这里：

- `周常副本` 和 `周奖励领取` 都可能属于同一个 `sectionId = 每周任务`
- 但两者 `indentLevel` 不同

所以不能用“缩进量”推断“块级归属”。

## 7. 开发者编辑器交互规则

### 7.1 标题行

当 `rowType = title` 时：

- 开发者看到可编辑文本框
- 可修改标题名称
- 不显示或禁用以下字段：
  - `triggerMode`
  - `recordSchedule`
  - `defaultTaskCycle`
  - `showEnabledToggle`
  - `defaultEnabled`
  - `taskTone`
  - UI 字段绑定相关内容

### 7.2 任务行

当 `rowType = task` 时：

- 显示完整任务设置
- “所属分组”字段应为下拉框
- 下拉选项来自当前脚本内所有 `rowType = title` 的任务
- 实际保存的是标题行 `id`，即 `sectionId`

### 7.3 新建任务默认值

新建普通任务时建议默认：

```ts
rowType = 'task'
triggerMode = 'rootOnly'
recordSchedule = true
sectionId = 最近的标题行 id（如果存在）
indentLevel = 0
defaultTaskCycle = 'everyRun'
showEnabledToggle = true
defaultEnabled = true
taskTone = 'normal'
```

新建标题行时建议默认：

```ts
rowType = 'title'
recordSchedule = false
sectionId = undefined
indentLevel = 0
showEnabledToggle = false
defaultEnabled = true
taskTone = 'normal'
```

## 8. 普通用户预览规则

### 8.1 标题行

`rowType = title` 时：

- 作为块级标题显示
- 不显示启用开关
- 不显示周期
- 不显示缩进
- 不参与任务运行

### 8.2 任务行

`rowType = task` 时：

- 按 `indentLevel` 做缩进
- 如果 `showEnabledToggle = true`，显示启用开关
- 否则只显示任务名称和配置项
- 周期信息显示在任务行末尾
- `taskTone` 影响任务名称 / 左侧强调条 / 风险标签

### 8.3 风险色建议

- `normal`
  - 默认文本色
- `warning`
  - 黄色 / 琥珀色强调
- `danger`
  - 红色强调

注意：

- 不建议直接存十六进制颜色
- 应存业务语义级别，再由前端主题统一映射成颜色表现

## 9. 状态控制节点扩展

当前 `StateTarget = Task` 时，仅有 `done / skip` 两种状态不足以表达“任务是否启用”。

因此建议在 `StateStatus` 中新增：

```ts
enabled { value: boolean }
```

对应 UI 语义：

- `value = true` 表示启用
- `value = false` 表示禁用

保留：

- `done`
- `skip`

这样：

- 流程步骤可以控制任务完成态
- 也可以控制任务启用态

## 10. 时间模板与默认值覆盖

脚本任务定义层提供默认值：

- `defaultTaskCycle`
- `defaultEnabled`

时间模板层只负责覆盖值，而不回写脚本定义。

优先级应为：

1. 脚本任务默认值
2. 时间模板覆盖值
3. 运行时临时状态

对应地，模板值结构中应允许覆盖：

- 变量值
- 任务启用状态
- 任务周期

## 11. 后端落地建议

### 11.1 ScriptTaskTable 新增字段

Rust `ScriptTaskTable` 需要新增：

- `row_type`
- `trigger_mode`
- `record_schedule`
- `section_id`
- `indent_level`
- `default_task_cycle`
- `show_enabled_toggle`
- `default_enabled`
- `task_tone`

### 11.2 数据库迁移

`script_tasks` 表需要新增对应列，并对旧数据做迁移：

- 旧 `task_type = main` -> `trigger_mode = rootOnly`
- 旧 `task_type = child` -> `trigger_mode = linkOnly`
- 旧数据统一补：
  - `row_type = task`
  - `record_schedule = true`
  - `indent_level = 0`
  - `default_task_cycle = everyRun`
  - `show_enabled_toggle = true`
  - `default_enabled = true`
  - `task_tone = normal`

### 11.3 调度记录

后续运行时接入真实任务执行链路时：

- 仅 `rowType = task` 的任务可执行
- 是否写调度记录由 `recordSchedule` 控制
- `defaultTaskCycle` 或模板覆盖值用于最终调度判断

## 12. 前端落地建议

### 12.1 编辑器基础面板

任务基础设置中新增：

- 行类型
- 进入方式
- 所属分组
- 缩进量
- 默认周期
- 是否记录调度
- 是否显示启用开关
- 默认启用
- 风险等级

### 12.2 脚本级任务预览

任务预览不再只看当前任务，而应能体现整张任务表：

- 标题行
- 普通任务行
- 分组归属
- 缩进
- 风险色
- 周期

否则 `title / sectionId / indentLevel / taskTone` 的效果无法被开发者直观看到。

## 13. 第一阶段范围

本次改造第一阶段优先落地：

- 数据模型字段
- 数据库存储与迁移
- 编辑器基础面板
- 普通用户任务预览
- `StateStatus.enabled`

暂不要求第一阶段完成：

- 运行时对 `triggerMode / recordSchedule / defaultTaskCycle` 的完整执行逻辑闭环
- 复杂的块级拖拽联动规则
- 时间模板覆盖值编辑界面
