use crate::api::backend_dto::BackendApiRes;
use crate::api::infrastructure::process_api::bundle_loader::LoadedScriptBundle;
use crate::api::infrastructure::profile_cache::{
    load_cached_user_profile, should_use_cached_profile,
};
use crate::domain::scripts::script_info::ScriptType;
use crate::infrastructure::http_client::HttpClient;

fn has_active_sponsor(sponsor_until: Option<&str>) -> bool {
    sponsor_until
        .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
        .map(|value| value.with_timezone(&chrono::Utc) > chrono::Utc::now())
        .unwrap_or(false)
}

pub(super) async fn validate_published_script_runtime_access(
    app_handle: &tauri::AppHandle,
    bundles: &[LoadedScriptBundle],
) -> Result<(), String> {
    if !bundles
        .iter()
        .any(|bundle| matches!(bundle.script_type, ScriptType::Published))
    {
        return Ok(());
    }

    let client = HttpClient::new(app_handle.clone());
    let session = client
        .get_auth_session()
        .ok_or_else(|| "请先登录后再运行云端下载脚本".to_string())?;

    if session.access_token.trim().is_empty() {
        return Err("请先登录后再运行云端下载脚本".to_string());
    }

    let payload = match client.get::<BackendApiRes<serde_json::Value>>("/user/profile").await {
        Ok(profile) if profile.code == 200 => profile
            .data
            .ok_or_else(|| "用户资料为空，无法校验云端脚本运行权限".to_string())?,
        Ok(profile) => {
            if should_use_cached_profile(profile.code, &profile.message) {
                load_cached_user_profile(app_handle, &session.username).ok_or_else(|| {
                    let message = profile.message.trim();
                    if message.is_empty() {
                        "校验云端脚本运行权限失败".to_string()
                    } else {
                        message.to_string()
                    }
                })?
            } else {
                let message = profile.message.trim();
                return Err(if message.is_empty() {
                    "校验云端脚本运行权限失败".to_string()
                } else {
                    message.to_string()
                });
            }
        }
        Err(error) => load_cached_user_profile(app_handle, &session.username)
            .ok_or_else(|| format!("校验云端脚本运行权限失败: {}", error))?,
    };
    let auth_stage = payload
        .get("authStage")
        .and_then(|value| value.as_i64())
        .unwrap_or(1);

    if auth_stage <= 1 {
        return Ok(());
    }

    let is_developer = payload
        .get("isDeveloper")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let is_sponsor = has_active_sponsor(payload.get("sponsorUntil").and_then(|value| value.as_str()));

    if auth_stage == 2 && (is_developer || is_sponsor) {
        return Ok(());
    }

    if auth_stage >= 3 && is_sponsor {
        return Ok(());
    }

    Err(if auth_stage == 2 {
        "当前阶段仅赞助用户或开发者可运行云端下载脚本".to_string()
    } else {
        "当前阶段仅赞助用户可运行云端下载脚本".to_string()
    })
}
