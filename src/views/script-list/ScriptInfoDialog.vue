<template>
  <AppDialog
    :open="open"
    :title="script ? '编辑脚本信息' : '新建脚本'"
    description="先补全基础信息，再决定是否进入编辑器继续处理任务结构和逻辑。"
    width-class="max-w-2xl"
    @close="$emit('close')"
  >
    <form class="space-y-6" @submit.prevent="submit">
      <div class="grid gap-4 md:grid-cols-2">
        <label class="space-y-2 md:col-span-2">
          <span class="text-sm font-medium text-[var(--app-text-strong)]">脚本名称</span>
          <input v-model.trim="form.name" class="app-input" maxlength="40" placeholder="例如：每日清体力" />
        </label>

        <label class="space-y-2 md:col-span-2">
          <span class="text-sm font-medium text-[var(--app-text-strong)]">描述</span>
          <textarea
            v-model="form.description"
            class="app-textarea min-h-[110px]"
            maxlength="240"
            placeholder="用一两句话说明脚本作用、运行前提和适用场景。"
          />
        </label>

        <label class="space-y-2">
          <span class="text-sm font-medium text-[var(--app-text-strong)]">运行时</span>
          <AppSelect v-model="form.runtimeType" :options="runtimeOptions" />
        </label>

        <label class="space-y-2">
          <span class="text-sm font-medium text-[var(--app-text-strong)]">版本号</span>
          <input v-model.trim="form.verName" class="app-input" maxlength="20" placeholder="0.1.0" />
        </label>

        <label class="space-y-2">
          <span class="text-sm font-medium text-[var(--app-text-strong)]">包名</span>
          <input v-model.trim="form.pkgName" class="app-input" maxlength="80" placeholder="com.example.app" />
        </label>

        <label class="space-y-2">
          <span class="text-sm font-medium text-[var(--app-text-strong)]">联系方式</span>
          <input v-model.trim="form.contactInfo" class="app-input" maxlength="80" placeholder="QQ / Telegram / Email" />
        </label>

        <label class="space-y-2 md:col-span-2">
          <span class="text-sm font-medium text-[var(--app-text-strong)]">赞助链接</span>
          <input v-model.trim="form.sponsorshipUrl" class="app-input" maxlength="160" placeholder="https://..." />
        </label>
      </div>

      <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
        <input v-model="form.allowClone" type="checkbox" class="h-4 w-4" style="accent-color: var(--app-accent)" />
        <span class="flex-1">
          <span class="block text-sm font-medium text-[var(--app-text-strong)]">允许克隆</span>
          <span class="block text-xs text-[var(--app-text-faint)]">关闭后，其他用户只能查看云端版本，不能直接复制为本地脚本。</span>
        </span>
      </label>

      <div class="flex justify-end gap-3">
        <button class="app-button app-button-ghost" type="button" @click="$emit('close')">取消</button>
        <button class="app-button app-button-primary" type="submit" :disabled="!form.name.trim()">
          {{ script ? '保存信息' : '创建脚本' }}
        </button>
      </div>
    </form>
  </AppDialog>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import AppDialog from '@/components/shared/AppDialog.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import type { ScriptInfoDraft, ScriptTableRecord } from '@/types/app/domain';

const props = defineProps<{
  open: boolean;
  script: ScriptTableRecord | null;
}>();

const emit = defineEmits<{
  close: [];
  save: [draft: ScriptInfoDraft];
}>();

const createDefaultDraft = (): ScriptInfoDraft => ({
  name: '',
  description: '',
  pkgName: '',
  runtimeType: 'rhai',
  verName: '0.1.0',
  allowClone: true,
  contactInfo: '',
  sponsorshipUrl: '',
  sponsorshipQr: '',
});

const form = reactive<ScriptInfoDraft>(createDefaultDraft());

const runtimeOptions = [
  { label: 'Rhai', value: 'rhai', description: '适合轻量流程和原生脚本。' },
  { label: 'JavaScript', value: 'javaScript', description: '适合复杂逻辑和扩展工具链。' },
  { label: 'Lua', value: 'lua', description: '适合轻量脚本运行时。' },
  { label: 'AI + Vision', value: 'aIAndVision', description: '适合依赖识别与视觉分析的脚本。' },
  { label: 'AI', value: 'aI', description: '适合纯 AI 推理流程。' },
];

const syncForm = () => {
  const next = props.script
    ? {
        name: props.script.data.name,
        description: props.script.data.description || '',
        pkgName: props.script.data.pkgName || '',
        runtimeType: props.script.data.runtimeType,
        verName: props.script.data.verName,
        allowClone: props.script.data.allowClone,
        contactInfo: props.script.data.contactInfo || '',
        sponsorshipUrl: props.script.data.sponsorshipUrl || '',
        sponsorshipQr: props.script.data.sponsorshipQr || '',
      }
    : createDefaultDraft();

  Object.assign(form, next);
};

const submit = () => {
  emit('save', { ...form, name: form.name.trim() });
};

watch(
  () => [props.open, props.script?.id],
  ([open]) => {
    if (open) {
      syncForm();
    }
  },
  { immediate: true },
);
</script>
