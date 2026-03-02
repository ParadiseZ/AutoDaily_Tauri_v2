<template>
  <div class="action-sequence-editor flex flex-col gap-4 p-2 relative">
    <div
      v-if="steps.length === 0"
      class="text-center py-16 bg-base-200/20 border-2 border-dashed border-base-300 rounded-4xl flex flex-col items-center justify-center"
    >
      <div
        class="w-16 h-16 rounded-3xl bg-base-200 flex items-center justify-center mb-4 text-base-content/20 shadow-inner"
      >
        <ListTodoIcon class="w-8 h-8" />
      </div>
      <p class="text-sm font-bold opacity-30 text-balance px-10">该序列为空，请使用下方的神奇按钮添加第一个步骤 🪄</p>
    </div>

    <!-- Steps List with Pointer Drag -->
    <div ref="stepsContainerRef" class="flex flex-col gap-4 select-none">
      <div
        v-for="(step, index) in steps"
        :key="step.id || index"
        class="relative group transition-colors"
        :class="[
          dragOverIndex === index && draggingIndex !== index ? 'ring-2 ring-primary/40 rounded-3xl' : '',
          draggingIndex === index ? 'opacity-30' : '',
        ]"
      >
        <!-- Connection Line -->
        <div
          v-if="index < steps.length - 1"
          class="absolute left-7 top-0 bottom-0 w-0.5 bg-linear-to-b from-base-300 via-base-300/50 to-transparent -z-10 group-hover:from-primary/30 transition-all duration-2000"
        ></div>

        <div class="flex items-start gap-1">
          <!-- Drag Handle -->
          <div
            class="cursor-grab active:cursor-grabbing opacity-20 hover:opacity-60 transition-opacity pt-5 touch-none shrink-0"
            @pointerdown="onPointerDown(index, $event)"
          >
            <GripVerticalIcon class="w-4 h-4" />
          </div>

          <!-- Step Editor -->
          <div class="flex-1 min-w-0">
            <StepItemEditor
              :step="step"
              :is-nested="isNested"
              @update="updateStep(index, $event)"
              @remove="removeStep(index)"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Add Step Picker Dropdown -->
    <div class="flex flex-col items-center pt-8 relative pb-12">
      <!-- Teleport popup to body to escape overflow-hidden ancestors -->
      <Teleport to="body">
        <div
          v-if="showPicker"
          class="fixed z-9999 p-4 bg-base-100 border border-base-300 shadow-2xl rounded-3xl backdrop-blur-xl animate-in fade-in zoom-in slide-in-from-bottom-4 duration-300"
          :style="pickerStyle"
        >
          <div class="text-[10px] font-bold uppercase tracking-widest opacity-30 mb-2 text-center">选择行为</div>

          <div v-for="(group, gName) in groupedActions" :key="gName" class="mb-2 last:mb-0">
            <div class="text-[9px] font-bold opacity-30 uppercase mb-2 pl-2">{{ gName }}</div>
            <div class="grid gap-1.5" :style="gridStyle(group.length)">
              <button
                v-for="kind in group"
                :key="kind.op"
                class="btn btn-sm h-10 px-3 bg-base-200/50 border-none hover:bg-primary hover:text-white justify-start gap-2 rounded-xl group/btn transition-all duration-300"
                @click="addStepWithType(kind.op)"
              >
                <IconRenderer
                  :icon="NODE_TYPES[kind.op]?.icon || 'box'"
                  class="w-3.5 h-3.5 group-hover/btn:scale-110 transition-transform shrink-0"
                />
                <span class="text-xs font-bold whitespace-nowrap overflow-hidden text-ellipsis">{{ kind.name }}</span>
              </button>
            </div>
          </div>

          <div class="divider opacity-20 my-2"></div>
          <button class="btn btn-sm btn-ghost w-full rounded-2xl opacity-50" @click="showPicker = false">取消</button>
        </div>
      </Teleport>

      <button
        ref="addBtnRef"
        class="btn btn-circle btn-primary btn-lg shadow-xl hover:scale-110 active:scale-95 transition-all group/plus"
        :class="{ 'rotate-45 btn-error': showPicker }"
        @click="togglePicker"
      >
        <PlusIcon class="w-8 h-8 transition-transform" />
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, reactive } from 'vue';
import { Plus as PlusIcon, ListTodo as ListTodoIcon, GripVertical as GripVerticalIcon } from 'lucide-vue-next';
import StepItemEditor from './StepItemEditor.vue';
import IconRenderer from '../IconRenderer.vue';
import { NODE_TYPES, NODE_CATEGORIES, getNodeDefaults } from '../config';
import type { Step } from '@/types/bindings';

