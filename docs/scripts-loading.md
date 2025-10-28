## 脚本加载与搜索（本地/云端，分页）

### 目录结构（本地）
- 根：`{scripts_dir}`（来自 `ScriptsConfig.dir`）
- 每个脚本：`{scripts_dir}/{script_id}/info.json`

`info.json` 为开发者编辑的脚本信息（ScriptInfo）。运行时分页展示使用 `ScriptMetadata`，仅在选中脚本时加载 `ScriptInfo` 全量。

### 扫描与分页
- 扫描一级目录（UUID），读取 `info.json`，构建索引 `ScriptMetadata`（名称、类型、优先级、创建与修改时间等）。
- 分页查询：先在内存索引中过滤/排序，再取页；仅对当前页所需脚本按需加载 `ScriptInfo` 并使用缓存。
- 保存：写入 `{script_id}/info.json`，并在索引中更新对应项；删除则移除整个 `{script_id}` 目录。

### 开发者编辑
- 编辑对象为 `info.json`；新字段通过 `#[serde(default)]` 兼容。
- 默认值与模板由上层合并器注入，保存时仍以脚本自身为准。

### 云端脚本搜索（预留）
- 后端提供搜索接口占位：根据关键词/类型/排序返回轻量元数据列表；不返回全量 `ScriptInfo`。
- 点击某条云端脚本时，可拉取详情或直接下载到本地 `{scripts_dir}/{script_id}`。

### 备注
- 本方案避免在分页中加载大体量 JSON，确保列表快速；与 UI 的“左列表+右详情”配合良好。

