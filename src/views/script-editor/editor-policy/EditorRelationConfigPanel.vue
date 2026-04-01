<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <template v-if="item">
      <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
        <div class="space-y-4">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.14em] text-[var(--app-text-faint)]">{{ nameLabel }}</span>
            <input :value="item.data.name" class="app-input" @input="$emit('update:name', ($event.target as HTMLInputElement).value)" />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.14em] text-[var(--app-text-faint)]">备注</span>
            <textarea :value="item.data.note" class="app-input min-h-[140px] resize-y" @input="$emit('update:note', ($event.target as HTMLTextAreaElement).value)" />
          </label>

          <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
          <p class="text-sm font-semibold text-[var(--app-text-strong)]">{{ relationTitle }}</p>
          <p class="mt-2 text-xs leading-5 text-[var(--app-text-faint)]">{{ relationDescription }}</p>
          </div>
        </div>
      </div>
    </template>
  </SurfacePanel>
</template>

<script setup lang="ts">
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { PolicyGroupTable } from '@/types/bindings/PolicyGroupTable';
import type { PolicySetTable } from '@/types/bindings/PolicySetTable';

defineProps<{
  item: PolicyGroupTable | PolicySetTable | null;
  nameLabel: string;
  relationTitle: string;
  relationDescription: string;
}>();

defineEmits<{
  'update:name': [value: string];
  'update:note': [value: string];
}>();
</script>
