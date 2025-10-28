use crate::constant::sys_conf_path::SYSTEM_SETTINGS_PATH;
use crate::domain::app_handle::get_app_handle;
use crate::domain::entities::app_result::{AppError, AppResult};
use crate::domain::entities::config::log_conf::{Log, Logger};
use crate::domain::entities::config::sys_conf::SystemConfig;
use crate::domain::manager::conf_mgr::{ConfMgr, ConfigManager};
use crate::infrastructure::entities::vision::base_traits::{TextDetector, TextRecognizer};
use crate::infrastructure::entities::vision::det::{
    paddle_dbnet::PaddleDetDbNet,
    yolo::YoloDet,
};
use crate::infrastructure::entities::vision::rec::paddle_crnn::PaddleRecCrnn;
use ahash::{AHashMap, AHasher};
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::sync::Arc;
use tauri::path::BaseDirectory;
use tauri::Manager;
use tokio::sync::RwLock;

/// 检测器类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DetectorType {
    Yolo11,
    PaddleDbNet,
}

/// 识别器类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RecognizerType {
    PaddleCrnn,
}

/// 路径类型枚举
#[derive(Debug, Clone, Serialize, Deserialize,PartialEq, Eq)]
pub enum ModelPathType {
    /// 使用Tauri的Resource目录
    Resource(String),
    /// 使用自定义绝对路径
    Custom(String),
    /// 绝对路径
    Absolute(String)
}

/// 检测器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorConfig {
    pub detector_type: DetectorType,
    pub model_path: ModelPathType,
    pub execution_provider: String, // "cuda", "dml", "cpu"
    pub input_width: u32,
    pub input_height: u32,

    pub intra_thread_num : usize,
    pub intra_spinning : bool,
    pub inter_thread_num: usize,
    pub inter_spinning : bool,
    
    // YOLO特有配置
    pub confidence_thresh: Option<f32>,
    pub iou_thresh: Option<f32>,
    pub class_count: Option<usize>,
    pub class_labels: Option<Vec<String>>,
    pub class_file_path: Option<String>,
    
    // DBNet特有配置
    pub db_thresh: Option<f32>,
    pub db_box_thresh: Option<f32>,
    pub unclip_ratio: Option<f32>,
    pub use_dilation: Option<bool>,
}

impl DetectorConfig{
    pub fn new(
        detector_type: DetectorType,
        model_path: ModelPathType,
        execution_provider: String, // "cuda", "dml", "cpu"
        input_width: u32,
        input_height: u32,

        intra_thread_num : usize,
        intra_spinning : bool,
        inter_thread_num: usize,
        inter_spinning : bool,

        // YOLO特有配置
        confidence_thresh: Option<f32>,
        iou_thresh: Option<f32>,
        class_count: Option<usize>,
        class_labels: Option<Vec<String>>,
        class_file_path: Option<String>,

        // DBNet特有配置
        db_thresh: Option<f32>,
        db_box_thresh: Option<f32>,
        unclip_ratio: Option<f32>,
        use_dilation: Option<bool>,
    ) -> Self{
        Self{
            detector_type,
            model_path,
            execution_provider,
            input_width,
            input_height,
            intra_thread_num,
            intra_spinning,
            inter_thread_num,
            inter_spinning,
            confidence_thresh,
            iou_thresh,
            class_count,
            class_labels,
            class_file_path,
            db_thresh,
            db_box_thresh,
            unclip_ratio,
            use_dilation,
        }
    }
}

/// 识别器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizerConfig {
    pub recognizer_type: RecognizerType,
    pub model_path: ModelPathType,
    pub execution_provider: String,
    pub input_width: u32,
    pub input_height: u32,
    pub dict_path: Option<ModelPathType>,
    pub beam_width: Option<i32>,

    pub intra_thread_num : usize,
    pub intra_spinning : bool,
    pub inter_thread_num: usize,
    pub inter_spinning : bool,
}

/// 模型实例的唯一标识符
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ModelKey {
    pub model_type: String,
    pub model_path: String,
    pub execution_provider: String,
    pub config_hash: u64, // 配置参数的哈希值
}

/// 全局模型管理器 - 负责模型的共享和生命周期管理
pub struct GlobalModelManager {
    detectors: RwLock<AHashMap<ModelKey, Arc<dyn TextDetector + Send + Sync>>>,
    recognizers: RwLock<AHashMap<ModelKey, Arc<dyn TextRecognizer + Send + Sync>>>,
}

impl GlobalModelManager {
    pub fn new() -> Self {
        Self {
            detectors: RwLock::new(AHashMap::new()),
            recognizers: RwLock::new(AHashMap::new()),
        }
    }

