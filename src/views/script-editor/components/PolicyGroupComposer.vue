<template>
  <div class="flex flex-col gap-4 h-full">
    <!-- 已关联的策略列表 -->
    <div class="flex-1 flex flex-col min-h-0">
      <div class="flex items-center justify-between mb-2">
        <h3 class="text-sm font-bold opacity-70 flex items-center gap-2">
          <LinkIcon class="w-4 h-4 text-primary" />
          已关联的策略
          <span class="badge badge-sm badge-primary">{{ selectedPolicies.length }}</span>
        </h3>
      </div>

      <!-- 已选列表 - 支持拖拽排序 -->
      <div
        class="flex-1 overflow-y-auto border border-base-300 rounded-xl bg-base-200/30 min-h-[120px]"
        @dragover.prevent="onDragOverSelected"
        @drop="onDropSelected"
      >
        <div
          v-if="selectedPolicies.length === 0"
          class="flex items-center justify-center h-full opacity-40 text-sm py-8"
        >
          <span>暂无关联策略，从下方候选列表中添加</span>
        </div>
        <div
          v-for="(policy, index) in selectedPolicies"
          :key="policy.id"
          class="flex items-center gap-2 px-3 py-2 border-b border-base-300/50 last:border-b-0 group transition-all"
          :class="[
            dragOverIndex === index ? 'bg-primary/10 border-t-2 border-t-primary' : 'hover:bg-base-200',
            draggingIndex === index ? 'opacity-30' : '',
          ]"
          draggable="true"
          @dragstart="onDragStart(index, $event)"
          @dragend="onDragEnd"
          @dragover.prevent="onDragOverItem(index)"
          @dragleave="onDragLeaveItem"
        >
          <!-- 拖拽手柄 -->
          <div class="cursor-grab active:cursor-grabbing opacity-30 group-hover:opacity-70 transition-opacity">
            <GripVerticalIcon class="w-4 h-4" />
          </div>

          <!-- 序号 -->
          <span class="text-xs font-mono opacity-40 w-5 text-center shrink-0">{{ index + 1 }}</span>

          <!-- 策略信息 -->
          <div class="flex-1 min-w-0">
            <div class="font-semibold text-sm truncate">{{ policy.data.name || '未命名' }}</div>
            <div v-if="policy.data.note" class="text-xs opacity-50 truncate">{{ policy.data.note }}</div>
          </div>

          <!-- 移除按钮 -->
          <button
            class="btn btn-xs btn-ghost btn-circle opacity-0 group-hover:opacity-70 hover:opacity-100! text-error transition-all"
            @click="removePolicy(index)"
            title="移除"
          >
            <XIcon class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>

    <!-- 分隔线 & 搜索 -->
    <div class="divider my-0 opacity-30"></div>

    <div class="flex-1 flex flex-col min-h-0">
      <div class="flex items-center gap-2 mb-2">
        <h3 class="text-sm font-bold opacity-70 flex items-center gap-2">
          <ListTodoIcon class="w-4 h-4" />
          候选策略
        </h3>
        <div class="flex-1"></div>
        <div class="form-control">
          <input
            type="text"
            v-model="searchQuery"
            placeholder="搜索策略名称或备注..."
            class="input input-bordered input-sm w-56"
          />
        </div>
      </div>

      <!-- 候选列表 -->
      <div class="flex-1 overflow-y-auto border border-base-300 rounded-xl bg-base-200/30 min-h-[120px]">
        <div
          v-if="filteredCandidates.length === 0"
          class="flex items-center justify-center h-full opacity-40 text-sm py-8"
        >
          <span>{{ searchQuery ? '未找到匹配的策略' : '所有策略已关联' }}</span>
        </div>
        <div
          v-for="policy in filteredCandidates"
          :key="policy.id"
          class="flex items-center gap-2 px-3 py-2 border-b border-base-300/50 last:border-b-0 hover:bg-primary/5 group transition-all cursor-pointer"
          @click="addPolicy(policy)"
        >
          <!-- 策略信息 -->
          <div class="flex-1 min-w-0">
            <div class="font-semibold text-sm truncate">{{ policy.data.name || '未命名' }}</div>
            <div v-if="policy.data.note" class="text-xs opacity-50 truncate">{{ policy.data.note }}</div>
          </div>

          <!-- 添加按钮 -->
          <button
            class="btn btn-xs btn-ghost btn-circle opacity-0 group-hover:opacity-70 hover:opacity-100! text-success transition-all"
            title="添加"
          >
            <PlusIcon class="w-3.5 h-3.5" />
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import {
  Link as LinkIcon,
  ListTodo as ListTodoIcon,
  GripVertical as GripVerticalIcon,
  X as XIcon,
  Plus as PlusIcon,
} from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';
import type { PolicyTable } from '@/types/bindings';

