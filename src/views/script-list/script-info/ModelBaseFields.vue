<template>
  <div class="grid gap-4 md:grid-cols-2">
    <label class="space-y-2">
      <span class="text-sm font-medium text-[var(--app-text-strong)]">模型来源</span>
      <AppSelect v-model="model.modelSource" :options="modelSourceOptions" />
    </label>

    <label class="space-y-2">
      <span class="text-sm font-medium text-[var(--app-text-strong)]">推理后端</span>
      <AppSelect v-model="model.executionProvider" :options="providerOptions" />
    </label>

    <label v-if="model.modelSource === 'Custom'" class="space-y-2 md:col-span-2">
      <span class="text-sm font-medium text-[var(--app-text-strong)]">模型路径</span>
      <input v-model.trim="model.modelPath" class="app-input" :placeholder="pathPlaceholder" />
    </label>

    <label class="space-y-2">
      <span class="text-sm font-medium text-[var(--app-text-strong)]">输入宽度</span>
      <input v-model.number="model.inputWidth" class="app-input" min="1" type="number" />
    </label>

    <label class="space-y-2">
      <span class="text-sm font-medium text-[var(--app-text-strong)]">输入高度</span>
      <input v-model.number="model.inputHeight" class="app-input" min="1" type="number" />
    </label>

    <label class="space-y-2">
      <span class="text-sm font-medium text-[var(--app-text-strong)]">Intra Threads</span>
      <input v-model.number="model.intraThreadNum" class="app-input" min="1" type="number" />
    </label>

    <label class="space-y-2">
      <span class="text-sm font-medium text-[var(--app-text-strong)]">Inter Threads</span>
      <input v-model.number="model.interThreadNum" class="app-input" min="1" type="number" />
    </label>

    <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
      <input v-model="model.intraSpinning" type="checkbox" class="h-4 w-4" style="accent-color: var(--app-accent)" />
      <span>
        <span class="block text-sm font-medium text-[var(--app-text-strong)]">Intra Spinning</span>
        <span class="block text-xs text-[var(--app-text-faint)]">启用后同一模型内部线程保持活跃。</span>
      </span>
    </label>

    <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
      <input v-model="model.interSpinning" type="checkbox" class="h-4 w-4" style="accent-color: var(--app-accent)" />
      <span>
        <span class="block text-sm font-medium text-[var(--app-text-strong)]">Inter Spinning</span>
        <span class="block text-xs text-[var(--app-text-faint)]">启用后多模型之间的线程保持活跃。</span>
      </span>
    </label>
  </div>
</template>

<script setup lang="ts">
import AppSelect from '@/components/shared/AppSelect.vue';
import type { BaseModel } from '@/types/bindings/BaseModel';

defineProps<{
  model: BaseModel;
  pathPlaceholder: string;
}>();

const modelSourceOptions = [
  { label: '内置', value: 'BuiltIn', description: '由客户端按脚本和内置资源解析。' },
  { label: '自定义', value: 'Custom', description: '手动指定本地模型文件。' },
];

const providerOptions = [
  { label: 'CPU', value: 'CPU', description: '通用兼容，部署门槛最低。' },
  { label: 'DirectML', value: 'DirectML', description: '适合 Windows 显卡推理。' },
  { label: 'Cuda', value: 'Cuda', description: '适合 NVIDIA CUDA 环境。' },
];
</script>
