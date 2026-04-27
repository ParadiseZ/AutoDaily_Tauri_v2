# 安卓服务端数据迁移临时梳理

生成时间：2026-04-25

## 数据来源

- 旧安卓项目本地数据库当前不可用。
- 本文只以 PostgreSQL 中的服务端表作为旧数据依据：
  - `public.script_info`
  - `public.script_set_info`
  - `public.script_action_info`
- 当前项目本地结构参考：
  - `scripts`
  - `script_tasks`
  - `policies`
  - `policy_groups`
  - `policy_sets`
  - `script_time_template_values`

## 已确认迁移口径

- 不需要持久保存旧 `script_id/flow_id/set_id/action_id` 作为当前项目的长期关联字段。
- 迁移期间可以临时用旧 id 做生成过程中的映射，但最终 `skipacid/rmacid/posadd/dropsetnext/action_ids` 等都应回填为当前项目新生成的 `task_id/policy_id/variable_id`。
- `script_id=0` 全局设置不需要按旧全局设置整体导入。当前项目已有大部分等价能力，缺口只剩点击坐标随机偏移。
- `flow_parent_id` 不需要按旧逻辑完整保留。旧项目用它关联 `script_action_info`；当前项目用策略集手动关联策略，不需要生成“父链 flow 策略组”的兼容结构。
- `SLIDER_SIXTH.action_ids` 不做静态降级，先补 counted loop，再迁移。
- 变量化文本和点击目标文本一起做，允许用户输入变量决定匹配文本和点击文本。
- 变量化文本第一期支持包含、不包含、等于、正则；不做忽略大小写。
- 点击坐标随机偏移放在脚本配置里，不放全局设置或设备信息页。
- `flow_id_type` 对应当前项目时间模板，本次迁移可以不管。
- `txt/txt_exc` 对应当前 `SearchRule` 的文本包含/不包含：`txt` 迁移为 `Txt`，`txt_exc` 迁移为 `Group(Not(Txt))`。
- `txt_label/txt_exc_label` 不需要迁移保留。
- `triggerMode` UI 不改成两个布尔选项；保留当前三态，但文案需要能识别“一级循环”以及“除一级循环外还可被跳转执行”的含义。
- 缩进量当前项目已经是 `script_tasks.indent_level` 顶层字段，Rust `ScriptTaskTable.indent_level` 已反序列化为结构体字段；前端也已有“缩进量”编辑项。它不属于 `ScriptTask.data.uiData`。

## 服务端数据概况

| 表 | 行数 | 备注 |
| --- | ---: | --- |
| `script_info` | 5 | 其中 `script_id=0` 是“全局设置”，不是具体游戏脚本。 |
| `script_set_info` | 163 | 设置/任务/标题混在一张表里。 |
| `script_action_info` | 693 | 旧动作、识别条件、执行次数、返回动作都在这里。 |

`script_info` 样本：

| script_id | script_name | package_name | model_path | classes_num | img_size |
| ---: | --- | --- | --- | ---: | ---: |
| 0 | 全局设置 | GLOBAL | n | 0 | 576 |
| 1 | 崩坏3rd(中文简体) | `com.miHoYo.enterprise.NGHSoD/com.miHoYo.overridenativeactivity.OverrideNativeActivity` | bh3/cn | 38 | 800 |
| 2 | 天下布魔(工口简体) | `com.pinkcore.tkfm/com.unity3d.player.UnityPlayerActivity` | tkfm/gk | 30 | 640 |
| 3 | 天下布魔(E服简体) | `com.pinkcore.tkfm.erolabs/com.unity3d.player.UnityPlayerActivity` | tkfm/ef | 30 | 640 |
| 4 | 交错战线(taptap) | `com.megagame.crosscore/com.mjsdk.app.MJUnityActivity` | jczx/cn | 22 | 640 |

## `script_set_info` 观察

总体分布：

