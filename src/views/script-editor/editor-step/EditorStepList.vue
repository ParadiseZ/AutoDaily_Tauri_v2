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
      <div class="flex items-start gap-2">
        <button
          v-if="allowReorder"
          class="app-drag-handle shrink-0"
          :class="{ 'app-drag-handle-active': draggingIndex === index }"
          :data-testid="`editor-step-drag-${index}`"
          type="button"
          aria-label="拖动排序"
          @mousedown.prevent="startDrag(index)"
          @click.stop
        >
          ::
        </button>

        <button class="min-w-0 flex-1 text-left" type="button" @click="$emit('select', index)">
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
            <p v-if="describeStepMetaText(step) !== describeStep(step)" class="mt-2 text-sm leading-6 text-(--app-text-soft)">{{ describeStepMetaText(step) }}</p>
            <p v-if="nestedSummary(step)" class="mt-2 text-xs text-(--app-text-faint)">{{ nestedSummary(step) }}</p>
          </div>
        </button>

        <button
          v-if="allowRemove"
          class="app-icon-button app-crash-icon app-icon-button-sec shrink-0 self-start"
          type="button"
          @click.stop="$emit('remove', index)"
        >
          <Trash2 class="h-4 w-4" />
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
  if (step.op === STEP_OP.sequence && step.steps.length) return `动作序列 · ${step.steps.length} 个子步骤`;
  if (step.op === STEP_OP.flowControl) {
    if (step.a.type === FLOW_TYPE.if) return `Then ${step.a.then.length} · Else ${(step.a.else_steps ?? []).length}`;
    if (step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.forEach || step.a.type === FLOW_TYPE.repeat) return `嵌套 ${step.a.flow.length} 个步骤`;
  }
  if (step.op === STEP_OP.vision && (step.a.type === VISION_TYPE.visionSearch || step.a.type === VISION_TYPE.countCompare) && step.a.then_steps.length) {
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
</style>
