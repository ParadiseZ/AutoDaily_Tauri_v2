<template>
  <div class="app-loading-state">
    <AppIcon name="loader-circle" :size="18" class="app-loading-spinner" />
    <span>{{ label }}</span>
  </div>
</template>

<script setup lang="ts">
import AppIcon from '@/components/shared/AppIcon.vue';

withDefaults(
  defineProps<{
    label?: string;
  }>(),
  {
    label: '正在加载...',
  },
);
</script>
