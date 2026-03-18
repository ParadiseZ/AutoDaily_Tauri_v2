<script setup lang="ts">
import { ref } from 'vue';
import { useUserStore } from '@/store/user';
import { invoke } from '@/utils/api';
import { showToast } from '@/utils/toast';

const userStore = useUserStore();
const activeTab = ref<'login' | 'register' | 'reset'>('login');

const formReset = ref({
    email: '',
    code: '',
    password: ''
});

const formLogin = ref({
    username: '',
    password: ''
});

const formRegister = ref({
    username: '',
    password: '',
    email: '',
    displayName: '',
    code: ''
});

const isSubmitting = ref(false);

const handleLogin = async () => {
    if (!formLogin.value.username || !formLogin.value.password) {
        showToast('请输入用户名和密码', 'warning');
        return;
    }
    isSubmitting.value = true;
    try {
        const res = await invoke('backend_login', { req: formLogin.value });
        if (res && res.success) {
            showToast('登录成功', 'success');
            userStore.closeAuthModal();
            // Refresh user profile after login
            await userStore.checkProfile();
        } else {
            showToast(res?.message || '登录失败，请检查账号密码', 'error');
        }
    } catch (e: any) {
        showToast(e.message || '网络异常，登录失败', 'error');
    } finally {
        isSubmitting.value = false;
    }
};

const handleRegister = async () => {
    isSubmitting.value = true;
    try {
        const res = await invoke('backend_register', { req: formRegister.value });
        if (res && res.success) {
            showToast('注册成功，请重新登录', 'success');
            activeTab.value = 'login'; // switch tab
            formLogin.value.username = formRegister.value.username;
        } else {
            showToast(res?.message || '注册失败', 'error');
        }
    } catch (e: any) {
        showToast(e.message || '注册异常出错', 'error');
    } finally {
        isSubmitting.value = false;
    }
};

const sendVerificationCode = async () => {
    if (!formRegister.value.email) {
        showToast('请先输入邮箱', 'warning');
        return;
    }
    try {
        const res = await invoke('backend_send_verification_code', { email: formRegister.value.email });
        if (res && res.success) {
            showToast('验证码发送成功', 'success');
        } else {
            showToast(res?.message || '发送失败', 'error');
        }
    } catch (e: any) {
        showToast(e.message || '网络异常', 'error');
    }
};

const handleReset = async () => {
    if (!formReset.value.email || !formReset.value.code || !formReset.value.password) {
        showToast('请完整填写邮箱、验证码和新密码', 'warning');
        return;
    }
    isSubmitting.value = true;
    try {
        const res = await invoke('backend_reset_password', { req: formReset.value });
        if (res && res.success) {
            showToast('重置密码成功，请使用新密码登录', 'success');
            activeTab.value = 'login';
            formLogin.value.username = formReset.value.email;
        } else {
            showToast(res?.message || '重置失败', 'error');
        }
    } catch (e: any) {
        showToast(e.message || '重置密码异常出错', 'error');
    } finally {
        isSubmitting.value = false;
    }
};

const sendResetVerificationCode = async () => {
    if (!formReset.value.email) {
        showToast('请先输入邮箱', 'warning');
        return;
    }
    try {
        const res = await invoke('backend_send_verification_code', { email: formReset.value.email });
        if (res && res.success) {
            showToast('验证码发送成功', 'success');
        } else {
            showToast(res?.message || '发送失败', 'error');
        }
    } catch (e: any) {
        showToast(e.message || '网络异常', 'error');
    }
};
</script>

