<template>
  <TransitionGroup tag="div" class="space-y-3" move-class="editor-step-reorder-move">
    <article
      v-for="item in previewSteps"
      :key="item.key"
      class="app-list-item editor-step-card"
      :class="{
        'app-list-item-active': selectedIndex === item.index,
        'editor-step-card-dragging': draggingIndex === item.index,
        'editor-step-card-drop-target': overIndex === item.index && draggingIndex !== null && draggingIndex !== item.index,
      }"
      :data-testid="`editor-step-card-${item.index}`"
      :data-step-index="item.index"
    >
      <div class="flex items-start gap-2">
        <button
          v-if="allowReorder"
          class="app-drag-handle shrink-0"
          :class="{ 'app-drag-handle-active': draggingIndex === item.index }"
          :data-testid="`editor-step-drag-${item.index}`"
          type="button"
          aria-label="拖动排序"
          @mousedown.prevent="startDrag(item.index, $event)"
          @click.stop
        >
          ::
        </button>

        <button class="min-w-0 flex-1 text-left" type="button" @click="$emit('select', item.index)">
          <div class="min-w-0">
            <div class="flex flex-wrap items-center gap-2">
              <AppIcon name="node" :size="16" class="text-(--app-vibrant-blue) opacity-80 shrink-0" />
              <p class="truncate text-sm font-semibold text-(--app-text-strong)">
                {{ describeStep(item.step) }}
              </p>
              <span class="rounded-full border border-(--app-border) bg-(--app-panel-muted) px-2 py-0.5 text-[11px] font-medium text-(--app-text-soft)">
                {{ item.step.op }}
              </span>
            </div>
            <p v-if="describeStepMetaText(item.step) !== describeStep(item.step)" class="mt-2 text-sm leading-6 text-(--app-text-soft)">{{ describeStepMetaText(item.step) }}</p>
            <p v-if="nestedSummary(item.step)" class="mt-2 text-xs text-(--app-text-faint)">{{ nestedSummary(item.step) }}</p>
          </div>
        </button>

        <button
          v-if="allowRemove"
          class="app-icon-button app-crash-icon app-icon-button-sec shrink-0 self-start"
          type="button"
          @click.stop="$emit('remove', item.index)"
        >
          <Trash2 class="h-4 w-4" />
        </button>
      </div>
    </article>
  </TransitionGroup>

  <Teleport to="body">
    <article v-if="draggingStep" class="editor-step-drag-overlay app-list-item" :style="dragOverlayStyle">
      <div class="flex items-start gap-2">
        <div class="app-drag-handle app-drag-handle-active shrink-0 pointer-events-none">::</div>
        <div class="min-w-0 flex-1 text-left">
          <div class="flex flex-wrap items-center gap-2">
            <AppIcon name="node" :size="16" class="text-(--app-vibrant-blue) opacity-80 shrink-0" />
            <p class="truncate text-sm font-semibold text-(--app-text-strong)">{{ describeStep(draggingStep) }}</p>
            <span class="rounded-full border border-(--app-border) bg-(--app-panel-muted) px-2 py-0.5 text-[11px] font-medium text-(--app-text-soft)">
              {{ draggingStep.op }}
            </span>
          </div>
          <p v-if="describeStepMetaText(draggingStep) !== describeStep(draggingStep)" class="mt-2 text-sm leading-6 text-(--app-text-soft)">
            {{ describeStepMetaText(draggingStep) }}
          </p>
          <p v-if="nestedSummary(draggingStep)" class="mt-2 text-xs text-(--app-text-faint)">{{ nestedSummary(draggingStep) }}</p>
        </div>
      </div>
    </article>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import type { Step } from '@/types/bindings/Step';