const props = defineProps({
  steps: {
    type: Array as () => Step[],
    default: () => [],
  },
  isNested: {
    type: Boolean,
    default: false,
  },
});

const emit = defineEmits<{
  (e: 'update:steps', steps: Step[]): void;
}>();

const showPicker = ref(false);
const addBtnRef = ref<HTMLButtonElement | null>(null);
const pickerPos = reactive({ top: 0, left: 0 });

// ─── Drag and Drop via Pointer Events ─────────────
const stepsContainerRef = ref<HTMLElement | null>(null);
const draggingIndex = ref<number | null>(null);
const dragOverIndex = ref<number | null>(null);

const getItemIndexFromPoint = (y: number): number | null => {
  if (!stepsContainerRef.value) return null;
  const children = stepsContainerRef.value.children;
  for (let i = 0; i < children.length; i++) {
    const rect = children[i].getBoundingClientRect();
    if (y >= rect.top && y <= rect.bottom) {
      return i;
    }
  }
  return null;
};

const onPointerDown = (index: number, event: PointerEvent) => {
  event.preventDefault();
  draggingIndex.value = index;

  const onPointerMove = (e: PointerEvent) => {
    const targetIndex = getItemIndexFromPoint(e.clientY);
    if (targetIndex !== null && targetIndex !== draggingIndex.value) {
      dragOverIndex.value = targetIndex;
    }
  };

  const onPointerUp = () => {
    if (draggingIndex.value !== null && dragOverIndex.value !== null && draggingIndex.value !== dragOverIndex.value) {
      const newSteps = [...props.steps];
      const [moved] = newSteps.splice(draggingIndex.value, 1);
      newSteps.splice(dragOverIndex.value, 0, moved);
      emit('update:steps', newSteps);
    }
    draggingIndex.value = null;
    dragOverIndex.value = null;
    document.removeEventListener('pointermove', onPointerMove);
    document.removeEventListener('pointerup', onPointerUp);
  };

  document.addEventListener('pointermove', onPointerMove);
  document.addEventListener('pointerup', onPointerUp);
};

// ─── Picker ───────────────────────────────────────

const togglePicker = () => {
  if (!showPicker.value && addBtnRef.value) {
    const rect = addBtnRef.value.getBoundingClientRect();
    pickerPos.top = rect.top - 8;
    pickerPos.left = rect.left + rect.width / 2;
  }
  showPicker.value = !showPicker.value;
};

const pickerStyle = computed(() => ({
  top: 'auto',
  bottom: `${window.innerHeight - pickerPos.top}px`,
  left: `${pickerPos.left}px`,
  transform: 'translateX(-50%)',
}));

/** 自适应网格：最多 5 列，不足时收缩 */
const gridStyle = (count: number) => {
  const cols = Math.min(count, 5);
  return { gridTemplateColumns: `repeat(${cols}, minmax(0, 1fr))` };
};

const groupedActions = computed(() => {
  const groups: Record<string, any[]> = {};
  for (const cat of NODE_CATEGORIES) {
    if (cat.key === 'special') continue;
    groups[cat.label] = cat.types.map((t) => ({
      name: NODE_TYPES[t]?.displayCn || t,
      op: t,
    }));
  }
  return groups;
});

const addStepWithType = (op: string) => {
  const newSteps = [...props.steps];
  const newStep = getNodeDefaults(op);
  newSteps.push(newStep as Step);
  emit('update:steps', newSteps);
  showPicker.value = false;
};

const updateStep = (index: number, newData: Step) => {
  const newSteps = [...props.steps];
  newSteps[index] = newData;
  emit('update:steps', newSteps);
};

const removeStep = (index: number) => {
  const newSteps = [...props.steps];
  newSteps.splice(index, 1);
  emit('update:steps', newSteps);
};
</script>
