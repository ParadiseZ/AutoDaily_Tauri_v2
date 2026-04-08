<template>
  <SurfacePanel class="space-y-4">
    <div class="flex items-center gap-3">
      <div v-if="icon" class="flex h-10 w-10 shrink-0 items-center justify-center rounded-[14px] bg-[var(--app-panel-muted)] border border-[var(--app-border)] text-[var(--app-text-strong)]">
        <AppIcon :name="icon" :size="20" />
      </div>
      <div class="space-y-1 pt-0.5">
        <p class="text-sm font-semibold text-[var(--app-text-strong)]">{{ title }}</p>
        <p v-if="description" class="text-xs text-[var(--app-text-soft)]">{{ description }}</p>
      </div>
    </div>
    <slot />
  </SurfacePanel>
</template>

<script setup lang="ts">
import AppIcon from '@/components/shared/AppIcon.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';

defineProps<{
  title: string;
  description?: string;
  icon?: string;
}>();
</script>