<template>
    <!-- Modal automatically opens when isAuthModalOpen is true -->
    <dialog class="modal modal-bottom sm:modal-middle" :class="{'modal-open': userStore.isAuthModalOpen}">
        <div class="modal-box p-0 overflow-hidden" v-if="userStore.isAuthModalOpen">
            <!-- Tabs Header -->
            <div class="flex w-full">
                <button 
                  class="flex-1 py-4 text-base font-semibold border-b-2 transition"
                  :class="activeTab === 'login' ? 'border-primary text-primary bg-base-200' : 'border-transparent text-base-content/60 hover:bg-base-200'"
                  @click="activeTab = 'login'"
                >
                    登录
                </button>
                <button 
                  class="flex-1 py-4 text-base font-semibold border-b-2 transition"
                  :class="activeTab === 'register' ? 'border-primary text-primary bg-base-200' : 'border-transparent text-base-content/60 hover:bg-base-200'"
                  @click="activeTab = 'register'"
                >
                    注册
                </button>
            </div>

            <div class="p-6 pb-8">
                <!-- Login Form -->
                <div v-show="activeTab === 'login'" class="flex flex-col gap-4">
                    <div class="form-control">
                        <label class="label"><span class="label-text font-medium">账户名</span></label>
                        <input type="text" placeholder="username" class="input input-bordered" v-model="formLogin.username" />
                    </div>
                    <div class="form-control">
                        <label class="label"><span class="label-text font-medium">密码</span></label>
                        <input type="password" placeholder="••••••••" class="input input-bordered" v-model="formLogin.password" @keyup.enter="handleLogin" />
                        <label class="label">
                            <a href="#" class="label-text-alt link link-hover" @click.prevent="activeTab = 'reset'">忘记密码？</a>
                        </label>
                    </div>
                    
                    <button class="btn btn-primary mt-4" :disabled="isSubmitting" @click="handleLogin">
                        <span v-if="isSubmitting" class="loading loading-spinner"></span>
                        登录
                    </button>
                </div>

                <!-- Register Form -->
                <div v-show="activeTab === 'register'" class="flex flex-col gap-4">
                    <div class="form-control">
                        <label class="label"><span class="label-text font-medium">账户名</span></label>
                        <input type="text" placeholder="仅限字母与数字" class="input input-bordered" v-model="formRegister.username" />
                    </div>
                    <!-- Display Name -->
                    <div class="form-control">
                        <label class="label"><span class="label-text font-medium">昵称</span></label>
                        <input type="text" placeholder="任意昵称" class="input input-bordered" v-model="formRegister.displayName" />
                    </div>
                    <div class="form-control">
                        <label class="label"><span class="label-text font-medium">密码</span></label>
                        <input type="password" placeholder="••••••••" class="input input-bordered" v-model="formRegister.password" />
                    </div>
                    <div class="form-control">
                        <label class="label"><span class="label-text font-medium">电子邮箱</span></label>
                        <div class="flex gap-2">
                            <input type="email" placeholder="example@email.com" class="input input-bordered flex-1" v-model="formRegister.email" />
                            <button class="btn btn-outline" @click.prevent="sendVerificationCode">发送验证码</button>
                        </div>
                    </div>
                    <div class="form-control">
                        <label class="label"><span class="label-text font-medium">邮箱验证码</span></label>
                        <input type="text" placeholder="6 digits" class="input input-bordered w-full max-w-xs" v-model="formRegister.code" />
                    </div>
                    
                    <button class="btn btn-primary mt-4" :disabled="isSubmitting" @click="handleRegister">
                        <span v-if="isSubmitting" class="loading loading-spinner"></span>
                        立即注册
                    </button>
                </div>
                
                <!-- Reset Password Form -->
                <div v-show="activeTab === 'reset'" class="flex flex-col gap-4">
                    <div class="form-control">
                        <label class="label"><span class="label-text font-medium">电子邮箱</span></label>
                        <div class="flex gap-2">
                            <input type="email" placeholder="注册时使用的邮箱" class="input input-bordered flex-1" v-model="formReset.email" />
                            <button class="btn btn-outline" @click.prevent="sendResetVerificationCode">发送验证码</button>
                        </div>
                    </div>
                    <div class="form-control">
                        <label class="label"><span class="label-text font-medium">邮箱验证码</span></label>
                        <input type="text" placeholder="6 digits" class="input input-bordered w-full max-w-xs" v-model="formReset.code" />
                    </div>
                    <div class="form-control">
                        <label class="label"><span class="label-text font-medium">新密码</span></label>
                        <input type="password" placeholder="••••••••" class="input input-bordered" v-model="formReset.password" />
                    </div>
                    
                    <button class="btn btn-primary mt-4" :disabled="isSubmitting" @click="handleReset">
                        <span v-if="isSubmitting" class="loading loading-spinner"></span>
                        重置密码
                    </button>
                    <div class="text-center mt-2">
                        <a href="#" class="text-sm link link-hover text-base-content/70" @click.prevent="activeTab = 'login'">返回登录</a>
                    </div>
                </div>
            </div>

            <!-- Close Button (X icon outside) -->
            <button class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2" @click="userStore.closeAuthModal">✕</button>
        </div>
        <form method="dialog" class="modal-backdrop" @click.prevent="userStore.closeAuthModal">
            <button>close</button>
        </form>
    </dialog>
</template>
