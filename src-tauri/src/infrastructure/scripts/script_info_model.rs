use crate::infrastructure::core::{Deserialize, HashMap, ScriptId, Serialize};
use crate::infrastructure::scripts::script_error::{LoadFromCacheErr, ScriptError, ScriptResult};
use crate::infrastructure::scripts::script_info::{RuntimeType, ScriptInfo, ScriptType};
use std::cmp::Ordering;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicU8;
use crate::infrastructure::core::time_format::LocalTimer;
use crate::infrastructure::logging::log_trait::Log;

/// 分页查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptPageReq {
    pub page: usize,
    pub page_size: usize,
    pub sort_by: SortField,
    pub sort_order: SortOrder,
    pub filter: Option<ScriptFilter>,
}
impl Default for ScriptPageReq {
    fn default() -> Self {
        Self {
            page: 0,
            page_size: 20,
            sort_by: SortField::LastModified,
            sort_order: SortOrder::Desc,
            filter: None,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortField {
    CreateTime,
    Name,
    ExecutionCount,
    LastModified
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptFilter {
    pub script_type: Option<ScriptType>,
    pub status: Option<String>,
    pub name_contains: Option<String>,
}

/// 分页结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptPageResp {
    pub scripts: Vec<ScriptInfo>,
    pub total_count: usize,
    pub page: usize,
    pub page_size: usize,
    pub total_pages: usize,
}

/// 脚本管理器 - 处理分页和排序
pub struct ScriptManager {
    // 使用索引而不是全量加载
    script_index: Arc<RwLock<HashMap<ScriptId, Arc<RwLock<ScriptMeta>>>>>,
    cache_size: AtomicU8,
    // LRU缓存最近访问的脚本
    script_cache: std::collections::BTreeMap<ScriptId, ScriptInfo>,
}

/// 脚本元数据（用于索引，内存占用小）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMeta {
    pub checked: bool,
    pub script_id: ScriptId,
    pub name: String,
    pub file_path : PathBuf,
    // 本地（已下载）/云端/本地（自定义）
    pub script_type: ScriptType,
    // 运行时类型：内置/自定义
    pub runtime_type: RuntimeType,
    pub create_time: LocalTimer::Time,
    pub last_modified: LocalTimer::Time,
    pub execution_count: u64,
}

impl ScriptManager {
    pub fn new(cache_size: usize) -> Self {
        Self {
            script_index: Arc::new(RwLock::new(HashMap::new())),
            cache_size: AtomicU8::from(cache_size),
            script_cache: std::collections::BTreeMap::new(),
        }
    }

    /// 分页查询脚本
    pub async fn get_scripts_page(&mut self, scripts_dir: &PathBuf, request: ScriptPageReq) -> ScriptResult<ScriptPageResp> {
        // 1. 从索引中获取符合条件的脚本元数据
        let mut filtered_metadata: Vec<_> = {
            self.script_index.read().await.values()
                .filter(async |meta| self.matches_filter(meta.read().await, &request.filter))
                .cloned()
                .collect()
        };

        // 2. 排序
        self.sort_metadata(&mut filtered_metadata, &request.sort_by, &request.sort_order);

        // 3. 分页
        let total_count = filtered_metadata.len();
        let start_idx = request.page * request.page_size;
        let end_idx = (start_idx + request.page_size).min(total_count);

        let page_metadata = &filtered_metadata[start_idx..end_idx];

        // 4. 按需加载完整的ScriptInfo（只加载当前页需要的）
        let scripts = page_metadata.iter()
            .map(|meta| self.load_script_info(meta.script_id))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| ScriptError::LoadFromFileErr { path: scripts_dir.to_string_lossy().to_string(), e: e.to_string() })?;

        Ok(ScriptPageResp {
            scripts,
            total_count,
            page: request.page,
            page_size: request.page_size,
            total_pages: (total_count + request.page_size - 1) / request.page_size,
        })
    }

    /// 从文件加载脚本信息（带缓存）
    pub async fn load_script_info(&mut self, script_id: ScriptId) -> ScriptResult<ScriptInfo> {
        // 先检查缓存
        /*if let Some(script) = self.script_cache.get(&script_id) {
            return Ok(script.clone());
        }*/

        // 从文件加载
        if let Some(entry) = self.script_index.read().await.get(&script_id).read().await {
            let script = self.load_from_file(&entry.file_path)?;

            // 更新缓存（LRU逻辑）
            if self.script_cache.len() >= self.cache_size {
                // 移除最老的项
                if let Some((oldest_id, _)) = self.script_cache.iter().next() {
                    let oldest_id = *oldest_id;
                    self.script_cache.remove(&oldest_id);
                }
            }
            self.script_cache.insert(script_id, script.clone());

            Ok(script)
        } else {
            Err(LoadFromCacheErr { script_id })
        }
    }

    fn load_from_file(&self, file_path: &PathBuf) -> Result<ScriptInfo, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(file_path)?;
        let script: ScriptInfo = serde_json::from_str(&content)?;
        Ok(script)
    }

    fn matches_filter(&self, metadata: &ScriptMeta, filter: &Option<ScriptFilter>) -> bool {
        if let Some(filter) = filter {
            if let Some(ref script_type) = filter.script_type {
                if metadata.script_type != *script_type {
                    return false;
                }
            }
            if let Some(ref name_contains) = filter.name_contains {
                if !metadata.name.contains(name_contains) {
                    return false;
                }
            }
        }
        true
    }

