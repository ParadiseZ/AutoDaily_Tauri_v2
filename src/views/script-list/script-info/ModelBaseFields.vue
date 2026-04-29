<template>
  <div class="space-y-4">
    <div class="dialog-form-grid" :class="{ 'dialog-form-grid-compact': compact }">
      <label class="dialog-form-row" :class="{ 'dialog-form-row-compact': compact }">
        <span class="dialog-form-label">模型来源</span>
        <AppSelect v-model="model.modelSource" :options="modelSourceOptions" :test-id="resolveTestId('model-source')" />
      </label>

      <label class="dialog-form-row" :class="{ 'dialog-form-row-compact': compact }">
        <span class="dialog-form-label">推理后端</span>
        <AppSelect v-model="model.executionProvider" :options="providerOptions" :test-id="resolveTestId('execution-provider')" />
      </label>
    </div>

    <label v-if="model.modelSource === 'Custom'" class="dialog-form-row dialog-form-row-wide" :class="{ 'dialog-form-row-compact': compact }">
      <span class="dialog-form-label">模型路径</span>
      <div class="dialog-path-row">
        <input v-model.trim="model.modelPath" class="app-input" :data-testid="resolveTestId('model-path')" :placeholder="pathPlaceholder" />
        <button class="app-button app-button-ghost dialog-path-button" type="button" @click="pickModelPath">
          <AppIcon name="folder-open" :size="16" />
        </button>
      </div>
    </label>

    <div class="dialog-form-grid" :class="{ 'dialog-form-grid-compact': compact }">
      <label class="dialog-form-row" :class="{ 'dialog-form-row-compact': compact }">
        <span class="dialog-form-label">输入宽度</span>
        <input v-model.number="model.inputWidth" class="app-input" :data-testid="resolveTestId('input-width')" min="1" type="number" />
      </label>

      <label class="dialog-form-row" :class="{ 'dialog-form-row-compact': compact }">
        <span class="dialog-form-label">输入高度</span>
        <input v-model.number="model.inputHeight" class="app-input" :data-testid="resolveTestId('input-height')" min="1" type="number" />
      </label>
    </div>

    <div class="dialog-form-grid" :class="{ 'dialog-form-grid-compact': compact }">
      <label class="dialog-form-row" :class="{ 'dialog-form-row-compact': compact }">
        <span class="dialog-form-label">Intra Threads</span>
        <input v-model.number="model.intraThreadNum" class="app-input" :data-testid="resolveTestId('intra-thread-num')" min="1" type="number" />
      </label>

      <label class="dialog-form-row" :class="{ 'dialog-form-row-compact': compact }">
        <span class="dialog-form-label">Inter Threads</span>
        <input v-model.number="model.interThreadNum" class="app-input" :data-testid="resolveTestId('inter-thread-num')" min="1" type="number" />
      </label>
    </div>

    <div class="dialog-form-grid" :class="{ 'dialog-form-grid-compact': compact }">
      <label v-if="!compact" class="dialog-form-row">
        <span class="dialog-form-label">Intra Spinning</span>
        <span class="dialog-form-inline-toggle">
          <input
            v-model="model.intraSpinning"
            type="checkbox"
            class="h-4 w-4"
            :data-testid="resolveTestId('intra-spinning')"
            style="accent-color: var(--app-accent)"
          />
          <span class="text-sm text-(--app-text-soft)">启用后同一模型内部线程保持活跃。</span>
        </span>
      </label>
      <label v-else class="dialog-toggle-card">
        <span class="dialog-toggle-head">
          <span class="dialog-form-label">Intra Spinning</span>
          <input
            v-model="model.intraSpinning"
            type="checkbox"
            class="h-4 w-4"
            :data-testid="resolveTestId('intra-spinning')"
            style="accent-color: var(--app-accent)"
          />
        </span>
        <span class="dialog-toggle-text">保持模型内部线程活跃。</span>
      </label>

      <label v-if="!compact" class="dialog-form-row">
        <span class="dialog-form-label">Inter Spinning</span>
        <span class="dialog-form-inline-toggle">
          <input
            v-model="model.interSpinning"
            type="checkbox"
            class="h-4 w-4"
            :data-testid="resolveTestId('inter-spinning')"
            style="accent-color: var(--app-accent)"
          />
          <span class="text-sm text-(--app-text-soft)">启用后多模型之间的线程保持活跃。</span>
        </span>
      </label>
      <label v-else class="dialog-toggle-card">
        <span class="dialog-toggle-head">
          <span class="dialog-form-label">Inter Spinning</span>
          <input
            v-model="model.interSpinning"
            type="checkbox"
            class="h-4 w-4"
            :data-testid="resolveTestId('inter-spinning')"
            style="accent-color: var(--app-accent)"
          />
        </span>
        <span class="dialog-toggle-text">保持模型间线程活跃。</span>
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import type { BaseModel } from '@/types/bindings/BaseModel';

const props = withDefaults(defineProps<{
  model: BaseModel;
  pathPlaceholder: string;
  testIdPrefix?: string;
  builtInEnabled?: boolean;
  compact?: boolean;
}>(), {
  builtInEnabled: true,
  compact: false,
});

const resolveTestId = (suffix: string) =>
  props.testIdPrefix ? `${props.testIdPrefix}-${suffix}` : undefined;

const modelSourceOptions = computed(() =>
  props.builtInEnabled
    ? [
        { label: '内置', value: 'BuiltIn', description: '由客户端按脚本和内置资源解析。' },
        { label: '自定义', value: 'Custom', description: '手动指定本地模型文件。' },
      ]
    : [{ label: '自定义', value: 'Custom', description: '手动指定本地模型文件。' }],
);

const providerOptions = [
  { label: 'CPU', value: 'CPU', description: '通用兼容，部署门槛最低。' },
  { label: 'DirectML', value: 'DirectML', description: '适合 Windows 显卡推理。' },
  { label: 'Cuda', value: 'Cuda', description: '适合 NVIDIA CUDA 环境。' },
];

const compact = computed(() => props.compact);

const pickModelPath = async () => {
  const value = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'ONNX Model', extensions: ['onnx'] }],
  });
  if (typeof value === 'string' && value) {
    props.model.modelPath = value;
  }
};
</script>

<style scoped>
.dialog-form-row {
  display: grid;
  gap: 0.9rem;
  align-items: center;
}

.dialog-form-grid {
  display: grid;
  gap: 1rem 1.25rem;
}

.dialog-form-label {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--app-text-strong);
}

.dialog-form-inline-toggle {
  display: inline-flex;
  align-items: center;
  gap: 0.75rem;
}

.dialog-form-row-compact {
  display: flex;
  flex-direction: column;
  align-items: stretch;
  gap: 0.45rem;
}

.dialog-toggle-card {
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
  border: 1px solid var(--app-border);
  border-radius: 16px;
  padding: 0.85rem 0.95rem;
  background: rgba(255, 255, 255, 0.48);
}

.dialog-toggle-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 0.75rem;
}

.dialog-toggle-text {
  font-size: 0.8rem;
  line-height: 1.45;
  color: var(--app-text-soft);
}

.dialog-path-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.65rem;
  align-items: center;
}

.dialog-path-button {
  min-width: 2.75rem;
  height: 2.75rem;
  padding: 0;
}

@media (min-width: 768px) {
  .dialog-form-row {
    grid-template-columns: 112px minmax(0, 1fr);
  }

  .dialog-form-grid {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  }

  .dialog-form-row-wide {
    grid-template-columns: 112px minmax(0, 1fr);
  }

  .dialog-form-grid-compact {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  }
}
</style>
