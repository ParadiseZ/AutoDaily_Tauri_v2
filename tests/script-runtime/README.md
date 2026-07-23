# AutoDaily 脚本运行时能力回归

`capability-suite.json` 只负责选择固定能力测试脚本、设备和运行范围，不再填写每个步骤的预期输出。

配置示例：

```json
{
  "schemaVersion": 1,
  "name": "能力测试",
  "scriptId": "SCRIPT_UUID",
  "deviceId": "DEVICE_UUID",
  "baselineDir": "baselines",
  "requiredCapabilities": ["action.click", "vision.ocr"],
  "scenarios": [
    {
      "name": "OCR 命中",
      "taskId": "TASK_UUID"
    },
    {
      "name": "完整流程"
    }
  ]
}
```

- 配置 `taskId` 时，通过 `RunTarget::Task` 单独执行该任务。
- 不配置 `taskId` 时，通过 `RunTarget::FullScript` 执行完整脚本。
- 数据库只有一个设备时可以省略 `deviceId`；存在多个设备时必须明确指定。
- `templateValues` 仍可用于固定脚本的输入变量。
- `requiredCapabilities` 继续作为覆盖门，检查固定脚本是否包含要求的步骤类型。

## 使用流程

1. 在专用模拟器中准备固定画面；视觉测试可以用图片查看器展示固定图片。
2. 首次运行 `record`。
3. 检查生成的基准文件，确认步骤轨迹、打印、变量、状态、视觉结果和设备操作正确。
4. 后续发布前运行 `verify`。
5. 测试脚本或正确行为有意变化时，再运行 `record` 并检查基准差异。

基准文件按脚本和运行目标自动命名：

```text
baselines/
  SCRIPT_UUID/
    task-TASK_UUID.json
    full-script.json
```

点击静态图片时，基准可以确认目标解析坐标、真实 ADB 操作及其执行结果，但不能证明静态图片会产生页面响应。
