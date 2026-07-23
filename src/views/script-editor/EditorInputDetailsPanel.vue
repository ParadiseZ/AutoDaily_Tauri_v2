<template>
  <EditorOverviewPanel>
    <EditorOverviewSection
      v-if="selectedInputEntry"
      :key="`${selectedInputEntry.id}:${selectedInputEntry.type}`"
      title="变量详情"
      heading-tag="h1"
      width="wide"
    >
      <EditorOverviewField label="名称" width="compact">
        <input
          :value="selectedInputEntry.name"
          class="app-input"
          placeholder="例如：扫荡次数"
          @input="$emit('update-input', selectedInputEntry.id, 'name', ($event.target as HTMLInputElement).value)"
        />
      </EditorOverviewField>

      <EditorOverviewField label="键 *（必填）" width="compact">
        <input
          :value="selectedInputEntry.key"
          class="app-input"
          :class="{ 'app-input-invalid': isKeyMissing }"
          placeholder="例如：activitySweepCount"
          :aria-invalid="isKeyMissing"
          :data-testid="selectedInputIndex === 0 ? 'editor-input-key-0' : undefined"
          @input="$emit('update-input', selectedInputEntry.id, 'key', ($event.target as HTMLInputElement).value)"
        />
      </EditorOverviewField>

      <EditorOverviewField label="类型" width="compact">
        <EditorSelectField
          :model-value="selectedInputEntry.type"
          :options="inputTypeOptions"
          placeholder="选择类型"
          :test-id="selectedInputIndex === 0 ? 'editor-input-type-0' : undefined"
          @update:model-value="$emit('update-input', selectedInputEntry.id, 'type', String($event))"
        />
      </EditorOverviewField>

      <template v-if="selectedInputEntry.namespace === 'input'">
        <EditorOverviewField v-if="selectedInputEntry.type === 'bool'" label="默认值" width="compact">
          <span class="editor-detail-toggle">
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
        </EditorOverviewField>

        <EditorOverviewField v-else label="默认值" width="compact">
          <textarea
            v-if="selectedInputEntry.type === 'json'"
            :value="selectedInputEntry.stringValue"
            class="app-textarea min-h-[120px] max-w-[38rem]"
            spellcheck="false"
            @input="$emit('update-input', selectedInputEntry.id, 'stringValue', ($event.target as HTMLTextAreaElement).value)"
          />
          <input
            v-else-if="selectedInputEntry.type === 'image'"
            value=""
            placeholder="不支持input图像变量默认值"
            disabled
            class="app-input"
            type="text"
            :data-testid="selectedInputIndex === 0 ? 'editor-input-value-0' : undefined"
            @input="$emit('update-input', selectedInputEntry.id, 'stringValue', ($event.target as HTMLInputElement).value)"
          />
          <input
            v-else
            :value="selectedInputEntry.stringValue"
            class="app-input"
            :type="selectedInputEntry.type === 'string' ? 'text' : 'number'"
            :data-testid="`editor-input-value-${selectedInputIndex}`"
            @input="$emit('update-input', selectedInputEntry.id, 'stringValue', ($event.target as HTMLInputElement).value)"
          />
        </EditorOverviewField>
      </template>

      <div
        v-else

      >
          {{ selectedInputEntry.namespace === 'runtime' ? 'Runtime 变量只定义结构和来源，不在这里设置默认值。' : 'System 变量由运行时注入，只在这里保留元数据。' }}
      </div>

      <EditorOverviewField label="作用域" width="radio">
        <EditorSelectField
            :model-value="selectedInputEntry.namespace"
            :options="scopeOptions"
            placeholder="选择作用域"
            @update:model-value="$emit('update-input', selectedInputEntry.id, 'namespace', String($event))"
        />
      </EditorOverviewField>

      <EditorOverviewField label="备注">
        <input
            :value="selectedInputEntry.description"
            class="app-input"
            placeholder="用于后续检索、绑定和变量引用"
            @input="$emit('update-input', selectedInputEntry.id, 'description', ($event.target as HTMLInputElement).value)"
        />
      </EditorOverviewField>
    </EditorOverviewSection>

    <EmptyState
      v-else
      title="中部区域选择一个变量后编辑"
    />
  </EditorOverviewPanel>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import EditorOverviewField from '@/views/script-editor/EditorOverviewField.vue';
import EditorOverviewPanel from '@/views/script-editor/EditorOverviewPanel.vue';
import EditorOverviewSection from '@/views/script-editor/EditorOverviewSection.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import { editorInputTypeOptions, type EditorInputEntry } from '@/views/script-editor/editorVariables';

defineOptions({ name: 'EditorInputDetailsPanel' });

const props = defineProps<{
  selectedInputEntry: EditorInputEntry | null;
  selectedInputIndex: number;
}>();

defineEmits<{
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
}>();

const inputTypeOptions = editorInputTypeOptions;
const isKeyMissing = computed(() => Boolean(props.selectedInputEntry && !props.selectedInputEntry.key.trim()));
const scopeOptions = [
  { label: 'Input', value: 'input', description: '用户可配置并持久化的输入变量。' },
  { label: 'Runtime', value: 'runtime', description: '步骤执行过程中的运行时变量。' },
  { label: 'System', value: 'system', description: '运行时注入的只读系统变量。' },
];
</script>

<style scoped>
@reference "../../style.css";

.editor-detail-toggle {
  @apply flex min-h-[44px] items-center gap-3 rounded-[16px] border border-(--app-border) px-4 py-3 text-sm text-(--app-text-soft);
}

.editor-detail-note {
  @apply max-w-[38rem] rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4 text-sm leading-6 text-(--app-text-soft);
}
</style>