- 总数 163。
- `flow_id IS NULL` 有 11 条，主要是 `script_id=0` 全局设置。
- `flow_parent_id` 非空 133 条。
- `action_ids` 非空 3 条，全部是 `SLIDER_SIXTH`。
- `is_show != 1` 有 30 条。
- `is_valid != 1` 有 15 条。
- `checked_flag=false` 有 11 条。
- `back_flag=1` 有 14 条。
- `flow_id_type` 当前全部是 4。

按 `set_type`：

| set_type | 行数 | 迁移倾向 |
| --- | ---: | --- |
| `CHECK_BOX` | 133 | 普通任务启用项，或隐藏/返回/无效任务。 |
| `TITLE` | 13 | 当前项目 `row_type=title`。注意服务端标题大多有 `flow_id`，不只是 `flow_id=null`。 |
| `DROPDOWN_MENU` | 7 | 当前项目 select/radio UI 变量。 |
| `RADIO_BUTTON` | 2 | 当前项目 radio UI 变量，当前样本在全局设置里。 |
| `SLIDER` / `SLIDER_SECOND` / `SLIDER_THIRD` / `SLIDER_FOURTH` / `SLIDER_FIFTH` | 各 1 | 当前项目 slider UI 变量或全局运行配置。 |
| `SLIDER_SIXTH` | 3 | 特殊：通过 `action_ids` 修改动作执行次数。 |

层级和执行选择：

- `set_level` 可直接迁移到当前项目 `indent_level`。
- `flow_parent_id` 不是纯展示层级。旧运行逻辑会把它拆成多个 flow id，并要求这些父链 flow 对应的设置都处于 checked 状态，才会执行当前设置。
- 当前项目迁移不需要保留这套父链兼容逻辑；只需要按当前项目的策略集设计，把旧 action 转成策略后手动或规则化关联到目标任务/策略集。
- `is_max_level` 是旧项目决定一级循环遍历候选的字段，并进一步决定会关联到哪些 `script_action_info`。
- `is_max_level=1` 是旧项目一级循环里的普通叶子执行项。
- `is_max_level=2` 是旧项目为了“不选择子项时父项也能正常工作”而设置的兜底入口；如果子项未选中则执行父项，如果子项选中则执行子项。
- `is_max_level=0` 多数是父级/分组节点，也可能包含废弃或中间节点。
- 当前项目不需要迁移 `is_max_level` 自身。是否一级循环、仅跳转或两者皆可由 `trigger_mode` 表达；旧 `is_max_level=2` 的场景如果仍需要，可由当前项目条件分支步骤表达。

隐藏和有效性：

- 旧前端列表按 `is_show=1` 显示。
- 当前项目已有 `script_tasks.is_hidden`，迁移时可按 `is_hidden = is_show != 1` 写入。
- 但当前项目需要确认：普通用户预览、模板值页、任务选择器是否都正确尊重 `is_hidden`。
- `is_valid=0` 是服务端更新删除语义。导入时不建议作为可见任务保留，更新同步时应删除或软删除对应当前数据。

`checked_flag`：

- 对应当前项目任务默认启用或模板启用覆盖。
- 迁移默认建议：
  - 写入 `script_tasks.default_enabled = checked_flag`。
  - 如果迁移要模拟旧用户当前配置，再写一份 `script_time_template_values.values_json.taskSettings[taskId].enabled`。

`back_flag`：

- 服务端样本 14 条，全部是 `CHECK_BOX`。
- 多数 `is_show=0`，说明旧项目把它作为隐藏返回动作配置。
- 当前项目更适合迁移成脚本的恢复/返回类 `policy_set`，并写到 `ScriptInfo.runtime_settings.recovery_task_id` 或等价恢复入口，而不是作为普通可见任务执行。

## `script_action_info` 观察

总体分布：

- 总数 693。
- `flow_id < 0` 有 32 条，属于返回类动作。
- `flow_id = 0` 有 37 条，属于脚本级公共兜底动作。
- `set_value` 为空 142 条。
- `execute_max > 0` 有 24 条。
- `label_pos != 0` 有 45 条，取值为 1、2、3、10。
- `rgb` 非空 64 条。
- `txt` 非空 653 条，`txt_exc` 非空 151 条。

动作 token 频次：

