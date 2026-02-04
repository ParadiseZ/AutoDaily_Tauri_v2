<template>
  <div class="policy-editor flex flex-col h-full overflow-hidden">
    <!-- Metadata Header -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4 p-4 bg-base-200/30 rounded-xl mb-6">
      <div class="form-control">
        <label class="label py-1"><span class="label-text font-bold">策略名称</span></label>
        <input v-model="localPolicy.name" type="text" class="input input-bordered w-full" />
      </div>
      <div class="form-control">
        <label class="label py-1"><span class="label-text font-bold">备注说明</span></label>
        <input
          v-model="localPolicy.note"
          type="text"
          class="input input-bordered w-full"
          placeholder="简要描述此策略的用途..."
        />
      </div>
      <div class="form-control md:col-span-2">
        <label class="label py-1"><span class="label-text font-bold">日志打印 (Hit Log)</span></label>
        <input
          v-model="localPolicy.logPrint"
          type="text"
          class="input input-bordered w-full font-mono text-sm"
          placeholder="当命中该策略时打印的自定义日志内容..."
        />
      </div>

      <div class="form-control">
        <label class="label py-1"><span class="label-text font-bold">点击索引 (CurPos)</span></label>
        <input v-model.number="localPolicy.curPos" type="number" class="input input-bordered w-full" />
      </div>
      <div class="form-control">
        <label class="label py-1"><span class="label-text font-bold">最大次数 (ExecMax)</span></label>
        <input v-model.number="localPolicy.execMax" type="number" class="input input-bordered w-full" />
      </div>
    </div>

    <!-- Tabs for Condition and Actions -->
    <div class="tabs tabs-boxed mb-4">
      <a class="tab" :class="{ 'tab-active': activeTab === 'cond' }" @click="activeTab = 'cond'">命中条件 (Cond)</a>
      <a class="tab" :class="{ 'tab-active': activeTab === 'before' }" @click="activeTab = 'before'"
        >前置动作 (Before)</a
      >
      <a class="tab" :class="{ 'tab-active': activeTab === 'after' }" @click="activeTab = 'after'">命中动作 (After)</a>
    </div>

    <!-- Tab Contents -->
    <div class="flex-1 overflow-y-auto pr-2 pb-10">
      <div v-show="activeTab === 'cond'" class="p-1">
        <SearchRuleEditor :rule="localPolicy.cond" @update="localPolicy.cond = $event" />
      </div>

      <div v-show="activeTab === 'before'" class="p-1">
        <div class="alert alert-info text-xs mb-4">
          <span>在匹配 `Cond` 的视觉搜索执行**之前**执行的操作。</span>
        </div>
        <ActionSequenceEditor v-model:steps="localPolicy.beforeAction" @add-step="addBeforeStep" />
      </div>

      <div v-show="activeTab === 'after'" class="p-1">
        <div class="alert alert-success text-xs mb-4">
          <span>在 `Cond` 成功匹配并执行确认动作（如有）**之后**执行的操作。</span>
        </div>
        <ActionSequenceEditor v-model:steps="localPolicy.afterAction" @add-step="addAfterStep" />
      </div>
    </div>

    <!-- Save Button Floating (or managed by parent) -->
    <div class="absolute bottom-6 right-6">
      <button class="btn btn-primary shadow-lg" @click="onSave">保存当前修改</button>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue';
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

const addBeforeStep = (idx) => {
  localPolicy.value.beforeAction.splice(idx, 0, createDefaultStep());
};

const addAfterStep = (idx) => {
  localPolicy.value.afterAction.splice(idx, 0, createDefaultStep());
};

const createDefaultStep = () => ({
  op: 'WaitMs',
  label: '',
  ms: 1000,
  execMax: 0,
  skipFlag: false,
});

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
