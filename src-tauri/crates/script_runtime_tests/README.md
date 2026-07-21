# Script Runtime Tests

本包只包含测试运行器代码。测试 bundle、场景 JSON 和图片位于仓库根目录 `tests/script-runtime`，均在运行时读取，不使用 `include_str!`，修改测试数据不会触发重新编译。

运行器为每个场景执行以下真实链路：

```text
ScriptBundleSnapshot
  -> RuntimeSessionSnapshot
  -> RunTarget::Task
  -> ExecutionPlanAssembler
  -> ScriptScheduler
  -> ScriptExecutor
```

每个任务是一个独立场景。每次运行都会替换 session、清空停止标记并重新初始化脚本运行状态，避免场景之间相互污染；完整 bundle 中的 tasks、policies、policy groups、policy sets 和关联表仍全部可用。

## 运行

在 `src-tauri` 目录执行：

```powershell
cargo test -p script-runtime-tests
cargo run -p script-runtime-tests -- ../tests/script-runtime/capability-suite.json
```

命令输出完整 JSON 报告，任一能力缺失或场景断言失败时退出码为 `1`。
`capability-suite.json` 是本地运行文件：先从仓库根目录下的
`tests/script-runtime/capability-suite.template.json` 复制，再配置应用当前构建链导出的
`ScriptBundleSnapshot` 和视觉样本路径。

具体配置格式、任务划分和 bundle 准备方式见：

- `tests/script-runtime/README.md`
- `tests/script-runtime/capability-suite.template.json`