| token | 次数 | 当前迁移状态 |
| --- | ---: | --- |
| `click` | 538 | 可映射。 |
| `finish` | 196 | 可映射到任务完成状态。 |
| `skipacid` | 107 | 已补策略状态设置；迁移时旧 action id 需映射当前策略 id。 |
| `sleep` | 70 | 可映射到 waitMs。 |
| `rmacid` | 56 | 已补策略状态设置。 |
| `skipflowid` | 39 | 可映射到任务跳过状态。 |
| `skip` | 19 | 跳过当前策略。 |
| `1swipe*` / `2swipe*` | 28 | 可映射到百分比滑动预设。 |
| `mrmacids` | 8 | 当前项目语义按“批量设置指定策略为跳过”。 |
| `posadd` / `posminus` | 12 | 已补运行时策略点击索引变更；迁移时旧 action id 需映射当前策略 id。 |
| `uc` | 7 | 可映射为“指定任务未启用”的条件。 |
| `clickc` | 7 | 可映射到中心百分比点击；随机偏移需要补运行时配置。 |
| `clickPer` | 6 | 可映射到百分比点击。 |
| `relFAC` | 6 | 相对查找/点击筛选结果。 |
| `relLabFAC` | 2 | 基于 label 的相对查找/点击。 |
| `dropsetnext` | 2 | 已补 UI 变量轮转。 |
| `reboot` | 2 | 当前项目按重启目标应用映射。 |
| `nflow` | 1 | 已补当前任务条件。 |

识别条件字段：

- `int_label` / `int_exc_label`：目标检测 label 包含/排除。
- `txt_label` / `txt_exc_label`：旧 OCR 字典 label 包含/排除；本次迁移不保留。
- `txt` / `txt_exc`：对应当前 `SearchRule` 的 keyword/txt 文本规则，分别表示包含/不包含。当前模型里包含可迁移为 `SearchRule::Txt { pattern }`，不包含可迁移为 `SearchRule::Group { op: Not, items: [Txt] }`。
- `oper_txt`：旧运行时用来决定执行点击时偏向文本目标还是检测目标。当前项目的点击步骤已经限制了此内容（点击标签/文字）。
- `rgb`：颜色约束，当前项目应迁移成 color compare / 颜色过滤条件。
- `label_pos`：旧点击索引。当前项目对应 `PolicyInfo.cur_pos`，10代表最后一个元素，当前项目还未做此内容（用999表示）。

`flow_id` 的三类语义：

- `flow_id > 0`：普通任务相关动作。
- `flow_id = 0`：脚本级公共动作，旧查询会对每个任务额外 union 进来。当前项目应迁移成公共策略组/策略集，供多个任务复用。
- `flow_id < 0`：返回动作。当前项目应迁移到恢复/返回策略集。

`set_value` 的语义：

- 旧查询条件是 `(a.set_value = b.set_value OR a.set_value IS NULL)`。
- 因此 `set_value IS NULL` 的动作是该 flow 下所有配置值通用动作。
- 非空 `set_value` 是 select/radio/dropdown 当前值分支动作。
- 当前项目迁移时，建议转成条件分支：
  - 通用动作：不加变量值条件。
  - 特定值动作：增加变量等于某选项的条件。

## `action_ids` 与执行次数

`script_set_info.action_ids` 当前只有 3 条：

| set_id | script_id | set_name | set_type | set_value | set_default_value | action_ids | 关联动作 |
| ---: | ---: | --- | --- | --- | --- | --- | --- |
| 73 | 2 | 调教3* | `SLIDER_SIXTH` | `true` | `20` | `219` | action 219，`execute_max=0`，旧样本可能有服务端/本地值不一致。 |
| 110 | 3 | 调教3* | `SLIDER_SIXTH` | `true` | `20` | `219` | 另一个脚本内容，是服务端数据异常。当前项目两个不同服务器的相同游戏脚本通过任务级if分支决定启动目标的app、activity名称 |
| 156 | 4 | 扫荡1+ | `SLIDER_SIXTH` | `8` | `8` | `646,653` | 两个 action 当前 `execute_max=8`。 |

