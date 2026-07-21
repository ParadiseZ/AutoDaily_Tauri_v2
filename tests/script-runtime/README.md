# AutoDaily 脚本运行时能力回归

这里存放运行时读取的测试数据，不属于 Cargo 源码：

```text
tests/script-runtime/
├── capability-suite.template.json
├── fixtures/
│   ├── capability-bundle.json        # 运行前由应用当前 bundle 构建链导出
│   ├── screenshots/
│   └── real-vision/
└── README.md
```

仓库只提交场景模板，不提交伪造的 bundle。首次使用时复制模板为
`capability-suite.json`，再把应用导出的完整能力脚本保存为
`fixtures/capability-bundle.json`。这样脚本结构始终以程序当前真实类型和构建链为准，
不会在测试包里演变出第二套需要同步维护的脚本定义。

## 一份脚本，多项任务

能力测试脚本应把任务作为隔离场景，例如：

1. OCR 命中与文字点击。
2. OCR 未命中。
3. 检测命中与标签点击。
4. 搜索条件文字及策略集命中。
5. 策略集未命中和结果清理。
6. If、While、ForEach、Repeat 及 Break/Continue。
7. 变量、Rhai、Filter、ColorCompare、RelativeFilter。
8. task/policy 状态设置。
9. 策略集、策略组和策略的动态绑定与卸载。
10. Task Link、StopScript 和执行次数限制。
11. 所有设备动作。
12. 固定图片真实视觉回归。

运行器为每个场景使用 `RunTarget::Task` 单独运行对应任务。任务可以访问完整 bundle 内的其他任务、策略、策略组和策略集，因此 `Link`、`HandlePolicySet`、状态设置及动态绑定仍执行真实逻辑。

## Bundle

`bundle` 文件直接使用 `runner_protocol::ScriptBundleSnapshot`，不定义第二套脚本 DTO。它应由当前应用的 bundle 构建链导出，包含：

- `scriptJson`
- `tasksJson`
- `policiesJson`
- `policyGroupsJson`
- `policySetsJson`
- `groupPoliciesJson`
- `setGroupsJson`

场景可以使用唯一的 `taskName`；脚本存在同名任务时必须改用 `taskId`。

## 视觉输入

注入模式直接使用真实 `DetResult` 和 `OcrResult` JSON，不再使用 `InjectedDetResult` 或 `InjectedOcrResult`。建议从 VisionLab 或调试日志导出真实结果，避免手写派生字段。

```json
{
  "mode": "injected",
  "ocrFrames": [
    {
      "ocrResults": [
        {
          "bounding_box": { "x1": 100, "y1": 200, "x2": 300, "y2": 260 },
          "stable_box": { "x1": 104, "y1": 200, "x2": 304, "y2": 264 },
          "stable_center": { "x": 200, "y": 232 },
          "txt": "开始游戏",
          "score": [0.99],
          "index": [0]
        }
      ]
    }
  ]
}
```

真实视觉模式的 `images.path` 可以是单图或目录。相对路径以场景 JSON 所在目录为准：

```json
{
  "mode": "real",
  "images": {
    "path": "fixtures/real-vision",
    "recursive": true,
    "extensions": ["png", "jpg", "jpeg"]
  },
  "capturesPerCase": 1
}
```

## 断言

`expected` 是最终完整结果的递归子集，不需要为每类断言增加 Rust 结构体：

```json
{
  "expected": {
    "execution": { "outcome": "completed" },
    "operations": [{ "type": "click", "x": 200, "y": 230 }],
    "taskStates": {
      "TASK_UUID": { "done": true, "execCount": 1 }
    }
  }
}
```

可断言的顶层结果包括：

- `execution`
- `operations`
- `variables`
- `taskStates`
- `policyStates`
- `actionStates`
- `policySetBindings`
- `policyGroupBindings`
- `vision`

对象按递归子集比较；数组按完整顺序比较。

## 能力覆盖门

`requiredCapabilities` 声明发布前必须出现在能力脚本中的步骤。运行器递归扫描所有任务、Sequence、If/Else、循环和 `then_steps`，缺少任一能力时整套测试失败。模板已经列出当前领域枚举支持的全部步骤。
