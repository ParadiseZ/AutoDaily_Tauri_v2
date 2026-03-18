import { invoke as tauriInvoke, InvokeArgs, InvokeOptions } from '@tauri-apps/api/core';
import { useUserStore } from '../store/user';
import { showToast } from './toast';

/**
 * A wrapper around Tauri's `invoke` that intercepts HTTP 401 responses
 * and triggers the global login modal.
 */
export async function invoke<T>(cmd: string, args?: InvokeArgs, options?: InvokeOptions): Promise<any> {
    try {
        const response: any = await tauriInvoke(cmd, args, options);
        // The backend might return success: false when there's an API error
        if (response && response.success === false) {
            // Check if it's an authorization error (e.g. 401 string buried in message)
            if (response.message && (response.message.includes('401') || response.message.includes('未登录') || response.message.includes('认证失败'))) {
                const userStore = useUserStore();
                userStore.openAuthModal();
                showToast('登录已过期，请重新登录', 'warning');
                throw new Error('Unauthorized');
            }
        }
        return response;
    } catch (error: any) {
        // If Rust panicked or we explicitly threw HttpErr that gets serialized as string '401 Unauthorized'
        const errMsg = typeof error === 'string' ? error : (error?.message || '');
        if (errMsg.includes('401') || errMsg.includes('未登录') || errMsg.includes('认证失败')) {
            const userStore = useUserStore();
            userStore.openAuthModal();
            showToast('未登录或登录态失效，请登录', 'warning');
        }
        throw error;
    }
}
