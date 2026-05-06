<template>
  <div v-if="html" class="markdown-view" v-html="html" />
  <p v-else class="text-sm text-(--app-text-faint)">{{ emptyText }}</p>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { renderMarkdown } from '@/utils/markdown';

const props = withDefaults(
  defineProps<{
    content?: string | null;
    emptyText?: string;
  }>(),
  {
    content: '',
    emptyText: '暂无内容。',
  },
);

const html = computed(() => renderMarkdown((props.content ?? '').trim()));
</script>

<style scoped>
.markdown-view {
  color: var(--app-text-soft);
  font-size: 0.9rem;
  line-height: 1.75;
  word-break: break-word;
}

.markdown-view :deep(h1),
.markdown-view :deep(h2),
.markdown-view :deep(h3) {
  color: var(--app-text-strong);
  font-weight: 700;
  line-height: 1.3;
}

.markdown-view :deep(h1) {
  font-size: 1.25rem;
  margin: 0 0 0.75rem;
}

.markdown-view :deep(h2) {
  font-size: 1.05rem;
  margin: 1rem 0 0.5rem;
}

.markdown-view :deep(h3) {
  font-size: 0.95rem;
  margin: 0.85rem 0 0.4rem;
}

.markdown-view :deep(p) {
  margin: 0.45rem 0;
}

.markdown-view :deep(ul) {
  margin: 0.55rem 0;
  padding-left: 1.25rem;
}

.markdown-view :deep(li) {
  margin: 0.25rem 0;
}

.markdown-view :deep(strong) {
  color: var(--app-text-strong);
}
</style>
