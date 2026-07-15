<template>
  <div class="space-y-4">
    <div class="grid grid-cols-[minmax(0,1fr)_44px] items-center gap-2">
      <input v-model="search" class="app-input" type="search" placeholder="按名称 / 键 / 备注搜索变量" />
      <button class="app-button app-button-primary app-toolbar-button justify-center" type="button" title="添加变量" aria-label="添加变量" data-testid="editor-input-add" @click="$emit('add')">
        <Plus class="h-4 w-4" />
      </button>
    </div>

    <div v-if="filteredEntries.length" class="space-y-2">
      <article
        v-for="(entry, index) in filteredEntries"
        :key="entry.id"
        class="app-list-item cursor-pointer"
        :class="{ 'app-list-item-active': selectedInputId === entry.id }"
        :data-testid="`editor-input-item-${index}`"
        @click="$emit('select', entry.id)"
      >
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <p
              class="truncate text-sm font-semibold"
              :class="entryReferenceState[entry.id]?.referenced ? 'text-emerald-600' : 'text-red-600'"
            >
              {{ entry.name || entry.key || '未命名变量' }}
            </p>
            <p class="mt-1 text-xs text-(--app-text-faint)">{{ entry.key || '未设置键' }} · {{ getScopeLabel(entry.namespace) }} · {{ getInputTypeLabel(entry.type) }}</p>
          </div>
          <button
            class="app-icon-button app-crash-icon app-icon-button-sec shrink-0"
            type="button"
            :title="removeTitle"
            :aria-label="removeTitle"
            :data-testid="`editor-input-remove-${index}`"
            @click.stop="$emit('remove', entry.id)"
          >
            <Trash2 class="h-4 w-4" />
          </button>
        </div>
      </article>
    </div>

    <EmptyState v-else title="还没有变量" />

    <p v-if="inputError" class="text-sm text-red-700">{{ inputError }}</p>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import { Plus, Trash2 } from '@lucide/vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import { getInputTypeLabel, type EditorInputEntry } from '@/views/script-editor/editorVariables';

defineOptions({ name: 'EditorVariableListPanel' });

const props = withDefaults(
  defineProps<{
    entries: EditorInputEntry[];
    selectedInputId: string | null;
    inputError?: string | null;
    removeTitle?: string;
    entryReferenceState?: Record<string, { referenced: boolean }>;
  }>(),
  {
    inputError: null,
    removeTitle: '删除',
    entryReferenceState: () => ({}),
  },
);

defineEmits<{
  add: [];
  select: [entryId: string];
  remove: [entryId: string];
}>();

const search = ref('');

const filteredEntries = computed(() => {
  const keyword = search.value.trim().toLowerCase();
  if (!keyword) {
    return props.entries;
  }
  return props.entries.filter((entry) =>
    `${entry.name} ${entry.key} ${entry.description}`.toLowerCase().includes(keyword),
  );
});

const getScopeLabel = (scope: EditorInputEntry['namespace']) => {
  if (scope === 'runtime') return '运行时';
  if (scope === 'system') return '系统';
  return '输入';
};
</script>