    fn sort_metadata(&self, metadata: &mut Vec<ScriptMeta>, sort_by: &SortField, order: &SortOrder) {
        metadata.sort_by(|a, b| {
            let cmp = self.get_ordering(sort_by);

            match order {
                SortOrder::Asc => cmp,
                SortOrder::Desc => cmp.reverse(),
            }
        });
    }

    fn get_ordering(a: &ScriptMeta, b: &ScriptMeta, sort_field: &SortField) -> Ordering {
        match sort_field {
            SortField::CreateTime => a.create_time.cmp(&b.create_time),
            SortField::Name => a.name.cmp(&b.name),
            SortField::ExecutionCount => a.execution_count.cmp(&b.execution_count),
            SortField::LastModified => a.last_modified.cmp(&b.last_modified),
        }
    }

    /// 从脚本根目录初始化脚本索引
    /// 目录结构：{scripts_dir}/{script_id}/info.json
    pub async fn load_from_directory(&mut self, scripts_dir: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        {
            self.script_index.write().await.clear();
        }
        // 一级目录：script_id（UUID）
        let mut entries = tokio::fs::read_dir(scripts_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let dir_path = entry.path();
            if !dir_path.is_dir() { continue; }
            let info_path = dir_path.join("info.json");
            if !info_path.exists() { continue; }
            match self.load_script_metadata_from_file(&info_path).await {
                Ok(meta) => {
                    self.script_index.write().await.insert(meta.script_id, meta);
                },
                Err(e) => {
                    Log::error(&format!("加载索引数据失败，路径 {:?}: {}", info_path, e));
                }
            }
        }
        Log::error(&format!("从{}文件夹加载{}个脚本", scripts_dir.to_string_lossy().to_string(), self.script_index.read().await.len));
        Ok(())
    }

    /// 从 info.json 加载脚本元数据
    async fn load_script_metadata_from_file(&self, file_path: &PathBuf) -> Result<ScriptMeta, Box<dyn std::error::Error>> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let script: ScriptInfo = serde_json::from_str(&content)?;
        let last_modified = self.compute_last_modified(file_path).await?;
        let mut meta = script.script_meta;
        meta.last_modified = last_modified.to_string();
        Ok(meta)
    }

    async fn compute_last_modified(&self, info_path: &PathBuf) -> Result<u64, Box<dyn std::error::Error>> {
        // 轻量方案：仅 info.json；后续可扩展 decision/*.json 的最大mtime
        let file_meta = tokio::fs::metadata(info_path).await?;
        let last_modified = file_meta.modified()?.duration_since(std::time::UNIX_EPOCH)?.as_secs();
        Ok(last_modified)
    }

    /// 保存脚本到 info.json,参数路径包含script_id+文件名称
    pub async fn save_script(&mut self, path: &PathBuf, script: &ScriptInfo) -> Result<(), Box<dyn std::error::Error>> {
        //let file_path = scripts_dir.join(script.script_meta.script_id.to_string()).join("info.json");
        if let Some(parent) = path.parent() { tokio::fs::create_dir_all(parent).await?; }
        let content = serde_json::to_string_pretty(script)?;
        tokio::fs::write(path, content).await?;

        // 更新索引
        let metadata = script.script_meta.clone();
        {
            self.script_index.write().await.insert(metadata.script_id, metadata.clone());
        }

        // 更新缓存
        self.script_cache.insert(metadata.script_id, script.clone());

        Ok(())
    }

    /// 删除脚本目录（含 info.json 与其他附属文件）,参数不包含script_id
    pub async fn delete_script(&mut self, script_dir: &PathBuf, script_id: ScriptId) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(_entry) = self.script_index.write().await.remove(&script_id) {
            let dir = script_dir.join(script_id.to_string());
            let _ = tokio::fs::remove_dir_all(dir).await;
            self.script_cache.remove(&script_id);
        }
        Ok(())
    }

    /// 根据多个字段进行复合排序
    pub async fn get_scripts_with_complex_sort(
        &mut self,
        page: usize,
        page_size: usize,
        sort_rules: Vec<(SortField, SortOrder)>
    ) -> Result<ScriptPageResp, Box<dyn std::error::Error>> {
        let mut all_metadata: Vec<_> = self.script_index.read().await.values().map(|e| e.clone()).collect();

        //self.sort_metadata(all_metadata)

        // 复合排序
        all_metadata.sort_by(|a, b| {
            for (field, order) in &sort_rules {
                let cmp = self.get_ordering(field);

                let final_cmp = match order {
                    SortOrder::Asc => cmp,
                    SortOrder::Desc => cmp.reverse(),
                };

                if final_cmp != Ordering::Equal {
                    return final_cmp;
                }
            }
            Ordering::Equal
        });

        // 分页
        let total_count = all_metadata.len();
        let start_idx = page * page_size;
        let end_idx = (start_idx + page_size).min(total_count);

        let page_metadata = &all_metadata[start_idx..end_idx];
        let scripts = page_metadata.iter()
            .map(|meta| self.load_script_info(meta.script_id))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ScriptPageResp {
            scripts,
            total_count,
            page,
            page_size,
            total_pages: (total_count + page_size - 1) / page_size,
        })
    }
}