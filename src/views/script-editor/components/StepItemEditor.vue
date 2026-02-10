<template>
  <div class="step-item p-1 bg-transparent group-step hover:translate-x-1 transition-transform duration-300">
    <div
      class="bg-base-100 border border-base-300 rounded-3xl shadow-sm hover:shadow-xl transition-all overflow-hidden"
    >
      <!-- Header -->
      <div class="flex items-center gap-3 p-4 cursor-pointer select-none" @click="isExpanded = !isExpanded">
        <div
          class="w-10 h-10 rounded-2xl flex items-center justify-center transition-transform hover:scale-110 shadow-sm"
          :class="categoryColor"
        >
          <StepIcon :type="step.op" :category="category" class-name="w-5 h-5 text-white" />
        </div>

        <div class="flex-2 min-w-0">
          <div class="font-black text-sm tracking-tight">
            {{ stepTitle }}<span v-if="step.label">{{ step.label }}}</span>
          </div>
          <div class="text-[14px] font-mono opacity-60 tracking-tight mt-0.5 truncate">{{ stepSummary }}</div>
        </div>

        <div class="flex items-center gap-1">
          <div class="flex flex-col gap-0.5 mr-2">
            <button
              class="btn btn-ghost min-h-0! h-4 w-6 p-0 hover:bg-primary/10 hover:text-primary transition-colors"
              @click.stop="$emit('move-up')"
            >
              <ChevronUpIcon class="w-3 h-3" />
            </button>
            <button
              class="btn btn-ghost min-h-0! h-4 w-6 p-0 hover:bg-primary/10 hover:text-primary transition-colors"
              @click.stop="$emit('move-down')"
            >
              <ChevronDownIcon class="w-3 h-3" />
            </button>
          </div>
          <button
            class="btn btn-ghost btn-sm btn-circle text-error/40 hover:text-error hover:bg-error/10 transition-all"
            @click.stop="$emit('remove')"
          >
            <TrashIcon class="w-4 h-4 text-error" />
          </button>
          <div class="divider divider-horizontal mx-1 w-px opacity-10"></div>
          <ChevronRightIcon
            class="w-4 h-4 transition-transform duration-300 opacity-30"
            :class="{ 'rotate-90 opacity-100 text-primary': isExpanded }"
          />
        </div>
      </div>

      <!-- Content (Expanded) -->
      <div
        v-if="isExpanded"
        class="p-6 border-t border-base-200 bg-base-200/5 animate-in slide-in-from-top-2 duration-300"
      >
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <!-- Metadata Section -->
          <div class="form-control col-span-full">
            <label class="label-text text-[10px] font-black uppercase opacity-30 mb-2 block">步骤说明 / Remark</label>
            <input
              type="text"
              v-model="localStep.label"
              class="input input-bordered w-full rounded-xl focus:ring-2 focus:ring-primary/20 font-medium text-sm"
              placeholder="说明该步骤的功能，e.g. 点击确认按钮..."
              @change="onDataUpdate(localStep)"
            />
          </div>

          <!-- Dynamic Body -->
          <div class="col-span-full space-y-6">
            <!-- WaitMs Form -->
            <div v-if="step.op === 'WaitMs'" class="form-control">
              <label class="label-text text-[10px] font-black uppercase opacity-30 mb-2 block"
                >等候时长 (Milliseconds)</label
              >
              <input
                type="number"
                v-model="localStep.ms"
                class="input input-bordered w-32 font-mono"
                @change="onDataUpdate(localStep)"
              />
            </div>

            <!-- If/While Form -->
            <div v-if="['If', 'While'].includes(step.op)" class="space-y-6">
              <div class="p-5 bg-base-100 rounded-[2rem] border border-base-300 shadow-sm">
                <SearchRuleEditor
                  :rule="localStep.cond"
                  @update="
                    localStep.cond = $event;
                    onDataUpdate(localStep);
                  "
                />
              </div>

              <div class="p-5 bg-base-100 rounded-[2rem] border border-base-300 shadow-sm relative">
                <div
                  class="absolute -top-3 left-8 bg-base-100 px-3 py-1 border border-base-300 rounded-full text-[9px] font-black uppercase tracking-tighter shadow-sm"
                >
                  符合条件时执行序列
                </div>
                <ActionSequenceEditor
                  v-model:steps="localStep.steps"
                  :is-nested="true"
                  @update:steps="
                    localStep.steps = $event;
                    onDataUpdate(localStep);
                  "
                />
              </div>
            </div>

            <!-- Sequence Form -->
            <div
              v-if="step.op === 'Sequence'"
              class="p-5 bg-base-100 rounded-[2rem] border border-base-300 shadow-sm relative"
            >
              <div
                class="absolute -top-3 left-8 bg-base-100 px-3 py-1 border border-base-300 rounded-full text-[9px] font-black uppercase tracking-tighter shadow-sm"
              >
                子步骤编排
              </div>
              <ActionSequenceEditor
                v-model:steps="localStep.steps"
                :is-nested="true"
                @update:steps="
                  localStep.steps = $event;
                  onDataUpdate(localStep);
                "
              />
            </div>

            <!-- ClickAction Form -->
            <div v-if="step.op === 'ClickAction'" class="space-y-4">
              <div class="form-control">
                <label class="label-text text-[10px] font-black uppercase opacity-30 mb-2 block">点击模式</label>
                <select
                  v-model="clickMode"
                  class="select select-bordered select-sm w-full max-w-xs"
                  @change="onClickModeChange"
                >
                  <option value="Point">坐标点击</option>
                  <option value="Percent">百分比点击</option>
                  <option value="Label">YOLO 标签点击</option>
                  <option value="Txt">文字点击</option>
                  <option value="Var">变量点击</option>
                </select>
              </div>

              <!-- Point Mode -->
              <div v-if="clickMode === 'Point'" class="grid grid-cols-2 gap-4">
                <div class="form-control">
                  <label class="label-text text-[10px] font-bold opacity-40 mb-1">X 坐标</label>
                  <input
                    type="number"
                    v-model.number="localStep.Point.x"
                    class="input input-bordered input-sm font-mono"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
                <div class="form-control">
                  <label class="label-text text-[10px] font-bold opacity-40 mb-1">Y 坐标</label>
                  <input
                    type="number"
                    v-model.number="localStep.Point.y"
                    class="input input-bordered input-sm font-mono"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
              </div>

              <!-- Percent Mode -->
              <div v-if="clickMode === 'Percent'" class="grid grid-cols-2 gap-4">
                <div class="form-control">
                  <label class="label-text text-[10px] font-bold opacity-40 mb-1">X 百分比 (0-1)</label>
                  <input
                    type="number"
                    step="0.01"
                    min="0"
                    max="1"
                    v-model.number="localStep.Percent.x"
                    class="input input-bordered input-sm font-mono"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
                <div class="form-control">
                  <label class="label-text text-[10px] font-bold opacity-40 mb-1">Y 百分比 (0-1)</label>
                  <input
                    type="number"
                    step="0.01"
                    min="0"
                    max="1"
                    v-model.number="localStep.Percent.y"
                    class="input input-bordered input-sm font-mono"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
              </div>

              <!-- Label Mode -->
              <div v-if="clickMode === 'Label'" class="form-control">
                <label class="label-text text-[10px] font-bold opacity-40 mb-1">YOLO 标签索引 (多个用逗号分隔)</label>
                <input
                  type="text"
                  :value="localStep.Label?.join(', ') || ''"
                  class="input input-bordered input-sm font-mono"
                  placeholder="e.g. 0, 1, 5"
                  @change="onLabelChange($event.target.value)"
                />
              </div>

              <!-- Txt Mode -->
              <div v-if="clickMode === 'Txt'" class="form-control">
                <label class="label-text text-[10px] font-bold opacity-40 mb-1">匹配文字 (多个用逗号分隔)</label>
                <input
                  type="text"
                  :value="localStep.Txt?.join(', ') || ''"
                  class="input input-bordered input-sm"
                  placeholder="e.g. 确认, 提交"
                  @change="onTxtChange($event.target.value)"
                />
              </div>

              <!-- Var Mode -->
              <div v-if="clickMode === 'Var'" class="form-control">
                <label class="label-text text-[10px] font-bold opacity-40 mb-1">变量名</label>
                <input
                  type="text"
                  v-model="localStep.Var"
                  class="input input-bordered input-sm font-mono"
                  placeholder="hit_target"
                  @change="onDataUpdate(localStep)"
                />
              </div>
            </div>

            <!-- SwipePoint Form -->
            <div v-if="step.op === 'SwipePoint'" class="space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div class="p-3 bg-base-200/50 rounded-xl">
                  <div class="text-[9px] font-black uppercase opacity-40 mb-2">起点 (From)</div>
                  <div class="grid grid-cols-2 gap-2">
                    <input
                      type="number"
                      v-model.number="localStep.from.x"
                      class="input input-bordered input-xs font-mono"
                      placeholder="X"
                      @change="onDataUpdate(localStep)"
                    />
                    <input
                      type="number"
                      v-model.number="localStep.from.y"
                      class="input input-bordered input-xs font-mono"
                      placeholder="Y"
                      @change="onDataUpdate(localStep)"
                    />
                  </div>
                </div>
                <div class="p-3 bg-base-200/50 rounded-xl">
                  <div class="text-[9px] font-black uppercase opacity-40 mb-2">终点 (To)</div>
                  <div class="grid grid-cols-2 gap-2">
                    <input
                      type="number"
                      v-model.number="localStep.to.x"
                      class="input input-bordered input-xs font-mono"
                      placeholder="X"
                      @change="onDataUpdate(localStep)"
                    />
                    <input
                      type="number"
                      v-model.number="localStep.to.y"
                      class="input input-bordered input-xs font-mono"
                      placeholder="Y"
                      @change="onDataUpdate(localStep)"
                    />
                  </div>
                </div>
              </div>
            </div>

            <!-- SwipePercent Form -->
            <div v-if="step.op === 'SwipePercent'" class="space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div class="p-3 bg-base-200/50 rounded-xl">
                  <div class="text-[9px] font-black uppercase opacity-40 mb-2">起点百分比</div>
                  <div class="grid grid-cols-2 gap-2">
                    <input
                      type="number"
                      step="0.01"
                      v-model.number="localStep.from.x"
                      class="input input-bordered input-xs font-mono"
                      placeholder="X%"
                      @change="onDataUpdate(localStep)"
                    />
                    <input
                      type="number"
                      step="0.01"
                      v-model.number="localStep.from.y"
                      class="input input-bordered input-xs font-mono"
                      placeholder="Y%"
                      @change="onDataUpdate(localStep)"
                    />
                  </div>
                </div>
                <div class="p-3 bg-base-200/50 rounded-xl">
                  <div class="text-[9px] font-black uppercase opacity-40 mb-2">终点百分比</div>
                  <div class="grid grid-cols-2 gap-2">
                    <input
                      type="number"
                      step="0.01"
                      v-model.number="localStep.to.x"
                      class="input input-bordered input-xs font-mono"
                      placeholder="X%"
                      @change="onDataUpdate(localStep)"
                    />
                    <input
                      type="number"
                      step="0.01"
                      v-model.number="localStep.to.y"
                      class="input input-bordered input-xs font-mono"
                      placeholder="Y%"
                      @change="onDataUpdate(localStep)"
                    />
                  </div>
                </div>
              </div>
            </div>

            <!-- VisionSearch Form -->
            <div v-if="step.op === 'VisionSearch'" class="space-y-4">
              <div class="form-control">
                <label class="label-text text-[10px] font-black uppercase opacity-30 mb-2 block">输出变量名</label>
                <input
                  type="text"
                  v-model="localStep.output_var"
                  class="input input-bordered input-sm font-mono w-48"
                  placeholder="vision_hits"
                  @change="onDataUpdate(localStep)"
                />
              </div>
              <div class="p-5 bg-base-100 rounded-[2rem] border border-base-300 shadow-sm">
                <SearchRuleEditor
                  :rule="localStep.rule || { type: 'Group', op: 'And', scope: 'Global', items: [] }"
                  @update="
                    localStep.rule = $event;
                    onDataUpdate(localStep);
                  "
                />
              </div>
            </div>

            <!-- SetVar Form -->
            <div v-if="step.op === 'SetVar'" class="space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div class="form-control">
                  <label class="label-text text-[10px] font-black uppercase opacity-30 mb-2 block">变量名</label>
                  <input
                    type="text"
                    v-model="localStep.name"
                    class="input input-bordered input-sm font-mono"
                    placeholder="my_var"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
                <div class="form-control">
                  <label class="label-text text-[10px] font-black uppercase opacity-30 mb-2 block"
                    >值表达式 (Rhai)</label
                  >
                  <input
                    type="text"
                    v-model="localStep.value_expr"
                    class="input input-bordered input-sm font-mono"
                    placeholder="42 或 'hello'"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
              </div>
            </div>

            <!-- GetVar Form -->
            <div v-if="step.op === 'GetVar'" class="form-control">
              <label class="label-text text-[10px] font-black uppercase opacity-30 mb-2 block">变量名</label>
              <input
                type="text"
                v-model="localStep.name"
                class="input input-bordered input-sm font-mono w-48"
                placeholder="my_var"
                @change="onDataUpdate(localStep)"
              />
            </div>

            <!-- TakeScreenshot Form -->
            <div v-if="step.op === 'TakeScreenshot'" class="form-control">
              <label class="label-text text-[10px] font-black uppercase opacity-30 mb-2 block">输出变量名</label>
              <input
                type="text"
                v-model="localStep.output_var"
                class="input input-bordered input-sm font-mono w-48"
                placeholder="screenshot_path"
                @change="onDataUpdate(localStep)"
              />
            </div>

            <!-- SetState Form -->
            <div v-if="step.op === 'SetState'" class="space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div class="form-control">
                  <label class="label-text text-[10px] font-black uppercase opacity-30 mb-2 block">目标类型</label>
                  <select
                    v-model="localStep.target.type"
                    class="select select-bordered select-sm"
                    @change="onDataUpdate(localStep)"
                  >
                    <option value="Policy">策略 (Policy)</option>
                    <option value="Task">任务 (Task)</option>
                  </select>
                </div>
                <div class="form-control">
                  <label class="label-text text-[10px] font-black uppercase opacity-30 mb-2 block">目标 ID</label>
                  <input
                    type="text"
                    v-model="localStep.target.id"
                    class="input input-bordered input-sm font-mono"
                    placeholder="目标 ID"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
              </div>
              <div class="grid grid-cols-2 gap-4">
                <div class="form-control">
                  <label class="label-text text-[10px] font-black uppercase opacity-30 mb-2 block">状态类型</label>
                  <select
                    v-model="localStep.status.type"
                    class="select select-bordered select-sm"
                    @change="onDataUpdate(localStep)"
                  >
                    <option value="Skip">跳过 (Skip)</option>
                    <option value="Done">完成 (Done)</option>
                  </select>
                </div>
                <div class="form-control">
                  <label class="label-text text-[10px] font-black uppercase opacity-30 mb-2 block">状态值</label>
                  <input
                    type="checkbox"
                    v-model="localStep.status.value"
                    class="checkbox checkbox-primary"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
              </div>
            </div>

            <!-- Placeholder for remaining forms -->
            <div
              v-if="!supportedOps.includes(step.op)"
              class="p-8 text-center bg-base-200/20 rounded-[2rem] border-2 border-dashed border-base-300 opacity-40"
            >
              <div class="text-[10px] font-black uppercase tracking-widest mb-2">配置项建设中</div>
              <p class="text-[10px]">该类型 ({{ step.op }}) 的可视化表单正在开发中...</p>
            </div>
          </div>

          <!-- Footer Controls -->
          <div class="col-span-full mt-2 grid grid-cols-2 gap-4 p-4 bg-base-200/50 rounded-2xl border border-base-200">
            <div class="form-control">
              <label class="label py-0"
                ><span class="label-text text-[10px] font-bold opacity-40 uppercase">最大次数限制</span></label
              >
              <input
                type="number"
                v-model="localStep.execMax"
                class="input input-bordered input-sm font-mono w-24 rounded-lg mt-1"
                @change="onDataUpdate(localStep)"
              />
            </div>
            <div class="flex items-center justify-end gap-3 px-2">
              <span class="text-[10px] font-bold opacity-40 uppercase">禁用步骤</span>
              <input
                type="checkbox"
                v-model="localStep.skipFlag"
                class="checkbox checkbox-primary checkbox-sm rounded-lg"
                @change="onDataUpdate(localStep)"
              />
            </div>
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
import SearchRuleEditor from './SearchRuleEditor.vue';

