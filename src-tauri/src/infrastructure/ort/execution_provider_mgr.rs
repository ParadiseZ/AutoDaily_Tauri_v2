use crate::infrastructure::logging::log_trait::Log;
use crate::infrastructure::ort::ort_error::OrtError;
use ort::execution_providers::ExecutionProviderDispatch;
use ort::session::builder::SessionBuilder;
use ort::session::Session;
use crate::infrastructure::core::{Deserialize, Serialize};

// 执行器类型枚举（便于扩展）
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InferenceBackend {
    Cuda,
    DirectML,
    CPU,
}

impl std::fmt::Display for InferenceBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl InferenceBackend {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "cuda" => InferenceBackend::Cuda,
            "dml" | "directml" => InferenceBackend::DirectML,
            "cpu" => InferenceBackend::CPU,
            _ => InferenceBackend::CPU,
        }
    }

    pub(crate) fn name(&self) -> &'static str {
        match self {
            InferenceBackend::Cuda => "CUDA",
            InferenceBackend::DirectML => "DirectML",
            InferenceBackend::CPU => "CPU",
        }
    }

    fn build_ep(&self) -> ExecutionProviderDispatch {
        match self {
            InferenceBackend::Cuda => {
                ort::execution_providers::CUDAExecutionProvider::default().build()
            }
            InferenceBackend::DirectML => {
                ort::execution_providers::DirectMLExecutionProvider::default().build()
            }
            InferenceBackend::CPU => {
                ort::execution_providers::CPUExecutionProvider::default().build()
            }
        }
    }
}

// 配置结果
pub struct ProviderConfigResult {
    pub builder: SessionBuilder,
    pub active_backend: InferenceBackend,
}

// 主函数：配置或切换执行器
pub fn configure_or_switch_provider(
    current_builder: Option<SessionBuilder>,
    target: &str,
) -> Result<ProviderConfigResult, OrtError> {
    let target_backend = InferenceBackend::from_str(target); // 未知默认 CPU

    // 获取基础 builder（首次运行 or 复用）
    let base_builder = match current_builder {
        Some(b) => b,
        None => Session::builder().map_err(|e| OrtError::LoadModelErr {
            method: "configure_or_switch_provider".to_string(),
            e: e.to_string(),
        })?,
    };

    // 尝试按优先级启用执行器
    try_enable_provider(base_builder, &target_backend)
}

// 内部：尝试启用指定后端，带 fallback
fn try_enable_provider(
    mut builder: SessionBuilder,
    target: &InferenceBackend,
) -> Result<ProviderConfigResult, OrtError> {
    // 定义 fallback 链（仅 CUDA 有二级 fallback）
    let fallback_chain = if *target == InferenceBackend::Cuda {
        vec![
            InferenceBackend::Cuda,
            InferenceBackend::DirectML,
            InferenceBackend::CPU,
        ]
    } else if *target == InferenceBackend::DirectML {
        vec![InferenceBackend::DirectML, InferenceBackend::CPU]
    } else {
        vec![InferenceBackend::CPU]
    };

    for backend in fallback_chain {
        let ep = backend.build_ep();
        match builder.with_execution_providers([ep]) {
            Ok(b) => {
                Log::debug(&format!("模型启用{}执行器成功", backend.name()));
                return Ok(ProviderConfigResult {
                    builder: b,
                    active_backend: backend,
                });
            }
            Err(e) => {
                Log::warn(&format!("为模型配置{}失败: {}。", backend.name(), e));
                // 为下一次尝试重建 builder（避免状态污染）
                builder = Session::builder().map_err(|e| OrtError::LoadModelErr {
                    method: "try_enable_provider".to_string(),
                    e: e.to_string(),
                })?;
            }
        }
    }

    // 理论上不会走到这里（CPU 应该总能成功）
    Err(OrtError::LoadModelErr {
        method: "try_enable_provider".to_string(),
        e: "所有执行器配置均失败".to_string(),
    })
}
