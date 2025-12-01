use crate::domain::vision::result::{DetResult, OcrResult};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::ort::execution_provider_mgr::{
    configure_or_switch_provider, InferenceBackend,
};
use crate::infrastructure::vision::base_traits::ModelHandler;
use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};
use memmap2::Mmap;
use ndarray::{ArrayD, ArrayViewD};
use ort::inputs;
use ort::logging::LogLevel;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::TensorRef;
use std::sync::Mutex;

/// 基础模型结构 - 包含所有模型的通用字段

pub struct BaseModel {
    pub session: Option<Mutex<Session>>,
    pub intra_thread_num: usize,
    pub intra_spinning: bool,
    pub inter_thread_num: usize,
    pub inter_spinning: bool,
    pub execution_provider: InferenceBackend,
    pub input_width: u32,
    pub input_height: u32,
    //pub model_path : Option<String>,
    pub model_bytes_map: Mmap,
    pub is_loaded: bool,
    pub model_type: ModelType,
}

impl std::fmt::Debug for BaseModel{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BaseModel[session:hidden, intra_thread_num: {}, intra_spinning: {}, inter_thread_num: {}, inter_spinning: {}, execution_provider: {:?}, input_width: {}, input_height: {}, model_bytes_map: hidden, is_loaded: {}, model_type: {:?}]",
            self.intra_thread_num,
            self.intra_spinning,
            self.inter_thread_num,
            self.inter_spinning,
            self.execution_provider,
            self.input_width,
            self.input_height,
            self.is_loaded,
            self.model_type
        )
    }
}

#[derive(Debug)]
pub enum ModelType {
    Yolo11,
    PaddleDet5,
    PaddleCrnn5,
}

#[derive(Debug)]
pub enum PostprocessRes{
    Detection(Vec<DetResult>),
    Recognition(Vec<OcrResult>),
}

impl BaseModel {
    pub fn new(
        input_width: u32,
        input_height: u32,
        model_bytes_map: Mmap,
        execution_provider: InferenceBackend,
        intra_thread_num: usize,
        intra_spinning: bool,
        inter_thread_num: usize,
        inter_spinning: bool,
        model_type: ModelType,
    ) -> Self {
        Self {
            session: None,
            intra_thread_num,
            intra_spinning,
            inter_thread_num,
            inter_spinning,
            execution_provider,
            input_width,
            input_height,
            model_bytes_map,
            is_loaded: false,
            model_type,
        }
    }

    /// 通用的模型加载方法 - 消除重复代码
    pub fn load_model_base<T: ModelHandler>(
        &mut self,
        model_type_name: &str,
    ) -> VisionResult<()> {
        // 1. 解析模型路径

        Log::info(&format!("加载{}模型", model_type_name));

        // 2. 创建session builder
        let result = configure_or_switch_provider(None, "cuda").map_err(|e| {
            VisionError::SessionConfigFailed {
                method: "load_model_base".to_string(),
                e: e.to_string(),
            }
        })?;

        let session_builder = result.builder;
        Log::info(&format!("当前使用执行器: {}", result.active_backend.name()));

        // 4. 加载模型文件
        let session = session_builder
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| VisionError::SessionConfigFailed {
                method: "load_model_base".to_string(),
                e: e.to_string(),
            })?
            .with_intra_threads(self.intra_thread_num)
            .map_err(|e| VisionError::SessionConfigFailed {
                method: "load_model_base".to_string(),
                e: e.to_string(),
            })?
            .with_log_level(LogLevel::Error)
            .map_err(|e| VisionError::SessionConfigFailed {
                method: "load_model_base".to_string(),
                e: e.to_string(),
            })?
            .with_intra_op_spinning(self.intra_spinning)
            .map_err(|e| VisionError::SessionConfigFailed {
                method: "load_model_base".to_string(),
                e: e.to_string(),
            })?
            .with_inter_threads(self.inter_thread_num)
            .map_err(|e| VisionError::SessionConfigFailed {
                method: "load_model_base".to_string(),
                e: e.to_string(),
            })?
            .with_inter_op_spinning(self.inter_spinning)
            .map_err(|e| VisionError::SessionConfigFailed {
                method: "load_model_base".to_string(),
                e: e.to_string(),
            })?
            .commit_from_memory(&self.model_bytes_map)
            .map_err(|e| VisionError::SessionConfigFailed {
                method: "load_model_base".to_string(),
                e: e.to_string(),
            })?;


        // 5. 更新状态
    self.session = Some(Mutex::new(session));
    self.is_loaded = true;

        Log::debug(&format!("{}模型加载成功", model_type_name));
        Ok(())
    }

    /// 通用的推理方法 - 消除推理代码重复 🆕
    /// 正确使用ORT线程设置和Rayon线程池配合
    pub fn inference_base(
        &self,
        input: ArrayViewD<'_, f32>,
        input_node_name: &str,
        output_node_name: &str,
    ) -> VisionResult<ArrayD<f32>> {
        if let Some(session_mutex) = self.session.as_ref() {
            // 创建输入张量
            let input_tensor =
                TensorRef::from_array_view(input).map_err(|e| VisionError::DataProcessingErr {
                    method: "inference_base".to_string(),
                    e: e.to_string(),
                })?;

            // 获取锁
            let mut session = session_mutex.lock().map_err(|_| VisionError::InferenceErr {
                method: "inference_base".to_string(),
                e: "获取Session锁失败".to_string(),
            })?;

            // 执行推理
            let outputs = session
                .run(inputs![input_node_name => input_tensor])
                .map_err(|e| VisionError::InferenceErr {
                    method: "inference_base".to_string(),
                    e: e.to_string(),
                })?;

            // 提取输出
            let view = outputs[output_node_name]
                .try_extract_array::<f32>()
                .map_err(|e| VisionError::DataProcessingErr {
                    method: "inference_base".to_string(),
                    e: e.to_string(),
                })?;
            Log::debug(&format!("模型输出维度: {}", view.ndim()));
            // 处理不同的输出格式
            let output = match self.model_type {
                // YOLO需要转置
                ModelType::Yolo11 => view.t().to_owned(),
                ModelType::PaddleCrnn5 => view.to_owned(),
                ModelType::PaddleDet5 => view.to_owned(),
            };

            // 直接返回 ArrayDyn，由调用者处理具体的维度逻辑
            Ok(output)
        } else {
            Err(VisionError::IoError {
                path: "[推理阶段]".to_string(),
                e: "模型未加载".to_string(),
            })
        }
    }
}
