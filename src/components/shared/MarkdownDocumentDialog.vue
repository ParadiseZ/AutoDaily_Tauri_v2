<template>
  <AppDialog :open="open" :title="title" :description="description" width-class="max-w-4xl" @close="$emit('close')">
    <div class="space-y-4">
      <div v-if="loading" class="rounded-[16px] border border-(--app-border) bg-(--app-panel-muted)/60 px-4 py-5 text-sm text-(--app-text-soft)">
        正在加载内容...
      </div>
      <div v-else-if="error" class="rounded-[16px] border border-red-300/60 bg-red-50 px-4 py-5 text-sm text-red-700">
        {{ error }}
      </div>
      <div v-else class="max-h-[65vh] overflow-y-auto pr-1 custom-scrollbar rounded-[16px] border border-(--app-border) bg-(--app-panel-muted)/40 px-4 py-4">
        <MarkdownView :content="content" :empty-text="emptyText" />
      </div>

      <div class="flex justify-end">
        <button class="app-button app-button-primary" type="button" @click="$emit('close')">
          关闭
        </button>
      </div>
    </div>
  </AppDialog>
</template>

<script setup lang="ts">
import AppDialog from '@/components/shared/AppDialog.vue';
import MarkdownView from '@/components/shared/MarkdownView.vue';

withDefaults(
  defineProps<{
    open: boolean;
    title: string;
    description?: string;
    content?: string | null;
    loading?: boolean;
    error?: string;
    emptyText?: string;
  }>(),
  {
    description: '',
    content: '',
    loading: false,
    error: '',
    emptyText: '暂无内容。',
  },
);

defineEmits<{
  close: [];
}>();
</script>
