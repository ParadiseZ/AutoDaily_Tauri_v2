use crate::infra::ort::{backend_name, configure_or_switch_provider};
use crate::infra::vision::base_traits::ModelHandler;
use crate::infra::vision::vision_error::{VisionError, VisionResult};
use domain_vision::{BaseModel as ModelConfig, InferenceBackend, ModelSource, ModelType};
use infra_logging::Log;

use ndarray::{ArrayD, ArrayViewD};
use ort::inputs;
use ort::logging::LogLevel;
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::TensorRef;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, TryLockError};

/// 基础模型结构 - 包含所有模型的通用字段

pub(crate) struct BaseModel {
    pub session: Option<Mutex<Session>>,
    session_pool: Vec<Mutex<Session>>,
    session_cursor: AtomicUsize,
    pub intra_thread_num: usize,
    pub intra_spinning: bool,
    pub inter_thread_num: usize,
    pub inter_spinning: bool,
    pub execution_provider: InferenceBackend,
    pub input_width: u32,
    pub input_height: u32,
    /// 模型来源 - BuiltIn 或 Custom
    pub model_source: ModelSource,
    /// 模型路径
    /// - BuiltIn: 此字段被忽略，由程序自动解析
    /// - Custom + Dev: 开发者指定的绝对路径
    /// - Custom + Published: 此字段被忽略，由程序解析为 scripts/{id}/models/
    pub model_path: std::path::PathBuf,
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

impl BaseModel {
    fn default_session_cursor() -> AtomicUsize {
        AtomicUsize::new(0)
    }

    pub fn resolve_model_path(&self) -> VisionResult<PathBuf> {
        match self.model_source {
            ModelSource::BuiltIn => Self::resolve_builtin_model_path(self.model_type),
            ModelSource::Custom => Ok(self.model_path.clone()),
        }
    }

    pub(crate) fn resolve_model_config_path(config: &ModelConfig) -> VisionResult<PathBuf> {
        match config.model_source {
            ModelSource::BuiltIn => Self::resolve_builtin_model_path(config.model_type),
            ModelSource::Custom => Ok(config.model_path.clone()),
        }
    }

    fn resolve_builtin_model_path(model_type: ModelType) -> VisionResult<PathBuf> {
        let relative = match model_type {
            ModelType::PaddleDet5 => PathBuf::from("ppocr").join("ch_mobile_v5_det.onnx"),
            ModelType::PaddleCrnn5 => PathBuf::from("ppocr").join("ch_mobile_v5_rec.onnx"),
            ModelType::PaddleDet6 => PathBuf::from("ppocr").join("small_v6_det.onnx"),
            ModelType::PaddleCrnn6 => PathBuf::from("ppocr").join("small_v6_rec.onnx"),
            ModelType::Yolo11 | ModelType::Yolo26 => {
                return Err(VisionError::IoError {
                    path: "[built-in-detector]".to_string(),
                    e: "当前不提供内置目标检测/文字检测模型，请切换为自定义路径".to_string(),
                });
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
            session_pool: Vec::new(),
            session_cursor: Self::default_session_cursor(),
        }
    }

    pub(crate) fn use_parallel_cpu_sessions(&self, session_intra_threads: usize) -> bool {
        matches!(
            self.model_type,
            ModelType::PaddleCrnn5 | ModelType::PaddleCrnn6
        ) && self.execution_provider == InferenceBackend::CPU
            && session_intra_threads > 0
            && self.intra_thread_num > session_intra_threads
            && self.intra_thread_num % session_intra_threads == 0
    }

    fn parallel_session_count(&self, session_intra_threads: usize) -> usize {
        if self.use_parallel_cpu_sessions(session_intra_threads) {
            self.intra_thread_num / session_intra_threads
        } else {
            1
        }
    }

    pub(crate) fn has_parallel_session_pool(&self) -> bool {
        self.session_pool.len() > 1
    }

    fn build_session(&self, final_path: &PathBuf, intra_threads: usize) -> VisionResult<Session> {
        let result = configure_or_switch_provider(None, &self.execution_provider).map_err(|e| {
            VisionError::SessionConfigFailed {
                method: "load_model_base".to_string(),
                e: e.to_string(),
            }
        })?;

        let session_builder = result.builder;
        Log::info(&format!(
            "当前使用执行器: {}",
            backend_name(&result.active_backend)
        ));

        session_builder
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| VisionError::SessionConfigFailed {
                method: "load_model_base".to_string(),
                e: e.to_string(),
            })?
            .with_intra_threads(intra_threads)
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
            .commit_from_file(final_path)
            .map_err(|e| VisionError::SessionConfigFailed {
                method: "load_model_base".to_string(),
                e: e.to_string(),
            })
    }

