## 反应式决策执行器（DDD 设计小结）

目标：以“感知→守卫→策略→执行”的反应式模型，替代线性流水线作为顶层调度；通过可复用子流程承载可视化、可编排的细节步骤。适配未知/非线性页面、用户干预、弹窗干扰。

### 数据形态（领域模型）
- GuardDef：全局拦截处理（高优先级），命中即先处理；用于更新弹窗、断网、误触等。
- PolicyDef：策略条目。针对某个 when_goal，若条件命中则执行对应动作（内置动作或 SubFlow）。
- SubFlowDef：可复用子流程（小型图/序列），用于“进入舰团入口”“回主页”“购买流程”等幂等流程。
- ConditionGroup：支持 And/Or 组合；叶子条件用字符串表达式，后续可替换为更强表达式引擎。
- ActionRef：引用内置动作或子流程（按 id 复用）。

### 执行循环（最小骨架）
1. Perceive：采集页面态/弹窗/按钮/文字等，写入黑板（上下文）。
2. Guards：按优先级遍历守卫，命中即执行并短路，确保稳定状态。
3. Policy Select：根据当前目标（任务目标）与上下文，选择命中的最高优先级策略，得到一个 Action。
4. Act：执行 Action；若为 SubFlow，则进入子流程解释器（顺序/If/While/Router）。
5. 迭代：最小等待后回到 Perceive。

### 复用与公共策略
- 公共策略/子流程/守卫放在 common 仓库（JSON 文件），脚本内策略可按需覆盖或增补。
- ActionRef::SubFlow 通过 id 进行共享复用，例如“舰团入口”。

### 仓储（Repository）
定义接口 GuardRepository / PolicyRepository / SubFlowRepository；当前用 JSON 文件实现，后续可替换为数据库或远端服务。

### 与图式流水线的关系
- 顶层决策用反应式（策略选择），避免“第N步指针”依赖。
- 子流程依然可视化编排（If/While/Router/Click/Wait），利于开发者复用与调试。
- 子流程要求幂等：从任意相邻状态调用都安全。

### 取消与中断
- 后台循环持有 CancellationToken（或原子布尔），长操作拆分为短片段并在片段间检查取消。
- 守卫可触发取消（例如用户请求停止、发现长时间无响应）。

### 示例 JSON（公共 + 脚本内）
```json
// policies.common.json
[
  {"id":"goto_fleet","when_goal":"提交委托","priority":80, "condition":{"type":"Group","group":{"op":"And","items":[{"type":"Leaf","leaf":{"expr":"page=='主页'"}}]}}, "action":{"kind":"SubFlow","id":"ensure_fleet","params":{}}}
]
```

```json
// subflows.common.json
[
  {"id":"ensure_fleet","name":"确保进入舰团页","steps":[{"op":"Router","to":"舰团"},{"op":"WaitMs","ms":300}]}
]
```

脚本内（policies.json）可追加：
```json
[
  {"id":"submit_commission","when_goal":"提交委托","priority":100, "condition":{"type":"Group","group":{"op":"And","items":[{"type":"Leaf","leaf":{"expr":"page=='申请新委托'"}}]}}, "action":{"kind":"Builtin","name":"click","params":{"text":"提交"}}}
]
```

### 后续工作
- 表达式求值器与页面分类器对接（OCR/Det/模板匹配/ROI/缓存）。
- 子流程解释器实现（If/While/Router/Click/Verify）。
- 公共库与脚本库的合并/覆盖策略与校验。