旧安卓前端对 `SLIDER_SIXTH` 的处理是：用户改 slider 后，把 `set_value.toInt()` 写到这些 action 的 `execute_max`。

当前项目情况：

- `Step.op=action` 有静态 `exec_max`。
- `Policy` / `ScriptTask` 也有静态 `exec_max`。
- 目前没有“执行次数来自 UI 变量”的字段。
- `forEach` 只能遍历输入集合，不是直接的 counted loop。
- `while` 可以表达条件循环，但没有天然“循环 N 次并自动计数”的简洁模板。

迁移建议：

1. 新增 counted loop，例如 `repeat(count_expr, flow)`。
2. `count_expr` 应能引用当前项目 UI 变量，所以 `SLIDER_SIXTH` 可迁移成一个 slider 变量驱动的 repeat。
3. 不把 `SLIDER_SIXTH` 强行迁移成多个复制出来的动作，也不做静态 `exec_max` 降级。

连带问题：

- 变量化执行次数解决的是“一个动作序列执行 N 次”。
- 迁移时还可能需要变量化文本，例如点击目标文本由用户输入的变量决定。
- 当前项目如果只有固定 OCR/text label 或固定文本条件，还需要补“文本条件/点击目标可引用变量”的能力。
- 变量化文本第一期支持：包含、不包含、等于、正则；不做忽略大小写。
- 这类能力建议和 counted loop 分开做：counted loop 属于流程控制，变量化文本属于策略匹配/点击目标参数化。

## 与当前项目的映射建议

### 脚本

- `script_info.script_name` -> `ScriptInfo.name`
- `package_name` -> 拆成 `pkg_name` 和 `activity_name`
- `model_path/classes_num/img_size` -> 当前模型配置
- `last_version` -> 云端/脚本版本字段
- `is_show` -> 脚本是否展示
- `script_id=0` -> 不导入为普通脚本；旧全局设置也不整体迁移，只保留其中当前项目仍缺的能力需求。

### 任务和标题

- `script_set_info` 迁移到 `script_tasks`。
- `set_type=TITLE` -> `row_type=title`。
- `CHECK_BOX` 且普通 flow -> `row_type=task`，`default_enabled=checked_flag`。
- `set_name` -> `script_tasks.name`
- `set_level` -> `indent_level`
- `sort` -> `index`
- `is_show` -> `is_hidden = is_show != 1`
- `is_valid=0` -> 导入时跳过或更新时删除/软删除。
- `flow_id`、`set_id`、旧 `script_id` 只作为迁移过程中的临时映射输入，不作为当前项目长期 metadata 保存。

### UI 变量

- `DROPDOWN_MENU` / `RADIO_BUTTON` / `SLIDER*` 迁移成当前任务 UI 字段 + variable catalog 条目。
- `set_default_value`：
  - select/radio/dropdown：候选项。
  - slider：默认值。
  - 但 `SLIDER_SIXTH.set_default_value` 和 `set_value` 样本不完全可靠，需要按类型特殊处理。
- `set_value`：
  - 如果代表脚本默认值，写 variable 默认值。
  - 如果代表用户当前值，写 `script_time_template_values.values_json.variables`。
  - 服务端没有设备/账号/时间模板上下文，迁移时需要决定放到哪个默认模板。
- `flow_id_type`：
  - 旧项目用于按时间段选择设置。
  - 当前项目对应时间模板，本轮迁移不处理。

### 动作与策略

建议把旧 `script_action_info` 每一条迁移成当前项目的策略或策略元素：

- `page_desc` -> 策略名或说明。
- `int_label/rgb/label_pos/txt/txt_exc` -> 策略匹配条件。
- `action_string` -> 策略命中后的步骤序列。
- `flow_id + set_value` -> 策略归属和条件分支。
- `sort` -> 策略顺序。
- `execute_max` -> 策略或动作执行次数；如果来自 `action_ids`，迁移为 counted loop。

复用关系：

