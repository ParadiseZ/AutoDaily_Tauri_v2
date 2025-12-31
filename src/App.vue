<template>
<!--
  <component :is="layout">
-->
  <component :is="MainLayout">
    <router-view />
  </component>
</template>

<script setup>
import { onMounted,computed } from 'vue';
import { useRoute } from 'vue-router';
import MainLayout from './layouts/MainLayout.vue';
import { useThemeManager } from './views/script-editor/composables/index.js';
import {appThemeKey} from './store/store.js'

const { initTheme } = useThemeManager();

const route = useRoute();
const layout = computed(() => {
  return route.path === '/editor' ? 'div' : MainLayout;
});

// 生命周期
onMounted(() => {
  initTheme(appThemeKey)
});
</script>
