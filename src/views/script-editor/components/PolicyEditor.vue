<template>
  <div class="policy-editor flex h-full gap-6 overflow-hidden p-1">
    <!-- Left Sidebar: Metadata -->
    <div class="w-80 flex-none flex flex-col gap-4 p-5 bg-base-200/50 rounded-3xl border border-base-300 shadow-sm">
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-2">
          <div class="w-2 h-6 bg-primary rounded-full"></div>
          <h4 class="text-xs font-bold uppercase tracking-wider opacity-60">策略配置</h4>
        </div>
        <button
          class="btn btn-primary btn-sm px-4 rounded-full shadow-lg hover:scale-105 active:scale-95 transition-all"
          @click="onSave"
        >
          保存修改
        </button>
      </div>

      <div class="space-y-4 flex-1">
        <div class="form-control">
          <label class="label py-1"><span class="label-text text-xs font-bold opacity-60">策略名称</span></label>
          <input
            v-model="localPolicy.name"
            type="text"
            class="input input-bordered input-sm w-full font-bold focus:ring-2 focus:ring-primary/20"
          />
        </div>

        <div class="form-control">
          <label class="label py-1"><span class="label-text text-xs font-bold opacity-60">备注说明</span></label>
          <textarea
            v-model="localPolicy.note"
            class="textarea textarea-bordered textarea-sm w-full h-24 resize-none focus:ring-2 focus:ring-primary/20"
            placeholder="简要描述此策略的用途..."
          ></textarea>
        </div>

        <div class="divider opacity-30 my-1"></div>

        <div class="grid grid-cols-2 gap-4">
          <div class="form-control">
            <label class="label py-1"
              ><span class="label-text text-[10px] uppercase font-bold opacity-60">点击索引 (CurPos)</span></label
            >
            <input
              v-model.number="localPolicy.curPos"
              type="number"
              class="input input-bordered input-sm w-full font-mono"
            />
          </div>
          <div class="form-control">
            <label class="label py-1"
              ><span class="label-text text-[10px] uppercase font-bold opacity-60">最大次数 (Limit)</span></label
            >
            <input
              v-model.number="localPolicy.execMax"
              type="number"
              class="input input-bordered input-sm w-full font-mono"
            />
          </div>
        </div>

        <div class="form-control">
          <label class="label py-1"
            ><span class="label-text text-[10px] uppercase font-bold opacity-60">日志打印 (Hit Log)</span></label
          >
          <textarea
            v-model="localPolicy.logPrint"
            class="textarea textarea-bordered textarea-xs w-full font-mono h-20 resize-none"
            placeholder="当命中该策略时打印的自定义日志内容..."
          ></textarea>
        </div>
      </div>

      <div class="p-3 bg-base-300/30 rounded-2xl">
        <div class="text-[9px] opacity-40 uppercase font-bold mb-1">策略 ID</div>
        <div class="text-[10px] font-mono opacity-50 break-all select-all">{{ localPolicy.id || 'Draft' }}</div>
      </div>
    </div>

    <!-- Right Content: Logic -->
    <div
      class="flex-1 flex flex-col min-w-0 bg-base-100 rounded-3xl border border-base-300 overflow-hidden shadow-inner"
    >
      <!-- Custom Tabs -->
      <div class="flex bg-base-200/50 p-1 mb-1">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          class="flex-1 py-3 px-4 text-sm font-bold transition-all duration-200 rounded-2xl flex items-center justify-center gap-2"
          :class="
            activeTab === tab.id
              ? 'bg-base-100 shadow-md text-primary scale-[0.98]'
              : 'text-base-content/40 hover:text-base-content/70'
          "
          @click="activeTab = tab.id"
        >
          <component :is="tab.icon" class="w-4 h-4" />
          {{ tab.label }}
        </button>
      </div>

      <!-- Tab Contents -->
      <div class="flex-1 overflow-y-auto p-6 custom-scrollbar">
        <div v-show="activeTab === 'cond'" class="animate-in fade-in slide-in-from-bottom-2 duration-300">
          <SearchRuleEditor :rule="localPolicy.cond" @update="localPolicy.cond = $event" />
        </div>

        <div v-show="activeTab === 'before'" class="animate-in fade-in slide-in-from-bottom-2 duration-300">
          <div
            class="alert bg-primary/10 border-primary/20 text-primary text-xs mb-6 rounded-2xl flex items-start gap-3"
          >
            <InfoIcon class="w-4 h-4 mt-0.5" />
            <div class="flex-1">
              <div class="font-bold mb-0.5 underline">前置钩子 (Before Hook)</div>
              <span>在视觉匹配执行前运行。通常用于清理状态、滚动页面或预置变量。</span>
            </div>
          </div>
          <ActionSequenceEditor v-model:steps="localPolicy.beforeAction" />
        </div>

        <div v-show="activeTab === 'after'" class="animate-in fade-in slide-in-from-bottom-2 duration-300">
          <div
            class="alert bg-emerald-500/10 border-emerald-500/20 text-emerald-600 text-xs mb-6 rounded-2xl flex items-start gap-3"
          >
            <InfoIcon class="w-4 h-4 mt-0.5" />
            <div class="flex-1">
              <div class="font-bold mb-0.5 underline">后置钩子 (After Hook)</div>
              <span>在视觉匹配成功且确认后运行。通常用于处理业务逻辑、状态流转或关闭弹窗。</span>
            </div>
          </div>
          <ActionSequenceEditor v-model:steps="localPolicy.afterAction" />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue';
import { Target, ArrowBigUpDash, ArrowBigDownDash, Info as InfoIcon } from 'lucide-vue-next';
import SearchRuleEditor from './SearchRuleEditor.vue';
import ActionSequenceEditor from './ActionSequenceEditor.vue';

const props = defineProps({
  policy: {
    type: Object,
    required: true,
  },
});

const emit = defineEmits(['save', 'update']);

const activeTab = ref('cond');
const localPolicy = ref({ ...props.policy });

const tabs = [
  { id: 'cond', label: '命中条件', icon: Target },
  { id: 'before', label: '前置动作', icon: ArrowBigUpDash },
  { id: 'after', label: '命中动作', icon: ArrowBigDownDash },
];

const onSave = () => {
  emit('save', localPolicy.value);
};

watch(
  () => props.policy,
  (newVal) => {
    localPolicy.value = { ...newVal };
  },
  { deep: true }
);
</script>
