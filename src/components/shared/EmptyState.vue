<template>
  <div class="app-empty-state">
    <div v-if="icon" class="app-empty-icon">
      <component :is="icon" class="h-5 w-5" />
    </div>
    <div class="space-y-1 text-center">
      <h3 class="text-sm font-semibold text-[var(--app-text-strong)]">{{ title }}</h3>
      <p class="max-w-sm text-sm text-[var(--app-text-soft)]">{{ description }}</p>
    </div>
    <div v-if="$slots.default" class="mt-2">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import type { Component } from 'vue';

defineProps<{
  title: string;
  description: string;
  icon?: Component;
}>();
</script>
