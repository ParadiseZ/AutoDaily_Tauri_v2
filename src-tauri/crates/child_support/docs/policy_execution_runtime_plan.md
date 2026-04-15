# Policy Execution Runtime Plan

## Scope

This document records the unfinished child-side execution work for:

- `FlowControl::HandlePolicySet`
- `FlowControl::HandlePolicy`
- `ConditionNode::PolicyCondition`

Current status:

- editor / bindings / model definitions are in place
- `vision_core` already supports:
  - first-stage recall with `SearchRule`
  - second-stage refine with `PolicyConditionRule`
  - `VisionSnapshot` layout sorting and relative-position matching
- child executor 已完成第一轮最小闭环：
  - 图像变量读取与 `VisionSnapshot` 刷新已接入
  - `HandlePolicySet / HandlePolicy` 已可执行首个命中的策略
  - `PolicyCondition.input_var` 与 `PolicySetResult` 已接入
  - `PolicyExecutionResult.rounds[*].pageFingerprints / actionSignatures / actions` 已接入真实记录
    - `pageFingerprints` 在策略入口图像激活、后续显式 `Capture` / 图像变量刷新时写入
    - `actionSignatures / actions` 在真实设备动作执行时按实际目标写入

## Expected Runtime Flow

### 1. Resolve image input

`HandlePolicySet.input_var`, `HandlePolicy.input_var`, and explicit `PolicyCondition.input_var`
all point to an image variable.

Child executor should:

1. read the variable from `Scope`
2. decode / fetch the captured image
3. build one `VisionSnapshot`
4. store it in `RuntimeContext.last_snapshot`

When `PolicyCondition.input_var` is `None`, it should reuse the current step context image / snapshot
instead of rebuilding one.

### 2. HandlePolicySet

Expected single-run semantics:

1. resolve input image
2. build `VisionSnapshot`
3. expand `target: Vec<PolicySetId>` into ordered policy sets, groups, and policies
4. run first-stage recall with `SearchRule`
5. iterate candidate policies in configured order
6. execute that policy's `before_action`
7. allow linear `PolicyCondition` / task status / variable comparisons inside the policy steps
8. execute the policy's `after_action` if the policy path succeeds
9. write one `PolicyExecutionRound`
10. update top-level `PolicyExecutionResult` summary
11. stop after the first successful policy for this step

If no policy succeeds, output should still contain:

- `matched = false`
- empty or partial `rounds`

### 3. HandlePolicy

Expected single-run semantics:

1. resolve input image
2. build `VisionSnapshot`
3. iterate `target: Vec<PolicyId>` directly in order
4. execute each policy once
5. stop after the first successful policy
6. write `PolicyExecutionResult`

Unlike `HandlePolicySet`, this path does not need policy-set/group expansion.

### 4. PolicyCondition

`PolicyCondition` is a linear flow condition, not a precompiled policy-phase only rule.

Expected behavior:

- if `input_var` is present:
  - resolve that image variable
  - build / refresh `VisionSnapshot`
- else:
  - reuse current `RuntimeContext.last_snapshot`
- evaluate `PolicyConditionRule` against the snapshot
- return boolean result to surrounding `if / while / for`

If no valid snapshot exists, the condition should return `false` and emit a debug log.

## Result Model

`PolicyExecutionResult` is now split into:

- top-level summary
- `rounds: Vec<PolicyExecutionRound>`

Meaning:

- top-level `matched / policySetId / policyGroupId / policyId`
  represent the final summary for this step execution
- each `round` records one attempt / one policy execution pass

Each round contains:

- `matched`
- `policySetId`
- `policyGroupId`
- `policyId`
- `pageFingerprints`
- `actionSignatures`
- `actions`

## Action Recording Rules

`PolicyActionTargetRole` only describes targets inside one action:

- click: usually one `primary`
- swipe: `start` + `end`

To distinguish repeated actions in a single round, use:

- `round.actions.len()`
- `round.actions[n].actionIndex`
- `round.actions[n].kind`
- `round.actionSignatures`

Examples:

- single-round two clicks:
  - `actions = [{ actionIndex: 0, kind: Click }, { actionIndex: 1, kind: Click }]`
- single-round one swipe:
  - `actions = [{ actionIndex: 0, kind: Swipe, targets: [start, end] }]`
- single-round two swipes:
  - `actions = [{ actionIndex: 0, kind: Swipe }, { actionIndex: 1, kind: Swipe }]`

## Fingerprint Recording

Do not store page fingerprints only once at top level.

Fingerprints must be appended inside each round so later analysis can inspect:

- repeated page states
- delayed page response
- repeated identical action signatures

The child executor should only provide evidence.
Whether that evidence means "stuck" must be decided by later analysis logic, not by the recorder itself.

## Remaining Implementation Items

1. Add richer debug logs for snapshot reuse / rebuild decisions.
2. If later需要“动作后强制观察”，再单独评估是否要在当前显式图像变量模型之外引入自动截图，不在本版本默认开启。
