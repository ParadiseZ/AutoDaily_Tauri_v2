<template>
  <div class="step-item p-3 bg-base-100 border border-base-300 rounded-xl shadow-sm hover:shadow-md transition-all">
    <!-- Header -->
    <div class="flex items-center gap-3 cursor-pointer" @click="isCollapsed = !isCollapsed">
      <div class="p-2 rounded-lg" :class="categoryColor">
        <StepIcon :type="step.op" :category="category" class-name="w-5 h-5 text-white" />
      </div>

      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <span class="font-bold text-sm truncate">{{ stepTitle }}</span>
          <span v-if="step.label" class="badge badge-sm badge-ghost opacity-60">{{ step.label }}</span>
        </div>
        <div v-if="stepSummary" class="text-[10px] opacity-50 truncate mt-0.5">{{ stepSummary }}</div>
      </div>

      <div class="flex items-center gap-1">
        <button class="btn btn-ghost btn-xs btn-circle" @click.stop="$emit('move-up')">
          <ChevronUpIcon class="w-3 h-3" />
        </button>
        <button class="btn btn-ghost btn-xs btn-circle" @click.stop="$emit('move-down')">
          <ChevronDownIcon class="w-3 h-3" />
        </button>
        <button class="btn btn-ghost btn-xs btn-circle text-error" @click.stop="$emit('remove')">
          <TrashIcon class="w-3 h-3" />
        </button>
        <div class="divider divider-horizontal mx-0 w-2"></div>
        <ChevronRightIcon class="w-4 h-4 transition-transform" :class="{ 'rotate-90': !isCollapsed }" />
      </div>
    </div>

    <!-- Content (Expanded) -->
    <div v-if="!isCollapsed" class="mt-4 pt-4 border-t border-base-200">
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Common Label/Remark -->
        <div class="form-control col-span-full">
          <label class="label py-1"
            ><span class="label-text text-xs opacity-60 font-bold uppercase">备注 / Label</span></label
          >
          <input
            type="text"
            v-model="localStep.label"
            class="input input-bordered input-sm w-full"
            placeholder="说明该步骤的作用..."
          />
        </div>

        <!-- Dynamic Fields based on OP -->
        <component :is="currentForm" :data="localStep" @update="onDataUpdate" />

        <!-- Execution Controls -->
        <div class="col-span-full mt-2 grid grid-cols-3 gap-2 p-3 bg-base-200/50 rounded-lg">
          <div class="form-control">
            <label class="label py-0"><span class="label-text text-[10px] opacity-70">最大执行次数</span></label>
            <input type="number" v-model="localStep.execMax" class="input input-bordered input-xs" />
          </div>
          <div class="flex items-center gap-2 pt-2">
            <input type="checkbox" v-model="localStep.skipFlag" class="checkbox checkbox-xs" />
            <span class="text-[10px]">禁用此步骤</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, defineAsyncComponent } from 'vue';
import {
  ChevronUp as ChevronUpIcon,
  ChevronDown as ChevronDownIcon,
  ChevronRight as ChevronRightIcon,
  Trash2 as TrashIcon,
  Plus as PlusIcon,
} from 'lucide-vue-next';
import StepIcon from './StepIcon.vue';

const props = defineProps({
  step: {
    type: Object,
    required: true,
  },
  isNested: Boolean,
});

const emit = defineEmits(['update', 'remove', 'move-up', 'move-down', 'add-after']);

const isCollapsed = ref(true);
const localStep = ref({ ...props.step });

// Step Categorization (Maps to StepKind in Rust)
const category = computed(() => {
  const op = props.step.op;
  if (['Sequence', 'If', 'While', 'ForEachActivity', 'Continue', 'Break', 'WaitUntil'].includes(op)) return 'control';
  if (['ClickAction', 'SwipePoint', 'SwipePercent', 'SwipeDet', 'SwipeTxt', 'WaitMs'].includes(op))
    return 'interaction';
  if (['VisionSearch', 'Ocr', 'DetRec', 'FindObject', 'TakeScreenshot'].includes(op)) return 'vision';
  if (['SetVar', 'GetVar', 'FilterHits', 'IncIndex', 'ResetIndex'].includes(op)) return 'logic';
  if (['SetState', 'GetState', 'StopPolicy', 'FinishTask'].includes(op)) return 'state';
  return 'other';
});

const categoryColor = computed(() => {
  const map = {
    control: 'bg-yellow-500',
    interaction: 'bg-blue-500',
    vision: 'bg-purple-500',
    logic: 'bg-orange-500',
    state: 'bg-emerald-500',
    other: 'bg-slate-500',
  };
  return map[category.value];
});

const stepTitle = computed(() => {
  const opMap = {
    ClickAction: '点击操作',
    WaitMs: '延时等待',
    If: '逻辑判断 (If)',
    While: '循环 (While)',
    SetVar: '赋值变量',
    VisionSearch: '强化视觉搜索',
    TakeScreenshot: '屏幕截图',
    Sequence: '步骤序列',
  };
  return opMap[props.step.op] || props.step.op;
});

const stepSummary = computed(() => {
  // TODO: Logic for brief summary based on data
  return '';
});

// Lazy load specific forms for each step kind
const currentForm = computed(() => {
  const op = props.step.op;
  try {
    // Expected to be implemented in StepForms/ directory later
    // return defineAsyncComponent(() => import(`./StepForms/${op}Form.vue`));
    return null; // Mocked for now
  } catch (e) {
    return null;
  }
});

const onDataUpdate = (newData) => {
  localStep.value = { ...localStep.value, ...newData };
  emit('update', localStep.value);
};

watch(
  () => props.step,
  (val) => {
    localStep.value = { ...val };
  },
  { deep: true }
);
</script>
