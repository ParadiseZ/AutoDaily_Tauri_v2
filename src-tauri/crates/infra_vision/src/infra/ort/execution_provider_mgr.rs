use crate::infra::ort::ort_error::OrtError;
use domain_vision::InferenceBackend;
use infra_logging::Log;
use ort::execution_providers::ExecutionProviderDispatch;
use ort::session::Session;
use ort::session::builder::SessionBuilder;

pub(crate) fn backend_name(backend: &InferenceBackend) -> &'static str {
    backend.as_str()
}

fn build_ep(backend: &InferenceBackend) -> ExecutionProviderDispatch {
    match backend {
        InferenceBackend::Cuda => {
            ort::execution_providers::CUDAExecutionProvider::default().build()
        }
        InferenceBackend::DirectML => {
            ort::execution_providers::DirectMLExecutionProvider::default().build()
        }
        InferenceBackend::CPU => ort::execution_providers::CPUExecutionProvider::default().build(),
    }
}

// 配置结果
pub(crate) struct ProviderConfigResult {
    pub builder: SessionBuilder,
    pub active_backend: InferenceBackend,
}

// 主函数：配置或切换执行器
pub(crate) fn configure_or_switch_provider(
    current_builder: Option<SessionBuilder>,
    target: &InferenceBackend,
) -> Result<ProviderConfigResult, OrtError> {
    // 获取基础 builder（首次运行 or 复用）
    let base_builder = match current_builder {
        Some(b) => b,
        None => Session::builder().map_err(|e| OrtError::LoadModelErr {
            method: "configure_or_switch_provider".to_string(),
            e: e.to_string(),
        })?,
    };

    // 尝试按优先级启用执行器
    try_enable_provider(base_builder, target)
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
        let ep = build_ep(&backend);
        match builder.with_execution_providers([ep]) {
            Ok(b) => {
                Log::debug(&format!("模型启用{}执行器成功", backend_name(&backend)));
                return Ok(ProviderConfigResult {
                    builder: b,
                    active_backend: backend,
                });
            }
            Err(e) => {
                Log::warn(&format!(
                    "为模型配置{}失败: {}。",
                    backend_name(&backend),
                    e
                ));
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
