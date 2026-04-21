use crate::infrastructure::core::{Deserialize, Serialize};
use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::ort::execution_provider_mgr::{
    configure_or_switch_provider, InferenceBackend,
};
use crate::infrastructure::vision::base_traits::ModelHandler;
use crate::infrastructure::vision::vision_error::{VisionError, VisionResult};

use ndarray::{ArrayD, ArrayViewD};
use ort::inputs;
use ort::logging::LogLevel;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::TensorRef;
use std::path::PathBuf;
use std::sync::Mutex;

/// 基础模型结构 - 包含所有模型的通用字段

/// 模型来源
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Default, ts_rs::TS)]
#[ts(export)]
pub enum ModelSource {
    /// 内置模型 - 从 resources/models/ 加载
    /// 路径由程序自动解析，无需用户指定
    BuiltIn,

    /// 自定义模型
    /// - Dev 脚本: 使用 model_path 中的绝对路径
    /// - Published 脚本: 从 scripts/{id}/models/ 加载
    #[default]
    Custom,
}

#[derive(Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct BaseModel {
    #[serde(skip)]
    #[ts(skip)]
    pub session: Option<Mutex<Session>>,
    pub intra_thread_num: usize,
    pub intra_spinning: bool,
    pub inter_thread_num: usize,
    pub inter_spinning: bool,
    pub execution_provider: InferenceBackend,
    pub input_width: u32,
    pub input_height: u32,
    /// 模型来源 - BuiltIn 或 Custom
    #[serde(default)]
    pub model_source: ModelSource,
    /// 模型路径
    /// - BuiltIn: 此字段被忽略，由程序自动解析
    /// - Custom + Dev: 开发者指定的绝对路径
    /// - Custom + Published: 此字段被忽略，由程序解析为 scripts/{id}/models/
    #[ts(as = "String")]
    pub model_path: std::path::PathBuf,
    #[serde(skip)]
    #[ts(skip)]
    pub is_loaded: bool,
    pub model_type: ModelType,
}

impl std::fmt::Debug for BaseModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BaseModel[session:hidden, intra_thread_num: {}, intra_spinning: {}, inter_thread_num: {}, inter_spinning: {}, execution_provider: {:?}, input_width: {}, input_height: {}, model_source: {:?}, model_path: {:?}, is_loaded: {}, model_type: {:?}]",
            self.intra_thread_num,
            self.intra_spinning,
            self.inter_thread_num,
            self.inter_spinning,
            self.execution_provider,
            self.input_width,
            self.input_height,
            self.model_source,
            self.model_path,
            self.is_loaded,
            self.model_type
        )
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
pub enum ModelType {
    Yolo11,
    Yolo26,
    PaddleDet5,
    PaddleCrnn5,
}
impl BaseModel {
    fn resolve_builtin_model_path(&self) -> VisionResult<PathBuf> {
        let relative = match self.model_type {
            ModelType::PaddleDet5 => PathBuf::from("ppocr").join("ch_mobile_v5_det.onnx"),
            ModelType::PaddleCrnn5 => PathBuf::from("ppocr").join("ch_mobile_v5_rec.onnx"),
            ModelType::Yolo11 | ModelType::Yolo26 => {
                return Err(VisionError::IoError {
                    path: "[built-in-detector]".to_string(),
                    e: "当前不提供内置目标检测/文字检测模型，请切换为自定义路径".to_string(),
                })
            }
        };

        let mut candidates = vec![
            PathBuf::from("src-tauri").join("models").join(&relative),
            PathBuf::from("models").join(&relative),
            PathBuf::from("resources").join("models").join(&relative),
        ];

        if let Ok(current_exe) = std::env::current_exe() {
            if let Some(exe_dir) = current_exe.parent() {
                candidates.push(exe_dir.join("models").join(&relative));
                candidates.push(exe_dir.join("resources").join("models").join(&relative));
            }
        }

        candidates
            .into_iter()
            .find(|path| path.exists())
            .ok_or_else(|| VisionError::IoError {
                path: relative.to_string_lossy().to_string(),
                e: "未找到内置模型文件".to_string(),
            })
    }

    pub fn new(
        input_width: u32,
        input_height: u32,
        model_source: ModelSource,
        model_path: std::path::PathBuf,
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
            model_source,
            model_path,
            is_loaded: false,
            model_type,
        }
    }

    /// 加载 ONNX 模型并创建会话。
    ///
    /// 这里统一处理：
    /// - 内置/自定义模型路径解析
    /// - 执行器切换
    /// - ORT 线程与图优化配置
    pub fn load_model_base<T: ModelHandler>(&mut self, model_type_name: &str) -> VisionResult<()> {
        // 1. 解析模型路径
        let final_path = match self.model_source {
            ModelSource::BuiltIn => self.resolve_builtin_model_path()?,
            ModelSource::Custom => self.model_path.clone(),
        };

        Log::debug(&format!(
            "加载{}模型, 路径: {:?}",
            model_type_name, final_path
        ));

        // 2. 创建session builder
        let result =
            configure_or_switch_provider(None, self.execution_provider.name()).map_err(|e| {
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
            .commit_from_file(&final_path)
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

    /// 执行一次推理，并在 ORT 输出 view 仍然有效时直接消费输出。
    ///
    /// 这个入口主要给检测链路使用，用来绕开“先拷贝整块输出再后处理”的额外开销。
    pub fn inference_with_output_view<R, F>(
        &self,
        input: ArrayViewD<'_, f32>,
        input_node_name: &str,
        output_node_name: &str,
        process_output: F,
    ) -> VisionResult<R>
    where
        F: for<'a> FnOnce(ArrayViewD<'a, f32>) -> VisionResult<R>,
    {
        if let Some(session_mutex) = self.session.as_ref() {
            let standard_input = (!input.is_standard_layout()).then(|| {
                Log::debug("推理输入不是标准连续布局，复制为标准布局后再送入ORT");
                input.to_owned()
            });
            let input_view = standard_input
                .as_ref()
                .map(|array| array.view())
                .unwrap_or(input);

            // 创建输入张量
            let input_tensor = TensorRef::from_array_view(input_view).map_err(|e| {
                VisionError::DataProcessingErr {
                    method: "inference_base".to_string(),
                    e: e.to_string(),
                }
            })?;

            // 获取锁
            let mut session = session_mutex
                .lock()
                .map_err(|_| VisionError::InferenceErr {
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
                    method: "inference_with_output_view".to_string(),
                    e: e.to_string(),
                })?;
            Log::debug(&format!("模型输出维度: {}", view.ndim()));
            process_output(view)
        } else {
            Err(VisionError::IoError {
                path: "[推理阶段]".to_string(),
                e: "模型未加载".to_string(),
            })
        }
    }

    /// 执行一次通用推理，并返回拥有所有权的输出张量。
    ///
    /// 识别模型和其他仍然需要持有输出数据的链路继续走这个入口。
    pub fn inference_base(
        &self,
        input: ArrayViewD<'_, f32>,
        input_node_name: &str,
        output_node_name: &str,
    ) -> VisionResult<ArrayD<f32>> {
        self.inference_with_output_view(input, input_node_name, output_node_name, |view| {
            Ok(view.to_owned())
        })
    }
}
