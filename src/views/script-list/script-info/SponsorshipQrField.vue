<template>
  <div class="space-y-4">
    <div class="flex items-start justify-between gap-3">
      <div>
        <p class="text-sm font-medium text-[var(--app-text-strong)]">赞助二维码</p>
        <p class="mt-1 text-xs text-[var(--app-text-faint)]">支持本地图片路径、远程链接和 data URL。优先推荐本地图片文件。</p>
      </div>
      <div class="flex items-center gap-2">
        <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="pickImage">
          选择图片
        </button>
        <button
          class="app-button app-button-ghost app-toolbar-button"
          type="button"
          :disabled="!modelValue"
          @click="$emit('update:modelValue', null)"
        >
          清空
        </button>
      </div>
    </div>

    <label class="space-y-2">
      <span class="text-sm font-medium text-[var(--app-text-strong)]">图片地址</span>
      <input :value="modelValue || ''" class="app-input" maxlength="240" placeholder="data URL / 本地路径 / 远程地址" @input="handleInput" />
    </label>

    <div class="grid gap-4 lg:grid-cols-[180px_minmax(0,1fr)]">
      <div class="flex min-h-[180px] items-center justify-center overflow-hidden rounded-[20px] border border-[var(--app-border)] bg-[color-mix(in_srgb,var(--app-panel-muted)_92%,transparent)]">
        <img
          v-if="previewSrc && !previewFailed"
          :src="previewSrc"
          alt="赞助二维码预览"
          class="h-full w-full object-contain p-4"
          @error="previewFailed = true"
        />
        <div v-else class="px-5 text-center text-xs leading-6 text-[var(--app-text-faint)]">
          {{
            previewLoading
              ? '正在生成预览...'
              : modelValue
                ? '当前地址无法预览，请检查文件路径或图片格式。'
                : '选择二维码图片后，这里显示预览。'
          }}
        </div>
      </div>

      <div class="space-y-3">
        <div class="rounded-[16px] border border-[var(--app-border)] px-4 py-3 text-sm">
          <p class="text-xs text-[var(--app-text-faint)]">当前来源</p>
          <p class="mt-1 break-all text-[var(--app-text-strong)]">{{ sourceLabel }}</p>
        </div>

        <div class="rounded-[16px] border border-dashed border-[var(--app-border-strong)] px-4 py-3 text-xs leading-6 text-[var(--app-text-faint)]">
          本地路径会原样保存到脚本信息里，预览时通过 Tauri 命令转换成 base64。这样不会依赖 `asset.localhost`，也不会把图片内容直接写回脚本记录。
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { scriptService } from '@/services/scriptService';
import { showToast } from '@/utils/toast';

const props = defineProps<{
  modelValue: string | null;
}>();

const emit = defineEmits<{
  'update:modelValue': [value: string | null];
}>();

const imageExtensions = ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'gif', 'svg'];
const previewLoading = ref(false);
const previewFailed = ref(false);
const previewSrc = ref('');

const sourceLabel = computed(() => {
  if (!props.modelValue) {
    return '未设置';
  }

  if (props.modelValue.startsWith('data:')) {
    return 'Data URL';
  }

  if (props.modelValue.startsWith('http://') || props.modelValue.startsWith('https://')) {
    return '远程链接';
  }

  return '本地文件';
});

const isLocalPath = (value: string) =>
  !value.startsWith('data:') && !value.startsWith('http://') && !value.startsWith('https://');

const handleInput = (event: Event) => {
  const value = (event.target as HTMLInputElement).value.trim();
  emit('update:modelValue', value || null);
};

const pickImage = async () => {
  const value = await open({
    multiple: false,
    directory: false,
    filters: [
      {
        name: 'Images',
        extensions: imageExtensions,
      },
    ],
  });

  if (typeof value !== 'string') {
    return;
  }

  emit('update:modelValue', value);
  showToast('二维码图片已选择', 'success');
};

watch(
  () => props.modelValue,
  async (value) => {
    previewSrc.value = '';
    previewFailed.value = false;
    previewLoading.value = false;

    if (!value) {
      return;
    }

    if (!isLocalPath(value)) {
      previewSrc.value = value;
      return;
    }

    previewLoading.value = true;
    try {
      previewSrc.value = await scriptService.convertLocalImageToDataUrl(value);
    } catch {
      previewFailed.value = true;
    } finally {
      previewLoading.value = false;
    }
  },
  { immediate: true },
);
</script>
