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
- `is_max_level=1` 是普通叶子执行项。
- `is_max_level=2` 是“如果子级没有选中，则执行自己；如果子级选中，则执行子级”的兜底项。
- `is_max_level=0` 多数是父级/分组节点，也可能包含废弃或中间节点。

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
| `skip` | 19 | 跳过当前策略。                                               |
| `1swipe*` / `2swipe*` | 28 | 可映射到百分比滑动预设。 |
| `mrmacids` | 8 | 当前项目语义按“批量设置指定策略为跳过”。 |
| `posadd` / `posminus` | 12 | 已补运行时策略点击索引变更；迁移时旧 action id 需映射当前策略 id。 |
| `uc` | 7 | 可映射为“指定任务未启用”的条件。 |
| `clickc` | 7 | 可映射到中心百分比点击；随机偏移暂不做。 |
| `clickPer` | 6 | 可映射到百分比点击。 |
| `relFAC` | 6 | 相对查找/点击筛选结果。 |
| `relLabFAC` | 2 | 基于 label 的相对查找/点击。 |
| `dropsetnext` | 2 | 已补 UI 变量轮转。 |
| `reboot` | 2 | 当前项目按重启目标应用映射。 |
| `nflow` | 1 | 已补当前任务条件。 |

识别条件字段：

- `int_label` / `int_exc_label`：目标检测 label 包含/排除。
- `txt_label` / `txt_exc_label`：OCR字典 label 包含/排除。
- `txt` / `txt_exc`：服务端保留的人类可读文本，样本中覆盖面很高；迁移时可写入策略说明/名称辅助字段，但执行匹配不能只靠它。
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

1. 优先新增一种可变量化的执行次数能力，例如：
   - `Step.action.exec_max_expr`
   - 或新增 counted loop：`repeat(count_expr, flow)`
2. 不建议把 `SLIDER_SIXTH` 强行迁移成多个复制出来的动作，因为用户调整 slider 后无法自然联动。
3. 若短期只做静态导入，可先把当前 `set_value` 写入静态 `exec_max`，但必须标记为兼容降级，不能算完整迁移。

## 与当前项目的映射建议

### 脚本

- `script_info.script_name` -> `ScriptInfo.name`
- `package_name` -> 拆成 `pkg_name` 和 `activity_name`
- `model_path/classes_num/img_size` -> 当前模型配置
- `last_version` -> 云端/脚本版本字段
- `is_show` -> 脚本是否展示
- `script_id=0` -> 不导入为普通脚本；应拆成全局默认设置或迁移到项目配置。

### 任务和标题

- `script_set_info` 迁移到 `script_tasks`。
- `set_type=TITLE` -> `row_type=title`。
- `CHECK_BOX` 且普通 flow -> `row_type=task`，`default_enabled=checked_flag`。
- `set_name` -> `script_tasks.name`
- `set_level` -> `indent_level`
- `sort` -> `index`
- `is_show` -> `is_hidden = is_show != 1`
- `is_valid=0` -> 导入时跳过或更新时删除/软删除。
- `flow_id`、`set_id`、旧 `script_id` 应保存到迁移 metadata，方便 action 和状态命令回填映射。

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

### 动作与策略

建议把旧 `script_action_info` 每一条迁移成当前项目的策略或策略元素：

- `page_desc` -> 策略名或说明。
- `int_label/rgb/label_pos` -> 策略匹配条件。
- `action_string` -> 策略命中后的步骤序列。
- `flow_id + set_value` -> 策略归属和条件分支。
- `sort` -> 策略顺序。
- `execute_max` -> 策略或动作执行次数；如果来自 `action_ids`，需要变量化执行次数能力。

复用关系：

- 旧 `flow_parent_id` 会让当前任务执行多个父链 flow 的 action。
- 当前项目可用“策略、策略组、策略集”表达：
  - 一个旧 flow 对应一个 policy group。
  - 一个旧任务的 policy set 包含它 `flow_parent_id` 中所有 flow 对应的 policy group，再加 `flow_id=0` 公共组。
  - `back_flag` 或 `flow_id<0` 进入恢复/返回 policy set。

## 需要当前项目补或改

1. `is_hidden` 的完整前端/运行时语义
   - DB 已有字段。
   - 需要确认普通用户预览、设置页、任务选择器、运行计划是否都按隐藏规则处理。

