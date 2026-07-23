# Script Runtime Tests

该包用于验证 AutoDaily 自身提供的脚本步骤是否能在正式运行链中正常工作。它不面向脚本开发者，也不替代脚本编辑器现有的脚本、任务和策略调试功能。

每个场景都会从 SQLite 读取固定能力测试脚本及其任务、策略、策略组、策略集和关联关系，然后执行：

```text
ScriptBundleSnapshot
  -> RuntimeSessionSnapshot
  -> RunTarget::Task / RunTarget::FullScript
  -> ExecutionPlanAssembler
  -> ScriptScheduler
  -> ScriptExecutor
```

设备截图、OCR、目标检测、等待和 ADB 操作都使用正式实现。测试记录器只记录步骤轨迹、打印、实际设备操作和最终运行状态，不注入视觉结果，也不会跳过设备操作或等待。

## 运行方式

在 `src-tauri` 目录执行：

```powershell
cargo run -p script-runtime-tests -- record ../tests/script-runtime/capability-suite.json
cargo run -p script-runtime-tests -- verify ../tests/script-runtime/capability-suite.json
```

- `record`：正式执行成功后，将规范化结果写入配置的 `baselineDir`。已有基准会被覆盖，因此执行后需要人工检查 Git 差异。
- `verify`：重新正式执行，只读取基准并逐字段比较；缺少基准或结果变化时退出码为 `1`。

规范化会移除视觉置信度并将浮点数保留四位小数。动态会话 ID、时间戳和耗时不进入结果。图片变量只记录类型、宽度和高度。

## 安全提示

能力脚本中的点击、滑动、返回、停止应用、启动应用和重启会作用于配置的真实设备。执行前应使用专用模拟器，确认当前画面和应用状态，并检查固定测试脚本没有作用于个人数据。

测试框架自身的单元测试不会连接设备：

```powershell
cargo test -p script-runtime-tests
```
