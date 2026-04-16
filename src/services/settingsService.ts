import { invoke } from '@/utils/api';
import type { LogLevel } from '@/types/bindings/LogLevel';
import type { EmailConfig, LogConfig, SystemConfigPayload, UpdateInfo, VisionTextCacheConfig } from '@/types/app/domain';

interface ApiEnvelope<T> {
    success: boolean;
    data?: T;
    message?: string;
}

export const settingsService = {
    getLogConfig: () => invoke('get_log_config_cmd') as Promise<LogConfig>,
    updateLogLevel: (logLevel: LogLevel) => invoke('update_log_level_cmd', { logLevel }) as Promise<string>,
    updateLogDir: (logDir: string) => invoke('update_log_dir_cmd', { logDir }) as Promise<string>,
    updateRetentionDays: (days: number) => invoke('update_retention_days_cmd', { days }) as Promise<string>,
    cleanLogs: () => invoke('clean_logs_now_cmd') as Promise<string>,
    getEmailConfig: () => invoke('get_email_config_cmd') as Promise<EmailConfig>,
    updateEmailConfig: (config: EmailConfig) => invoke('set_email_config_cmd', { config }) as Promise<string>,
    sendTestEmail: (config: EmailConfig) => invoke('send_test_email_cmd', { config }) as Promise<string>,
    getVisionTextCacheConfig: () => invoke('get_vision_text_cache_config_cmd') as Promise<VisionTextCacheConfig>,
    updateVisionTextCacheConfig: (config: VisionTextCacheConfig) =>
        invoke('set_vision_text_cache_config_cmd', { config }) as Promise<string>,
    applySystemConfig: (systemConfig: SystemConfigPayload) =>
        invoke('set_system_settings_cmd', { systemConfig }) as Promise<string>,
    checkUpdate: async (): Promise<UpdateInfo | null> => {
        const response = (await invoke('backend_check_update')) as ApiEnvelope<UpdateInfo>;
        return response.success ? response.data ?? null : null;
    },
};
