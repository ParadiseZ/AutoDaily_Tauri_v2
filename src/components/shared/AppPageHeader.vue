<template>
  <header class="app-page-header">
    <div class="space-y-2">
      <p v-if="eyebrow" class="app-eyebrow">{{ eyebrow }}</p>
      <div class="space-y-1">
        <p class="app-page-title">{{ title }}</p>
        <p v-if="description" class="app-page-description">{{ description }}</p>
      </div>
    </div>
    <div v-if="$slots.actions" class="flex items-center gap-2">
      <slot name="actions" />
    </div>
  </header>
</template>

<script setup lang="ts">
defineProps<{
  title: string;
  description?: string;
  eyebrow?: string;
}>();
</script>
