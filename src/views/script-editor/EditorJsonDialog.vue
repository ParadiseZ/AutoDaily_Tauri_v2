<template>
  <AppDialog
    :open="open"
    :title="title"
    :description="description"
    width-class="max-w-4xl"
    @close="$emit('close')"
  >
    <div class="space-y-4">
      <textarea
        :value="modelValue"
        class="editor-codearea"
        rows="20"
        spellcheck="false"
        @input="$emit('update:modelValue', ($event.target as HTMLTextAreaElement).value)"
      />

      <p v-if="error" class="text-sm text-red-700">{{ error }}</p>

      <div class="flex justify-end gap-2">
        <button class="app-button app-button-ghost" type="button" @click="$emit('format')">格式化</button>
        <button class="app-button app-button-ghost" type="button" @click="$emit('close')">关闭</button>
        <button class="app-button app-button-primary" type="button" @click="$emit('apply')">应用草稿</button>
      </div>
    </div>
  </AppDialog>
</template>

<script setup lang="ts">
import AppDialog from '@/components/shared/AppDialog.vue';

defineProps<{
  open: boolean;
  title: string;
  description: string;
  modelValue: string;
  error: string | null;
}>();

defineEmits<{
  close: [];
  apply: [];
  format: [];
  'update:modelValue': [value: string];
}>();
</script>

<style scoped>
.editor-codearea {
  width: 100%;
  resize: vertical;
  border-radius: 22px;
  border: 1px solid var(--app-border);
  background:
    linear-gradient(180deg, rgba(15, 23, 42, 0.03), rgba(15, 23, 42, 0.02)),
    color-mix(in srgb, var(--app-panel-muted) 96%, transparent);
  color: var(--app-text-strong);
  padding: 1rem 1.1rem;
  font-family: "SF Mono", "Consolas", "Liberation Mono", monospace;
  font-size: 0.86rem;
  line-height: 1.7;
}

.editor-codearea:focus {
  outline: none;
  border-color: rgba(70, 110, 255, 0.28);
  box-shadow: 0 0 0 4px rgba(70, 110, 255, 0.08);
}
</style>
