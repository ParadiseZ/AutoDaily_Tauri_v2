<template>
  <span class="app-badge" :class="`app-badge-${tone}`">
    <span class="app-badge-dot" />
    <span>{{ label }}</span>
  </span>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string;
    tone?: 'neutral' | 'info' | 'success' | 'warning' | 'danger';
  }>(),
  {
    tone: 'neutral',
  },
);
</script>