const ActionSequenceEditor = defineAsyncComponent(() => import('./ActionSequenceEditor.vue'));

const props = defineProps({
  step: {
    type: Object,
    required: true,
  },
  isNested: Boolean,
});

const emit = defineEmits(['update', 'remove', 'move-up', 'move-down']);

const isExpanded = ref(false);
const localStep = ref({ ...props.step });

// 支持的操作类型列表
const supportedOps = [
  'WaitMs',
  'If',
  'While',
  'Sequence',
  'ClickAction',
  'SwipePoint',
  'SwipePercent',
  'VisionSearch',
  'SetVar',
  'GetVar',
  'TakeScreenshot',
  'SetState',
];

// ClickAction 模式检测
const detectClickMode = (step) => {
  if (step.Point) return 'Point';
  if (step.Percent) return 'Percent';
  if (step.Label) return 'Label';
  if (step.Txt) return 'Txt';
  if (step.Var) return 'Var';
  return 'Point'; // 默认
};

const clickMode = ref(detectClickMode(props.step));

const onClickModeChange = () => {
  // 清理旧模式数据，初始化新模式
  delete localStep.value.Point;
  delete localStep.value.Percent;
  delete localStep.value.Label;
  delete localStep.value.Txt;
  delete localStep.value.Var;

  switch (clickMode.value) {
    case 'Point':
      localStep.value.Point = { x: 0, y: 0 };
      break;
    case 'Percent':
      localStep.value.Percent = { x: 0.5, y: 0.5 };
      break;
    case 'Label':
      localStep.value.Label = [];
      break;
    case 'Txt':
      localStep.value.Txt = [];
      break;
    case 'Var':
      localStep.value.Var = '';
      break;
  }
  onDataUpdate(localStep.value);
};