    /// 获取或创建检测器实例（线程安全）
    pub async fn get_or_create_detector(
        &self,
        config: DetectorConfig,
    ) -> AppResult<Arc<dyn TextDetector + Send + Sync>> {
        let model_key = Self::create_detector_key(&config)?;
        
        // 首先尝试读锁获取现有实例
        {
            let detectors = self.detectors.read().await;
            if let Some(detector) = detectors.get(&model_key) {
                return Ok(detector.clone());
            }
        }
        
        // 如果不存在，获取写锁创建新实例
        let mut detectors = self.detectors.write().await;
        
        // 双重检查 - 防止并发创建
        if let Some(detector) = detectors.get(&model_key) {
            return Ok(detector.clone());
        }
        
        // 创建新的检测器实例
        let mut detector = OcrModelFactory::create_detector_impl(config.clone()).await?;
        detector.load_model();
        
        let arc_detector: Arc<dyn TextDetector + Send + Sync> = Arc::from(detector as Box<dyn TextDetector + Send + Sync>);
        
        detectors.insert(model_key, arc_detector.clone());
        Ok(arc_detector)
    }

    /// 获取或创建识别器实例（线程安全）
    pub async fn get_or_create_recognizer(
        &self,
        config: RecognizerConfig,
    ) -> AppResult<Arc<dyn TextRecognizer + Send + Sync>> {
        let model_key = Self::create_recognizer_key(&config)?;
        
        // 首先尝试读锁获取现有实例
        {
            let recognizers = self.recognizers.read().await;
            if let Some(recognizer) = recognizers.get(&model_key) {
                return Ok(recognizer.clone());
            }
        }
        
        // 如果不存在，获取写锁创建新实例
        let mut recognizers = self.recognizers.write().await;
        
        // 双重检查
        if let Some(recognizer) = recognizers.get(&model_key) {
            return Ok(recognizer.clone());
        }
        
        // 创建新的识别器实例
        let mut recognizer = OcrModelFactory::create_recognizer_impl(config.clone()).await?;
        recognizer.load_model();
        
        let arc_recognizer: Arc<dyn TextRecognizer + Send + Sync> = Arc::from(recognizer as Box<dyn TextRecognizer + Send + Sync>);
        
        recognizers.insert(model_key, arc_recognizer.clone());
        Ok(arc_recognizer)
    }

    /// 创建检测器的唯一键
    fn create_detector_key(config: &DetectorConfig) -> AppResult<ModelKey> {
        let mut hasher = AHasher::default();
        /*hasher.write_u32(config.input_width);
        hasher.write_u32(config.input_height);
        if let Some(c) = config.confidence_thresh{
            hasher.write_u32(c.to_bits());
        }
        if let Some(iou) = config.iou_thresh{
            hasher.write_u32(iou.to_bits());
        }
        if let Some(db) = config.db_thresh{
            hasher.write_u32(db.to_bits());
        }*/
        config.input_width.hash(&mut hasher);
        config.input_height.hash(&mut hasher);
        config.confidence_thresh.map(|f| f.to_bits()).hash(&mut hasher);
        config.iou_thresh.map(|f| f.to_bits()).hash(&mut hasher);
        config.db_thresh.map(|f| f.to_bits()).hash(&mut hasher);
        // ... 其他配置参数
        
        Ok(ModelKey {
            model_type: format!("{:?}", config.detector_type),
            model_path: Self::extract_path_string(&config.model_path),
            execution_provider: config.execution_provider.clone(),
            config_hash: hasher.finish(),
        })
    }

    /// 创建识别器的唯一键
    fn create_recognizer_key(config: &RecognizerConfig) -> AppResult<ModelKey> {
        let mut hasher = AHasher::default();
        config.input_width.hash(&mut hasher);
        config.input_height.hash(&mut hasher);
        config.beam_width.hash(&mut hasher);
        if let Some(dict_path) = &config.dict_path {
            Self::extract_path_string(dict_path).hash(&mut hasher);
        }
        
        Ok(ModelKey {
            model_type: format!("{:?}", config.recognizer_type),
            model_path: Self::extract_path_string(&config.model_path),
            execution_provider: config.execution_provider.clone(),
            config_hash: hasher.finish(),
        })
    }

    /// 从ModelPathType中提取路径字符串
    fn extract_path_string(path_type: &ModelPathType) -> String {
        match path_type {
            ModelPathType::Resource(path) => format!("resource:{}", path),
            ModelPathType::Custom(path) => format!("custom:{}", path),
            ModelPathType::Absolute(path) => path.into()
        }
    }

    /// 获取当前缓存的模型数量信息
    pub async fn get_cache_info(&self) -> (usize, usize) {
        let detectors = self.detectors.read().await;
        let recognizers = self.recognizers.read().await;
        (detectors.len(), recognizers.len())
    }

    /// 清理未使用的模型（可选的内存管理）
    pub async fn cleanup_unused_models(&self) {
        // TODO: 实现基于引用计数的清理逻辑
        // 当某个模型的Arc引用计数为1时（只有这里持有），可以考虑清理
    }
}

/// 全局模型管理器单例
static GLOBAL_MODEL_MANAGER: tokio::sync::OnceCell<GlobalModelManager> = tokio::sync::OnceCell::const_new();

/// 获取全局模型管理器实例
pub async fn get_global_model_manager() -> &'static GlobalModelManager {
    GLOBAL_MODEL_MANAGER
        .get_or_init(|| async { GlobalModelManager::new() })
        .await
}

/// OCR模型工厂
pub struct OcrModelFactory;