    fn with_session<R, F>(&self, run: F) -> VisionResult<R>
    where
        F: FnOnce(&mut Session) -> VisionResult<R>,
    {
        if !self.session_pool.is_empty() {
            let pool_len = self.session_pool.len();
            let start = self.session_cursor.fetch_add(1, Ordering::Relaxed) % pool_len;

            for offset in 0..pool_len {
                let idx = (start + offset) % pool_len;
                match self.session_pool[idx].try_lock() {
                    Ok(mut session) => return run(&mut session),
                    Err(TryLockError::WouldBlock) => {}
                    Err(TryLockError::Poisoned(_)) => {
                        return Err(VisionError::InferenceErr {
                            method: "inference_base".to_string(),
                            e: "获取Session池锁失败".to_string(),
                        });
                    }
                }
            }

            let mut session =
                self.session_pool[start]
                    .lock()
                    .map_err(|_| VisionError::InferenceErr {
                        method: "inference_base".to_string(),
                        e: "获取Session池锁失败".to_string(),
                    })?;
            return run(&mut session);
        }

        if let Some(session_mutex) = self.session.as_ref() {
            let mut session = session_mutex
                .lock()
                .map_err(|_| VisionError::InferenceErr {
                    method: "inference_base".to_string(),
                    e: "获取Session锁失败".to_string(),
                })?;
            run(&mut session)
        } else {
            Err(VisionError::IoError {
                path: "[推理阶段]".to_string(),
                e: "模型未加载".to_string(),
            })
        }
    }

    /// 加载 ONNX 模型并创建会话。
    ///
    /// 这里统一处理：
    /// - 内置/自定义模型路径解析
    /// - 执行器切换
    /// - ORT 线程与图优化配置
    pub(crate) fn load_model_base<T: ModelHandler>(
        &mut self,
        model_type_name: &str,
    ) -> VisionResult<()> {
        self.load_model_base_with_session_intra_threads::<T>(model_type_name, None)
    }

    pub(crate) fn load_model_base_with_session_intra_threads<T: ModelHandler>(
        &mut self,
        model_type_name: &str,
        session_intra_threads: Option<usize>,
    ) -> VisionResult<()> {
        // 1. 解析模型路径
        let final_path = self.resolve_model_path()?;

        Log::debug(&format!(
            "加载{}模型, 路径: {:?}",
            model_type_name, final_path
        ));

        let session_intra_threads = session_intra_threads
            .filter(|value| *value > 0)
            .unwrap_or(self.intra_thread_num);
        let session_count = self.parallel_session_count(session_intra_threads);

        if session_count > 1 {
            Log::debug(&format!(
                "识别模型启用并行Session池: {}个Session x {}个intra线程",
                session_count, session_intra_threads
            ));
        }

        let mut sessions = Vec::with_capacity(session_count);
        for _ in 0..session_count {
            sessions.push(Mutex::new(
                self.build_session(&final_path, session_intra_threads)?,
            ));
        }

        // 5. 更新状态
        if session_count == 1 {
            self.session = sessions.pop();
            self.session_pool.clear();
        } else {
            self.session = None;
            self.session_pool = sessions;
            self.session_cursor.store(0, Ordering::Relaxed);
        }
        self.is_loaded = true;

        Log::debug(&format!("{}模型加载成功", model_type_name));
        Ok(())
    }

