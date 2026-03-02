<template>
  <div class="step-item p-1 bg-transparent group-step">
    <div class="bg-base-100 border border-base-300 rounded-3xl shadow-sm overflow-hidden">
      <!-- Header -->
      <div
        v-if="!isPropertiesPanel"
        class="flex items-center gap-3 p-4 cursor-pointer select-none"
        @click="isExpanded = !isExpanded"
      >
        <div
          class="w-10 h-10 rounded-2xl flex items-center justify-center transition-transform hover:scale-110 shadow-sm"
          :class="categoryColor"
        >
          <IconRenderer :icon="getNodeIcon(displayVirtualOp)" class="w-5 h-5 text-white" />
        </div>

        <div class="flex-2 min-w-0">
          <div class="font-black text-sm tracking-tight">
            {{ stepTitle }}<span v-if="step.label"> {{ step.label }}</span>
          </div>
          <div class="text-[14px] font-mono opacity-60 tracking-tight mt-0.5 truncate">{{ stepSummary }}</div>
        </div>

        <div class="flex items-center gap-1">
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
            <label class="label-text text-[16px] font-black uppercase opacity-30 mb-2 block">备注</label>
            <input
              type="text"
              v-model="localStep.label"
              class="input input-bordered w-full rounded-xl focus:ring-2 focus:ring-primary/20 font-medium text-sm"
              placeholder="记个笔记免得忘了作用..."
              @change="onDataUpdate(localStep)"
            />
          </div>

          <!-- Dynamic Body -->
          <div class="col-span-full space-y-6">
            <!-- ClickPoint Form -->
            <div v-if="virtualOp === 'clickPoint'" class="grid grid-cols-2 gap-4">
              <div class="form-control">
                <label class="label-text text-[14px] font-bold opacity-40 mb-1">X 坐标</label>
                <input
                  type="number"
                  v-model.number="(localStep as any).a.p.x"
                  class="input input-bordered input-sm font-mono"
                  @change="onDataUpdate(localStep)"
                />
              </div>
              <div class="form-control">
                <label class="label-text text-[14px] font-bold opacity-40 mb-1">Y 坐标</label>
                <input
                  type="number"
                  v-model.number="(localStep as any).a.p.y"
                  class="input input-bordered input-sm font-mono"
                  @change="onDataUpdate(localStep)"
                />
              </div>
            </div>

            <!-- ClickPercent Form -->
            <div v-if="virtualOp === 'clickPercent'" class="grid grid-cols-2 gap-4">
              <div class="form-control">
                <label class="label-text text-[14px] font-bold opacity-40 mb-1">X 百分比 (0-1)</label>
                <input
                  type="number"
                  step="0.01"
                  min="0"
                  max="1"
                  v-model.number="(localStep as any).a.p.x"
                  class="input input-bordered input-sm font-mono"
                  @change="onDataUpdate(localStep)"
                />
              </div>
              <div class="form-control">
                <label class="label-text text-[14px] font-bold opacity-40 mb-1">Y 百分比 (0-1)</label>
                <input
                  type="number"
                  step="0.01"
                  min="0"
                  max="1"
                  v-model.number="(localStep as any).a.p.y"
                  class="input input-bordered input-sm font-mono"
                  @change="onDataUpdate(localStep)"
                />
              </div>
            </div>

            <!-- ClickLabel Form -->
            <div v-if="virtualOp === 'clickLabel'" class="form-control">
              <label class="label-text text-[14px] font-bold opacity-40 mb-1">YOLO 标签索引</label>
              <input
                type="number"
                v-model.number="(localStep as any).a.idx"
                class="input input-bordered input-sm font-mono w-32"
                placeholder="e.g. 0"
                @change="onDataUpdate(localStep)"
              />
            </div>

            <!-- ClickTxt Form -->
            <div v-if="virtualOp === 'clickTxt'" class="form-control">
              <label class="label-text text-[14px] font-bold opacity-40 mb-1">匹配文字</label>
              <input
                type="text"
                v-model="(localStep as any).a.txt"
                class="input input-bordered input-sm"
                placeholder="e.g. 确认"
                @change="onDataUpdate(localStep)"
              />
            </div>

            <!-- SwipePoint Form -->
            <div v-if="virtualOp === 'swipePoint'" class="space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div class="p-3 bg-base-200/50 rounded-xl">
                  <div class="text-[14px] font-black uppercase opacity-40 mb-2">起点 (From)</div>
                  <div class="grid grid-cols-2 gap-2">
                    <input
                      type="number"
                      v-model.number="(localStep as any).a.from.x"
                      class="input input-bordered input-xs font-mono"
                      placeholder="X"
                      @change="onDataUpdate(localStep)"
                    />
                    <input
                      type="number"
                      v-model.number="(localStep as any).a.from.y"
                      class="input input-bordered input-xs font-mono"
                      placeholder="Y"
                      @change="onDataUpdate(localStep)"
                    />
                  </div>
                </div>
                <div class="p-3 bg-base-200/50 rounded-xl">
                  <div class="text-[14px] font-black uppercase opacity-40 mb-2">终点 (To)</div>
                  <div class="grid grid-cols-2 gap-2">
                    <input
                      type="number"
                      v-model.number="(localStep as any).a.to.x"
                      class="input input-bordered input-xs font-mono"
                      placeholder="X"
                      @change="onDataUpdate(localStep)"
                    />
                    <input
                      type="number"
                      v-model.number="(localStep as any).a.to.y"
                      class="input input-bordered input-xs font-mono"
                      placeholder="Y"
                      @change="onDataUpdate(localStep)"
                    />
                  </div>
                </div>
              </div>
              <div class="form-control">
                <label class="label-text text-[14px] font-bold opacity-40 mb-1">持续时间 (ms)</label>
                <input
                  type="number"
                  v-model.number="(localStep as any).a.duration"
                  class="input input-bordered input-sm font-mono w-32"
                  @change="onDataUpdate(localStep)"
                />
              </div>
            </div>

            <!-- SwipePercent Form -->
            <div v-if="virtualOp === 'swipePercent'" class="space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div class="p-3 bg-base-200/50 rounded-xl">
                  <div class="text-[14px] font-black uppercase opacity-40 mb-2">起点百分比</div>
                  <div class="grid grid-cols-2 gap-2">
                    <input
                      type="number"
                      step="0.01"
                      v-model.number="(localStep as any).a.from.x"
                      class="input input-bordered input-xs font-mono"
                      placeholder="X%"
                      @change="onDataUpdate(localStep)"
                    />
                    <input
                      type="number"
                      step="0.01"
                      v-model.number="(localStep as any).a.from.y"
                      class="input input-bordered input-xs font-mono"
                      placeholder="Y%"
                      @change="onDataUpdate(localStep)"
                    />
                  </div>
                </div>
                <div class="p-3 bg-base-200/50 rounded-xl">
                  <div class="text-[14px] font-black uppercase opacity-40 mb-2">终点百分比</div>
                  <div class="grid grid-cols-2 gap-2">
                    <input
                      type="number"
                      step="0.01"
                      v-model.number="(localStep as any).a.to.x"
                      class="input input-bordered input-xs font-mono"
                      placeholder="X%"
                      @change="onDataUpdate(localStep)"
                    />
                    <input
                      type="number"
                      step="0.01"
                      v-model.number="(localStep as any).a.to.y"
                      class="input input-bordered input-xs font-mono"
                      placeholder="Y%"
                      @change="onDataUpdate(localStep)"
                    />
                  </div>
                </div>
              </div>
              <div class="form-control">
                <label class="label-text text-[14px] font-bold opacity-40 mb-1">持续时间 (ms)</label>
                <input
                  type="number"
                  v-model.number="(localStep as any).a.duration"
                  class="input input-bordered input-sm font-mono w-32"
                  @change="onDataUpdate(localStep)"
                />
              </div>
            </div>

            <!-- SwipeLabel Form -->
            <div v-if="virtualOp === 'swipeLabel'" class="space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div class="form-control">
                  <label class="label-text text-[14px] font-bold opacity-40 mb-1">起始标签索引</label>
                  <input
                    type="number"
                    v-model.number="(localStep as any).a.from"
                    class="input input-bordered input-sm font-mono"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
                <div class="form-control">
                  <label class="label-text text-[14px] font-bold opacity-40 mb-1">终止标签索引</label>
                  <input
                    type="number"
                    v-model.number="(localStep as any).a.to"
                    class="input input-bordered input-sm font-mono"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
              </div>
              <div class="form-control">
                <label class="label-text text-[14px] font-bold opacity-40 mb-1">持续时间 (ms)</label>
                <input
                  type="number"
                  v-model.number="(localStep as any).a.duration"
                  class="input input-bordered input-sm font-mono w-32"
                  @change="onDataUpdate(localStep)"
                />
              </div>
            </div>

            <!-- SwipeTxt Form -->
            <div v-if="virtualOp === 'swipeTxt'" class="space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div class="form-control">
                  <label class="label-text text-[14px] font-bold opacity-40 mb-1">起始文字</label>
                  <input
                    type="text"
                    v-model="(localStep as any).a.from"
                    class="input input-bordered input-sm"
                    placeholder="起始文字"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
                <div class="form-control">
                  <label class="label-text text-[14px] font-bold opacity-40 mb-1">终止文字</label>
                  <input
                    type="text"
                    v-model="(localStep as any).a.to"
                    class="input input-bordered input-sm"
                    placeholder="终止文字"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
              </div>
              <div class="form-control">
                <label class="label-text text-[14px] font-bold opacity-40 mb-1">持续时间 (ms)</label>
                <input
                  type="number"
                  v-model.number="(localStep as any).a.duration"
                  class="input input-bordered input-sm font-mono w-32"
                  @change="onDataUpdate(localStep)"
                />
              </div>
            </div>

            <!-- WaitMs Form -->
            <div v-if="virtualOp === 'waitMs'" class="form-control">
              <label class="label-text text-[14px] font-black uppercase opacity-30 mb-2 block"
                >等候时长 (Milliseconds)</label
              >
              <input
                type="number"
                v-model.number="(localStep as any).a.ms"
                class="input input-bordered w-32 font-mono"
                @change="onDataUpdate(localStep)"
              />
            </div>

            <!-- If/While/For Form -->
            <div v-if="['if', 'while', 'for'].includes(virtualOp)" class="space-y-6">
              <div class="p-5 bg-base-100 rounded-4xl border border-base-300 shadow-sm">
                <ConditionNodeEditor
                  :condition="(localStep as any).a?.con || (localStep as any).cond"
                  :isRoot="true"
                  @update="
                  if ((localStep as any).a) (localStep as any).a.con = $event;
                  else (localStep as any).cond = $event;
                  onDataUpdate(localStep);"
                />
              </div>
              <div class="p-5 bg-base-100 rounded-4xl border border-base-300 shadow-sm relative">
                <div
                  class="absolute -top-3 left-8 bg-base-100 px-3 py-1 border border-base-300 rounded-full text-[14px] font-black uppercase tracking-tighter shadow-sm"
                >
                  {{ virtualOp === 'if' ? '符合条件时执行序列' : '循环体' }}
                </div>
                <ActionSequenceEditor
                  v-model:steps="conditionSteps"
                  :is-nested="true"
                  @update:steps="onConditionStepsUpdate"
                />
              </div>
            </div>

            <!-- Sequence Form -->
            <div
              v-if="virtualOp === 'sequence'"
              class="p-5 bg-base-100 rounded-4xl border border-base-300 shadow-sm relative"
            >
              <div
                class="absolute -top-3 left-8 bg-base-100 px-3 py-1 border border-base-300 rounded-full text-[14px] font-black uppercase tracking-tighter shadow-sm"
              >
                子步骤编排
              </div>
              <ActionSequenceEditor
                v-model:steps="(localStep as any).steps"
                :is-nested="true"
                @update:steps="
                  (localStep as any).steps = $event;
                  onDataUpdate(localStep);
                "
              />
            </div>

            <!-- TakeScreenshot Form -->
            <div v-if="virtualOp === 'takeScreenshot'" class="form-control">
              <label class="label-text text-[14px] font-black uppercase opacity-30 mb-2 block">输出变量名</label>
              <input
                type="text"
                v-model="(localStep as any).a.output_var"
                class="input input-bordered input-sm font-mono w-48"
                placeholder="screenshot_path"
                @change="onDataUpdate(localStep)"
              />
            </div>

            <!-- LaunchApp / StopApp Form -->
            <div v-if="virtualOp === 'launchApp' || virtualOp === 'stopApp'" class="form-control">
              <label class="label-text text-[14px] font-black uppercase opacity-30 mb-2 block">应用包名</label>
              <input
                type="text"
                v-model="(localStep as any).a.pkg_name"
                class="input input-bordered input-sm font-mono w-64"
                placeholder="com.example.app"
                @change="onDataUpdate(localStep)"
              />
            </div>

            <!-- Reboot: 无需表单 -->
            <div
              v-if="virtualOp === 'reboot'"
              class="p-6 text-center bg-base-200/20 rounded-3xl border border-dashed border-base-300"
            >
              <div class="text-sm font-bold opacity-50">此操作将重启设备，无需额外配置</div>
            </div>

            <!-- Continue/Break: 无需表单 -->
            <div
              v-if="virtualOp === 'continue' || virtualOp === 'break'"
              class="p-6 text-center bg-base-200/20 rounded-3xl border border-dashed border-base-300"
            >
              <div class="text-sm font-bold opacity-50">
                {{ virtualOp === 'continue' ? '跳过当前循环迭代' : '跳出当前循环' }}，无需额外配置
              </div>
            </div>

            <!-- VisionSearch Form -->
            <div v-if="virtualOp === 'visionSearch'" class="space-y-4">
              <div class="form-control">
                <label class="label-text text-[14px] font-black uppercase opacity-30 mb-2 block">输出变量名</label>
                <input
                  type="text"
                  v-model="(localStep as any).a.out_var"
                  class="input input-bordered input-sm font-mono w-48"
                  placeholder="vision_hits"
                  @change="onDataUpdate(localStep)"
                />
              </div>
              <div class="p-5 bg-base-100 rounded-4xl border border-base-300 shadow-sm">
                <SearchRuleEditor
                  :rule="(localStep as any).a?.rule || { type: 'group', op: 'And', scope: 'Global', items: [] }"
                  @update="
                    (localStep as any).a.rule = $event;
                    onDataUpdate(localStep);
                  "
                />
              </div>
            </div>

            <!-- SetVar Form -->
            <div v-if="virtualOp === 'setVar'" class="space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div class="form-control">
                  <label class="label-text text-[14px] font-black uppercase opacity-30 mb-2 block">变量名</label>
                  <input
                    type="text"
                    v-model="(localStep as any).a.name"
                    class="input input-bordered input-sm font-mono"
                    placeholder="my_var"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
                <div class="form-control">
                  <label class="label-text text-[14px] font-black uppercase opacity-30 mb-2 block">表达式 (Rhai)</label>
                  <input
                    type="text"
                    v-model="(localStep as any).a.expr"
                    class="input input-bordered input-sm font-mono"
                    placeholder="42 或 'hello'"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
              </div>
            </div>

            <!-- GetVar Form -->
            <div v-if="virtualOp === 'getVar'" class="form-control">
              <label class="label-text text-[14px] font-black uppercase opacity-30 mb-2 block">变量名</label>
              <input
                type="text"
                v-model="(localStep as any).a.name"
                class="input input-bordered input-sm font-mono w-48"
                placeholder="my_var"
                @change="onDataUpdate(localStep)"
              />
            </div>

            <!-- SetState / GetState Form -->
            <div v-if="virtualOp === 'setState' || virtualOp === 'getState'" class="space-y-4">
              <div class="grid grid-cols-2 gap-4">
                <div class="form-control">
                  <label class="label-text text-[14px] font-black uppercase opacity-30 mb-2 block">目标类型</label>
                  <select
                    v-model="(localStep as any).a.target.type"
                    class="select select-bordered select-sm"
                    @change="onDataUpdate(localStep)"
                  >
                    <option value="Policy">策略 (Policy)</option>
                    <option value="Task">任务 (Task)</option>
                  </select>
                </div>
                <div class="form-control">
                  <label class="label-text text-[14px] font-black uppercase opacity-30 mb-2 block">目标 ID</label>
                  <select
                    v-if="(localStep as any).a.target.type === 'Policy'"
                    class="select select-bordered select-sm"
                    v-model="(localStep as any).a.target.id"
                    @change="onDataUpdate(localStep)"
                  >
                    <option v-for="p in policies" :key="p.id" :value="p.id">{{ p.data?.name || '未命名' }}</option>
                  </select>
                  <select
                    v-else-if="(localStep as any).a.target.type === 'Task'"
                    class="select select-bordered select-sm"
                    v-model="(localStep as any).a.target.id"
                    @change="onDataUpdate(localStep)"
                  >
                    <option v-for="t in tasks" :key="t.id" :value="t.id">{{ t.name || '未命名' }}</option>
                  </select>
                </div>
              </div>
              <div class="grid grid-cols-2 gap-4">
                <div class="form-control">
                  <label class="label-text text-[14px] font-black uppercase opacity-30 mb-2 block">状态类型</label>
                  <select
                    v-model="(localStep as any).a.status.type"
                    class="select select-bordered select-sm"
                    @change="onDataUpdate(localStep)"
                  >
                    <option value="Skip">跳过 (Skip)</option>
                    <option value="Done">完成 (Done)</option>
                  </select>
                </div>
                <div class="form-control">
                  <label class="label-text text-[14px] font-black uppercase opacity-30 mb-2 block">状态值</label>
                  <input
                    type="checkbox"
                    v-model="(localStep as any).a.status.value"
                    class="checkbox checkbox-primary"
                    @change="onDataUpdate(localStep)"
                  />
                </div>
              </div>
            </div>

            <!-- Placeholder for unsupported types -->
            <div
              v-if="!supportedFormOps.includes(virtualOp)"
              class="p-8 text-center bg-base-200/20 rounded-4xl border-2 border-dashed border-base-300 opacity-40"
            >
              <div class="text-[14px] font-black uppercase tracking-widest mb-2">配置项建设中</div>
              <p class="text-[14px]">该类型 ({{ virtualOp }}) 的可视化表单正在开发中...</p>
            </div>
          </div>

          <!-- Footer Controls -->
          <div class="col-span-full mt-2 grid grid-cols-2 gap-4 p-4 bg-base-200/50 rounded-2xl border border-base-200">
            <div class="form-control">
              <span class="label-text text-[14px] font-bold opacity-40 uppercase"
              >次数(0不限)</span
              >
              <input
                type="number"
                v-model="localStep.exec_max"
                class="input input-bordered input-sm font-mono w-24 rounded-lg mt-1"
                @change="onDataUpdate(localStep)"
              />
            </div>
            <div class="flex items-center justify-end gap-3 px-2">
              <span class="text-[14px] font-bold opacity-40 uppercase">禁用</span>
              <input
                type="checkbox"
                v-model="localStep.skip_flag"
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