const props = defineProps<{
  groupId: string;
  scriptId: string;
}>();

// 所有策略
const allPolicies = ref<PolicyTable[]>([]);
// 已选中策略的 ID 列表（有序）
const selectedPolicyIds = ref<string[]>([]);
// 搜索
const searchQuery = ref('');

// 拖拽状态
const draggingIndex = ref<number | null>(null);
const dragOverIndex = ref<number | null>(null);

// 已选策略（完整对象，保持顺序）
const selectedPolicies = computed(() => {
  return selectedPolicyIds.value
    .map((id) => allPolicies.value.find((p) => p.id === id))
    .filter(Boolean) as PolicyTable[];
});

// 候选策略（未被选中的）
const candidatePolicies = computed(() => {
  const selectedSet = new Set(selectedPolicyIds.value);
  return allPolicies.value.filter((p) => !selectedSet.has(p.id));
});

// 过滤后的候选策略
const filteredCandidates = computed(() => {
  if (!searchQuery.value) return candidatePolicies.value;
  const q = searchQuery.value.toLowerCase();
  return candidatePolicies.value.filter(
    (p) => p.data.name.toLowerCase().includes(q) || (p.data.note && p.data.note.toLowerCase().includes(q))
  );
});

// 加载数据
const loadAllPolicies = async () => {
  try {
    allPolicies.value = await invoke<PolicyTable[]>('get_all_policies_cmd', {
      scriptId: props.scriptId,
    });
  } catch (e) {
    console.error('加载策略列表失败:', e);
  }
};

const loadGroupPolicies = async () => {
  try {
    selectedPolicyIds.value = await invoke<string[]>('get_group_policies_cmd', {
      groupId: props.groupId,
    });
  } catch (e) {
    console.error('加载策略组关联失败:', e);
  }
};

// 操作
const addPolicy = (policy: PolicyTable) => {
  if (!selectedPolicyIds.value.includes(policy.id)) {
    selectedPolicyIds.value.push(policy.id);
  }
};

const removePolicy = (index: number) => {
  selectedPolicyIds.value.splice(index, 1);
};

// 拖拽排序
const onDragStart = (index: number, event: DragEvent) => {
  draggingIndex.value = index;
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
    event.dataTransfer.setData('text/plain', String(index));
  }
};

const onDragEnd = () => {
  draggingIndex.value = null;
  dragOverIndex.value = null;
};

const onDragOverSelected = (event: DragEvent) => {
  event.preventDefault();
};

const onDragOverItem = (index: number) => {
  if (draggingIndex.value !== null && draggingIndex.value !== index) {
    dragOverIndex.value = index;
  }
};

const onDragLeaveItem = () => {
  dragOverIndex.value = null;
};

const onDropSelected = (event: DragEvent) => {
  event.preventDefault();
  if (draggingIndex.value === null || dragOverIndex.value === null) {
    dragOverIndex.value = null;
    return;
  }

  const from = draggingIndex.value;
  const to = dragOverIndex.value;

  if (from !== to) {
    const list = [...selectedPolicyIds.value];
    const [moved] = list.splice(from, 1);
    list.splice(to, 0, moved);
    selectedPolicyIds.value = list;
  }

  draggingIndex.value = null;
  dragOverIndex.value = null;
};

// 保存方法（暴露给父组件）
const saveGroupPolicies = async () => {
  await invoke('update_group_policies_cmd', {
    groupId: props.groupId,
    policyIds: selectedPolicyIds.value,
  });
};

// 获取当前数据（暴露给父组件，与 saveScript 集成）
const getComposerData = () => ({
  groupId: props.groupId,
  policyIds: [...selectedPolicyIds.value],
});

defineExpose({
  saveGroupPolicies,
  getComposerData,
});

// 监听 groupId 变化时重新加载
watch(
  () => props.groupId,
  async () => {
    if (props.groupId) {
      await loadGroupPolicies();
    }
  }
);

onMounted(async () => {
  await loadAllPolicies();
  if (props.groupId) {
    await loadGroupPolicies();
  }
});
</script>
