<template>
  <div class="action-sequence-editor flex flex-col gap-2 p-2">
    <div
      v-if="steps.length === 0"
      class="text-center py-8 border-2 border-dashed border-base-300 rounded-lg opacity-40"
    >
      <p class="text-sm">暂无操作步骤</p>
      <button class="btn btn-xs btn-ghost mt-2" @click="$emit('add-step', 0)">+ 添加第一个步骤</button>
    </div>

    <div v-for="(step, index) in steps" :key="step.id || index" class="relative group">
      <!-- Connection Line -->
      <div
        v-if="index < steps.length - 1"
        class="absolute left-6 top-12 bottom-0 w-0.5 bg-base-300 -z-10 group-hover:bg-primary/30 transition-colors"
      ></div>

      <StepItemEditor
        :step="step"
        :is-nested="isNested"
        @update="updateStep(index, $event)"
        @remove="removeStep(index)"
        @move-up="moveStep(index, -1)"
        @move-down="moveStep(index, 1)"
        @add-after="addStep(index + 1)"
      />
    </div>

    <div class="flex justify-center pt-2" v-if="steps.length > 0">
      <button class="btn btn-circle btn-sm btn-ghost hover:btn-primary" @click="addStep(steps.length)">
        <PlusIcon class="w-4 h-4" />
      </button>
    </div>
  </div>
</template>

<script setup>
import { Plus as PlusIcon } from 'lucide-vue-next';
import StepItemEditor from './StepItemEditor.vue';

const props = defineProps({
  steps: {
    type: Array,
    required: true,
  },
  isNested: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits(['update:steps', 'add-step']);

const updateStep = (index, newData) => {
  const newSteps = [...props.steps];
  newSteps[index] = newData;
  emit('update:steps', newSteps);
};

const removeStep = (index) => {
  const newSteps = [...props.steps];
  newSteps.splice(index, 1);
  emit('update:steps', newSteps);
};

const moveStep = (index, direction) => {
  if (index + direction < 0 || index + direction >= props.steps.length) return;
  const newSteps = [...props.steps];
  const [movedItem] = newSteps.splice(index, 1);
  newSteps.splice(index + direction, 0, movedItem);
  emit('update:steps', newSteps);
};

const addStep = (atIndex) => {
  emit('add-step', atIndex);
};
</script>
