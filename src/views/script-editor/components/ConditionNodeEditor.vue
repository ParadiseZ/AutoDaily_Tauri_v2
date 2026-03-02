<template>
  <div class="condition-node-editor space-y-4">
    <!-- Group Settings -->
    <div
      v-if="localCondition.type === 'group'"
      class="bg-base-200/40 rounded-3xl p-4 border border-base-300 shadow-inner"
    >
      <div class="flex flex-wrap items-center gap-4 mb-3">
        <div class="join bg-base-100 p-1 rounded-2xl shadow-sm border border-base-300">
          <select class="select select-ghost select-sm join-item w-20 text-xs font-bold" v-model="localCondition.op">
            <option value="And">AND</option>
            <option value="Or">OR</option>
            <option value="Not">NOT</option>
          </select>
        </div>
        <div class="badge badge-sm badge-outline opacity-40 rounded-md font-mono">
          {{ localCondition.items?.length || 0 }} 项
        </div>
        <button v-if="!isRoot" class="btn btn-ghost btn-xs text-error ml-auto" @click="$emit('remove')">
          移除此组
        </button>
      </div>

      <!-- Rule Items -->
      <div class="space-y-3">
        <div v-for="(item, idx) in localCondition.items" :key="idx" class="relative">
          <div class="pl-4 border-l-2 border-base-300 relative group">
            <div class="absolute -left-2 top-3 w-4 border-t-2 border-base-300"></div>
            <ConditionNodeEditor
              :condition="item as any"
              :isRoot="false"
              @update="updateNestedCondition(idx as number, $event)"
              @remove="removeConditionItem(idx as number)"
            />
          </div>
        </div>

        <!-- Add Rule Type Selector for Group -->
        <div class="grid grid-cols-2 md:grid-cols-5 gap-2 pt-2 mt-4 border-t border-base-300/50">
          <button class="btn btn-xs btn-outline btn-ghost" @click="addCondition('rawExpr')">Rhai 表达式</button>
          <button class="btn btn-xs btn-outline btn-ghost" @click="addCondition('execNumCompare')">执行次数</button>
          <button class="btn btn-xs btn-outline btn-ghost" @click="addCondition('taskStatus')">任务状态</button>
          <button class="btn btn-xs btn-outline btn-ghost" @click="addCondition('colorCompare')">色彩对比</button>
          <button class="btn btn-xs btn-outline btn-ghost" @click="addCondition('varCompare')">变量对比</button>
          <button class="btn btn-xs btn-outline border-primary/50 text-primary" @click="addCondition('group')">
            + 逻辑子组
          </button>
        </div>
      </div>
    </div>

    <!-- Leaf Item: Single Condition -->
    <div
      v-else
      class="bg-base-100 p-3 rounded-xl border border-base-300 hover:border-primary/30 transition-all flex flex-col gap-3 group relative"
    >
      <!-- Header -->
      <div class="flex justify-between items-center text-xs font-bold text-base-content/50 uppercase tracing-wide">
        <span>{{ displayConditionType(localCondition.type) }}</span>
        <div class="flex items-center gap-2">
          <button
            class="btn btn-ghost btn-xs btn-circle text-error opacity-0 group-hover:opacity-100 transition-opacity"
            @click="$emit('remove')"
            v-if="!isRoot"
          >
            ×
          </button>
        </div>
      </div>

      <!-- Details Editor -->
      <div class="space-y-2">
        <!-- RawExpr -->
        <div v-if="localCondition.type === 'rawExpr'">
          <input
            type="text"
            v-model="localCondition.expr"
            placeholder="输入 rhai 表达式 (例: count > 10)"
            class="input input-sm input-bordered w-full font-mono text-xs"
          />
        </div>

        <!-- ExecNumCompare -->
        <div v-else-if="localCondition.type === 'execNumCompare'" class="flex gap-2">
          <select class="select select-bordered select-sm flex-1" v-model="localCondition.a.type">
            <option value="policy">策略 (Policy)</option>
            <option value="task">任务 (Task)</option>
          </select>
          <select
            v-if="localCondition.a.type === 'policy'"
            class="select select-bordered select-sm w-32"
            v-model="localCondition.a.id"
          >
            <option v-for="p in policies" :key="p.id" :value="p.id">{{ p.data?.name || '未命名策略' }}</option>
          </select>
          <select
            v-else-if="localCondition.a.type === 'task'"
            class="select select-bordered select-sm w-32"
            v-model="localCondition.a.id"
          >
            <option v-for="t in tasks" :key="t.id" :value="t.id">{{ t.name || '未命名任务' }}</option>
          </select>
        </div>

        <!-- TaskStatus -->
        <div v-else-if="localCondition.type === 'taskStatus'" class="space-y-2">
          <div class="flex gap-2">
            <select class="select select-bordered select-sm flex-1" v-model="localCondition.a.type">
              <option value="getState">取状态</option>
              <option value="setState">设状态</option>
            </select>
            <select class="select select-bordered select-sm flex-1" v-model="localCondition.a.target.type">
              <option value="policy">Policy</option>
              <option value="task">Task</option>
            </select>
            <select
              v-if="localCondition.a.target.type === 'policy'"
              class="select select-bordered select-sm w-24"
              v-model="localCondition.a.target.id"
            >
              <option v-for="p in policies" :key="p.id" :value="p.id">{{ p.data?.name || '未命名' }}</option>
            </select>
            <select
              v-else-if="localCondition.a.target.type === 'task'"
              class="select select-bordered select-sm w-24"
              v-model="localCondition.a.target.id"
            >
              <option v-for="t in tasks" :key="t.id" :value="t.id">{{ t.name || '未命名' }}</option>
            </select>
          </div>
          <div class="flex gap-2 items-center">
            <select class="select select-bordered select-sm flex-1" v-model="localCondition.a.status.type">
              <option value="skip">跳过状态</option>
              <option value="done">完成状态</option>
            </select>
            <label class="cursor-pointer flex items-center gap-2 pr-2">
              <input type="checkbox" v-model="localCondition.a.status.value" class="checkbox checkbox-sm" />
              <span class="text-xs">预期</span>
            </label>
          </div>
        </div>

        <!-- ColorCompare -->
        <div v-else-if="localCondition.type === 'colorCompare'" class="space-y-2 flex flex-col">
          <input
            type="text"
            v-model="localCondition.txtTarget"
            placeholder="定位文本..."
            class="input input-sm input-bordered w-full text-xs"
          />
          <div class="flex gap-2 items-center">
            <label class="cursor-pointer flex items-center space-x-2 text-xs">
              <input type="radio" :value="true" v-model="localCondition.isFont" class="radio radio-xs" />
              <span>字体色</span>
            </label>
            <label class="cursor-pointer flex items-center space-x-2 text-xs ml-2">
              <input type="radio" :value="false" v-model="localCondition.isFont" class="radio radio-xs" />
              <span>背景色</span>
            </label>
            <input
              type="number"
              v-model="localCondition.r"
              min="0"
              max="255"
              placeholder="R"
              class="input input-xs input-bordered w-14"
            />
            <input
              type="number"
              v-model="localCondition.g"
              min="0"
              max="255"
              placeholder="G"
              class="input input-xs input-bordered w-14"
            />
            <input
              type="number"
              v-model="localCondition.b"
              min="0"
              max="255"
              placeholder="B"
              class="input input-xs input-bordered w-14"
            />
          </div>
        </div>

        <!-- VarCompare -->
        <div v-else-if="localCondition.type === 'varCompare'" class="space-y-2">
          <input
            type="text"
            v-model="localCondition.varName"
            placeholder="变量名..."
            class="input input-sm input-bordered w-full text-xs"
          />
          <div class="flex gap-1">
            <select class="select select-bordered select-sm w-20" v-model="localCondition.op">
              <option value="eq">==</option>
              <option value="ne">!=</option>
              <option value="lt">&lt;</option>
              <option value="le">&lt;=</option>
              <option value="gt">&gt;</option>
              <option value="ge">&gt;=</option>
              <option value="contains">包含</option>
              <option value="notContains">不包含</option>
            </select>
            <select
              class="select select-bordered select-sm w-20"
              v-model="localCondition.value.type"
              @change="onVarCompareTypeChange"
            >
              <option value="int">Int</option>
              <option value="float">Float</option>
              <option value="bool">Bool</option>
              <option value="string">String</option>
            </select>
            <input
              v-if="localCondition.value.type === 'int'"
              type="number"
              v-model="localCondition.value.value"
              class="input input-sm input-bordered flex-1"
            />
            <input
              v-else-if="localCondition.value.type === 'float'"
              type="number"
              step="0.1"
              v-model="localCondition.value.value"
              class="input input-sm input-bordered flex-1"
            />
            <input
              v-else-if="localCondition.value.type === 'string'"
              type="text"
              v-model="localCondition.value.value"
              class="input input-sm input-bordered flex-1"
            />
            <div v-else-if="localCondition.value.type === 'bool'" class="flex items-center pl-2 flex-1">
              <input type="checkbox" v-model="localCondition.value.value" class="checkbox checkbox-sm" />
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, inject, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const scriptInfo: any = inject('scriptInfo', ref(null));
const policies = ref<any[]>([]);
const tasks = ref<any[]>([]);

