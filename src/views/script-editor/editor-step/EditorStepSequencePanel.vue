<template>
  <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
    <input
      :checked="reverse"
      type="checkbox"
      class="h-4 w-4"
      style="accent-color: var(--app-accent)"
      @change="$emit('update-reverse', ($event.target as HTMLInputElement).checked)"
    />
    <span class="text-sm text-[var(--app-text-soft)]">倒序执行子步骤</span>
  </label>
</template>

<script setup lang="ts">
defineOptions({ name: 'EditorStepSequencePanel' });

defineProps<{
  reverse: boolean;
}>();

defineEmits<{
  'update-reverse': [value: boolean];
}>();
</script>