impl OcrModelFactory {
    /// 创建检测器实例（通过全局管理器，支持共享）
    pub async fn create_detector(
        config: DetectorConfig,
    ) -> AppResult<Arc<dyn TextDetector + Send + Sync>> {
        let manager = get_global_model_manager().await;
        manager.get_or_create_detector(config).await
    }
    
    /// 创建识别器实例（通过全局管理器，支持共享）
    pub async fn create_recognizer(
        config: RecognizerConfig,
    ) -> AppResult<Arc<dyn TextRecognizer + Send + Sync>> {
        let manager = get_global_model_manager().await;
        manager.get_or_create_recognizer(config).await
    }

    /// 内部方法：创建检测器实现（不通过管理器）
    async fn create_detector_impl(config: DetectorConfig) -> AppResult<Box<dyn TextDetector>> {
        let model_path_str = Self::resolve_model_path(&config.model_path).await?;
        
        match config.detector_type {
            DetectorType::Yolo11 => {
                let detector = YoloDet::new(
                    config.input_width,
                    config.input_height,
                    config.intra_thread_num,
                    config.intra_spinning,
                    config.inter_thread_num,
                    config.inter_spinning,
                    model_path_str,
                    config.execution_provider,
                    config.class_count.unwrap_or(1),
                    config.class_labels.unwrap_or_else(|| vec!["text".to_string()]),
                    config.confidence_thresh.unwrap_or(0.5),
                    config.iou_thresh.unwrap_or(0.4),
                );
                Ok(Box::new(detector))
            }
            DetectorType::PaddleDbNet => {
                let detector = PaddleDetDbNet::new(
                    config.input_width,
                    config.input_height,
                    config.intra_thread_num,
                    config.intra_spinning,
                    config.inter_thread_num,
                    config.inter_spinning,
                    model_path_str,
                    config.execution_provider,
                    config.db_thresh.unwrap_or(0.3),
                    config.db_box_thresh.unwrap_or(0.6),
                    config.unclip_ratio.unwrap_or(1.5),
                    config.use_dilation.unwrap_or(false),
                );
                Ok(Box::new(detector))
            }
        }
    }

    /// 内部方法：创建识别器实现（不通过管理器）
    async fn create_recognizer_impl(config: RecognizerConfig) -> AppResult<Box<dyn TextRecognizer>> {
        let model_path_str = Self::resolve_model_path(&config.model_path).await?;
        
        match config.recognizer_type {
            RecognizerType::PaddleCrnn => {
                // 加载字典
                let dict = if let Some(dict_path) = config.dict_path {
                    Self::load_dict_from_path_type(&dict_path).await?
                } else {
                    // 默认字符集
                    (0..=9).map(|i| i.to_string())
                        .chain(('a'..='z').map(|c| c.to_string()))
                        .chain(('A'..='Z').map(|c| c.to_string()))
                        .collect()
                };
                
                let recognizer = PaddleRecCrnn::new(
                    config.input_width,
                    config.input_height,
                    config.intra_thread_num,
                    config.intra_spinning,
                    config.inter_thread_num,
                    config.inter_spinning,
                    model_path_str,
                    config.execution_provider,
                    dict,
                    config.beam_width.unwrap_or(1),
                );
                Ok(Box::new(recognizer))
            }
        }
    }

    /// 解析模型路径
    async fn resolve_model_path(path_type: &ModelPathType) -> AppResult<String> {


        match path_type {
            ModelPathType::Resource(path) => {
                let absolute_path = get_app_handle().await.path()
                    .resolve(path, BaseDirectory::Resource)
                    .map_err(|e| AppError::UnknownError(format!("解析路径失败: {}", e)))?
                    .to_string_lossy()
                    .into_owned();
                Log::info(&format!("默认模型路径转换：{}", absolute_path));
                Ok(absolute_path)
            },
            ModelPathType::Custom(path) => {
                let absolute_path = get_app_handle().await.state::<ConfigManager>().get_conf::<SystemConfig>(SYSTEM_SETTINGS_PATH).await?.scripts_path + "/ "+ path;
                Log::info(&format!("自定义模型路径转换[{}]", absolute_path));
                Ok(absolute_path)
            },
            ModelPathType::Absolute(path)=>{
                Ok(path.into())
            }
        }
    }

    /// 从路径类型加载字典
    async fn load_dict_from_path_type(path_type: &ModelPathType) -> AppResult<Vec<String>> {
        let path_str = Self::resolve_model_path(path_type).await?;
        Self::load_dict(&path_str)
    }
    
    /// 加载字典文件
    fn load_dict(dict_path: &str) -> AppResult<Vec<String>> {
        use std::fs;
        let content = fs::read_to_string(dict_path)
            .map_err(|e| AppError::IoError(format!("无法读取字典文件 {}: {}", dict_path, e)))?;
        
        let dict: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();
            
        if dict.is_empty() {
            return Err(AppError::IoError(format!("字典文件{}为空", dict_path)));
        }
        
        Ok(dict)
    }


    /// 获取缓存统计信息
    pub async fn get_cache_statistics() -> (usize, usize) {
        let manager = get_global_model_manager().await;
        manager.get_cache_info().await
    }
}