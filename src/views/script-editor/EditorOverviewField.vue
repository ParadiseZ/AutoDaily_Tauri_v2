<template>
  <div class="editor-overview-field" :class="width === 'compact' ? 'editor-overview-field-compact' : ''">
    <span class="editor-overview-field-label">{{ label }}</span>
    <slot />
  </div>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string;
    width?: 'full' | 'compact';
  }>(),
  {
    width: 'full',
  },
);

defineOptions({ name: 'EditorOverviewField' });
</script>

<style scoped>
@reference "../../style.css";

.editor-overview-field {
  @apply flex w-full flex-col gap-1.5;
}

.editor-overview-field-compact {
  @apply max-w-[16rem];
}

.editor-overview-field-label {
  @apply text-xs font-semibold tracking-wide text-(--app-text-soft);
}
</style>
