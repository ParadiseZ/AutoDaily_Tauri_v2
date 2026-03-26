<template>
  <div class="flex h-full min-h-[640px] flex-col gap-4">
    <SurfacePanel padding="sm" class="space-y-4">
      <div class="flex items-center justify-between gap-3">
        <div>
          <p class="text-xs uppercase tracking-[0.18em] text-[var(--app-text-faint)]">Status</p>
          <h2 class="text-xl font-semibold text-[var(--app-text-strong)]">检查器</h2>
        </div>
        <span
          class="rounded-full px-3 py-1 text-xs font-semibold"
          :class="
            hasErrors
              ? 'bg-red-500/12 text-red-700'
              : dirty
                ? 'bg-amber-500/12 text-amber-700'
                : 'bg-emerald-500/12 text-emerald-700'
          "
        >
          {{ hasErrors ? '存在校验错误' : dirty ? '存在未保存修改' : '已同步' }}
        </span>
      </div>

      <div class="grid gap-3 sm:grid-cols-3 xl:grid-cols-1">
        <div class="rounded-[20px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-3">
          <p class="text-xs uppercase tracking-[0.12em] text-[var(--app-text-faint)]">任务数</p>
          <p class="mt-1 text-xl font-semibold text-[var(--app-text-strong)]">{{ tasks.length }}</p>
        </div>
        <div class="rounded-[20px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-3">
          <p class="text-xs uppercase tracking-[0.12em] text-[var(--app-text-faint)]">步骤数</p>
          <p class="mt-1 text-xl font-semibold text-[var(--app-text-strong)]">
            {{ currentTask ? currentTask.data.steps.length : 0 }}
          </p>
        </div>
        <div class="rounded-[20px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-3">
          <p class="text-xs uppercase tracking-[0.12em] text-[var(--app-text-faint)]">最近保存</p>
          <p class="mt-1 text-sm font-semibold text-[var(--app-text-strong)]">{{ saveLabel }}</p>
        </div>
      </div>

      <p class="rounded-[18px] border border-[var(--app-border)] bg-white/30 px-4 py-3 text-sm leading-6 text-[var(--app-text-soft)]">
        {{ saveStatus }}
      </p>
    </SurfacePanel>

    <SurfacePanel padding="sm" class="space-y-4">
      <div>
        <p class="text-sm font-semibold text-[var(--app-text-strong)]">脚本上下文</p>
        <p class="mt-1 text-sm text-[var(--app-text-soft)]">{{ scriptName || '未选择脚本' }}</p>
      </div>

      <div class="space-y-3 text-sm text-[var(--app-text-soft)]">
        <div class="flex justify-between gap-4">
          <span>运行时</span>
          <span class="text-[var(--app-text-strong)]">{{ runtimeLabel }}</span>
        </div>
        <div class="flex justify-between gap-4">
          <span>当前任务</span>
          <span class="truncate text-[var(--app-text-strong)]">{{ currentTask?.name || '未选中' }}</span>
        </div>
        <div class="flex justify-between gap-4">
          <span>变量键</span>
          <span class="text-[var(--app-text-strong)]">{{ variableCount }}</span>
        </div>
        <div class="flex justify-between gap-4">
          <span>UI 字段</span>
          <span class="text-[var(--app-text-strong)]">{{ uiDataCount }}</span>
        </div>
      </div>
    </SurfacePanel>

    <SurfacePanel padding="sm" class="space-y-4">
      <div>
        <p class="text-sm font-semibold text-[var(--app-text-strong)]">验证结果</p>
        <p class="mt-1 text-xs text-[var(--app-text-faint)]">保存前会先检查 JSON 结构和任务完整性。</p>
      </div>

      <div v-if="validationItems.length" class="space-y-2">
        <div
          v-for="item in validationItems"
          :key="item.label"
          class="rounded-[18px] border px-4 py-3 text-sm"
          :class="item.error ? 'border-red-500/16 bg-red-500/8 text-red-700' : 'border-emerald-500/16 bg-emerald-500/8 text-emerald-700'"
        >
          <p class="font-semibold">{{ item.label }}</p>
          <p class="mt-1 text-xs leading-5">{{ item.message }}</p>
        </div>
      </div>
    </SurfacePanel>

    <SurfacePanel padding="sm" class="space-y-4">
      <div>
        <p class="text-sm font-semibold text-[var(--app-text-strong)]">当前步骤</p>
        <p class="mt-1 text-xs text-[var(--app-text-faint)]">
          选中步骤后，这里会展示摘要，方便在修改 JSON 前先做一次核对。
        </p>
      </div>

      <template v-if="selectedStep">
        <div class="rounded-[20px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
          <p class="text-sm font-semibold text-[var(--app-text-strong)]">{{ stepTitle }}</p>
          <p class="mt-2 text-sm leading-6 text-[var(--app-text-soft)]">{{ stepSummary }}</p>
        </div>
        <pre class="max-h-[320px] overflow-auto rounded-[20px] border border-[var(--app-border)] bg-slate-950 px-4 py-4 text-xs leading-6 text-slate-100">{{ stepSource }}</pre>
      </template>

      <p v-else class="rounded-[18px] border border-dashed border-[var(--app-border)] px-4 py-6 text-sm text-[var(--app-text-soft)]">
        当前还没有选中步骤。可以在中央工作区点选步骤卡片后回来查看。
      </p>
    </SurfacePanel>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import { formatDateTime, formatRuntimeLabel } from '@/utils/presenters';
import { describeStep } from '@/views/script-editor/editorStepTemplates';
import type { Step } from '@/types/bindings/Step';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';

const props = defineProps<{
  tasks: ScriptTaskTable[];
  currentTask: ScriptTaskTable | null;
  saveStatus: string;
  dirty: boolean;
  saveTime: string | null;
  scriptName: string;
  runtimeType: string | null;
  variableCount: number;
  uiDataCount: number;
  variablesError: string | null;
  uiDataError: string | null;
  stepsError: string | null;
  selectedStep: Step | null;
}>();

const hasErrors = computed(() => Boolean(props.variablesError || props.uiDataError || props.stepsError));

const saveLabel = computed(() => (props.saveTime ? formatDateTime(props.saveTime) : '未保存'));
const runtimeLabel = computed(() => formatRuntimeLabel(props.runtimeType));

const validationItems = computed(() => [
  {
    label: '变量 JSON',
    error: Boolean(props.variablesError),
    message: props.variablesError || '结构有效，可以直接保存。',
  },
  {
    label: 'UI 数据 JSON',
    error: Boolean(props.uiDataError),
    message: props.uiDataError || '结构有效，可以直接保存。',
  },
  {
    label: '步骤 JSON',
    error: Boolean(props.stepsError),
    message: props.stepsError || '数组结构有效，可以继续保存。',
  },
]);

const stepTitle = computed(() => {
  if (!props.selectedStep) {
    return '';
  }

  return props.selectedStep.label?.trim() || `步骤 · ${props.selectedStep.op}`;
});

const stepSummary = computed(() => {
  if (!props.selectedStep) {
    return '';
  }

  return describeStep(props.selectedStep);
});

const stepSource = computed(() =>
  props.selectedStep
    ? JSON.stringify(
        props.selectedStep,
        (_key, value) => (typeof value === 'bigint' ? Number(value) : value),
        2,
      )
    : '',
);
</script>
