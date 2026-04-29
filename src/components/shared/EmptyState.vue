<template>
  <div class="app-empty-state">
    <div v-if="icon" class="app-empty-icon">
      <AppIcon :name="icon" :size="20" />
    </div>
    <div class="space-y-1 text-center">
      <h3 class="text-sm font-semibold text-(--app-text-strong)">{{ title }}</h3>
      <p v-if="description" class="max-w-sm text-sm text-(--app-text-soft)">{{ description }}</p>
    </div>
    <div v-if="$slots.default" class="mt-2">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import AppIcon from '@/components/shared/AppIcon.vue';

defineProps<{
  title: string;
  description?: string;
  icon?: string;
}>();
</script>
