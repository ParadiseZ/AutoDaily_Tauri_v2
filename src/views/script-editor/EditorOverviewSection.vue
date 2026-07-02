<template>
  <section class="editor-overview-section">
    <div v-if="title" class="editor-overview-section-header">
      <div class="min-w-0">
        <slot name="header">
          <component :is="headingTag" class="editor-overview-section-title">{{ title }}</component>
          <p v-if="description" class="editor-overview-section-description">{{ description }}</p>
        </slot>
      </div>

      <div v-if="$slots.actions" class="shrink-0">
        <slot name="actions" />
      </div>
    </div>

    <div
      class="editor-overview-section-content"
      :class="width === 'wide' ? 'editor-overview-section-content-wide' : 'editor-overview-section-content-default'"
    >
      <slot />
    </div>
  </section>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    title?: string;
    headingTag?: 'h1' | 'h2' | 'h3';
    width?: 'default' | 'wide';
    description?: string;
  }>(),
  {
    headingTag: 'h3',
    width: 'default',
    description: '',
  },
);

defineOptions({ name: 'EditorOverviewSection' });
</script>

<style scoped>
@reference "../../style.css";

.editor-overview-section {
  @apply rounded-[16px] border border-(--app-border) bg-(--app-panel) px-5 py-5 shadow-[0_4px_12px_rgba(15,23,42,0.03)];
}
/* 
.editor-overview-section-header {
  @apply mb-4 flex items-start justify-between gap-3 border-b border-(--app-border) pb-3;
} */
 .editor-overview-section-header {
  @apply mb-4 flex items-start justify-between gap-3  pb-1;
}

.editor-overview-section-title {
  @apply text-sm font-semibold text-(--app-text-strong);
}

.editor-overview-section-description {
  @apply mt-1 text-xs text-(--app-text-faint);
}

.editor-overview-section-content {
  @apply flex w-full flex-col gap-4;
}

.editor-overview-section-content-default {
  @apply max-w-[38rem];
}

.editor-overview-section-content-wide {
  @apply max-w-[42rem];
}
</style>
