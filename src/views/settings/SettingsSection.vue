<template>
  <SurfacePanel class="space-y-4">
    <div class="space-y-1">
      <p class="text-sm font-semibold text-[var(--app-text-strong)]">{{ title }}</p>
      <p v-if="description" class="text-sm text-[var(--app-text-soft)]">{{ description }}</p>
    </div>
    <slot />
  </SurfacePanel>
</template>

<script setup lang="ts">
import SurfacePanel from '@/components/shared/SurfacePanel.vue';

defineProps<{
  title: string;
  description?: string;
}>();
</script>
