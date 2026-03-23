<template>
  <Teleport to="body">
    <transition name="dialog-fade">
      <div v-if="open" class="app-dialog-backdrop" @click.self="$emit('close')">
        <div class="app-dialog" :class="widthClass">
          <header class="mb-5 flex items-start justify-between gap-4">
            <div class="space-y-1">
              <h2 class="text-lg font-semibold text-[var(--app-text-strong)]">{{ title }}</h2>
              <p v-if="description" class="text-sm text-[var(--app-text-soft)]">{{ description }}</p>
            </div>
            <button class="app-icon-button" type="button" @click="$emit('close')">
              <span class="text-lg leading-none">×</span>
            </button>
          </header>
          <slot />
        </div>
      </div>
    </transition>
  </Teleport>
</template>

<script setup lang="ts">
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
</style>