- 旧 `flow_parent_id` 会让当前任务执行多个父链 flow 的 action。
- 当前项目不按旧父链自动生成兼容结构。
- 迁移目标是把旧 action 转为策略/策略元素，再依当前项目的策略集设计关联。
- `back_flag` 或 `flow_id<0` 进入恢复/返回 policy set。

## 需要当前项目补或改

1. `is_hidden` 的完整前端/运行时语义
   - DB 已有字段。
   - 需要确认普通用户预览、设置页、任务选择器、运行计划是否都按隐藏规则处理。

2. `is_valid` 同步删除语义
   - 当前任务/策略层没有统一服务端有效性字段。
   - 因为不做历史兼容同步，首次迁移可直接跳过旧 `is_valid=0` 数据。

3. 点击坐标随机偏移
   - 旧全局设置里有“随机点击范围”。
   - 当前项目中心点击/百分比点击已有，但随机偏移还没有。
   - 已确认放到脚本配置里，运行时所有该脚本下的坐标点击按脚本配置应用偏移。

4. counted loop
   - 为 `SLIDER_SIXTH.action_ids` 准备。
   - 明确使用 counted loop，而不是 `exec_max_expr` 静态扩展。
   - 需要前端步骤编辑器、Rust `FlowControl`、执行器都支持。

5. 变量化文本/点击目标参数
   - 需要同时支持由用户输入变量决定匹配文本和点击目标文本。
   - 用于旧数据里未来可能从固定 `txt/txt_label` 升级成用户可配置目标的场景。
   - 第一期支持包含、不包含、等于、正则；不做忽略大小写。
   - 这和 counted loop 是两类能力，建议拆开实现。

6. 任务进入方式 UI 降低理解成本
   - 当前项目 `triggerMode` 有“一级循环 / 仅跳转 / 两者都可”三种。
   - 这与旧 `is_max_level` 不再一一对应，但迁移和手工配置时可能仍然费脑。
   - 不改成两个布尔选项；保留当前三态。
   - UI 文案需要让用户能识别“一级循环”，并理解“除一级循环外还可被跳转执行”的含义。

7. `flow_id=0` 公共动作
   - 当前项目可作为公共策略/策略组复用，或迁移后由开发者手动关联到需要的策略集。

8. `flow_id<0` 与 `back_flag=1` 返回动作
   - 应迁移为恢复/返回策略集。
   - 不建议显示为普通用户任务。

9. `relFAC` / `relLabFAC`
   - 服务端数据中存在 8 次。
   - 已确认语义方向分别是相对查找/点击筛选结果、基于 label 的相对查找/点击。
   - 当前项目已有 `PolicyConditionRule::Relative`，前端也已有“相对位置”策略条件编辑器。
   - 后端支持 OCR 文本锚点 / 检测标签锚点、方向、目标类型、取值类型和比较运算。
   - 迁移时只需要把旧 `relFAC/relLabFAC` 参数映射到当前 `policyCondition.relative`，不需要新增相对位置能力。

10. `skip`
   - 服务端有 19 次。
   - 已确认语义为跳过当前策略。

11. `oper_txt`
    - 已确认当前项目点击步骤通过“点击标签/文字”等目标类型限制表达，不再需要单独迁移为一个字段。

12. `label_pos=10`
    - 已确认表示最后一个元素。
    - 当前项目计划用 `999` 表示最后一个元素，需要补约定和 UI 文案。

## 仍需进一步确认的问题

暂无。

## 建议迁移准备顺序

1. 在脚本配置里补点击坐标随机偏移，并接入运行时坐标点击。
2. 补 counted loop，用于 `SLIDER_SIXTH.action_ids`。
3. 补变量化文本匹配和点击目标文本能力。
4. 补 `label_pos=999` 表示最后一个元素，并让 UI 文案明确。
5. 优化任务 `triggerMode` 的前端表达，降低“一级循环/仅跳转/两者皆可”的理解成本。
6. 补 `relFAC` / `relLabFAC` 到当前相对位置条件的导入映射。
7. 检查 `is_hidden` 在前端预览、设置页、选择器中的一致行为。
8. 再写迁移器读取 PostgreSQL 三张表，生成当前项目脚本、任务、变量、策略、策略组、策略集。
