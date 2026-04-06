<template>
  <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
    <div>
      <p class="text-sm font-semibold text-[var(--app-text-strong)]">变量详情</p>
      <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ displayKey || '未设置键' }}</p>
    </div>

    <div class="mt-4 detail-grid">
      <div class="detail-item">
        <span class="detail-label">名称</span>
        <div class="rounded-[16px] border border-[var(--app-border)] bg-white/45 px-4 py-3 text-sm text-[var(--app-text-strong)]">
          {{ displayName }}
        </div>
      </div>

      <div class="detail-item">
        <span class="detail-label">键</span>
        <div class="rounded-[16px] border border-[var(--app-border)] bg-white/45 px-4 py-3 text-sm text-[var(--app-text-strong)] break-all">
          {{ displayKey || '未设置键' }}
        </div>
      </div>

      <div class="detail-item">
        <span class="detail-label">类型</span>
        <div class="rounded-[16px] border border-[var(--app-border)] bg-white/45 px-4 py-3 text-sm text-[var(--app-text-strong)]">
          {{ typeLabel }}
        </div>
      </div>

      <div class="detail-item">
        <span class="detail-label">作用域</span>
        <div class="rounded-[16px] border border-[var(--app-border)] bg-white/45 px-4 py-3 text-sm text-[var(--app-text-strong)]">
          {{ namespaceLabel }}
        </div>
      </div>

      <div class="detail-item">
        <span class="detail-label">备注</span>
        <div class="rounded-[16px] border border-[var(--app-border)] bg-white/45 px-4 py-3 text-sm text-[var(--app-text-soft)]">
          {{ displayDescription }}
        </div>
      </div>

      <template v-if="variable.namespace === 'input'">
        <div class="detail-item detail-item-top">
          <span class="detail-label">默认值</span>
          <div class="rounded-[16px] border border-[var(--app-border)] bg-white/45 px-4 py-3 text-sm text-[var(--app-text-strong)] whitespace-pre-wrap break-all">
            {{ defaultValueLabel }}
          </div>
        </div>
      </template>

      <div
        v-else
        class="detail-item detail-span-2 detail-item-top rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4 text-sm leading-6 text-[var(--app-text-soft)]"
      >
        {{ variable.namespace === 'runtime' ? 'Runtime 变量只定义结构和来源，不在这里设置默认值。' : 'System 变量由运行时注入，只在这里保留元数据。' }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { JsonValue } from '@/types/app/domain';
import {
  getInputTypeLabel,
  getVariableDisplayKey,
  getVariableValueTypeLabel,
  type EditorInputEntry,
  type EditorVariableOption,
} from '@/views/script-editor/editorVariables';

const props = defineProps<{
  variable: EditorVariableOption;
  inputEntry?: EditorInputEntry | null;
}>();

const namespaceLabel = computed(() => {
  if (props.variable.namespace === 'runtime') return 'Runtime';
  if (props.variable.namespace === 'system') return 'System';
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
