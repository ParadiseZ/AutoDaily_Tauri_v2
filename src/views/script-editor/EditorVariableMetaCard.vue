<template>
  <div class="rounded-[14px] border border-[var(--app-border)] bg-white/45 px-4 py-3">
    <div class="grid gap-2 text-sm md:grid-cols-2">
      <div class="grid gap-1">
        <span class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">作用域</span>
        <span class="text-[var(--app-text-strong)]">{{ namespaceLabel }}</span>
      </div>
      <div class="grid gap-1">
        <span class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">类型</span>
        <span class="text-[var(--app-text-strong)]">{{ variable.valueType }}</span>
      </div>
      <div class="grid gap-1">
        <span class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">键</span>
        <span class="break-all text-[var(--app-text-strong)]">{{ variable.key }}</span>
      </div>
      <div class="grid gap-1">
        <span class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">备注</span>
        <span class="text-[var(--app-text-soft)]">{{ variable.description || '无' }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { EditorVariableOption } from '@/views/script-editor/editorVariables';

const props = defineProps<{
  variable: EditorVariableOption;
}>();

const namespaceLabel = computed(() => {
  if (props.variable.namespace === 'runtime') return 'Runtime';
  if (props.variable.namespace === 'system') return 'System';
  return 'Input';
});
</script>
