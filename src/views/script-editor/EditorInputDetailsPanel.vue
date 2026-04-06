<template>
  <div class="min-h-0 overflow-y-auto pr-1 custom-scrollbar">
    <div v-if="selectedInputEntry" class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
      <div class="flex items-start justify-between gap-3">
        <div>
          <p class="text-sm font-semibold text-[var(--app-text-strong)]">变量详情</p>
          <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ selectedInputEntry.key || '未设置键' }}</p>
        </div>
        <button class="app-button app-button-danger app-toolbar-button" type="button" @click="$emit('remove-input', selectedInputEntry.id)">
          删除变量
        </button>
      </div>

      <div class="mt-4 detail-grid">
        <label class="detail-item">
          <span class="detail-label">名称</span>
          <input
            :value="selectedInputEntry.name"
            class="app-input"
            placeholder="例如：扫荡次数"
            @input="$emit('update-input', selectedInputEntry.id, 'name', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <label class="detail-item">
          <span class="detail-label">键</span>
          <input
            :value="selectedInputEntry.key"
            class="app-input"
            placeholder="例如：activitySweepCount"
            :data-testid="selectedInputIndex === 0 ? 'editor-input-key-0' : undefined"
            @input="$emit('update-input', selectedInputEntry.id, 'key', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <div class="detail-item">
          <span class="detail-label">类型</span>
          <EditorSelectField
            :model-value="selectedInputEntry.type"
            :options="inputTypeOptions"
            placeholder="选择类型"
            :test-id="selectedInputIndex === 0 ? 'editor-input-type-0' : undefined"
            @update:model-value="$emit('update-input', selectedInputEntry.id, 'type', String($event))"
          />
        </div>

        <div class="detail-item">
          <span class="detail-label">作用域</span>
          <EditorSelectField
            :model-value="selectedInputEntry.namespace"
            :options="scopeOptions"
            placeholder="选择作用域"
            @update:model-value="$emit('update-input', selectedInputEntry.id, 'namespace', String($event))"
          />
        </div>

        <label class="detail-item">
          <span class="detail-label">备注</span>
          <input
            :value="selectedInputEntry.description"
            class="app-input"
            placeholder="用于后续检索、绑定和变量引用"
            @input="$emit('update-input', selectedInputEntry.id, 'description', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <template v-if="selectedInputEntry.namespace === 'input'">
          <label v-if="selectedInputEntry.type === 'bool'" class="detail-item">
            <span class="detail-label">默认值</span>
            <span class="flex min-h-[44px] items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3 text-sm text-[var(--app-text-soft)]">
              <input
                :checked="selectedInputEntry.booleanValue"
                type="checkbox"
                class="h-4 w-4"
                :data-testid="selectedInputIndex === 0 ? 'editor-input-bool-0' : undefined"
                style="accent-color: var(--app-accent)"
                @change="$emit('update-input', selectedInputEntry.id, 'booleanValue', ($event.target as HTMLInputElement).checked)"
              />
              <span>默认启用</span>
            </span>
          </label>

          <label v-else class="detail-item detail-item-top">
            <span class="detail-label">默认值</span>
            <textarea
              v-if="selectedInputEntry.type === 'json'"
              :value="selectedInputEntry.stringValue"
              class="app-textarea min-h-[120px]"
              spellcheck="false"
              @input="$emit('update-input', selectedInputEntry.id, 'stringValue', ($event.target as HTMLTextAreaElement).value)"
            />
            <input
              v-else
              :value="selectedInputEntry.stringValue"
              class="app-input"
              :type="selectedInputEntry.type === 'string' ? 'text' : 'number'"
              :data-testid="selectedInputIndex === 0 ? 'editor-input-value-0' : undefined"
              @input="$emit('update-input', selectedInputEntry.id, 'stringValue', ($event.target as HTMLInputElement).value)"
            />
          </label>
        </template>

        <div
          v-else
          class="detail-item detail-span-2 detail-item-top rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4 text-sm leading-6 text-[var(--app-text-soft)]"
        >
          {{ selectedInputEntry.namespace === 'runtime' ? 'Runtime 变量只定义结构和来源，不在这里设置默认值。' : 'System 变量由运行时注入，只在这里保留元数据。' }}
        </div>
      </div>
    </div>

    <EmptyState
      v-else
      title="选择一个变量"
      description="中间列表选中变量后，右侧才会显示名称、键、类型、作用域和值。"
    />
  </div>
</template>

<script setup lang="ts">
import EmptyState from '@/components/shared/EmptyState.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import { editorInputTypeOptions, type EditorInputEntry } from '@/views/script-editor/editorVariables';

defineOptions({ name: 'EditorInputDetailsPanel' });

const props = defineProps<{
  selectedInputEntry: EditorInputEntry | null;
  selectedInputIndex: number;
}>();

defineEmits<{
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
  'remove-input': [entryId: string];
}>();

const inputTypeOptions = editorInputTypeOptions;
const scopeOptions = [
  { label: 'Input', value: 'input', description: '用户可配置并持久化的输入变量。' },
  { label: 'Runtime', value: 'runtime', description: '步骤执行过程中的运行时变量。' },
  { label: 'System', value: 'system', description: '运行时注入的只读系统变量。' },
];
</script>

<style scoped>
.detail-grid {
  display: grid;
  gap: 0.9rem 1rem;
}

.detail-item {
  display: grid;
  gap: 0.75rem;
}

.detail-label {
  display: flex;
  align-items: center;
  min-height: 44px;
  color: var(--app-text-faint);
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

@media (min-width: 768px) {
  .detail-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .detail-item {
    grid-template-columns: 72px minmax(0, 1fr);
    align-items: center;
  }

  .detail-item-top {
    align-items: start;
  }

  .detail-span-2 {
    grid-column: 1 / -1;
  }
}

.editor-inline-grid {
  display: grid;
  gap: 0.75rem;
}

@media (min-width: 768px) {
  .editor-inline-grid {
    grid-template-columns: 72px minmax(0, 1fr) 72px minmax(0, 1fr);
    align-items: center;
  }
}

.editor-inline-label {
  display: flex;
  align-items: center;
  min-height: 44px;
  color: var(--app-text-faint);
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.editor-inline-content {
  min-height: 44px;
}
</style>
