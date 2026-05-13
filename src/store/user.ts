import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { requestAppConfirm } from '@/services/appDialogService';
import { invoke } from '@/utils/api';
import { sha256Hex } from '@/utils/passwordHash';
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
}

interface ResetPasswordPayload {
    email: string;
    code: string;
    newPassword: string;
}

const isAuthFailure = (message: string | undefined) =>
    Boolean(message && (message.includes('401') || message.includes('未登录') || message.includes('认证失败')));

const emailPattern = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const usernamePattern = /^[A-Za-z0-9_]{3,16}$/;
const passwordPattern = /^(?=.*[A-Za-z])(?=.*\d).{8,}$/;

const normalizeAuthStage = (value: unknown): 1 | 2 | 3 => {
    const stage = typeof value === 'number' ? value : typeof value === 'string' ? Number(value) : NaN;
    return stage === 2 || stage === 3 ? stage : 1;
};

const hasActiveSponsor = (profile: Pick<UserProfile, 'sponsorUntil'> | null | undefined) => {
    const sponsorUntil = profile?.sponsorUntil;
    if (!sponsorUntil) {
        return false;
    }

    const expiresAt = Date.parse(sponsorUntil);
    return Number.isFinite(expiresAt) && expiresAt > Date.now();
};

const assertEmail = (email: string) => {
    if (!emailPattern.test(email.trim())) {
        throw new Error('请输入有效邮箱地址');
    }
};

const assertPassword = (password: string) => {
    if (!passwordPattern.test(password)) {
        throw new Error('密码至少 8 位，且需同时包含字母和数字');
    }
};

const assertUsername = (username: string) => {
    const value = username.trim();
    if (emailPattern.test(value)) {
        throw new Error('用户名不能使用邮箱格式');
    }
    if (!usernamePattern.test(value)) {
        throw new Error('用户名需为 3-16 位字母、数字或下划线');
    }
};

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
        authStage: normalizeAuthStage(record.authStage),
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
    const authHydrated = ref(false);
    const profileLoading = ref(false);
    const authSubmitting = ref(false);
    const isLoggedIn = computed(() => Boolean(authSession.value?.accessToken));
    const authStage = computed(() => normalizeAuthStage(userProfile.value?.authStage));
    const isDeveloper = computed(() => userProfile.value?.isDeveloper ?? false);
    const isSponsor = computed(() => hasActiveSponsor(userProfile.value));
    const canUsePublishedCloudScripts = computed(() => {
        if (!isLoggedIn.value) {
            return false;
        }

        if (authStage.value === 1) {
            return true;
        }

        if (authStage.value === 2) {
            return isDeveloper.value || isSponsor.value;
        }

        return isSponsor.value;
    });

    const getPublishedCloudScriptAccessMessage = (
        action: '下载' | '运行',
        profileOverride: UserProfile | null = userProfile.value,
    ) => {
        if (!isLoggedIn.value) {
            return `请先登录后再${action}云端脚本`;
        }

        const stage = normalizeAuthStage(profileOverride?.authStage);
        const developer = Boolean(profileOverride?.isDeveloper);
        const sponsor = hasActiveSponsor(profileOverride);

        if (stage === 1) {
            return null;
        }

        if (stage === 2) {
            return developer || sponsor ? null : `当前阶段仅赞助用户或开发者可${action}云端脚本`;
        }

        return sponsor ? null : `当前阶段仅赞助用户可${action}云端脚本`;
    };

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
        applyAuthSession(res.success ? normalizeAuthSession(res.data) : null);
        authHydrated.value = true;
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
                    userProfile.value = null;
                    return null;
                }

                return userProfile.value;
            }

            const profile = normalizeProfile(res.data);
            if (profile) {
                userProfile.value = profile;
            }
            return userProfile.value;
        } catch (error) {
            const message = error instanceof Error ? error.message : '';
            if (isAuthFailure(message)) {
                applyAuthSession(null);
                userProfile.value = null;
                return null;
            }
            return userProfile.value;
        } finally {
            profileLoading.value = false;
        }
    };

    const ensureProfileForAction = async (actionLabel: string) => {
        if (!authSession.value) {
            return null;
        }

        const profile = userProfile.value ?? (await checkProfile());
        if (profile || !authSession.value) {
            return profile;
        }

        const retry = await requestAppConfirm({
            title: '账户信息同步失败',
            message: `当前无法获取账户信息，${actionLabel}前无法继续。你可以重试，或重新登录后再继续。`,
            confirmText: '重试',
            cancelText: '重新登录',
            tone: 'warning',
        });

        if (retry) {
            return checkProfile();
        }

        await logout({ silent: true });
        openAuthModal();
        showToast('请重新登录后继续', 'warning');
        return null;
    };

    const login = async (payload: LoginPayload) => {
        authSubmitting.value = true;
        try {
            assertPassword(payload.password);
            const res = (await invoke('backend_login', {
                req: {
                    ...payload,
                    password: await sha256Hex(payload.password),
                },
            })) as ApiEnvelope<unknown>;
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
            assertUsername(payload.username);
            assertEmail(payload.email);
            assertPassword(payload.password);
            const res = (await invoke('backend_register', {
                req: {
                    ...payload,
                    password: await sha256Hex(payload.password),
                },
            })) as ApiEnvelope<unknown>;
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
        assertEmail(email);
        const res = (await invoke('backend_send_verification_code', { email })) as ApiEnvelope<unknown>;
        if (!res.success) {
            throw new Error(res.message || '验证码发送失败');
        }
        showToast(res.message || '验证码已发送', 'success');
    };

    const resetPassword = async (payload: ResetPasswordPayload) => {
        authSubmitting.value = true;
        try {
            assertEmail(payload.email);
            assertPassword(payload.newPassword);
            const res = (await invoke('backend_reset_password', {
                req: {
                    ...payload,
                    newPassword: await sha256Hex(payload.newPassword),
                },
            })) as ApiEnvelope<unknown>;
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

    const logout = async (options?: { silent?: boolean }) => {
        try {
            await invoke('backend_logout');
            applyAuthSession(null);
            if (!options?.silent) {
                showToast('已退出登录', 'success');
            }
        } catch (error) {
            if (!options?.silent) {
                showToast(error instanceof Error ? error.message : '登出失败', 'error');
            }
        }
    };

    return {
        authSession,
        authHydrated,
        authStage,
        authSubmitting,
        canUsePublishedCloudScripts,
        checkProfile,
        closeAuthModal,
        ensureProfileForAction,
        getPublishedCloudScriptAccessMessage,
        hydrateAuthSession,
        isAuthModalOpen,
        isDeveloper,
        isLoggedIn,
        isSponsor,
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
