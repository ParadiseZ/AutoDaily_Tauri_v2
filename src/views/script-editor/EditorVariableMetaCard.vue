<template>
  <div class="rounded-[18px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
    <div>
      <p class="text-sm font-semibold text-(--app-text-strong)">变量详情</p>
      <p class="mt-1 text-xs text-(--app-text-faint)">{{ displayName }}</p>
      <p v-if="displayKey && displayKey !== displayName" class="mt-1 text-[11px] text-(--app-text-faint)">键：{{ displayKey }}</p>
    </div>

    <div class="mt-4 detail-grid">
      <label v-if="editableEntry" class="detail-item">
        <span class="detail-label">名称</span>
        <input :value="editableEntry.name" class="app-input" placeholder="例如：扫荡次数" @input="emitUpdate('name', ($event.target as HTMLInputElement).value)" />
      </label>
      <div v-else class="detail-item">
        <span class="detail-label">名称</span>
        <div class="rounded-[16px] border border-(--app-border) bg-white/45 px-4 py-3 text-sm text-(--app-text-strong)">{{ displayName }}</div>
      </div>

      <label v-if="editableEntry" class="detail-item">
        <span class="detail-label">键</span>
        <input :value="editableEntry.key" class="app-input" placeholder="例如：activitySweepCount" @input="emitUpdate('key', ($event.target as HTMLInputElement).value)" />
      </label>
      <div v-else class="detail-item">
        <span class="detail-label">键</span>
        <div class="rounded-[16px] border border-(--app-border) bg-white/45 px-4 py-3 text-sm text-(--app-text-strong) break-all">
          {{ displayKey || '未设置键' }}
        </div>
      </div>

      <div v-if="editableEntry" class="detail-item">
        <span class="detail-label">类型</span>
        <EditorSelectField :model-value="editableEntry.type" :options="inputTypeOptions" placeholder="选择类型" @update:model-value="emitUpdate('type', String($event || 'int'))" />
      </div>
      <div v-else class="detail-item">
        <span class="detail-label">类型</span>
        <div class="rounded-[16px] border border-(--app-border) bg-white/45 px-4 py-3 text-sm text-(--app-text-strong)">
          {{ typeLabel }}
        </div>
      </div>

      <div v-if="editableEntry" class="detail-item">
        <span class="detail-label">作用域</span>
        <EditorSelectField :model-value="editableEntry.namespace" :options="scopeOptions" placeholder="选择作用域" @update:model-value="emitUpdate('namespace', String($event || 'input'))" />
      </div>
      <div v-else class="detail-item">
        <span class="detail-label">作用域</span>
        <div class="rounded-[16px] border border-(--app-border) bg-white/45 px-4 py-3 text-sm text-(--app-text-strong)">
          {{ namespaceLabel }}
        </div>
      </div>

      <label v-if="editableEntry" class="detail-item">
        <span class="detail-label">备注</span>
        <input
          :value="editableEntry.description"
          class="app-input"
          placeholder="用于后续检索、绑定和变量引用"
          @input="emitUpdate('description', ($event.target as HTMLInputElement).value)"
        />
      </label>
      <div v-else class="detail-item">
        <span class="detail-label">备注</span>
        <div class="rounded-[16px] border border-(--app-border) bg-white/45 px-4 py-3 text-sm text-(--app-text-soft)">
          {{ displayDescription }}
        </div>
      </div>

      <template v-if="effectiveNamespace === 'input'">
        <label v-if="editableEntry?.type === 'bool'" class="detail-item">
          <span class="detail-label">默认值</span>
          <span class="flex min-h-[44px] items-center gap-3 rounded-[16px] border border-(--app-border) px-4 py-3 text-sm text-(--app-text-soft)">
            <input :checked="editableEntry.booleanValue" type="checkbox" class="h-4 w-4" style="accent-color: var(--app-accent)" @change="emitUpdate('booleanValue', ($event.target as HTMLInputElement).checked)" />
            <span>默认启用</span>
          </span>
        </label>

        <label v-else-if="editableEntry" class="detail-item detail-item-top">
          <span class="detail-label">默认值</span>
          <textarea
            v-if="editableEntry.type === 'json'"
            :value="editableEntry.stringValue"
            class="app-textarea min-h-[120px]"
            spellcheck="false"
            @input="emitUpdate('stringValue', ($event.target as HTMLTextAreaElement).value)"
          />
          <input
            v-else
            :value="editableEntry.stringValue"
            class="app-input"
            :type="editableEntry.type === 'string' ? 'text' : 'number'"
            @input="emitUpdate('stringValue', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <div v-else class="detail-item detail-item-top">
          <span class="detail-label">默认值</span>
          <div class="rounded-[16px] border border-(--app-border) bg-white/45 px-4 py-3 text-sm text-(--app-text-strong) whitespace-pre-wrap break-all">
            {{ defaultValueLabel }}
          </div>
        </div>
      </template>

      <div
        v-else
        class="detail-item detail-span-2 detail-item-top rounded-[16px] border border-(--app-border) bg-white/35 px-4 py-4 text-sm leading-6 text-(--app-text-soft)"
      >
        {{ effectiveNamespace === 'runtime' ? 'Runtime 变量只定义结构和来源，不在这里设置默认值。' : 'System 变量由运行时注入，只在这里保留元数据。' }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { JsonValue } from '@/types/app/domain';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import {
  editorInputTypeOptions,
  getInputTypeLabel,
  getVariableDisplayKey,
  getVariableValueTypeLabel,
  type EditorInputEntry,
  type EditorVariableOption,
} from '@/views/script-editor/editorVariables';

const props = defineProps<{
  variable: EditorVariableOption;
  inputEntry?: EditorInputEntry | null;
  editable?: boolean;
}>();

const emit = defineEmits<{
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
}>();

const inputTypeOptions = editorInputTypeOptions;
const scopeOptions = [
  { label: 'Input', value: 'input', description: '用户可配置并持久化的输入变量。' },
  { label: 'Runtime', value: 'runtime', description: '步骤执行过程中的运行时变量。' },
  { label: 'System', value: 'system', description: '运行时注入的只读系统变量。' },
];

const editableEntry = computed(() => (props.editable && props.inputEntry ? props.inputEntry : null));
const effectiveNamespace = computed(() => editableEntry.value?.namespace ?? props.variable.namespace);

const namespaceLabel = computed(() => {
  if (effectiveNamespace.value === 'runtime') return 'Runtime';
  if (effectiveNamespace.value === 'system') return 'System';
  return 'Input';
});

const displayName = computed(() => props.inputEntry?.name || props.variable.label || '未命名变量');
const displayKey = computed(() => props.inputEntry?.key || getVariableDisplayKey(props.variable.key, props.variable.namespace));
const typeLabel = computed(() => (props.inputEntry ? getInputTypeLabel(props.inputEntry.type) : getVariableValueTypeLabel(props.variable.valueType)));
const displayDescription = computed(() => props.inputEntry?.description || props.variable.description || '无');

const stringifyDefaultValue = (value: JsonValue) => {
  if (typeof value === 'boolean') return value ? 'true' : 'false';
  if (typeof value === 'number') return String(value);
  if (typeof value === 'string') return value || '空';
  return JSON.stringify(value, null, 2);
};

const defaultValueLabel = computed(() => {
  if (props.inputEntry) {
    if (props.inputEntry.type === 'bool') {
      return props.inputEntry.booleanValue ? 'true' : 'false';
    }

    return props.inputEntry.stringValue || (props.inputEntry.type === 'string' ? '空' : '未设置');
  }

  if (props.variable.defaultValue === null) {
    return '未设置';
  }

  return stringifyDefaultValue(props.variable.defaultValue);
});

const emitUpdate = (
  field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue',
  value: string | boolean,
) => {
  if (!editableEntry.value) {
    return;
  }
  emit('update-input', editableEntry.value.id, field, value);
};
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
</style>
