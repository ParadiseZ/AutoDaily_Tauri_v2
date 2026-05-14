mod market;
mod model_transfer;

pub use market::{
    backend_download_script, backend_get_script_change_logs, backend_get_script_cloud_summary,
    backend_preflight_download_script, backend_preflight_upload_script, backend_search_scripts,
    backend_upload_script,
};
pub use model_transfer::{backend_download_model, backend_upload_model};

#[cfg(test)]
mod tests {
    use super::model_transfer::{normalize_download_endpoint, runtime_type_param};
    use crate::domain::scripts::script_info::RuntimeType;

    #[test]
    fn runtime_type_query_values_match_server_contract() {
        assert_eq!(runtime_type_param(&RuntimeType::Rhai).unwrap(), "rhai");
        assert_eq!(
            runtime_type_param(&RuntimeType::JavaScript).unwrap(),
            "javaScript"
        );
        assert_eq!(runtime_type_param(&RuntimeType::Lua).unwrap(), "lua");
        assert_eq!(
            runtime_type_param(&RuntimeType::AIAndVision).unwrap(),
            "aIAndVision"
        );
        assert_eq!(runtime_type_param(&RuntimeType::AI).unwrap(), "aI");
    }

    #[test]
    fn download_endpoint_strips_api_prefix() {
        assert_eq!(
            normalize_download_endpoint("/api/scripts/download/model/1/2/x").unwrap(),
            "/scripts/download/model/1/2/x"
        );
        assert_eq!(
            normalize_download_endpoint("/scripts/download/model/1/2/x").unwrap(),
            "/scripts/download/model/1/2/x"
        );
    }
}