import { describeStep, describeStepMeta } from '@/views/script-editor/editor-step/editorStepTemplates';
import type { EditorReferenceOption } from '@/views/script-editor/editorReferences';
import { ACTION_TYPE, FLOW_TYPE, STEP_OP, TASK_CONTROL_TYPE, VISION_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';
import { Trash2 } from '@lucide/vue';

const props = withDefaults(defineProps<{
  steps: Step[];
  selectedIndex: number | null;
  allowRemove?: boolean;
  allowReorder?: boolean;
  taskReferenceOptions?: EditorReferenceOption[];
  policyReferenceOptions?: EditorReferenceOption[];
  policyGroupReferenceOptions?: EditorReferenceOption[];
  policySetReferenceOptions?: EditorReferenceOption[];
}>(), {
  allowRemove: true,
  allowReorder: true,
  taskReferenceOptions: () => [],
  policyReferenceOptions: () => [],
  policyGroupReferenceOptions: () => [],
  policySetReferenceOptions: () => [],
});

const emit = defineEmits<{
  select: [index: number];
  remove: [index: number];
  reorder: [from: number, to: number];
}>();

const draggingIndex = ref<number | null>(null);
const overIndex = ref<number | null>(null);
const dragPointer = ref({ x: 0, y: 0, width: 0, height: 0 });
const dragStartY = ref(0);
const dragTargetCenters = ref<Array<{ index: number; y: number }>>([]);

const indexedSteps = computed(() => props.steps.map((step, index) => ({ step, index, key: step.id ?? `${step.op}-${index}` })));
const previewSteps = computed(() => {
  if (draggingIndex.value === null || overIndex.value === null || draggingIndex.value === overIndex.value) {
    return indexedSteps.value;
  }
  const nextSteps = [...indexedSteps.value];
  const fromIndex = nextSteps.findIndex((item) => item.index === draggingIndex.value);
  const toIndex = nextSteps.findIndex((item) => item.index === overIndex.value);
  if (fromIndex < 0 || toIndex < 0) return indexedSteps.value;
  const [draggedStep] = nextSteps.splice(fromIndex, 1);
  if (!draggedStep) return indexedSteps.value;
  nextSteps.splice(toIndex, 0, draggedStep);
  return nextSteps;
});
const draggingStep = computed(() => (draggingIndex.value === null ? null : props.steps[draggingIndex.value] ?? null));
const dragOverlayStyle = computed(() => ({
  left: `${dragPointer.value.x}px`,
  top: `${dragPointer.value.y - dragPointer.value.height / 2}px`,
  ...(dragPointer.value.width ? { width: `${dragPointer.value.width}px` } : {}),
  ...(dragPointer.value.height ? { height: `${dragPointer.value.height}px` } : {}),
}));

const applyDraggingUi = (active: boolean) => {
  document.body.style.userSelect = active ? 'none' : '';
  document.body.style.cursor = active ? 'grabbing' : '';
};

const updateDragPointer = (
  event: MouseEvent,
  x = dragPointer.value.x,
  width = dragPointer.value.width,
  height = dragPointer.value.height,
) => {
  dragPointer.value = { x, y: event.clientY, width, height };
};

const resetDragState = () => {
  draggingIndex.value = null;
  overIndex.value = null;
  dragTargetCenters.value = [];
  applyDraggingUi(false);
};

const startDrag = (index: number, event: MouseEvent) => {
  if (!props.allowReorder) return;
  draggingIndex.value = index;
  overIndex.value = index;
  const sourceRect = (event.target as HTMLElement).closest<HTMLElement>('[data-step-index]')?.getBoundingClientRect();
  updateDragPointer(event, sourceRect?.left ?? event.clientX, sourceRect?.width ?? 0, sourceRect?.height ?? 0);
  dragStartY.value = event.clientY;
  dragTargetCenters.value = Array.from(document.querySelectorAll<HTMLElement>('[data-step-index]'))
    .filter((target) => Number(target.dataset.stepIndex) !== index)
    .map((target) => {
      const rect = target.getBoundingClientRect();
      return { index: Number(target.dataset.stepIndex), y: rect.top + rect.height / 2 };
    });
  applyDraggingUi(true);
};

const resolveTargetIndex = (event: MouseEvent) => {
  if (event.clientY > dragStartY.value) {
    return dragTargetCenters.value.filter((target) => target.y > dragStartY.value && target.y < event.clientY).at(-1)?.index ?? null;
  }
  if (event.clientY < dragStartY.value) {
    return dragTargetCenters.value.find((target) => target.y < dragStartY.value && target.y > event.clientY)?.index ?? null;
  }
  return null;
};

const handleWindowMouseMove = (event: MouseEvent) => {
  if (!props.allowReorder || draggingIndex.value === null) return;
  updateDragPointer(event);
  overIndex.value = resolveTargetIndex(event) ?? draggingIndex.value;
};

const handleWindowMouseUp = () => {
  if (props.allowReorder && draggingIndex.value !== null && overIndex.value !== null && draggingIndex.value !== overIndex.value) {
    emit('reorder', draggingIndex.value, overIndex.value);
  }
  resetDragState();
};

const nestedSummary = (step: Step) => {
  if (step.op === STEP_OP.sequence && step.steps.length) return `动作序列 · ${step.steps.length} 个子步骤`;
  if (step.op === STEP_OP.flowControl) {
    if (step.a.type === FLOW_TYPE.if) return `Then ${step.a.then.length} · Else ${(step.a.else_steps ?? []).length}`;
    if (step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.forEach || step.a.type === FLOW_TYPE.repeat) return `嵌套 ${step.a.flow.length} 个步骤`;
  }
  if (step.op === STEP_OP.vision && step.a.type === VISION_TYPE.visionSearch && step.a.then_steps.length) {
    return `命中后 ${step.a.then_steps.length} 个步骤`;
  }
  return '';
};

const resolveReferenceLabel = (
  id: string | null | undefined,
  options: EditorReferenceOption[],
  fallback = '未指定',
) => {
  const value = id?.trim();
  if (!value) {
    return fallback;
  }
  return options.find((option) => option.value === value)?.label || value;
};

const describeStepMetaText = (step: Step) => {
  if (step.op === STEP_OP.action) {
    if (step.a.ac === ACTION_TYPE.posAdd) {
      return `调整策略 ${resolveReferenceLabel(step.a.target, props.policyReferenceOptions)} · +1`;
    }
    if (step.a.ac === ACTION_TYPE.posMinus) {
      return `调整策略 ${resolveReferenceLabel(step.a.target, props.policyReferenceOptions)} · -1`;
    }
    if (step.a.ac === ACTION_TYPE.dropSetNext) {
      return `任务 ${resolveReferenceLabel(step.a.task, props.taskReferenceOptions)} · 变量 ${step.a.variable_id || '未指定'}`;
    }
  }

  if (step.op === STEP_OP.flowControl) {
    if (step.a.type === FLOW_TYPE.link) {
      return `跳转➡️[${resolveReferenceLabel(step.a.target, props.taskReferenceOptions)}]`;
    }
    if (step.a.type === FLOW_TYPE.addPolicies) {
      return `策略集 ${resolveReferenceLabel(step.a.source, props.policySetReferenceOptions)} -> ${resolveReferenceLabel(step.a.target, props.policySetReferenceOptions)}${step.a.top ? ' · 顶部插入' : ' · 底部插入'}${step.a.reverse ? ' · 逆序' : ''}`;
    }
    if (step.a.type === FLOW_TYPE.removePolicies) {
      return `移除策略集 ${resolveReferenceLabel(step.a.source, props.policySetReferenceOptions)} -> ${resolveReferenceLabel(step.a.target, props.policySetReferenceOptions)}`;
    }
    if (step.a.type === FLOW_TYPE.bindPolicyGroup) {
      return `策略组 ${resolveReferenceLabel(step.a.source, props.policyGroupReferenceOptions)} -> 策略集 ${resolveReferenceLabel(step.a.target, props.policySetReferenceOptions)}${step.a.top ? ' · 顶部插入' : ' · 底部插入'}${step.a.reverse ? ' · 逆序' : ''}`;
    }
    if (step.a.type === FLOW_TYPE.removePolicyGroup) {
      return `移除策略组 ${resolveReferenceLabel(step.a.source, props.policyGroupReferenceOptions)} -> 策略集 ${resolveReferenceLabel(step.a.target, props.policySetReferenceOptions)}`;
    }
    if (step.a.type === FLOW_TYPE.addPolicyGroups) {
      return `策略组 ${resolveReferenceLabel(step.a.source, props.policyGroupReferenceOptions)} -> 策略组 ${resolveReferenceLabel(step.a.target, props.policyGroupReferenceOptions)}${step.a.top ? ' · 顶部插入' : ' · 底部插入'}${step.a.reverse ? ' · 逆序' : ''}`;
    }
    if (step.a.type === FLOW_TYPE.unloadPolicyGroup) {
      return `卸载策略组 ${resolveReferenceLabel(step.a.source, props.policyGroupReferenceOptions)} -> 策略组 ${resolveReferenceLabel(step.a.target, props.policyGroupReferenceOptions)}`;
    }
    if (step.a.type === FLOW_TYPE.bindPolicy) {
      return `策略 ${resolveReferenceLabel(step.a.source, props.policyReferenceOptions)} -> 策略组 ${resolveReferenceLabel(step.a.target, props.policyGroupReferenceOptions)}${step.a.top ? ' · 顶部插入' : ' · 底部插入'}${step.a.reverse ? ' · 逆序' : ''}`;
    }
    if (step.a.type === FLOW_TYPE.unloadPolicy) {
      return `卸载策略 ${resolveReferenceLabel(step.a.source, props.policyReferenceOptions)} -> 策略组 ${resolveReferenceLabel(step.a.target, props.policyGroupReferenceOptions)}`;
    }
  }

  if (step.op === STEP_OP.taskControl && step.a.type === TASK_CONTROL_TYPE.setState) {
    return `设置状态 · ${step.a.target.type}:${step.a.target.type === 'task'
      ? resolveReferenceLabel(step.a.target.id, props.taskReferenceOptions)
      : resolveReferenceLabel(step.a.target.id, props.policyReferenceOptions)}`;
  }

  return describeStepMeta(step);
};

onMounted(() => {
  window.addEventListener('mousemove', handleWindowMouseMove);
  window.addEventListener('mouseup', handleWindowMouseUp);
});

onBeforeUnmount(() => {
  window.removeEventListener('mousemove', handleWindowMouseMove);
  window.removeEventListener('mouseup', handleWindowMouseUp);
  applyDraggingUi(false);
});
</script>

<style scoped>
.editor-step-card {
  transition: border-color 0.16s ease, background 0.16s ease, transform 0.16s ease;
}

.editor-step-card-dragging {
  opacity: 0;
}

.editor-step-card-drop-target {
  box-shadow: inset 0 0 0 1px rgba(70, 110, 255, 0.24);
  background: color-mix(in srgb, var(--app-state-active-bg) 84%, white);
  transform: translateX(6px);
}

.editor-step-reorder-move {
  transition: transform 180ms cubic-bezier(0.22, 1, 0.36, 1);
}

.editor-step-drag-overlay {
  position: fixed;
  z-index: 70;
  pointer-events: none;
  border-color: rgba(70, 110, 255, 0.24);
  background: color-mix(in srgb, var(--app-panel) 92%, white);
  box-shadow: 0 18px 36px rgba(15, 23, 42, 0.2);
  transform: scale(1.03);
}
</style>
