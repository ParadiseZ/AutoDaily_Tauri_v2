pub(crate) mod child_log;
pub(crate) use ad_kernel::LogLevel;
pub(crate) mod log_trait {
    pub(crate) use infra_logging::{Log, LogTrait};
}
