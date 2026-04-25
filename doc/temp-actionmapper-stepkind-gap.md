# ActionMapper 与当前 StepKind 能力临时对照

生成时间：2026-04-25

## 对照口径

- 安卓项目来源：`D:\Database\Project\AndroidStudio\AutoDaily\app\src\main\java\com\smart\autodaily\handler\ActionMapper.kt`
- 安卓动作常量：`D:\Database\Project\AndroidStudio\AutoDaily\app\src\main\java\com\smart\autodaily\constant\ActionString.kt`
- 安卓运行语义补充：`RunScript.kt`、`RunScriptExtension.kt`、`command\CommandImpl.kt`
- 当前项目步骤模型：`src-tauri/crates/runtime_engine/src/domain/scripts/script_decision.rs`
- 当前项目 Action / Flow / TaskControl：`src-tauri/crates/runtime_engine/src/domain/scripts/nodes/*.rs`
- 当前前端模板：`src/views/script-editor/editor-step/editorStepTemplates.ts`、`editorStepKinds.ts`

安卓项目没有 `StepKind`。这里比较的是：安卓 `ActionMapper` 映射出来的操作能力，当前项目是否已经能通过 `StepKind` 下的 `action`、`flowControl`、`taskControl`、`vision`、`dataHanding` 表达并执行。

## 已确认不需要补

| 安卓动作 | 处理结论 |
| --- | --- |
| `clickPart(type,part,idx)` | 不补。该能力基本不用，当前项目可用百分比点击替代。 |
| `jump` | 不补。安卓侧未使用。 |
| 设备级重启 | 不做。当前所谓 `reboot` 应按安卓项目语义理解为重启目标应用，不应该做重启设备。 |

## 已有或基本等价

| 安卓动作 | 当前项目对应 | 备注 |
| --- | --- | --- |
| `sleep` / `sleep(ms)` | `flowControl.waitMs` | 已有。 |
| `clickc` | `action.click` + `mode=percent` + `{ x: 0.5, y: 0.5 }` | 随机偏移当前没做，先不作为迁移必需项。 |
| `clickPer(x,y)` | `action.click` + `mode=percent` | 已有。 |
| `1swipeN` / `2swipeN` | `action.swipe` + `mode=percent` | 需要的是预设模板/导入映射，不是新底层能力。 |
| `swipePer(x1,y1,x2,y2)` | `action.swipe` + `mode=percent` | 已有。 |
| `back` | Android 返回键动作 | 已补为 `Action::Back` 和编辑器“返回”步骤。 |
| `reboot` | 重启目标应用 | 已把步骤文案改为“重启应用”，运行时按脚本配置的包名/Activity 执行停止后启动，不再触发设备重启。 |
| `finish` / `finish(taskId)` | 设置任务状态 `done=true` | 无参表示当前任务完成；有参表示指定任务完成。 |
| `skipflowid(id)` / `rmflowid(id)` | 设置任务状态 `skip=true/false` | 安卓 flowId 对应当前项目任务。 |

## 明确需要做或需要改

### 1. 状态设置从单目标改为多目标

当前 `TaskControl::SetState` 的目标是单个 `StateTarget`：

- 单个任务
- 单个策略

需要改成支持多目标，覆盖安卓这些能力：

- `skipacid(id)`：设置指定策略 `skip=true`。
- `rmacid(id)`：设置指定策略 `skip=false`。
- `mrmacids(ids...)`：安卓因为跳过项合并在同一个 HashSet 中，语义是批量移除跳过；当前项目不使用 HashSet 集合表达跳过，因此迁移目标是支持把指定的多条策略批量设置为跳过状态。
- `skipflowid(id)` / `rmflowid(id)`：设置指定任务 `skip=true/false`。
- `finish` / `finish(id)`：设置当前或指定任务 `done=true`。

建议模型方向：

- 后端把 `StateTarget` 扩展为支持 `policyIds` / `taskIds`，或在 `TaskControl::SetState` 上增加 `targets: Vec<StateTarget>`。
- 执行器批量遍历目标设置状态。
- 条件判断里的 `taskStatus` 是否也支持多目标，需要单独设计语义，见“待确认问题”。

前端建议：

- 不用 checkbox 展示所有项。
- 使用多选选择器，显示已选 chips/tag 列表。
- 选择器弹层内支持搜索任务/策略名称。
- 提供“添加目标”入口，单个下拉选择后追加到列表；列表项可删除、可点击定位。
- 对任务/策略分别使用当前已有的引用选项，不混成一张大列表。

### 2. 策略命中索引的运行时变更

安卓：

- `posadd(id)`：指定目标 action 的点击索引加 1。
- `posminus(id)`：指定目标 action 的点击索引减 1。
- 作用是多个相同元素命中时，改变下次命中的元素位置。
- 不持久化。

当前项目：

