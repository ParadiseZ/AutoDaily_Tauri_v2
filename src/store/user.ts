import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@/utils/api';
import { showToast } from '@/utils/toast';

export const useUserStore = defineStore('user', () => {
    const isAuthModalOpen = ref(false);
    const isLoggedIn = ref(false);
    const userProfile = ref<any>(null);

    const openAuthModal = () => {
        isAuthModalOpen.value = true;
    };

    const closeAuthModal = () => {
        isAuthModalOpen.value = false;
    };

    const checkProfile = async () => {
        try {
            const res: any = await invoke('backend_get_profile');
            if (res.success && res.data) {
                isLoggedIn.value = true;
                userProfile.value = res.data;
            } else {
                isLoggedIn.value = false;
                userProfile.value = null;
            }
        } catch (e) {
            isLoggedIn.value = false;
            userProfile.value = null;
        }
    };

    const logout = async () => {
        try {
            await invoke('backend_logout');
            isLoggedIn.value = false;
            userProfile.value = null;
            showToast('已退出登录', 'success');
        } catch (e) {
            showToast('登出失败', 'error');
        }
    };

    return {
        isAuthModalOpen,
        isLoggedIn,
        userProfile,
        openAuthModal,
        closeAuthModal,
        checkProfile,
        logout,
    };
});