<script setup lang="ts">
import { ref, computed, watch, defineAsyncComponent, inject, onMounted } from 'vue';
import { ChevronRight as ChevronRightIcon, Trash2 as TrashIcon } from 'lucide-vue-next';
import IconRenderer from '../IconRenderer.vue';
import SearchRuleEditor from './SearchRuleEditor.vue';
import ConditionNodeEditor from './ConditionNodeEditor.vue';
import type { Step } from '@/types/bindings';
import { invoke } from '@tauri-apps/api/core';
import { getNodeColor, getNodeDisplay, getNodeIcon, getStepVirtualOp, SUPPORTED_STEP_OPS } from '../config';

const ActionSequenceEditor = defineAsyncComponent(() => import('./ActionSequenceEditor.vue'));

const props = defineProps<{
  step: Step;
  isNested?: boolean;
  isPropertiesPanel?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update', step: Step): void;
  (e: 'remove'): void;
}>();

const isExpanded = ref(props.isPropertiesPanel || false);

const localStep = ref<Step>({ ...props.step } as Step);
const displayVirtualOp = computed(() => getStepVirtualOp(props.step));
const virtualOp = computed(() => getStepVirtualOp(localStep.value));

const scriptInfo: any = inject('scriptInfo', ref(null));
const policies = ref<any[]>([]);
const tasks = ref<any[]>([]);

