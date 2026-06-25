<template>
  <div class="space-y-4">
    <div class="flex items-center justify-between gap-3">
      <div>
        <p class="text-sm font-semibold text-(--app-text-strong)">快速模板</p>
      </div>
    </div>

    <div class="space-y-3">
      <details v-for="group in templateGroups" :key="group.name" class="rounded-[18px] border border-(--app-border) bg-(--app-panel-muted)" :open="true">
        <summary class="cursor-pointer list-none px-4 py-3 text-sm font-semibold text-(--app-text-strong)">
          <div class="flex items-center justify-between gap-3">
            <span>{{ group.name }}</span>
            <span class="text-xs text-(--app-text-faint)">{{ group.items.length }}</span>
          </div>
        </summary>

        <div class="grid gap-2 px-3 pb-3 sm:grid-cols-2">
          <button
            v-for="template in group.items"
            :key="template.id"
            class="editor-template-tile"
            :data-testid="`${testIdPrefix}-${template.id}`"
            type="button"
            @click="$emit('select', template.id)"
          >
            <span class="editor-template-icon" v-html="template.icon" />
            <span class="truncate">{{ template.label }}</span>
          </button>
        </div>
      </details>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import {
  editorStepTemplates,
  isActionSequenceTemplateId,
} from '@/views/script-editor/editor-step/editorStepTemplates';

const props = withDefaults(
  defineProps<{
    restrictSequenceTemplates?: boolean;
    testIdPrefix?: string;
  }>(),
  {
    restrictSequenceTemplates: false,
    testIdPrefix: 'editor-step-template',
  },
);

defineEmits<{
  select: [templateId: string];
}>();

const templateGroups = computed(() => {
  const grouped = new Map<string, typeof editorStepTemplates>();
  for (const template of editorStepTemplates) {
    if (props.restrictSequenceTemplates && !isActionSequenceTemplateId(template.id)) {
      continue;
    }
    const bucket = grouped.get(template.group) ?? [];
    bucket.push(template);
    grouped.set(template.group, bucket);
  }

  const groupOrder = props.restrictSequenceTemplates ? ['动作', '流程'] : ['动作', '流程', '数据', '视觉', '状态', '容器', '兼容'];
  return Array.from(grouped.entries())
    .sort(([left], [right]) => {
      const leftIndex = groupOrder.indexOf(left);
      const rightIndex = groupOrder.indexOf(right);
      return (leftIndex === -1 ? Number.MAX_SAFE_INTEGER : leftIndex) - (rightIndex === -1 ? Number.MAX_SAFE_INTEGER : rightIndex);
    })
    .map(([name, items]) => ({ name, items }));
});
</script>

<style scoped>
.editor-template-tile {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  width: 100%;
  border-radius: 14px;
  border: 1px solid var(--app-border);
  background: var(--app-panel-muted);
  padding: 0.7rem 0.8rem;
  text-align: left;
  color: var(--app-text-strong);
  transition: border-color 0.16s ease, background 0.16s ease;
}

.editor-template-tile:hover {
  border-color: rgba(70, 110, 255, 1);
  cursor: pointer;
}

.editor-template-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 1.15rem;
  height: 1.15rem;
  color: var(--app-accent);
  flex-shrink: 0;
}

.editor-template-icon :deep(svg) {
  width: 100%;
  height: 100%;
}
</style>
