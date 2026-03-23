import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { invoke } from '@/utils/api';
import { showToast } from '@/utils/toast';
import type { UserProfile } from '@/types/app/domain';

interface ApiEnvelope<T> {
    success: boolean;
    data?: T;
    message?: string;
}

interface LoginPayload {
    username: string;
    password: string;
}

interface RegisterPayload extends LoginPayload {
    email: string;
    code: string;
    phone?: string | null;
}

interface ResetPasswordPayload {
    email: string;
    code: string;
    newPassword: string;
}

const normalizeProfile = (payload: unknown): UserProfile | null => {
    if (!payload || typeof payload !== 'object') {
        return null;
    }

    const record = payload as Record<string, unknown>;
    if (typeof record.id !== 'string' || typeof record.username !== 'string' || typeof record.email !== 'string') {
        return null;
    }

    return {
        id: record.id,
        username: record.username,
        email: record.email,
        isDeveloper: Boolean(record.isDeveloper),
        lastScriptUploadTime: typeof record.lastScriptUploadTime === 'string' ? record.lastScriptUploadTime : '',
        lastUsernameChangeTime:
            typeof record.lastUsernameChangeTime === 'string' ? record.lastUsernameChangeTime : '',
        sponsorUntil: typeof record.sponsorUntil === 'string' ? record.sponsorUntil : null,
    };
};

export const useUserStore = defineStore('user', () => {
    const isAuthModalOpen = ref(false);
    const isLoggedIn = ref(false);
    const userProfile = ref<UserProfile | null>(null);
    const profileLoading = ref(false);
    const authSubmitting = ref(false);
    const isDeveloper = computed(() => userProfile.value?.isDeveloper ?? false);

    const openAuthModal = () => {
        isAuthModalOpen.value = true;
    };

    const closeAuthModal = () => {
        isAuthModalOpen.value = false;
    };

    const checkProfile = async () => {
        profileLoading.value = true;
        try {
            const res = (await invoke('backend_get_profile')) as ApiEnvelope<unknown>;
            const profile = res.success ? normalizeProfile(res.data) : null;
            isLoggedIn.value = Boolean(profile);
            userProfile.value = profile;
        } catch {
            isLoggedIn.value = false;
            userProfile.value = null;
        } finally {
            profileLoading.value = false;
        }
    };

    const login = async (payload: LoginPayload) => {
        authSubmitting.value = true;
        try {
            const res = (await invoke('backend_login', { req: payload })) as ApiEnvelope<unknown>;
            if (!res.success) {
                throw new Error(res.message || '登录失败');
            }
            await checkProfile();
            closeAuthModal();
            showToast('登录成功', 'success');
        } catch (error) {
            showToast(error instanceof Error ? error.message : '登录失败', 'error');
            throw error;
        } finally {
            authSubmitting.value = false;
        }
    };

    const register = async (payload: RegisterPayload) => {
        authSubmitting.value = true;
        try {
            const res = (await invoke('backend_register', { req: payload })) as ApiEnvelope<unknown>;
            if (!res.success) {
                throw new Error(res.message || '注册失败');
            }
            showToast(res.message || '注册成功，请登录', 'success');
        } catch (error) {
            showToast(error instanceof Error ? error.message : '注册失败', 'error');
            throw error;
        } finally {
            authSubmitting.value = false;
        }
    };

    const sendVerificationCode = async (email: string) => {
        const res = (await invoke('backend_send_verification_code', { email })) as ApiEnvelope<unknown>;
        if (!res.success) {
            throw new Error(res.message || '验证码发送失败');
        }
        showToast(res.message || '验证码已发送', 'success');
    };

    const resetPassword = async (payload: ResetPasswordPayload) => {
        authSubmitting.value = true;
        try {
            const res = (await invoke('backend_reset_password', { req: payload })) as ApiEnvelope<unknown>;
            if (!res.success) {
                throw new Error(res.message || '重置密码失败');
            }
            showToast(res.message || '密码已重置，请重新登录', 'success');
        } catch (error) {
            showToast(error instanceof Error ? error.message : '重置密码失败', 'error');
            throw error;
        } finally {
            authSubmitting.value = false;
        }
    };

    const updateUsername = async (newUsername: string) => {
        authSubmitting.value = true;
        try {
            const res = (await invoke('backend_update_username', { req: { newUsername } })) as ApiEnvelope<unknown>;
            if (!res.success) {
                throw new Error(res.message || '用户名更新失败');
            }
            await checkProfile();
            showToast('用户名已更新', 'success');
        } catch (error) {
            showToast(error instanceof Error ? error.message : '用户名更新失败', 'error');
            throw error;
        } finally {
            authSubmitting.value = false;
        }
    };

    const logout = async () => {
        try {
            await invoke('backend_logout');
            isLoggedIn.value = false;
            userProfile.value = null;
            showToast('已退出登录', 'success');
        } catch (error) {
            showToast(error instanceof Error ? error.message : '登出失败', 'error');
        }
    };

    return {
        authSubmitting,
        checkProfile,
        closeAuthModal,
        isAuthModalOpen,
        isDeveloper,
        isLoggedIn,
        login,
        logout,
        openAuthModal,
        profileLoading,
        register,
        resetPassword,
        sendVerificationCode,
        updateUsername,
        userProfile,
    };
});
