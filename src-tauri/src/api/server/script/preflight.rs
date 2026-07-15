use domain_script::ScriptProfile;
use domain_script::ScriptType;
use infra_sqlite::list_scripts;
use num::ToPrimitive;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptVersionPreflight {
    pub status: String,
    pub message: String,
    pub local_script_id: Option<String>,
    pub local_version_label: Option<String>,
    pub remote_version_label: Option<String>,
    pub local_ver_num: Option<u32>,
    pub remote_ver_num: Option<u32>,
}

pub fn format_version_label(ver_name: Option<&str>, ver_num: Option<u32>) -> String {
    if let Some(name) = ver_name.map(str::trim).filter(|value| !value.is_empty()) {
        return format!("v{}", name);
    }
    if let Some(number) = ver_num {
        return format!("版本 {}", number);
    }
    "未标记版本".to_string()
}

pub async fn find_replaceable_local_published_script(
    cloud_script_id: &str,
) -> Result<Option<ScriptProfile>, String> {
    let mut scripts = list_scripts()
        .await
        .map_err(|error| format!("读取本地脚本失败: {}", error))?;

    scripts.retain(|script| {
        script.info.script_type == ScriptType::Published
            && script
                .info
                .cloud_id
                .as_ref()
                .map(|value| value.to_string())
                .as_deref()
                == Some(cloud_script_id)
    });

    scripts.sort_by(|left, right| {
        let right_time = right
            .info
            .update_time
            .as_ref()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.timestamp_millis())
            .unwrap_or(0);
        let left_time = left
            .info
            .update_time
            .as_ref()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.timestamp_millis())
            .unwrap_or(0);
        right_time
            .cmp(&left_time)
            .then_with(|| right.info.ver_num.cmp(&left.info.ver_num))
    });

    Ok(scripts.into_iter().next())
}

pub fn build_download_preflight(
    existing_local_script: Option<&ScriptProfile>,
    remote_ver_name: Option<&str>,
    remote_ver_num: Option<u32>,
) -> ScriptVersionPreflight {
    let remote_version_label = format_version_label(remote_ver_name, remote_ver_num);
    let Some(local_script) = existing_local_script else {
        return ScriptVersionPreflight {
            status: "noLocalCopy".to_string(),
            message: format!(
                "本地还没有该云端脚本副本，将直接下载 {} 到本地库。",
                remote_version_label
            ),
            local_script_id: None,
            local_version_label: None,
            remote_version_label: Some(remote_version_label),
            local_ver_num: None,
            remote_ver_num,
        };
    };

    let local_ver_num = Some(local_script.info.ver_num);
    let local_version_label =
        format_version_label(Some(local_script.info.ver_name.as_str()), local_ver_num);

    if let (Some(local_ver_num), Some(remote_ver_num)) = (local_ver_num, remote_ver_num) {
        if remote_ver_num > local_ver_num {
            return ScriptVersionPreflight {
                status: "upgradeAvailable".to_string(),
                message: format!(
                    "本地已有 {}，云端当前为 {}。继续后会更新本地云端副本，并保留原有脚本关联。",
                    local_version_label, remote_version_label
                ),
                local_script_id: Some(local_script.id.to_string()),
                local_version_label: Some(local_version_label),
                remote_version_label: Some(remote_version_label),
                local_ver_num: Some(local_ver_num),
                remote_ver_num: Some(remote_ver_num),
            };
        }

        if remote_ver_num < local_ver_num {
            return ScriptVersionPreflight {
                status: "downgradeBlocked".to_string(),
                message: format!(
                    "本地已有 {}，云端当前仅为 {}。不允许用较旧的云端版本覆盖本地副本。",
                    local_version_label, remote_version_label
                ),
                local_script_id: Some(local_script.id.to_string()),
                local_version_label: Some(local_version_label),
                remote_version_label: Some(remote_version_label),
                local_ver_num: Some(local_ver_num),
                remote_ver_num: Some(remote_ver_num),
            };
        }
    }

    ScriptVersionPreflight {
        status: "sameVersion".to_string(),
        message: format!(
            "本地已有 {}，继续后会用云端 {} 覆盖当前本地云端副本，并保留原有脚本关联。",
            local_version_label, remote_version_label
        ),
        local_script_id: Some(local_script.id.to_string()),
        local_version_label: Some(local_version_label),
        remote_version_label: Some(remote_version_label),
        local_ver_num,
        remote_ver_num,
    }
}

pub fn extract_cloud_summary_version(summary: &serde_json::Value) -> (Option<String>, Option<u32>) {
    let ver_name = summary
        .get("verName")
        .and_then(|value| value.as_str())
        .map(str::to_string);
    let ver_num = summary
        .get("verNum")
        .and_then(|value| value.as_u64().and_then(|num| num.to_u32()));
    (ver_name, ver_num)
}

use crate::api::server::dto::current_client_capability;
use domain_script::ScriptInfo;
pub(crate) fn validate_script_compatibility(script: &ScriptInfo) -> Option<String> {
    let client = current_client_capability();

    if let Some(required_schema) = script.min_runtime_schema {
        if required_schema > client.runtime_schema {
            return Some(format!(
                "该脚本需要运行时结构版本 {}，当前程序仅支持 {}，请先更新程序",
                required_schema, client.runtime_schema
            ));
        }
    }

    if let Some(required_version) = script.min_app_version.as_deref() {
        if is_version_greater(required_version, &client.app_version) {
            return Some(format!(
                "该脚本需要 AutoDaily {} 或以上版本，当前版本为 {}，请先更新程序",
                required_version, client.app_version
            ));
        }
    }

    let missing_features: Vec<&str> = script
        .required_features
        .iter()
        .map(String::as_str)
        .filter(|feature| {
            !client
                .supported_features
                .iter()
                .any(|supported| supported == feature)
        })
        .collect();

    if !missing_features.is_empty() {
        return Some(format!(
            "该脚本需要当前程序不支持的能力: {}，请先更新程序",
            missing_features
                .iter()
                .map(|feature| script_feature_label(feature))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    None
}

fn script_feature_label(feature: &str) -> &str {
    match feature {
        "onnxInference" => "ONNX推理",
        "runtime:javascript" => "JavaScript运行时",
        "runtime:rhai" => "Rhai运行时",
        "runtime:lua" => "Lua运行时",
        "llmApi" => "LLM API接口",
        "device:android" => "Android目标设备",
        "device:desktop" => "Desktop目标设备",
        _ => feature,
    }
}

fn is_version_greater(required: &str, current: &str) -> bool {
    let parse = |version: &str| {
        version
            .split(|ch: char| !ch.is_ascii_digit())
            .filter(|part| !part.is_empty())
            .take(3)
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };

    let required_parts = parse(required);
    let current_parts = parse(current);

    for index in 0..3 {
        let required_part = *required_parts.get(index).unwrap_or(&0);
        let current_part = *current_parts.get(index).unwrap_or(&0);
        if required_part != current_part {
            return required_part > current_part;
        }
    }

    false
}
