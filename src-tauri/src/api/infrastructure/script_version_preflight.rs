use crate::domain::scripts::script_info::{ScriptTable, ScriptType};
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
) -> Result<Option<ScriptTable>, String> {
    let pool = crate::infrastructure::db::get_pool();
    let mut scripts: Vec<ScriptTable> = sqlx::query_as("SELECT id, `data` FROM scripts")
        .fetch_all(pool)
        .await
        .map_err(|error| format!("读取本地脚本失败: {}", error))?;

    scripts.retain(|script| {
        script.data.script_type == ScriptType::Published
            && script
                .data
                .cloud_id
                .as_ref()
                .map(|value| value.to_string())
                .as_deref()
                == Some(cloud_script_id)
    });

    scripts.sort_by(|left, right| {
        let right_time = right
            .data
            .update_time
            .as_ref()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.timestamp_millis())
            .unwrap_or(0);
        let left_time = left
            .data
            .update_time
            .as_ref()
            .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.timestamp_millis())
            .unwrap_or(0);
        right_time
            .cmp(&left_time)
            .then_with(|| right.data.ver_num.cmp(&left.data.ver_num))
    });

    Ok(scripts.into_iter().next())
}

pub fn build_download_preflight(
    existing_local_script: Option<&ScriptTable>,
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

    let local_ver_num = Some(local_script.data.ver_num);
    let local_version_label =
        format_version_label(Some(local_script.data.ver_name.as_str()), local_ver_num);

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
