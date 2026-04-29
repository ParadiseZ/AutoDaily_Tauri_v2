<template>
  <div class="space-y-3">
    <article
      v-for="(step, index) in steps"
      :key="step.id ?? `${step.op}-${index}`"
      class="app-list-item editor-step-card"
      :class="{
        'app-list-item-active': selectedIndex === index,
        'editor-step-card-dragging': draggingIndex === index,
        'editor-step-card-drop-target': overIndex === index && draggingIndex !== null && draggingIndex !== index,
      }"
      :data-testid="`editor-step-card-${index}`"
      @mouseenter="handleMouseEnter(index)"
      @mouseup="handleMouseUp(index)"
    >
      <div class="grid grid-cols-[34px_36px_minmax(0,1fr)_auto] items-start gap-2">
        <button
          v-if="allowReorder"
          class="editor-step-card-handle"
          :class="{ 'editor-step-card-handle-active': draggingIndex === index }"
          :data-testid="`editor-step-drag-${index}`"
          type="button"
          aria-label="拖动排序"
          @mousedown.prevent="startDrag(index)"
          @click.stop
        >
          ::
        </button>

        <button class="editor-step-order" type="button" @click="$emit('select', index)">
          {{ index + 1 }}
        </button>

        <button class="min-w-0 text-left" type="button" @click="$emit('select', index)">
          <div class="min-w-0">
            <div class="flex flex-wrap items-center gap-2">
              <AppIcon name="node" :size="16" class="text-(--app-vibrant-blue) opacity-80 shrink-0" />
              <p class="truncate text-sm font-semibold text-(--app-text-strong)">
                {{ describeStep(step) }}
              </p>
              <span class="rounded-full border border-(--app-border) bg-(--app-panel-muted) px-2 py-0.5 text-[11px] font-medium text-(--app-text-soft)">
                {{ step.op }}
              </span>
            </div>
            <p v-if="describeStepMeta(step) !== describeStep(step)" class="mt-2 text-sm leading-6 text-(--app-text-soft)">{{ describeStepMeta(step) }}</p>
            <p v-if="nestedSummary(step)" class="mt-2 text-xs text-(--app-text-faint)">{{ nestedSummary(step) }}</p>
          </div>
        </button>

        <button
          v-if="allowRemove"
          class="app-button app-button-danger app-toolbar-button shrink-0"
          type="button"
          @click.stop="$emit('remove', index)"
        >
          删除
        </button>
      </div>
    </article>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import type { Step } from '@/types/bindings/Step';
import { describeStep, describeStepMeta } from '@/views/script-editor/editor-step/editorStepTemplates';
import { FLOW_TYPE, STEP_OP, VISION_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';

const props = withDefaults(defineProps<{
  steps: Step[];
  selectedIndex: number | null;
  allowRemove?: boolean;
  allowReorder?: boolean;
}>(), {
  allowRemove: true,
  allowReorder: true,
});

const emit = defineEmits<{
  select: [index: number];
  remove: [index: number];
  reorder: [from: number, to: number];
}>();

const draggingIndex = ref<number | null>(null);
const overIndex = ref<number | null>(null);

const resetDragState = () => {
  draggingIndex.value = null;
  overIndex.value = null;
};

const startDrag = (index: number) => {
  if (!props.allowReorder) return;
  draggingIndex.value = index;
  overIndex.value = index;
};

const handleMouseEnter = (index: number) => {
  if (!props.allowReorder) return;
  if (draggingIndex.value === null) return;
  overIndex.value = index;
};

const handleMouseUp = (index: number) => {
  if (!props.allowReorder) return;
  if (draggingIndex.value === null) return;
  if (draggingIndex.value !== index) {
    emit('reorder', draggingIndex.value, index);
  }
  resetDragState();
};

const handleWindowMouseUp = () => {
  resetDragState();
};

const nestedSummary = (step: Step) => {
  if (step.op === STEP_OP.sequence && step.steps.length) return `顺序容器 · ${step.steps.length} 个子步骤`;
  if (step.op === STEP_OP.flowControl) {
    if (step.a.type === FLOW_TYPE.if) return `Then ${step.a.then.length} · Else ${(step.a.else_steps ?? []).length}`;
    if (step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.forEach || step.a.type === FLOW_TYPE.repeat) return `嵌套 ${step.a.flow.length} 个步骤`;
  }
  if (step.op === STEP_OP.vision && step.a.type === VISION_TYPE.visionSearch && step.a.then_steps.length) {
    return `命中后 ${step.a.then_steps.length} 个步骤`;
  }
  return '';
};

onMounted(() => {
  window.addEventListener('mouseup', handleWindowMouseUp);
});

onBeforeUnmount(() => {
  window.removeEventListener('mouseup', handleWindowMouseUp);
});
</script>

<style scoped>
.editor-step-card {
  transition: border-color 0.16s ease, background 0.16s ease, transform 0.16s ease;
}

.editor-step-card-dragging {
  border-color: rgba(70, 110, 255, 0.24);
  background: rgba(70, 110, 255, 0.08);
}

.editor-step-card-drop-target {
  box-shadow: inset 0 0 0 1px rgba(70, 110, 255, 0.24);
}

.editor-step-card-handle {
  align-self: center;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 32px;
  border-radius: 12px;
  border: 1px dashed var(--app-border);
  background: rgba(255, 255, 255, 0.55);
  padding: 0.25rem 0.2rem;
  color: var(--app-text-faint);
  font-size: 0.8rem;
  font-weight: 700;
  letter-spacing: -0.08em;
  cursor: grab;
}

.editor-step-card-handle:active,
.editor-step-card-handle-active {
  cursor: grabbing;
}

.editor-step-card-handle-active {
  border-color: var(--app-state-active-border);
  background: var(--app-state-active-bg);
  color: var(--app-text-strong);
}

.editor-step-order {
  width: 36px;
  height: 36px;
  border-radius: 12px;
  border: 1px solid rgba(255, 255, 255, 0.28);
  background: rgba(255, 255, 255, 0.34);
  font-size: 0.95rem;
  font-weight: 700;
  color: var(--app-text-strong);
}
</style>
