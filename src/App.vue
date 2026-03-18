<template>
  <component :is="layout">
    <router-view />
  </component>
  <AuthModal />
</template>

<script setup>
import { onMounted,computed } from 'vue';
import { useRoute } from 'vue-router';
import MainLayout from './layouts/MainLayout.vue';
import AuthModal from './components/AuthModal.vue';
import { useThemeManager } from './views/script-editor/composables/index.js';
import { appThemeKey } from './store/store.js'
import { useUserStore } from './store/user.js';

const { initTheme } = useThemeManager();

const route = useRoute();
const layout = computed(() => {
  return route.path === '/editor' ? 'div' : MainLayout;
});

// 生命周期
onMounted(async () => {
  initTheme(appThemeKey);
  
  // App start profile check (will auto populate if token is valid)
  const userStore = useUserStore();
  await userStore.checkProfile();
});
</script>
