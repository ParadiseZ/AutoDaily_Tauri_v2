<template>
  <AppDialog
    :open="userStore.isAuthModalOpen"
    title="欢迎回来"
    width-class="max-w-2xl"
    @close="userStore.closeAuthModal()"
  >
    <div class="space-y-5">
      <div class="flex flex-wrap gap-2">
        <button
          v-for="item in tabs"
          :key="item.value"
          type="button"
          class="app-tab"
          :class="{ 'app-tab-active': currentTab === item.value }"
          :disabled="isAuthBusy"
          @click="currentTab = item.value"
        >
          {{ item.label }}
        </button>
      </div>

      <form v-if="currentTab === 'login'" class="grid gap-4" @submit.prevent="handleLogin">
        <label class="grid gap-2">
          <span class="text-sm text-(--app-text-soft)">用户名 / 邮箱</span>
          <input v-model.trim="loginForm.username" class="app-input" placeholder="输入用户名或邮箱" :disabled="isAuthBusy" />
        </label>
        <label class="grid gap-2">
          <span class="text-sm text-(--app-text-soft)">密码</span>
          <input v-model="loginForm.password" class="app-input" type="password" placeholder="输入密码" :disabled="isAuthBusy" />
        </label>
        <div class="flex items-center justify-between gap-3 pt-2">
          <button class="app-button app-button-primary" type="submit" :disabled="isAuthBusy">
            {{ userStore.authPendingAction === 'login' ? '登录中...' : '登录' }}
          </button>
          <button class="app-button app-button-ghost" type="button" :disabled="isAuthBusy" @click="currentTab = 'reset'">
            忘记密码
          </button>
        </div>
        <p v-if="userStore.authPendingAction === 'login'" class="text-xs text-(--app-text-faint)">正在登录并同步账户状态，请稍候...</p>
      </form>

      <form v-else-if="currentTab === 'register'" class="grid gap-4" @submit.prevent="handleRegister">
        <div class="grid gap-4 md:grid-cols-2">
          <label class="grid gap-2">
            <span class="text-sm text-(--app-text-soft)">用户名</span>
            <input v-model.trim="registerForm.username" class="app-input" placeholder="输入用户名" :disabled="isAuthBusy" />
          </label>
          <label class="grid gap-2">
            <span class="text-sm text-(--app-text-soft)">邮箱</span>
            <input v-model.trim="registerForm.email" class="app-input" placeholder="name@example.com" :disabled="isAuthBusy" />
          </label>
        </div>
        <div class="grid gap-4 md:grid-cols-[1fr_auto]">
          <label class="grid gap-2">
            <span class="text-sm text-(--app-text-soft)">验证码</span>
            <input v-model.trim="registerForm.code" class="app-input" placeholder="输入邮箱验证码" :disabled="isAuthBusy" />
          </label>
          <button
            class="app-button app-button-ghost self-end"
            type="button"
            :disabled="isAuthBusy || userStore.verificationCodeSending"
            @click="sendCode(registerForm.email)"
          >
            {{ userStore.verificationCodeSending ? '发送中...' : '发送验证码' }}
          </button>
        </div>
        <label class="grid gap-2">
          <span class="text-sm text-(--app-text-soft)">密码</span>
          <input v-model="registerForm.password" class="app-input" type="password" placeholder="设置密码" :disabled="isAuthBusy" />
        </label>
        <div class="flex items-center justify-between gap-3 pt-2">
          <button class="app-button app-button-primary" type="submit" :disabled="isAuthBusy">
            {{ userStore.authPendingAction === 'register' ? '创建中...' : '创建账户' }}
          </button>
          <button class="app-button app-button-ghost" type="button" :disabled="isAuthBusy" @click="currentTab = 'login'">
            已有账户
          </button>
        </div>
        <p v-if="userStore.authPendingAction === 'register'" class="text-xs text-(--app-text-faint)">正在提交注册请求，请稍候...</p>
      </form>

      <form v-else class="grid gap-4" @submit.prevent="handleResetPassword">
        <label class="grid gap-2">
          <span class="text-sm text-(--app-text-soft)">邮箱</span>
          <input v-model.trim="resetForm.email" class="app-input" placeholder="输入注册邮箱" :disabled="isAuthBusy" />
        </label>
        <div class="grid gap-4 md:grid-cols-[1fr_auto]">
          <label class="grid gap-2">
            <span class="text-sm text-(--app-text-soft)">验证码</span>
            <input v-model.trim="resetForm.code" class="app-input" placeholder="输入邮箱验证码" :disabled="isAuthBusy" />
          </label>
          <button
            class="app-button app-button-ghost self-end"
            type="button"
            :disabled="isAuthBusy || userStore.verificationCodeSending"
            @click="sendCode(resetForm.email)"
          >
            {{ userStore.verificationCodeSending ? '发送中...' : '发送验证码' }}
          </button>
        </div>
        <label class="grid gap-2">
          <span class="text-sm text-(--app-text-soft)">新密码</span>
          <input v-model="resetForm.newPassword" class="app-input" type="password" placeholder="设置新密码" :disabled="isAuthBusy" />
        </label>
        <div class="flex items-center justify-between gap-3 pt-2">
          <button class="app-button app-button-primary" type="submit" :disabled="isAuthBusy">
            {{ userStore.authPendingAction === 'reset' ? '重置中...' : '重置密码' }}
          </button>
          <button class="app-button app-button-ghost" type="button" :disabled="isAuthBusy" @click="currentTab = 'login'">
            返回登录
          </button>
        </div>
        <p v-if="userStore.authPendingAction === 'reset'" class="text-xs text-(--app-text-faint)">正在重置密码，请稍候...</p>
      </form>
    </div>
  </AppDialog>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue';
import AppDialog from '@/components/shared/AppDialog.vue';
import { useUserStore } from '@/store/user';

const userStore = useUserStore();
const currentTab = ref<'login' | 'register' | 'reset'>('login');
const isAuthBusy = computed(() => userStore.authSubmitting || userStore.verificationCodeSending);

const tabs = [
  { label: '登录', value: 'login' as const },
  { label: '注册', value: 'register' as const },
  { label: '找回密码', value: 'reset' as const },
];

const loginForm = reactive({
  username: '',
  password: '',
});

const registerForm = reactive({
  username: '',
  email: '',
  code: '',
  password: '',
});

const resetForm = reactive({
  email: '',
  code: '',
  newPassword: '',
});

const handleLogin = async () => {
  await userStore.login({
    username: loginForm.username,
    password: loginForm.password,
  });
};

const handleRegister = async () => {
  await userStore.register({
    username: registerForm.username,
    email: registerForm.email,
    code: registerForm.code,
    password: registerForm.password,
  });
  currentTab.value = 'login';
};

const handleResetPassword = async () => {
  await userStore.resetPassword({
    email: resetForm.email,
    code: resetForm.code,
    newPassword: resetForm.newPassword,
  });
  currentTab.value = 'login';
};

const sendCode = async (email: string) => {
  if (!email) {
    return;
  }
  await userStore.sendVerificationCode(email);
};
</script>
