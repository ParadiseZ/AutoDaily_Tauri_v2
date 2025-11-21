use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::ort::execution_provider_mgr::{
    configure_or_switch_provider, InferenceBackend,
};
use crate::infrastructure::vision::base_traits::ModelHandler;
use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};
use memmap2::Mmap;
use ndarray::Array4;
use ort::inputs;
use ort::logging::LogLevel;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::TensorRef;

/// 基础模型结构 - 包含所有模型的通用字段
#[derive(Debug)]
pub struct BaseModel {
    pub session: Option<Session>,
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
#[derive(Debug)]
pub enum ModelType {
    Yolo11,
    PaddleDet5,
    PaddleCrnn5,
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
    pub async fn load_model_base<T: ModelHandler>(
        &mut self,
        model_type_name: &str,
    ) -> VisionResult<()> {
        // 1. 解析模型路径
        /* let model_path = get_app_handle().await
        .path()
        .resolve(&self.model_path.clone().unwrap(), tauri::path::BaseDirectory::Resource)
        .map_err(|e| VisionError::LoadModelErr(format!("解析{}模型路径失败: {}", model_type_name, e)))?
        .to_string_lossy()
        .to_string();*/

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
            .commit_from_memory_directly(&self.model_bytes_map.as_ref())
            .map_err(|e| VisionError::SessionConfigFailed {
                method: "load_model_base".to_string(),
                e: e.to_string(),
            })?;
        //.commit_from_file(model_path)
        //.map_err(|e| VisionError::LoadModelErr(format!("加载{}模型文件失败: {}", model_type_name, e)))?;

        // 5. 更新状态
        //self.session = Some(Arc::new(Mutex::new(session)));
        self.session = Some(*session);
        self.is_loaded = true;

        Log::debug(&format!("{}模型加载成功", model_type_name));
        Ok(())
    }

    /// 通用的推理方法 - 消除推理代码重复 🆕
    /// 正确使用ORT线程设置和Rayon线程池配合
    pub async fn inference_base<T: ModelHandler>(
        &self,
        input: Array4<f32>,
        handler: &T,
    ) -> VisionResult<Array4<f32>> {
        if let Some(session) = &self.session {
            // 尝试获取推理线程池，如果没有则回退到普通推理
            let mut session_guard = session.lock().await;

            // 创建输入张量
            let input_tensor =
                TensorRef::from_array_view(&input).map_err(|e| VisionError::DataProcessingErr {
                    method: "inference_base".to_string(),
                    e: e.to_string(),
                })?;

            // 执行推理 - ORT内部使用单线程(由with_intra_threads(1)控制)
            let outputs = session_guard
                .run(inputs![handler.get_input_node_name() => input_tensor])
                .map_err(|e| VisionError::InferenceErr {
                    method: "inference_base".to_string(),
                    e: e.to_string(),
                })?;

            // 提取输出
            let view = outputs[handler.get_output_node_name()]
                .try_extract_array::<f32>()
                .map_err(|e| VisionError::DataProcessingErr {
                    method: "inference_base".to_string(),
                    e: e.to_string(),
                })?;

            // 处理不同的输出格式
            let output = match self.model_type {
                // YOLO需要转置
                ModelType::Yolo11 => view.t().into_owned(),
                ModelType::PaddleCrnn5 => view.into_owned(),
                ModelType::PaddleDet5 => view.into_owned(),
            };

            // 重塑输出形状
            let shape: [usize; 4] = {
                let s = output.shape();
                [s[0], s[1], s[2], s[3]]
            };

            output
                .into_shape_with_order(ndarray::Ix4(shape[0], shape[1], shape[2], shape[3]))
                .map_err(|e| VisionError::DataProcessingErr {
                    method: "inference_base".to_string(),
                    e: e.to_string(),
                })?
        } else {
            Err(VisionError::IoError {
                path: "[推理阶段]".to_string(),
                e: "模型未加载",
            })
        }
    }
}
