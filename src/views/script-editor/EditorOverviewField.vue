<template>
  <div class="field" :class="{
    'field-compact': width === 'compact',
    'field-radio': width === 'radio'
  }">
    <span class="field-label">{{ label }}</span>
    <slot />
  </div>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    label: string;
    width?: 'full' | 'compact'| 'radio';
  }>(),
  {
    width: 'full',
  },
);

defineOptions({ name: 'EditorOverviewField' });
</script>

<style scoped>
@reference "../../style.css";

.field {
  @apply flex w-full flex-col gap-1.5;
}

.field-compact {
  @apply max-w-[16rem];
}

.field-radio{
  @apply max-w-[24rem];
}

.field-label {
  @apply text-xs font-semibold tracking-wide text-(--app-text-soft);
}
</style>
