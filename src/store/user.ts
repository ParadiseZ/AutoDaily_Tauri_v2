import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { invoke } from '@/utils/api';
import { showToast } from '@/utils/toast';
import type { AuthSession, UserProfile } from '@/types/app/domain';

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

const isAuthFailure = (message: string | undefined) =>
    Boolean(message && (message.includes('401') || message.includes('未登录') || message.includes('认证失败')));

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

const normalizeAuthSession = (payload: unknown): AuthSession | null => {
    if (!payload || typeof payload !== 'object') {
        return null;
    }

    const record = payload as Record<string, unknown>;
    if (
        typeof record.accessToken !== 'string' ||
        typeof record.refreshToken !== 'string' ||
        typeof record.username !== 'string'
    ) {
        return null;
    }

    return {
        accessToken: record.accessToken,
        refreshToken: record.refreshToken,
        username: record.username,
        message: typeof record.message === 'string' ? record.message : null,
    };
};

export const useUserStore = defineStore('user', () => {
    const isAuthModalOpen = ref(false);
    const authSession = ref<AuthSession | null>(null);
    const userProfile = ref<UserProfile | null>(null);
    const profileLoading = ref(false);
    const authSubmitting = ref(false);
    const isLoggedIn = computed(() => Boolean(authSession.value?.accessToken));
    const isDeveloper = computed(() => userProfile.value?.isDeveloper ?? false);

    const openAuthModal = () => {
        isAuthModalOpen.value = true;
    };

    const closeAuthModal = () => {
        isAuthModalOpen.value = false;
    };

    const applyAuthSession = (session: AuthSession | null) => {
        authSession.value = session;
        if (!session) {
            userProfile.value = null;
        }
    };

    const hydrateAuthSession = async () => {
        const res = (await invoke('backend_get_auth_session')) as ApiEnvelope<unknown>;
        authSession.value = res.success ? normalizeAuthSession(res.data) : null;
        return authSession.value;
    };

    const checkProfile = async () => {
        if (!authSession.value) {
            userProfile.value = null;
            return null;
        }

        profileLoading.value = true;
        try {
            const res = (await invoke('backend_get_profile')) as ApiEnvelope<unknown>;
            if (!res.success) {
                if (isAuthFailure(res.message)) {
                    applyAuthSession(null);
                }
                userProfile.value = null;
                return null;
            }

            const profile = normalizeProfile(res.data);
            userProfile.value = profile;
            return profile;
        } catch (error) {
            const message = error instanceof Error ? error.message : '';
            if (isAuthFailure(message)) {
                applyAuthSession(null);
            }
            userProfile.value = null;
            return null;
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

            const session = normalizeAuthSession(res.data);
            if (!session) {
                throw new Error('登录返回缺少必要信息');
            }

            applyAuthSession(session);
            closeAuthModal();
            showToast('登录成功', 'success');
            void checkProfile();
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

            const nextSession = normalizeAuthSession(res.data);
            if (nextSession) {
                applyAuthSession(nextSession);
            }

            if (userProfile.value) {
                userProfile.value = {
                    ...userProfile.value,
                    username: newUsername,
                };
            }

            void checkProfile();
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
            applyAuthSession(null);
            showToast('已退出登录', 'success');
        } catch (error) {
            showToast(error instanceof Error ? error.message : '登出失败', 'error');
        }
    };

    return {
        authSession,
        authSubmitting,
        checkProfile,
        closeAuthModal,
        hydrateAuthSession,
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
