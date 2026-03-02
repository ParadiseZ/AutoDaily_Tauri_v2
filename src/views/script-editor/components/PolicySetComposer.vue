<template>
  <div class="flex flex-col gap-4 h-full">
    <!-- 已关联的策略组列表 -->
    <div class="flex-1 flex flex-col min-h-0">
      <div class="flex items-center justify-between mb-2">
        <h3 class="text-sm font-bold opacity-70 flex items-center gap-2">
          <LinkIcon class="w-4 h-4 text-primary" />
          已关联的策略组
          <span class="badge badge-sm badge-primary">{{ selectedGroups.length }}</span>
        </h3>
      </div>

      <!-- 已选列表 - 支持指针拖拽排序 -->
      <div
        ref="listContainerRef"
        class="flex-1 overflow-y-auto border border-base-300 rounded-xl bg-base-200/30 min-h-[120px] relative select-none"
      >
        <div v-if="selectedGroups.length === 0" class="flex items-center justify-center h-full opacity-40 text-sm py-8">
          <span>暂无关联策略组，从下方候选列表中添加</span>
        </div>
        <div
          v-for="(group, index) in selectedGroups"
          :key="group.id"
          class="flex items-center gap-2 px-3 py-2 border-b border-base-300/50 last:border-b-0 group transition-colors"
          :class="[
            dragOverIndex === index && draggingIndex !== index
              ? 'bg-primary/10 border-t-2 border-t-primary'
              : 'hover:bg-base-200',
            draggingIndex === index ? 'opacity-30 bg-base-300/30' : '',
          ]"
        >
          <!-- 拖拽手柄 -->
          <div
            class="cursor-grab active:cursor-grabbing opacity-30 group-hover:opacity-70 transition-opacity touch-none"
            @pointerdown="onPointerDown(index, $event)"
          >
            <GripVerticalIcon class="w-4 h-4" />
          </div>

          <!-- 序号 -->
          <span class="text-xs font-mono opacity-40 w-5 text-center shrink-0">{{ index + 1 }}</span>

          <!-- 策略组信息 -->
          <div class="flex-1 min-w-0">
            <div class="font-semibold text-sm truncate">{{ group.data.name || '未命名' }}</div>
            <div v-if="group.data.note" class="text-xs opacity-50 truncate">{{ group.data.note }}</div>
          </div>

          <!-- 移除按钮 -->
          <button
            class="btn btn-xs btn-ghost btn-circle opacity-0 group-hover:opacity-70 hover:opacity-100! text-error transition-all"
            @click="removeGroup(index)"
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
          <LayoutGridIcon class="w-4 h-4" />
          候选策略组
        </h3>
        <div class="flex-1"></div>
        <div class="form-control">
          <input
            type="text"
            v-model="searchQuery"
            placeholder="搜索策略组名称或备注..."
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
          <span>{{ searchQuery ? '未找到匹配的策略组' : '所有策略组已关联' }}</span>
        </div>
        <div
          v-for="group in filteredCandidates"
          :key="group.id"
          class="flex items-center gap-2 px-3 py-2 border-b border-base-300/50 last:border-b-0 hover:bg-primary/5 group transition-all cursor-pointer"
          @click="addGroup(group)"
        >
          <!-- 策略组信息 -->
          <div class="flex-1 min-w-0">
            <div class="font-semibold text-sm truncate">{{ group.data.name || '未命名' }}</div>
            <div v-if="group.data.note" class="text-xs opacity-50 truncate">{{ group.data.note }}</div>
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
  LayoutGrid as LayoutGridIcon,
  GripVertical as GripVerticalIcon,
  X as XIcon,
  Plus as PlusIcon,
} from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';
import type { PolicyGroupTable } from '@/types/bindings';

const props = defineProps<{
  setId: string;
  scriptId: string;
}>();

// 所有策略组
const allGroups = ref<PolicyGroupTable[]>([]);
// 已选中策略组的 ID 列表（有序）
const selectedGroupIds = ref<string[]>([]);
// 搜索
const searchQuery = ref('');

// 拖拽状态
const draggingIndex = ref<number | null>(null);
const dragOverIndex = ref<number | null>(null);
const listContainerRef = ref<HTMLElement | null>(null);

// 已选策略组（完整对象，保持顺序）
const selectedGroups = computed(() => {
  return selectedGroupIds.value
    .map((id) => allGroups.value.find((g) => g.id === id))
    .filter(Boolean) as PolicyGroupTable[];
});

// 候选策略组（未被选中的）
const candidateGroups = computed(() => {
  const selectedSet = new Set(selectedGroupIds.value);
  return allGroups.value.filter((g) => !selectedSet.has(g.id));
});

// 过滤后的候选策略组
const filteredCandidates = computed(() => {
  if (!searchQuery.value) return candidateGroups.value;
  const q = searchQuery.value.toLowerCase();
  return candidateGroups.value.filter(
    (g) => g.data.name.toLowerCase().includes(q) || (g.data.note && g.data.note.toLowerCase().includes(q))
  );
});

// 加载数据
const loadAllGroups = async () => {
  try {
    allGroups.value = await invoke<PolicyGroupTable[]>('get_all_policy_groups_cmd', {
      scriptId: props.scriptId,
    });
  } catch (e) {
    console.error('加载策略组列表失败:', e);
  }
};

const loadSetGroups = async () => {
  try {
    selectedGroupIds.value = await invoke<string[]>('get_set_groups_cmd', {
      setId: props.setId,
    });
  } catch (e) {
    console.error('加载策略集关联失败:', e);
  }
};

// 操作
const addGroup = (group: PolicyGroupTable) => {
  if (!selectedGroupIds.value.includes(group.id)) {
    selectedGroupIds.value.push(group.id);
  }
};

const removeGroup = (index: number) => {
  selectedGroupIds.value.splice(index, 1);
};

// ============ 基于 Pointer Events 的拖拽排序 ============

const getItemIndexFromPoint = (y: number): number | null => {
  if (!listContainerRef.value) return null;
  const children = listContainerRef.value.children;
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
      const list = [...selectedGroupIds.value];
      const [moved] = list.splice(draggingIndex.value, 1);
      list.splice(dragOverIndex.value, 0, moved);
      selectedGroupIds.value = list;
    }

    draggingIndex.value = null;
    dragOverIndex.value = null;
    document.removeEventListener('pointermove', onPointerMove);
    document.removeEventListener('pointerup', onPointerUp);
  };

  document.addEventListener('pointermove', onPointerMove);
  document.addEventListener('pointerup', onPointerUp);
};

// 保存方法（暴露给父组件）
const saveSetGroups = async () => {
  await invoke('update_set_groups_cmd', {
    setId: props.setId,
    groupIds: selectedGroupIds.value,
  });
};

const getComposerData = () => ({
  setId: props.setId,
  groupIds: [...selectedGroupIds.value],
});

defineExpose({
  saveSetGroups,
  getComposerData,
});

// 监听 setId 变化时重新加载
watch(
  () => props.setId,
  async () => {
    if (props.setId) {
      await loadSetGroups();
    }
  }
);

onMounted(async () => {
  await loadAllGroups();
  if (props.setId) {
    await loadSetGroups();
  }
});
</script>