const loadData = async () => {
  if (!scriptInfo.value?.id) return;
  try {
    const rawPolicies = await invoke<any[]>('get_all_policies_cmd', { scriptId: scriptInfo.value.id });
    policies.value = rawPolicies || [];
  } catch (e) {
    console.error(e);
  }
  try {
    const rawTasks = await invoke<any[]>('get_script_tasks_cmd', { scriptId: scriptInfo.value.id });
    tasks.value = rawTasks || [];
  } catch (e) {
    console.error(e);
  }
};

onMounted(() => {
  loadData();
});
watch(
  () => scriptInfo.value?.id,
  () => loadData()
);

// 已有详细表单的操作类型
const supportedFormOps = [
  'clickPoint',
  'clickPercent',
  'clickLabel',
  'clickTxt',
  'swipePoint',
  'swipePercent',
  'swipeLabel',
  'swipeTxt',
  'waitMs',
  'if',
  'while',
  'for',
  'sequence',
  'takeScreenshot',
  'reboot',
  'launchApp',
  'stopApp',
  'continue',
  'break',
  'visionSearch',
  'setVar',
  'getVar',
  'setState',
  'getState',
];

// If/While/For 的子步骤
const conditionSteps = computed({
  get(): Step[] {
    const s = localStep.value as any;
    if (s.a?.then) return s.a.then;
    if (s.a?.flow) return s.a.flow;
    if (s.steps) return s.steps;
    return [];
  },
  set(val: Step[]) {
    const s = localStep.value as any;
    if (s.a?.then !== undefined) s.a.then = val;
    else if (s.a?.flow !== undefined) s.a.flow = val;
    else if (s.steps !== undefined) s.steps = val;
    onDataUpdate(localStep.value);
  },
});

