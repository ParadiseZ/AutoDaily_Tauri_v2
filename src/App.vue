<template>
<!--
  <component :is="layout">
-->
  <component :is="MainLayout">
    <router-view />
  </component>
</template>

<script setup>
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import MainLayout from './layouts/MainLayout.vue';

const route = useRoute();
const layout = computed(() => {
  return route.path === '/editor' ? 'div' : MainLayout;
});
</script>
