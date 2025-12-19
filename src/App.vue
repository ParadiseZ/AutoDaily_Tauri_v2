<template>
  <component :is="layout">
    <router-view />
  </component>
</template>

<script setup>
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import MainLayout from './layouts/MainLayout.vue';

const route = useRoute();
/*const layout = computed(() => {
  return route.path === '/editor' ? 'div' : MainLayout;
});*/
//先开发编辑器
const layout = 'div';
</script>