const onLabelChange = (value) => {
  localStep.value.Label = value
    .split(',')
    .map((s) => parseInt(s.trim(), 10))
    .filter((n) => !isNaN(n));
  onDataUpdate(localStep.value);
};

const onTxtChange = (value) => {
  localStep.value.Txt = value
    .split(',')
    .map((s) => s.trim())
    .filter((s) => s);
  onDataUpdate(localStep.value);
};

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
    control: 'bg-gradient-to-br from-amber-400 to-orange-500',
    interaction: 'bg-gradient-to-br from-blue-400 to-indigo-600',
    vision: 'bg-gradient-to-br from-purple-400 to-fuchsia-600',
    logic: 'bg-gradient-to-br from-orange-400 to-rose-500',
    state: 'bg-gradient-to-br from-emerald-400 to-teal-600',
    other: 'bg-gradient-to-br from-slate-400 to-slate-600',
  };
  return map[category.value];
});

const stepTitle = computed(() => {
  const opMap = {
    ClickAction: '点击交互',
    WaitMs: '延时等待',
    If: '条件分支 (If)',
    While: '循环控制 (While)',
    SetVar: '变量赋值',
    GetVar: '变量读取',
    VisionSearch: '强化视觉搜索',
    TakeScreenshot: '屏幕截图',
    Sequence: '步骤序列容器',
    SwipePoint: '坐标滑动',
    SwipePercent: '百分比滑动',
    SetState: '状态设置',
    GetState: '状态读取',
    StopPolicy: '停止策略',
    FinishTask: '结束任务',
  };
  return opMap[props.step.op] || props.step.op;
});

