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

    <div v-for="(step, index) in steps" :key="step.id || index" class="relative group">
      <!-- Connection Line -->
      <div
        v-if="index < steps.length - 1"
        class="absolute left-7 top-0 bottom-0 w-0.5 bg-linear-to-b from-base-300 via-base-300/50 to-transparent -z-10 group-hover:from-primary/30 transition-all duration-2000"
      ></div>

      <StepItemEditor
        :step="step"
        :is-nested="isNested"
        @update="updateStep(index, $event)"
        @remove="removeStep(index)"
        @move-up="moveStep(index, -1)"
        @move-down="moveStep(index, 1)"
      />
    </div>

    <!-- Add Step Picker Dropdown -->
    <div class="flex flex-col items-center pt-8 relative pb-12">
      <!-- Teleport popup to body to escape overflow-hidden ancestors -->
      <Teleport to="body">
        <div
          v-if="showPicker"
          class="fixed z-9999 p-4 bg-base-100 border border-base-300 shadow-2xl rounded-3xl w-80 backdrop-blur-xl animate-in fade-in zoom-in slide-in-from-bottom-4 duration-300"
          :style="pickerStyle"
        >
          <div class="text-[10px] font-bold uppercase tracking-widest opacity-30 mb-2 text-center">选择行为</div>

          <div v-for="(group, gName) in groupedActions" :key="gName" class="mb-2 last:mb-0">
            <div class="text-[9px] font-bold opacity-30 uppercase mb-2 pl-2">{{ gName }}</div>
            <div class="grid grid-cols-2 gap-1.5">
              <button
                v-for="kind in group"
                :key="kind.op"
                class="btn btn-sm h-10 px-3 bg-base-200/50 border-none hover:bg-primary hover:text-white justify-start gap-2 rounded-xl group/btn transition-all duration-300"
                @click="addStepWithType(kind.op)"
              >
                <IconRenderer
                  :icon="getIcon(kind.op)"
                  class="w-3.5 h-3.5 group-hover/btn:scale-110 transition-transform"
                />
                <span class="text-[10px] font-bold">{{ kind.name }}</span>
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
import { Plus as PlusIcon, ListTodo as ListTodoIcon } from 'lucide-vue-next';
import StepItemEditor from './StepItemEditor.vue';
import IconRenderer from '../IconRenderer.vue';
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

const togglePicker = () => {
  if (!showPicker.value && addBtnRef.value) {
    const rect = addBtnRef.value.getBoundingClientRect();
    pickerPos.top = rect.top - 8;
    pickerPos.left = rect.left + rect.width / 2 - 160;
  }
  showPicker.value = !showPicker.value;
};

const pickerStyle = computed(() => ({
  top: 'auto',
  bottom: `${window.innerHeight - pickerPos.top}px`,
  left: `${Math.max(8, pickerPos.left)}px`,
}));

const getIcon = (op: string) => {
  const map: Record<string, string> = {
    ClickAction: 'cursor',
    SwipePoint: 'move',
    WaitMs: 'clock',
    VisionSearch: 'zap',
    Ocr: 'type',
    If: 'branch',
    While: 'repeat',
    Sequence: 'layers',
    SetVar: 'variable',
    GetVar: 'terminal',
    SetState: 'settings',
  };
  return map[op] || 'box';
};

const groupedActions = {
  action: [
    { name: '点击操作', op: 'ClickAction' },
    { name: '滑动手势', op: 'SwipePoint' },
    { name: '毫秒等待', op: 'WaitMs' },
  ],
  vision: [
    { name: '视觉搜索', op: 'VisionSearch' },
    { name: 'OCR文字', op: 'Ocr' },
  ],
  flowControl: [
    { name: '判断 (If)', op: 'If' },
    { name: '循环 (While)', op: 'While' },
    { name: '序列 (Seq)', op: 'Sequence' },
  ],
  varSet: [
    { name: '变量赋值', op: 'SetVar' },
    { name: '变量获取', op: 'GetVar' },
    { name: '状态通知', op: 'SetState' },
  ],
};

const addStepWithType = (op: string) => {
  const newSteps = [...props.steps];
  const newStep: any = {
    op,
    label: '',
    skipFlag: false,
    execMax: 0,
    ...(op === 'WaitMs' ? { ms: 1000 } : {}),
    ...(op === 'If' || op === 'While'
      ? {
          cond: { type: 'Group', op: 'And', scope: 'Global', items: [] },
          steps: [],
        }
      : {}),
    ...(op === 'Sequence' ? { steps: [], reverse: false } : {}),
    ...(op === 'ClickAction' ? { Point: { x: 0, y: 0 } } : {}),
    ...(op === 'SwipePoint' ? { from: { x: 0, y: 0 }, to: { x: 0, y: 0 } } : {}),
    ...(op === 'VisionSearch'
      ? { rule: { type: 'Group', op: 'And', scope: 'Global', items: [] }, output_var: 'search_result' }
      : {}),
    ...(op === 'SetVar' ? { name: '', value_expr: '' } : {}),
    ...(op === 'GetVar' ? { name: '' } : {}),
    ...(op === 'SetState' ? { target: { type: 'Policy', id: '' }, status: { type: 'Skip', value: false } } : {}),
  };
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

const moveStep = (index: number, direction: number) => {
  const target = index + direction;
  if (target < 0 || target >= props.steps.length) return;

  const newSteps = [...props.steps];
  const item = newSteps[index];
  newSteps[index] = newSteps[target];
  newSteps[target] = item;

  emit('update:steps', newSteps);
};
</script>
