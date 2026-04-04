<template>
  <Teleport to="body">
    <transition name="dialog-fade">
      <div v-if="open" class="app-dialog-backdrop" @click.self="$emit('close')">
        <div class="app-dialog" :class="widthClass" aria-modal="true" :aria-label="title" role="dialog">
          <header class="mb-5 flex items-start justify-between gap-4">
            <div class="space-y-1">
              <h2 class="text-lg font-semibold text-[var(--app-text-strong)]">{{ title }}</h2>
              <p v-if="description" class="text-sm text-[var(--app-text-soft)]">{{ description }}</p>
            </div>
            <button class="app-icon-button app-dialog-close-button" type="button" @click="$emit('close')">
              <AppIcon name="x" :size="18" />
            </button>
          </header>
          <slot />
        </div>
      </div>
    </transition>
  </Teleport>
</template>

<script setup lang="ts">
import AppIcon from '@/components/shared/AppIcon.vue';

withDefaults(
  defineProps<{
    open: boolean;
    title: string;
    description?: string;
    widthClass?: string;
  }>(),
  {
    widthClass: 'max-w-3xl',
  },
);

defineEmits<{
  close: [];
}>();
</script>

<style scoped>
.dialog-fade-enter-active,
.dialog-fade-leave-active {
  transition: opacity 0.16s ease, transform 0.16s ease;
}

.dialog-fade-enter-from,
.dialog-fade-leave-to {
  opacity: 0;
}

.app-dialog-close-button {
  transition:
    transform 0.16s ease,
    background 0.18s ease,
    border-color 0.18s ease,
    color 0.18s ease,
    box-shadow 0.18s ease;
}

.app-dialog-close-button:hover {
  background: linear-gradient(135deg, rgba(248, 113, 113, 0.2), rgba(220, 38, 38, 0.34));
  border-color: rgba(220, 38, 38, 0.32);
  color: rgb(185, 28, 28);
  box-shadow: 0 10px 22px rgba(220, 38, 38, 0.12);
}
</style>