const loadData = async () => {
  if (!scriptInfo.value?.id) return;
  try {
    const rawPolicies = await invoke<any[]>('get_all_policies_cmd', { scriptId: scriptInfo.value.id });
    policies.value = rawPolicies || [];
  } catch (e) {
    alert(e);
  }
  try {
    const rawTasks = await invoke<any[]>('get_script_tasks_cmd', { scriptId: scriptInfo.value.id });
    tasks.value = rawTasks || [];
  } catch (e) {
    alert(e);
  }
};

onMounted(() => {
  loadData();
});
watch(
  () => scriptInfo.value?.id,
  () => loadData()
);

const props = defineProps<{
  condition: any;
  isRoot?: boolean;
}>();

const emit = defineEmits<{
  (e: 'update', condition: any): void;
  (e: 'remove'): void;
}>();

const ensureConditionStructure = (cond: any) => {
  if (!cond) return { type: 'rawExpr', expr: '' };
  if (cond.type === 'execNumCompare') {
    if (!cond.a) cond.a = { type: 'task', id: '' };
  } else if (cond.type === 'taskStatus') {
    if (!cond.a) cond.a = { type: 'getState', target: { type: 'task', id: '' }, status: { type: 'done', value: true } };
    if (!cond.a.target) cond.a.target = { type: 'task', id: '' };
    if (!cond.a.status) cond.a.status = { type: 'done', value: true };
  } else if (cond.type === 'varCompare') {
    if (!cond.value) cond.value = { type: 'string', value: '' };
  } else if (cond.type === 'colorCompare') {
    if (cond.isFont === undefined) cond.isFont = true;
  }
  return cond;
};

