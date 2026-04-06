<template>
  <div class="grid gap-3 xl:grid-cols-2">
    <div class="space-y-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
      <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标</p>
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

        <div class="editor-inline-label">目标资源</div>
        <div class="xl:col-span-3 space-y-3">
          <EditorSelectField
            :model-value="selectedTaskControl.target.id || null"
            :options="resolvedTargetOptions"
            placeholder="选择任务或策略"
            test-id="editor-task-control-target"
            @update:model-value="emit('update-target-id', String($event || ''))"
          />
          <div class="flex flex-wrap gap-2">
            <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="createTargetReference">
              <AppIcon name="plus" :size="14" />
              新建{{ selectedTaskControl.target.type === STATE_TARGET_TYPE.task ? '任务' : '策略' }}
            </button>
            <button
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              :disabled="!selectedTaskControl.target.id"
              @click="jumpToTargetReference"
            >
              <AppIcon name="locate-fixed" :size="14" />
              定位编辑
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="space-y-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
      <p class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">状态</p>
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
      <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] bg-white/45 px-4 py-3">
        <input
          :checked="Boolean(selectedTaskControl.status.value)"
          type="checkbox"
          class="h-4 w-4"
          style="accent-color: var(--app-accent)"
          @change="$emit('update-status-value', ($event.target as HTMLInputElement).checked)"
        />
        <span class="text-sm text-[var(--app-text-soft)]">状态值为真</span>
      </label>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import type { TaskControl } from '@/types/bindings/TaskControl';
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
  'update-target-id': [value: string];
  'update-status-type': [value: string];
  'update-status-value': [value: boolean];
}>();

const resolvedTargetOptions = computed(() =>
  withResolvedReferenceOption(
    props.selectedTaskControl.target.type === STATE_TARGET_TYPE.task ? props.taskReferenceOptions : props.policyReferenceOptions,
    props.selectedTaskControl.target.id,
    props.selectedTaskControl.target.type === STATE_TARGET_TYPE.task ? 'task' : 'policy',
  ),
);

const createTargetReference = async () => {
  const kind: EditorReferenceKind = props.selectedTaskControl.target.type === STATE_TARGET_TYPE.task ? 'task' : 'policy';
  const id = await props.createReference(kind);
  emit('update-target-id', id);
};

const jumpToTargetReference = () => {
  if (!props.selectedTaskControl.target.id) {
    return;
  }

  props.jumpToReference(
    props.selectedTaskControl.target.type === STATE_TARGET_TYPE.task ? 'task' : 'policy',
    props.selectedTaskControl.target.id,
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
</style>