- `PolicyInfo.cur_pos` 已有，UI 上叫“当前位置”。
- 目前核对下来，运行时点击 OCR/Det 的选择函数还是 `first()` 或按 label index 查找，没有看到 `cur_pos` 参与点击选点。
- 当前也没有“指定目标元素/指定策略元素”的状态变更步骤。

已完成第一版：

- `cur_pos` 按策略级命中游标处理。
- 策略执行时记录当前策略上下文；策略内 OCR/Det 点击按运行时 `click_pos` 或基础 `PolicyInfo.cur_pos` 选择第 N 个匹配项。
- 新增 `posAdd` / `posMinus` 动作步骤，目标为单个策略。
- `posAdd` / `posMinus` 只写运行时 `PolicyState.click_pos`，不持久化到策略配置。
- 选择越界时按最后一个匹配项兜底，减到 0 后不再继续减。

冲突点：

- 当前项目也支持直接点击变量里的 OCR/Det 筛选结果。
- 变量点击本身没有天然归属到某个 `PolicyInfo.cur_pos`。
- 需要决定变量点击的索引来源：使用动作自己的 index 字段，还是允许绑定某个策略的运行时 `cur_pos`。

### 3. 变动指定任务的 UI 变量值

安卓：

- `dropsetnext(setId)`：把某个设置项的当前值切到候选列表下一个值，落库。
- 目的：发现进入错误场景后，自动调整用户 UI 配置，避免下次继续进入错误场景。

当前项目：

- 还没有步骤可以修改指定任务的 UI 变量值。
- 目标主要对应当前 UI 里的 select / radio 类型选项。

已完成第一版：

- 新增 `dropSetNext` 动作步骤。
- 前端目标选择为“任务 + 该任务下绑定了持久化 Input 变量的 Select/Radio 字段”。
- 运行时按字段 `options` 把当前模板变量值切到下一个选项；当前值不存在或未命中选项时，从第一个选项开始。
- 运行时直接更新当前设备 + 当前时间模板 + 当前账号作用域的 `script_time_template_values.values_json.variables[variableId]`。
- 同步刷新当前执行上下文中的模板变量快照，后续任务 hydrate 时会读到新值。

前端建议：

- 第一级选择任务。
- 第二级只列出该任务 UI 中可轮转的 select/radio 字段。
- 展示当前候选项预览，不用 checkbox。
- 可提供“方向：下一个/上一个”，默认下一个。

### 4. 条件能力补齐：任务未启用与 nflow

`uc(flowIds...)`：

- 安卓语义：如果指定任务未启用，则执行后面的内容。
- 当前项目应对应“条件判断：任务 enabled=false”。
- 当前已有 `taskStatus` 条件，任务目标支持 `enabled` 判断；需要确认前端/模板是否足够顺手，必要时加一个明确模板“任务未启用”。

`nflow(id)`：

- 安卓用途：页面元素一致时，只有当前任务是指定任务时才执行后续内容。
- 已补条件节点 `currentTaskIn`，表示当前任务属于指定任务列表。

### 5. back 步骤动作

已完成：

- 在 `Action` 枚举增加 `Back`。
- `DeviceAdapter` / `DeviceCtx` 增加 `back()`，Android adapter 封装已有 `ADBCommand::Back`。
- `execute_action_step` 分发 `Action::Back`，trace 使用 `PolicyActionKind::Back`。
- 前端动作类型、模板补“返回”。

### 6. 点击选点与 cur_pos

需要确认并补齐：

- 策略命中多个相同 OCR/Det 元素时，点击是否应该按 `PolicyInfo.cur_pos` 选第 N 个。
- 当前执行器的 `select_ocr_result` / `select_det_result` 没看到 `cur_pos` 参与。
- 如果要实现安卓等价行为，需要把当前策略上下文传给策略内动作执行，或在策略执行时把当前策略 id / cur_pos 写入 runtime context。

## 待确认问题

暂无。

## 已确认决策

- 状态批量目标保留旧 `target` 字段，同时新增 `targets` 列表；执行时优先用 `targets`，为空则回退旧 `target`。已实现。
- 变量点击与 `PolicyInfo.cur_pos` 的冲突先按策略作用域处理：策略内 action 使用策略 `cur_pos`，变量点击保持自身 first/match 行为；后续需要再给变量点击单独设计 `position/index`。
- `reboot` 直接改为重启目标应用，不新增设备重启入口。

## 建议实施顺序

1. 状态目标批量化：任务/策略 skip/done/enabled 的模型、执行器、前端多选。已完成第一版。
2. 条件节点：当前任务是/属于指定任务。已完成第一版；任务未启用可由现有任务状态条件表达。
3. `back` 与“重启应用”动作。已完成第一版。
4. `dropsetnext` 对应的任务 UI 变量轮转。已完成第一版。
5. 策略 `cur_pos` 的运行时应用与 `posadd/posminus`。已完成第一版。
6. 后续如需要，再给变量点击单独设计显式 `position/index`，不复用策略 `cur_pos`。