const localCondition = ref(
  ensureConditionStructure(JSON.parse(JSON.stringify(props.condition || { type: 'rawExpr', expr: '' })))
);

const onUpdate = () => {
  emit('update', localCondition.value);
};

const displayConditionType = (type: string) => {
  switch (type) {
    case 'rawExpr':
      return 'Rhai 表达式';
    case 'execNumCompare':
      return '执行次数比照';
    case 'taskStatus':
      return '任务状态核对';
    case 'colorCompare':
      return 'OCR 颜色检测';
    case 'varCompare':
      return '变量比照';
    default:
      return type;
  }
};

const addCondition = (type: string) => {
  if (localCondition.value.type !== 'group') return;
  if (!localCondition.value.items) localCondition.value.items = [];

  let newCond: any = { type };
  if (type === 'rawExpr') newCond = { type, expr: '' };
  else if (type === 'execNumCompare') newCond = { type, a: { type: 'task', id: '' } };
  else if (type === 'taskStatus')
    newCond = {
      type,
      a: { type: 'getState', target: { type: 'task', id: '' }, status: { type: 'done', value: true } },
    };
  else if (type === 'colorCompare') newCond = { type, txtTarget: '', isFont: true, r: 0, g: 0, b: 0 };
  else if (type === 'varCompare') newCond = { type, varName: '', op: 'eq', value: { type: 'string', value: '' } };
  else if (type === 'group') newCond = { type, op: 'And', items: [] };

  localCondition.value.items.push(newCond);
  onUpdate();
};

const removeConditionItem = (idx: number) => {
  localCondition.value.items.splice(idx, 1);
  onUpdate();
};

const updateNestedCondition = (idx: number, newCond: any) => {
  localCondition.value.items[idx] = newCond;
  onUpdate();
};

const onVarCompareTypeChange = () => {
  const t = localCondition.value.value.type;
  if (t === 'int' || t === 'float') localCondition.value.value.value = 0;
  else if (t === 'bool') localCondition.value.value.value = true;
  else localCondition.value.value.value = '';
  onUpdate();
};

watch(localCondition, onUpdate, { deep: true });
watch(
  () => props.condition,
  (newVal) => {
    const parsed = ensureConditionStructure(JSON.parse(JSON.stringify(newVal || { type: 'rawExpr', expr: '' })));
    if (JSON.stringify(parsed) !== JSON.stringify(localCondition.value)) {
      localCondition.value = parsed;
    }
  },
  { deep: true }
);
</script>
