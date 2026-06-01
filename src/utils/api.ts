import { invoke as tauriInvoke, InvokeArgs, InvokeOptions } from '@tauri-apps/api/core';
import { useUserStore } from '@/store/user';
import { showToast } from './toast';

export interface ApiEnvelope<T> {
    success: boolean;
    data?: T;
    message?: string;
    details?: unknown;
}

const authCommands = new Set([
    'backend_login',
    'backend_register',
    'backend_send_verification_code',
    'backend_reset_password',
]);

const backendCommandFallbackMessages: Record<string, string> = {
    backend_login: '登录失败，请检查账号信息或稍后重试。',
    backend_register: '注册失败，请稍后重试。',
    backend_send_verification_code: '验证码发送失败，请稍后重试。',
    backend_reset_password: '重置密码失败，请稍后重试。',
    backend_update_username: '用户名更新失败，请稍后重试。',
    backend_get_profile: '获取账户信息失败，请稍后重试。',
    backend_get_cached_profile: '读取账户缓存失败，请稍后重试。',
    backend_get_auth_session: '读取登录状态失败，请稍后重试。',
    backend_logout: '退出登录失败，请稍后重试。',
    backend_search_scripts: '搜索脚本市场失败，请检查网络后重试。',
    backend_get_script_change_logs: '加载脚本更新日志失败，请稍后重试。',
    backend_get_script_cloud_summary: '获取云端脚本信息失败，请稍后重试。',
    backend_preflight_download_script: '获取下载版本信息失败，请稍后重试。',
    backend_preflight_upload_script: '获取上传版本信息失败，请稍后重试。',
    backend_download_script: '下载脚本失败，请检查网络后重试。',
    backend_upload_script: '上传脚本失败，请检查网络后重试。',
    backend_download_model: '下载模型文件失败，请检查网络后重试。',
    backend_upload_model: '上传模型文件失败，请检查网络后重试。',
};

const technicalErrorPattern =
    /(exception|traceback|panic|stack|reqwest|sqlx|serde|json|uuid|parse|syntaxerror|typeerror|referenceerror|timed?\s*out|timeout|connection|network|fetch|refused|reset|status\s*\d{3}|http\s*\d{3}|unexpected|os error|io error|econn|enotfound|error sending request for url|https?:\/\/|localhost:|\/api\/|<html|<\/html>)/i;

export const isAuthFailure = (message: string | undefined | null) =>
    Boolean(
        message &&
            (
                message.includes('401') ||
                message.includes('未登录') ||
                message.includes('认证失败') ||
                message.includes('登录状态已失效') ||
                message.includes('登录已过期') ||
                message.includes('Unauthorized')
            ),
    );

const isBackendCommand = (cmd: string) => cmd.startsWith('backend_');

export const toErrorText = (error: unknown) => {
    if (typeof error === 'string') {
        return error;
    }

    if (error instanceof Error) {
        return [error.name, error.message, error.stack].filter(Boolean).join('\n');
    }

    try {
        return JSON.stringify(error);
    } catch {
        return String(error);
    }
};

const formatValidationDetails = (details: unknown) => {
    if (!details || typeof details !== 'object' || !('issues' in details)) {
        return null;
    }

    const issues = (details as { issues?: Array<{ path?: string; message?: string }> }).issues;
    if (!Array.isArray(issues) || issues.length === 0) {
        return null;
    }

    const lines = issues
        .map((issue) => {
            const message = issue?.message?.trim();
            if (!message) {
                return null;
            }

            const path = issue.path?.trim();
            return path ? `- ${path}: ${message}` : `- ${message}`;
        })
        .filter((line): line is string => Boolean(line));

    return lines.length ? lines.join('\n') : null;
};

const appendValidationDetails = (message: string, details?: unknown) => {
    const detailText = formatValidationDetails(details);
    if (!detailText || message.includes(detailText)) {
        return message;
    }
    return `${message}\n${detailText}`;
};

const isUserFriendlyServerMessage = (message: string) => {
    const trimmed = message.trim();
    if (!trimmed) {
        return false;
    }

    if (technicalErrorPattern.test(trimmed)) {
        return false;
    }

    return !(trimmed.length > 240 && !trimmed.includes('\n- '));
};

export const logServerAccessError = (label: string, rawError: unknown, payload?: unknown) => {
    console.error(`[server-access:${label}]`, rawError, payload ?? '');
};

export const getServerFriendlyErrorMessage = (
    label: string,
    rawMessage?: string | null,
    fallback?: string,
    details?: unknown,
) => {
    const trimmed = rawMessage?.trim() ?? '';
    if (isAuthFailure(trimmed) && !authCommands.has(label)) {
        return '登录状态已失效，请重新登录后再试。';
    }

    if (isUserFriendlyServerMessage(trimmed)) {
        return appendValidationDetails(trimmed, details);
    }

    return fallback ?? backendCommandFallbackMessages[label] ?? '服务器暂时不可用，请稍后重试。';
};

export const createServerRequestError = (label: string, rawError: unknown, fallback?: string) => {
    logServerAccessError(label, rawError);
    const rawMessage = typeof rawError === 'string' ? rawError : rawError instanceof Error ? rawError.message : toErrorText(rawError);
    return new Error(getServerFriendlyErrorMessage(label, rawMessage, fallback));
};

export const createServerResponseError = (
    label: string,
    response: Pick<ApiEnvelope<unknown>, 'message' | 'details'>,
    fallback?: string,
) => {
    logServerAccessError(label, response.message ?? '[empty message]', response);
    return new Error(getServerFriendlyErrorMessage(label, response.message, fallback, response.details));
};

/**
 * A wrapper around Tauri's `invoke` that intercepts HTTP 401 responses
 * and triggers the global login modal.
 */
export async function invoke(cmd: string, args?: InvokeArgs, options?: InvokeOptions): Promise<any> {
    try {
        const response: any = await tauriInvoke(cmd, args, options);
        // The backend might return success: false when there's an API error
        if (response && response.success === false) {
            // Check if it's an authorization error (e.g. 401 string buried in message)
            if (!authCommands.has(cmd)) {
                if (isAuthFailure(response.message)) {
                    const userStore = useUserStore();
                    userStore.openAuthModal();
                    showToast('登录已过期，请重新登录', 'warning');
                    throw new Error('Unauthorized');
                }
            }
        }
        return response;
    } catch (error: any) {
        // If Rust panicked, we explicitly threw HttpErr that gets serialized as string '401 Unauthorized'
        const errMsg = typeof error === 'string' ? error : (error?.message || '');
        if (!authCommands.has(cmd)) {
            if (isAuthFailure(errMsg)) {
                const userStore = useUserStore();
                userStore.openAuthModal();
                showToast('未登录或登录态失效，请登录', 'warning');
                console.warn(`[server-access:${cmd}] auth expired`, error);
                throw new Error('Unauthorized');
            }
        }

        if (isBackendCommand(cmd)) {
            throw createServerRequestError(cmd, error);
        }

        throw error;
    }
}
