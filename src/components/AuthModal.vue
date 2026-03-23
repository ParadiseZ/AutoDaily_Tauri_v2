<template>
  <AppDialog
    :open="userStore.isAuthModalOpen"
    title="账户中心"
    description="登录后可同步脚本、访问脚本市场和管理账户状态。"
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
          @click="currentTab = item.value"
        >
          {{ item.label }}
        </button>
      </div>

      <form v-if="currentTab === 'login'" class="grid gap-4" @submit.prevent="handleLogin">
        <label class="grid gap-2">
          <span class="text-sm text-[var(--app-text-soft)]">用户名 / 邮箱</span>
          <input v-model.trim="loginForm.username" class="app-input" placeholder="输入用户名或邮箱" />
        </label>
        <label class="grid gap-2">
          <span class="text-sm text-[var(--app-text-soft)]">密码</span>
          <input v-model="loginForm.password" class="app-input" type="password" placeholder="输入密码" />
        </label>
        <div class="flex items-center justify-between gap-3 pt-2">
          <button class="app-button app-button-primary" type="submit" :disabled="userStore.authSubmitting">
            登录
          </button>
          <button class="app-button app-button-ghost" type="button" @click="currentTab = 'reset'">
            忘记密码
          </button>
        </div>
      </form>

      <form v-else-if="currentTab === 'register'" class="grid gap-4" @submit.prevent="handleRegister">
        <div class="grid gap-4 md:grid-cols-2">
          <label class="grid gap-2">
            <span class="text-sm text-[var(--app-text-soft)]">用户名</span>
            <input v-model.trim="registerForm.username" class="app-input" placeholder="输入用户名" />
          </label>
          <label class="grid gap-2">
            <span class="text-sm text-[var(--app-text-soft)]">邮箱</span>
            <input v-model.trim="registerForm.email" class="app-input" placeholder="name@example.com" />
          </label>
        </div>
        <div class="grid gap-4 md:grid-cols-[1fr_auto]">
          <label class="grid gap-2">
            <span class="text-sm text-[var(--app-text-soft)]">验证码</span>
            <input v-model.trim="registerForm.code" class="app-input" placeholder="输入邮箱验证码" />
          </label>
          <button class="app-button app-button-ghost self-end" type="button" @click="sendCode(registerForm.email)">
            发送验证码
          </button>
        </div>
        <div class="grid gap-4 md:grid-cols-2">
          <label class="grid gap-2">
            <span class="text-sm text-[var(--app-text-soft)]">密码</span>
            <input v-model="registerForm.password" class="app-input" type="password" placeholder="设置密码" />
          </label>
          <label class="grid gap-2">
            <span class="text-sm text-[var(--app-text-soft)]">手机号（可选）</span>
            <input v-model.trim="registerForm.phone" class="app-input" placeholder="便于后续找回" />
          </label>
        </div>
        <div class="flex items-center justify-between gap-3 pt-2">
          <button class="app-button app-button-primary" type="submit" :disabled="userStore.authSubmitting">
            创建账户
          </button>
          <button class="app-button app-button-ghost" type="button" @click="currentTab = 'login'">
            已有账户
          </button>
        </div>
      </form>

      <form v-else class="grid gap-4" @submit.prevent="handleResetPassword">
        <label class="grid gap-2">
          <span class="text-sm text-[var(--app-text-soft)]">邮箱</span>
          <input v-model.trim="resetForm.email" class="app-input" placeholder="输入注册邮箱" />
        </label>
        <div class="grid gap-4 md:grid-cols-[1fr_auto]">
          <label class="grid gap-2">
            <span class="text-sm text-[var(--app-text-soft)]">验证码</span>
            <input v-model.trim="resetForm.code" class="app-input" placeholder="输入邮箱验证码" />
          </label>
          <button class="app-button app-button-ghost self-end" type="button" @click="sendCode(resetForm.email)">
            发送验证码
          </button>
        </div>
        <label class="grid gap-2">
          <span class="text-sm text-[var(--app-text-soft)]">新密码</span>
          <input v-model="resetForm.newPassword" class="app-input" type="password" placeholder="设置新密码" />
        </label>
        <div class="flex items-center justify-between gap-3 pt-2">
          <button class="app-button app-button-primary" type="submit" :disabled="userStore.authSubmitting">
            重置密码
          </button>
          <button class="app-button app-button-ghost" type="button" @click="currentTab = 'login'">
            返回登录
          </button>
        </div>
      </form>
    </div>
  </AppDialog>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue';
import AppDialog from '@/components/shared/AppDialog.vue';
import { useUserStore } from '@/store/user';

const userStore = useUserStore();
const currentTab = ref<'login' | 'register' | 'reset'>('login');

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
  phone: '',
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
    phone: registerForm.phone || null,
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
