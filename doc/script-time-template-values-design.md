# 脚本时间模板变量值设计

本文档用于明确“时间模板”和“脚本 UI 设置值”之间的关系，避免后续把值存错层。

## 1. 核心结论

- `time_templates` 只表示普通用户定义的时间段。
- `task.data.uiData` 只表示变量如何给用户配置，不存最终运行值。
- `script.data.variableCatalog` 只表示变量定义，不存模板下的最终配置值。
- 真正的“某个脚本在某个时间模板下的一整套变量值”，需要单独存储。

这层值的归属不是：

- 时间模板本身
- 脚本本身
- UI 字段本身

而是这对组合：

- `script_id`
- `time_template_id`

## 2. 用户操作语义

正确的用户流程是：

1. 普通用户先定义一个时间模板。
2. 给某台设备分配脚本时，选择该脚本使用哪个时间模板。
3. 用户在“该脚本 + 该时间模板”上下文里调整 UI 设置。
4. 保存时，落下这套模板专属变量值。

因此，“时间模板下的变量值”不是创建模板时生成的，而是在用户实际修改脚本设置后才保存。

## 3. UI 与变量的关系

- UI 是变量的配置入口，不是值的最终存储层。
- UI 字段绑定的是变量定义。
- 用户在 UI 上改出来的内容，最终应写成变量值快照。
- 运行时真正读取的是变量，而不是 UI 字段。

所以：

- `uiData` 负责“怎么配”
- `values_json` 负责“配成了什么值”

## 4. 表结构设计

新增表：

```sql
CREATE TABLE IF NOT EXISTS script_time_template_values (
    id TEXT PRIMARY KEY,
    script_id TEXT NOT NULL,
    time_template_id TEXT NOT NULL,
    values_json JSON NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE (script_id, time_template_id),
    FOREIGN KEY (script_id) REFERENCES scripts(id) ON DELETE CASCADE,
    FOREIGN KEY (time_template_id) REFERENCES time_templates(id) ON DELETE CASCADE
)
```

字段含义：

- `id`
  记录主键，便于单条保存、更新和删除。
- `script_id`
  这套值属于哪个脚本。
- `time_template_id`
  这套值属于哪个时间模板。
- `values_json`
  当前脚本在当前时间模板下保存的一整套变量值。
- `created_at`
  首次创建时间。
- `updated_at`
  最后一次保存时间。

## 5. values_json 的建议形态

`values_json` 建议拆成“变量值”和“任务级覆盖值”两部分：

```json
{
  "variables": {
    "var_pkg_name_id": "官服",
    "var_sweep_count_id": 5
  },
  "taskSettings": {
    "task_sign_in": {
      "enabled": true,
      "taskCycle": "daily"
    },
    "task_weekly_reward": {
      "enabled": false,
      "taskCycle": {
        "weekDay": 1
      }
    }
  }
}
```

其中：

- `variables`
  - 按变量目录里的稳定 `variableId` 存，不按 UI 字段存
- `taskSettings`
  - 存模板对任务默认值的覆盖
  - 第一阶段仅覆盖：
    - `enabled`
    - `taskCycle`

原因：

- UI 字段可能改名、删改、换控件
- 变量定义才是运行时真正依赖的对象
- 任务启用状态和任务周期属于“任务定义默认值”的模板覆盖，不属于变量值本身

脚本定义层提供默认值：

- `defaultEnabled`
- `defaultTaskCycle`

时间模板层只负责覆盖这些默认值，而不回写脚本定义。

## 6. 运行时读取方式

运行时不应该让条件步骤或表达式直接去查 `values_json`。

正确顺序：

1. 根据当前设备的 `device_script_assignments.time_template_id` 确定使用哪个时间模板。
2. 读取当前 `script_id + time_template_id` 对应的 `values_json`。
3. 先把 `taskSettings` 合并到任务默认配置，再把 `variables` 结合 `variableCatalog` 装载进运行时 `scope`。
4. `if / while / for / setVar(expr)` 等表达式都只从 `scope` 取值。

因此：

- `values_json` 是持久化层
- `scope` 是运行时上下文
- Rhai 负责在 `scope` 上执行表达式

额外约束：

- 模板值层不应直接修改 `ScriptTaskTable`
- 模板值层只保存“覆盖项”
- 如果普通用户没有改某个任务的启用状态或周期，运行时应回退到脚本定义层的默认值

## 7. 表达式命名建议

基于当前执行器实现，第一阶段建议仍采用平铺变量名，而不是命名空间对象。

推荐写法：

```rhai
var_pkg_name == "官服"
```

而不是：

```rhai
input.var_pkg_name == "官服"
```

原因：

- 当前执行器使用的是平铺 `Scope`
- 运行时主要通过 `scope.set_value(name, value)` 注入变量
- 在真正支持命名空间对象前，直接平铺变量名更稳

## 8. 对脚本编辑器的影响

当前脚本编辑器只需要承担两件事：

- 编辑变量定义
- 编辑 UI 绑定关系

以及补充：

- 编辑任务默认值（如 `defaultEnabled`、`defaultTaskCycle`）
- 编辑任务展示元数据（如标题行、分组、缩进、风险等级）

当前不应该在脚本编辑器里承担：

- 时间模板变量值的保存
- 设备分配时的模板值切换
- 模板值运行前装载调试

也就是说，脚本编辑器现在主要是“定义层”，不是“用户配置值层”。

其中需要明确：

- `defaultEnabled`
- `defaultTaskCycle`

属于脚本定义层默认值，目的是减少普通用户在时间模板里逐个任务重复设置的成本。

对应要求：

- UI 预览应尽量接近最终设置界面，而不是展示编辑器内部字段结构
- UI 字段不应该再把“字段键”当成主编辑入口
- 绑定关系要明确服务于变量，而不是服务于 UI 自身

## 9. 与 device_script_assignments 的关系

- `device_script_assignments.time_template_id` 只表示当前设备上，这个脚本使用哪个时间模板
- 它不负责存整套模板变量值
- `account_data` 继续保留作账号或设备专属扩展数据，本设计不占用它

## 10. 下一阶段工作

后续真正接设置页或设备运行时前，再继续补：

- 查询 / 保存 `script_time_template_values` 的 Tauri 命令
- 前端“脚本设置”页：先选时间模板，再改 UI 值
- 运行前把模板值装载进 `scope`
- 明确默认值、模板值、运行时临时值三者的覆盖顺序