2. `is_valid` 同步删除语义
   - 当前任务/策略层没有统一服务端有效性字段。
   - 迁移工具需要根据旧 id 映射删除或软删除本地数据。

3. 旧 id 映射表或 metadata
   - 迁移必须能从旧 `flow_id/set_id/action_id/script_id` 找到当前 `task_id/policy_id/group_id/set_id`。
   - 否则 `skipacid/rmacid/posadd/dropsetnext/action_ids` 无法可靠回填。

4. 变量化执行次数
   - 为 `SLIDER_SIXTH.action_ids` 准备。
   - 推荐新增 counted loop 或 `exec_max_expr`。

5. `flow_parent_id` 到策略集的批量关联
   - 需要迁移器能生成“当前任务 = 多个旧 flow group 的组合”。
   - 这比单任务单策略更接近旧运行逻辑。

6. `is_max_level=2` 的兜底选择
   - 当前任务结构没有“子级没启用时执行父级”的内置语义。
   - 可迁移成条件任务，也可在导入时展开成显式条件分支。

7. `flow_id=0` 公共动作
   - 当前项目应作为公共策略组复用，而不是复制到每个任务。

8. `flow_id<0` 与 `back_flag=1` 返回动作
   - 应迁移为恢复/返回策略集。
   - 不建议显示为普通用户任务。

9. `relFAC` / `relLabFAC`
   - 服务端数据中存在 8 次，当前文档还不能确认完整语义。
   - 迁移前需要看旧 `ActionMapper` / `CommandImpl` 的具体实现。

10. `skip`
    - 服务端有 19 次。
    - 需要确认它在旧运行时是跳过当前 action、当前 flow，还是标记当前 action skipFlag。

11. `oper_txt`
    - 需要明确在当前策略点击里如何表达 OCR/Det 点击优先级。

12. `label_pos=10`
    - 不能直接等价为第 10 个候选。
    - 需要确认旧项目中是否有特殊含义，或只是业务数据中的第 10 个。

## 需要和你确认的问题

1. 旧 id 是否要在当前项目持久保存？
   - 我建议保存迁移 metadata，例如 `legacy.android.script_id/flow_id/set_id/action_id`。
   - 这样后续增量同步、`action_ids`、`skipacid`、`dropsetnext` 才能可靠回填。

2. `script_id=0` 全局设置怎么落？
   - 方案 A：导入到项目全局设置。
   - 方案 B：每个脚本复制一份默认变量。
   - 方案 C：先不导入，只作为迁移配置参考。

3. `flow_parent_id` 是否按旧逻辑完整保留？
   - 如果保留，应生成“任务策略集 = 父链 flow 策略组 + 自身策略组 + 公共组”。
   - 如果不保留，可以只迁移叶子任务动作，但会丢失旧项目复用父级动作的行为。

4. `is_max_level=2` 怎么表达？
   - 我倾向迁移成显式条件：“如果子任务都未启用，则执行该任务策略集”。
   - 但这会增加迁移出的步骤复杂度。

5. `flow_id_type=4` 当前全量数据都是全天。
   - 是否仍要为 1/2/3 预留迁移设计？
   - 如果要预留，应决定映射到当前 `defaultTaskCycle`、时间模板，还是新增任务时间段条件。

6. `SLIDER_SIXTH.action_ids` 要做完整支持还是先静态降级？
   - 完整支持需要变量化执行次数。
   - 静态降级只能按当前服务端值导入，后续用户调 slider 不会联动动作次数。

7. `txt/txt_exc` 是否需要作为可编辑文案保留？
   - 它们对调试和人工维护很有用。
   - 当前执行匹配仍应使用 label/id/颜色等结构化字段。

8. `relFAC` / `relLabFAC` 是否还有实际迁移需求？
   - 数据里出现次数少，但不是 0。
   - 如果这些脚本要完整跑通，需要补语义。

## 建议迁移准备顺序

1. 先定旧 id metadata 方案。
2. 补 `is_hidden` 在前端预览/设置页/选择器中的一致行为。
3. 定 `script_id=0` 全局设置落点。
4. 设计 `flow_parent_id` 到策略组/策略集的生成规则。
5. 补变量化执行次数能力，用于 `SLIDER_SIXTH.action_ids`。
6. 再写迁移器读取 PostgreSQL 三张表，生成当前项目脚本、任务、变量、策略、策略组、策略集。
7. 最后补 `relFAC` / `relLabFAC` / `skip` 等少量旧命令。