    /// 执行一次推理，并在 ORT 输出 view 仍然有效时直接消费输出。
    ///
    /// 这个入口主要给检测链路使用，用来绕开“先拷贝整块输出再后处理”的额外开销。
    pub(crate) fn inference_with_output_view<R, F>(
        &self,
        input: ArrayViewD<'_, f32>,
        input_node_name: &str,
        output_node_name: &str,
        process_output: F,
    ) -> VisionResult<R>
    where
        F: for<'a> FnOnce(ArrayViewD<'a, f32>) -> VisionResult<R>,
    {
        let standard_input = (!input.is_standard_layout()).then(|| {
            Log::debug("推理输入不是标准连续布局，复制为标准布局后再送入ORT");
            input.to_owned()
        });
        let input_view = standard_input
            .as_ref()
            .map(|array| array.view())
            .unwrap_or(input);

        self.with_session(|session| {
            let input_tensor = TensorRef::from_array_view(input_view).map_err(|e| {
                VisionError::DataProcessingErr {
                    method: "inference_base".to_string(),
                    e: e.to_string(),
                }
            })?;

            let outputs = session
                .run(inputs![input_node_name => input_tensor])
                .map_err(|e| VisionError::InferenceErr {
                    method: "inference_base".to_string(),
                    e: e.to_string(),
                })?;

            let view = outputs[output_node_name]
                .try_extract_array::<f32>()
                .map_err(|e| VisionError::DataProcessingErr {
                    method: "inference_with_output_view".to_string(),
                    e: e.to_string(),
                })?;
            Log::debug(&format!("模型输出维度: {}", view.ndim()));
            process_output(view)
        })
    }

    /// 执行一次通用推理，并返回拥有所有权的输出张量。
    ///
    /// 识别模型和其他仍然需要持有输出数据的链路继续走这个入口。
    pub(crate) fn inference_base(
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

pub(crate) fn resolve_model_config_path(config: &ModelConfig) -> VisionResult<PathBuf> {
    BaseModel::resolve_model_config_path(config)
}

impl From<ModelConfig> for BaseModel {
    fn from(config: ModelConfig) -> Self {
        Self::new(
            config.input_width,
            config.input_height,
            config.model_source,
            config.model_path,
            config.execution_provider,
            config.intra_thread_num,
            config.intra_spinning,
            config.inter_thread_num,
            config.inter_spinning,
            config.model_type,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_model(
        model_type: ModelType,
        execution_provider: InferenceBackend,
        intra: usize,
    ) -> BaseModel {
        BaseModel::new(
            320,
            48,
            ModelSource::Custom,
            PathBuf::new(),
            execution_provider,
            intra,
            false,
            1,
            false,
            model_type,
        )
    }

    #[test]
    fn enables_parallel_session_pool_only_for_even_cpu_crnn() {
        let cpu_even = build_model(ModelType::PaddleCrnn5, InferenceBackend::CPU, 6);
        assert!(cpu_even.use_parallel_cpu_sessions(2));
        assert_eq!(cpu_even.parallel_session_count(2), 3);

        let cpu_odd = build_model(ModelType::PaddleCrnn5, InferenceBackend::CPU, 5);
        assert!(!cpu_odd.use_parallel_cpu_sessions(2));

        let cpu_custom = build_model(ModelType::PaddleCrnn5, InferenceBackend::CPU, 6);
        assert!(cpu_custom.use_parallel_cpu_sessions(3));
        assert_eq!(cpu_custom.parallel_session_count(3), 2);

        let dml_even = build_model(ModelType::PaddleCrnn5, InferenceBackend::DirectML, 6);
        assert!(!dml_even.use_parallel_cpu_sessions(2));

        let det_even = build_model(ModelType::PaddleDet5, InferenceBackend::CPU, 6);
        assert!(!det_even.use_parallel_cpu_sessions(2));
    }
}