const stepSummary = computed(() => {
  const op = props.step.op;
  if (op === 'WaitMs') return `等待 ${localStep.value.ms} 毫秒`;
  if (op === 'Sequence') return `包含 ${localStep.value.steps?.length || 0} 个子动作`;
  if (op === 'If' || op === 'While') return `判断条件后执行 ${localStep.value.steps?.length || 0} 个逻辑`;
  if (op === 'ClickAction') return `${clickMode.value} 模式点击`;
  if (op === 'SwipePoint' || op === 'SwipePercent') return `滑动手势`;
  if (op === 'VisionSearch') return `搜索结果 → ${localStep.value.output_var || '?'}`;
  if (op === 'SetVar') return `${localStep.value.name || '?'} = ${localStep.value.value_expr || '?'}`;
  if (op === 'GetVar') return `读取 ${localStep.value.name || '?'}`;
  if (op === 'TakeScreenshot') return `保存到 ${localStep.value.output_var || '?'}`;
  if (op === 'SetState') return `设置 ${localStep.value.target?.type || '?'} 状态`;
  return `OP: ${op}`;
});

const onDataUpdate = (newData) => {
  emit('update', { ...newData });
};

watch(
  () => props.step,
  (val) => {
    if (JSON.stringify(val) !== JSON.stringify(localStep.value)) {
      localStep.value = { ...val };
    }
  },
  { deep: true }
);
</script>
