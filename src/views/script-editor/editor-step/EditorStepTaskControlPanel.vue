<template>
  <div class="grid gap-3 xl:grid-cols-2">
    <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-white/35 px-4 py-4">
      <p class="text-[11px] uppercase tracking-[0.12em] text-(--app-text-faint)">目标</p>
      <div class="grid gap-3 xl:grid-cols-[78px_minmax(0,1fr)_78px_minmax(0,1fr)] xl:items-center">
        <div class="editor-inline-label">动作类型</div>
        <div>
          <EditorSelectField
            :model-value="selectedTaskControl.type"
            :options="taskControlTypeOptions"
            placeholder="动作类型"
            @update:model-value="$emit('update-type', String($event || TASK_CONTROL_TYPE.setState))"
          />
        </div>

        <div class="editor-inline-label">目标类型</div>
        <div>
          <EditorSelectField
            :model-value="selectedTaskControl.target.type"
            :options="stateTargetTypeOptions"
            placeholder="目标类型"
            @update:model-value="$emit('update-target-type', String($event || STATE_TARGET_TYPE.task))"
          />
        </div>

        <div class="editor-inline-label">添加目标</div>
        <div class="xl:col-span-3 space-y-3">
          <EditorSelectField
            :model-value="null"
            :options="resolvedTargetOptions"
            placeholder="搜索任务或策略后添加"
            test-id="editor-task-control-target"
            @update:model-value="addTarget(String($event || ''))"
          />
          <div v-if="selectedTargets.length" class="editor-target-list">
            <div v-for="target in selectedTargetOptions" :key="target.value" class="editor-target-chip">
              <span class="min-w-0 flex-1 truncate">{{ target.label }}</span>
              <button class="editor-target-remove" type="button" @click="emit('remove-target-id', target.value)">
                <AppIcon name="x" :size="13" />
              </button>
            </div>
          </div>
          <div v-else class="rounded-[12px] border border-dashed border-(--app-border) px-3 py-2 text-xs text-(--app-text-faint)">
            尚未选择目标。
          </div>
          <div class="flex flex-wrap gap-2">
            <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="createTargetReference">
              <AppIcon name="plus" :size="14" />
              新建{{ selectedTaskControl.target.type === STATE_TARGET_TYPE.task ? '任务' : '策略' }}
            </button>
            <button
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              :disabled="selectedTargets.length !== 1"
              @click="jumpToTargetReference"
            >
              <AppIcon name="locate-fixed" :size="14" />
              定位编辑
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-white/35 px-4 py-4">
      <p class="text-[11px] uppercase tracking-[0.12em] text-(--app-text-faint)">状态</p>
      <div class="grid gap-3 xl:grid-cols-[78px_minmax(0,1fr)] xl:items-center">
        <div class="editor-inline-label">状态类型</div>
        <div>
          <EditorSelectField
            :model-value="selectedTaskControl.status.type"
            :options="stateStatusTypeOptions"
            placeholder="状态类型"
            @update:model-value="$emit('update-status-type', String($event || STATE_STATUS_TYPE.done))"
          />
        </div>
      </div>
      <label class="flex items-center gap-3 rounded-[16px] border border-(--app-border) bg-white/45 px-4 py-3">
        <input
          :checked="Boolean(selectedTaskControl.status.value)"
          type="checkbox"
          class="h-4 w-4"
          style="accent-color: var(--app-accent)"
          @change="$emit('update-status-value', ($event.target as HTMLInputElement).checked)"
        />
        <span class="text-sm text-(--app-text-soft)">状态值为真</span>
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import type { TaskControl } from '@/types/bindings/TaskControl';
import type { StateTarget } from '@/types/bindings/StateTarget';
import type { EditorReferenceKind, EditorReferenceOption } from '@/views/script-editor/editorReferences';
import { withResolvedReferenceOption } from '@/views/script-editor/editorReferences';
import { STATE_STATUS_TYPE, STATE_TARGET_TYPE, TASK_CONTROL_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';

defineOptions({ name: 'EditorStepTaskControlPanel' });

const props = defineProps<{
  selectedTaskControl: TaskControl;
  taskControlTypeOptions: Array<{ label: string; value: string; description: string }>;
  stateTargetTypeOptions: Array<{ label: string; value: string; description: string }>;
  stateStatusTypeOptions: Array<{ label: string; value: string; description: string }>;
  taskReferenceOptions: EditorReferenceOption[];
  policyReferenceOptions: EditorReferenceOption[];
  createReference: (kind: EditorReferenceKind) => Promise<string>;
  jumpToReference: (kind: EditorReferenceKind, id: string) => void;
}>();

const emit = defineEmits<{
  'update-type': [value: string];
  'update-target-type': [value: string];
  'add-target-id': [value: string];
  'remove-target-id': [value: string];
  'update-status-type': [value: string];
  'update-status-value': [value: boolean];
}>();

const selectedTargets = computed<StateTarget[]>(() =>
  props.selectedTaskControl.targets?.length
    ? props.selectedTaskControl.targets
    : props.selectedTaskControl.target.id
      ? [props.selectedTaskControl.target]
      : [],
);
const resolvedTargetOptions = computed(() =>
  withResolvedReferenceOption(
    props.selectedTaskControl.target.type === STATE_TARGET_TYPE.task ? props.taskReferenceOptions : props.policyReferenceOptions,
    null,
    props.selectedTaskControl.target.type === STATE_TARGET_TYPE.task ? 'task' : 'policy',
  ).filter((option) => !selectedTargets.value.some((target) => target.id === option.value)),
);
const selectedTargetOptions = computed(() => {
  const sourceOptions = props.selectedTaskControl.target.type === STATE_TARGET_TYPE.task ? props.taskReferenceOptions : props.policyReferenceOptions;
  return selectedTargets.value.map((target) => {
    const resolved = sourceOptions.find((option) => option.value === target.id);
    return {
      label: resolved?.label ?? `未解析${target.type === STATE_TARGET_TYPE.task ? '任务' : '策略'} ${target.id}`,
      value: target.id,
    };
  });
});

const addTarget = (id: string) => {
  if (!id) return;
  emit('add-target-id', id);
};

const createTargetReference = async () => {
  const kind: EditorReferenceKind = props.selectedTaskControl.target.type === STATE_TARGET_TYPE.task ? 'task' : 'policy';
  const id = await props.createReference(kind);
  emit('add-target-id', id);
};

const jumpToTargetReference = () => {
  const target = selectedTargets.value[0];
  if (!target || selectedTargets.value.length !== 1) {
    return;
  }

  props.jumpToReference(
    target.type === STATE_TARGET_TYPE.task ? 'task' : 'policy',
    target.id,
  );
};
</script>

<style scoped>
.editor-inline-label {
  color: var(--app-text-faint);
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.editor-target-list {
  display: flex;
  flex-wrap: wrap;
  gap: 0.5rem;
}

.editor-target-chip {
  display: inline-flex;
  min-width: 0;
  max-width: 100%;
  align-items: center;
  gap: 0.45rem;
  border: 1px solid var(--app-border);
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.62);
  padding: 0.35rem 0.45rem 0.35rem 0.7rem;
  color: var(--app-text-soft);
  font-size: 0.82rem;
}

.editor-target-remove {
  display: inline-flex;
  height: 1.35rem;
  width: 1.35rem;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  color: var(--app-text-faint);
}

.editor-target-remove:hover {
  background: rgba(15, 23, 42, 0.08);
  color: var(--app-text);
}
</style>
