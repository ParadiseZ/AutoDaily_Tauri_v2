<template>
  <AppDialog
    :open="open"
    :closable="false"
    title="使用 AutoDaily 前请先确认"
    description="请完整阅读免责声明；不同意将退出程序。"
    width-class="max-w-3xl"
  >
    <div class="flex min-h-0 flex-col gap-4">
      <div class="min-h-0 flex-1 overflow-auto rounded-md border border-(--app-border) bg-(--app-bg-muted) p-4 pr-3 custom-scrollbar">
        <MarkdownView :content="disclaimerContent" />
      </div>
      <p v-if="error" class="rounded-md bg-red-500/10 px-3 py-2 text-sm text-red-700">{{ error }}</p>
      <div class="flex flex-wrap justify-end gap-2">
        <button class="app-button app-button-ghost" type="button" :disabled="busy" @click="$emit('decline')">不同意并退出</button>
        <button class="app-button app-button-primary" type="button" :disabled="busy" @click="$emit('accept')">
          {{ busy ? '正在保存...' : '我已阅读并同意' }}
        </button>
      </div>
    </div>
  </AppDialog>
</template>

<script setup lang="ts">
import disclaimerContent from '../../../doc/上线文档/免责声明.md?raw';
import AppDialog from '@/components/shared/AppDialog.vue';
import MarkdownView from '@/components/shared/MarkdownView.vue';

defineProps<{ open: boolean; busy: boolean; error: string }>();
defineEmits<{ accept: []; decline: [] }>();
</script>