const onConditionStepsUpdate = (val: Step[]) => {
  conditionSteps.value = val;
};

// ========== 全部从 config.ts 统一获取，不再本地硬编码 ==========
const categoryColor = computed(() => {
  const color = getNodeColor(displayVirtualOp.value);
  return color || 'bg-base-300';
});

const stepTitle = computed(() => {
  return getNodeDisplay(displayVirtualOp.value, 'cn') || displayVirtualOp.value;
});

const stepSummary = computed(() => {
  const op = virtualOp.value;
  const s = localStep.value as any;
  if (op === 'waitMs') return `等待 ${s.a?.ms || 0} 毫秒`;
  if (op === 'sequence') return `包含 ${s.steps?.length || 0} 个子动作`;
  if (op === 'if') return `判断条件后执行`;
  if (op === 'while' || op === 'for') return `循环`;
  if (op.startsWith('click')) return `点击`;
  if (op.startsWith('swipe')) return `滑动`;
  if (op === 'visionSearch') return `搜索结果 → ${s.a?.out_var || '?'}`;
  if (op === 'setVar') return `${s.a?.name || '?'} = ${s.a?.expr || '?'}`;
  if (op === 'getVar') return `读取 ${s.a?.name || '?'}`;
  if (op === 'takeScreenshot') return `保存到 ${s.a?.output_var || '?'}`;
  if (op === 'setState' || op === 'getState') return `${s.a?.target?.type || '?'} 状态`;
  if (op === 'launchApp' || op === 'stopApp') return s.a?.pkg_name || '未指定';
  if (op === 'reboot') return '重启设备';
  if (op === 'continue') return '继续循环';
  if (op === 'break') return '跳出循环';
  return `OP: ${op}`;
});

const onDataUpdate = (newData: Step) => {
  emit('update', { ...newData });
};

watch(
  () => props.step,
  (val) => {
    if (JSON.stringify(val) !== JSON.stringify(localStep.value)) {
      localStep.value = { ...val } as Step;
    }
  },
  { deep: true }
);
</script>
