<template>
  <div class="flex-1 flex flex-col overflow-hidden bg-base-100">
    <!-- Header with Search -->
    <div class="p-4 border-b border-base-300 flex items-center justify-between bg-base-200/50">
      <div class="flex items-center gap-4 flex-1">
        <h2 class="text-xl font-bold flex items-center gap-2">
          <component :is="activeIcon" class="w-6 h-6 text-primary" />
          {{ title }}
        </h2>
        <div class="form-control flex-1 max-w-md">
          <input
            type="text"
            v-model="searchQuery"
            :placeholder="`搜索${title}...`"
            class="input input-bordered w-full h-10"
          />
        </div>
      </div>
      <button class="btn btn-primary btn-sm gap-2" @click="addNewItem">
        <PlusIcon class="w-4 h-4" />
        新建{{ title }}
      </button>
    </div>

    <!-- Main List/Content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Left: List -->
      <div class="w-80 border-r border-base-300 flex flex-col bg-base-100">
        <div v-if="loading" class="flex-1 flex items-center justify-center">
          <span class="loading loading-spinner loading-md"></span>
        </div>
        <div v-else class="flex-1 overflow-y-auto p-2">
          <div
            v-for="item in filteredItems"
            :key="item.id"
            class="p-3 rounded-lg cursor-pointer mb-1 transition-all"
            :class="[
              selectedItem?.id === item.id
                ? 'bg-primary text-primary-content'
                : 'hover:bg-base-200 text-base-content/70',
            ]"
            @click="selectedItem = item"
          >
            <div class="font-bold truncate">{{ item.data.name }}</div>
            <div class="text-xs opacity-60 truncate">{{ item.data.note || '无备注' }}</div>
          </div>

          <div v-if="filteredItems.length === 0" class="text-center py-10 opacity-40">未找到{{ title }}</div>
        </div>
      </div>

      <!-- Right: Details/Composition -->
      <div class="flex-1 bg-base-100 flex flex-col overflow-hidden">
        <div v-if="selectedItem" class="flex-1 flex flex-col p-6 overflow-y-auto">
          <div class="flex justify-between items-start mb-6">
            <div>
              <h3 class="text-2xl font-bold">{{ selectedItem.data.name }}</h3>
              <p class="text-base-content/60 mt-1">{{ selectedItem.data.note }}</p>
            </div>
            <div class="flex gap-2">
              <button class="btn btn-outline btn-sm" @click="editItem">编辑</button>
              <button class="btn btn-error btn-sm btn-outline" @click="deleteItem">删除</button>
            </div>
          </div>

          <!-- Composition View based on tab -->
          <div class="divider">组成结构</div>

          <div v-if="activeTab === 'policy_set'">
            <div class="alert alert-info shadow-sm mb-4">
              <InfoIcon class="w-5 h-5 shrink-0" />
              <span>这里可以编排策略组。</span>
            </div>
            <!-- TODO: Implement actual list of sub-groups -->
          </div>

          <div v-else-if="activeTab === 'policy_group'">
            <div class="alert alert-info shadow-sm mb-4">
              <InfoIcon class="w-5 h-5 shrink-0" />
              <span>这里可以编排策略。</span>
            </div>
            <!-- TODO: Implement actual list of sub-policies -->
          </div>

          <div v-else-if="activeTab === 'policy'">
            <div class="bg-base-200 p-4 rounded-xl">
              <h4 class="font-bold mb-2 text-sm uppercase opacity-50">命中条件</h4>
              <div v-if="selectedItem.data.conditions && selectedItem.data.conditions.length > 0">
                <div
                  v-for="(cond, idx) in selectedItem.data.conditions"
                  :key="idx"
                  class="badge badge-primary mr-2 mb-2"
                >
                  {{ cond }}
                </div>
              </div>
              <div v-else class="opacity-40 italic">暂无命中条件</div>
            </div>
          </div>
        </div>
        <div v-else class="flex-1 flex flex-col items-center justify-center opacity-30">
          <component :is="activeIcon" class="w-20 h-20 mb-4" />
          <p class="text-xl">请从左侧选择一个{{ title }}</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted } from 'vue';
import { Library, LayoutGrid, ListTodo, Plus as PlusIcon, Info as InfoIcon } from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps({
  activeTab: {
    type: String,
    required: true,
  },
  scriptId: {
    type: String,
    required: true,
  },
});

const searchQuery = ref('');
const selectedItem = ref(null);
const items = ref([]);
const loading = ref(false);

const title = computed(() => {
  switch (props.activeTab) {
    case 'policy_set':
      return '策略集';
    case 'policy_group':
      return '策略组';
    case 'policy':
      return '策略';
    default:
      return '';
  }
});

const activeIcon = computed(() => {
  switch (props.activeTab) {
    case 'policy_set':
      return Library;
    case 'policy_group':
      return LayoutGrid;
    case 'policy':
      return ListTodo;
    default:
      return null;
  }
});

const filteredItems = computed(() => {
  if (!searchQuery.value) return items.value;
  const q = searchQuery.value.toLowerCase();
  return items.value.filter(
    (item) => item.data.name.toLowerCase().includes(q) || (item.data.note && item.data.note.toLowerCase().includes(q))
  );
});

const loadData = async () => {
  loading.value = true;
  try {
    let command = '';
    switch (props.activeTab) {
      case 'policy_set':
        command = 'get_all_policy_sets_cmd';
        break;
      case 'policy_group':
        command = 'get_all_policy_groups_cmd';
        break;
      case 'policy':
        command = 'get_all_policies_cmd';
        break;
    }

    if (command) {
      items.value = await invoke(command, { scriptId: props.scriptId });
    }
  } catch (e) {
    console.error('Failed to load policy data:', e);
  } finally {
    loading.value = false;
  }
};

const addNewItem = async () => {
  // Basic implementation for demonstration
  const newItemName = `新${title.value} ${items.value.length + 1}`;
  const id = crypto.randomUUID(); // Placeholder UUID logic

  // In reality, this would open a modal
  const item = {
    id,
    scriptId: props.scriptId,
    data: {
      name: newItemName,
      note: '',
      conditions: [],
    },
  };

  try {
    let command = '';
    switch (props.activeTab) {
      case 'policy_set':
        command = 'save_policy_set_cmd';
        break;
      case 'policy_group':
        command = 'save_policy_group_cmd';
        break;
      case 'policy':
        command = 'save_policy_cmd';
        break;
    }

    await invoke(command, {
      [props.activeTab.replace('policy_', 'set').replace('policy_group', 'group').replace('policy', 'policy')]: item,
    });
    await loadData();
    selectedItem.value = items.value.find((i) => i.id === id);
  } catch (e) {
    console.error('Failed to save item:', e);
  }
};

const deleteItem = async () => {
  if (!selectedItem.value) return;

  try {
    let command = '';
    switch (props.activeTab) {
      case 'policy_set':
        command = 'delete_policy_set_cmd';
        break;
      case 'policy_group':
        command = 'delete_policy_group_cmd';
        break;
      case 'policy':
        command = 'delete_policy_cmd';
        break;
    }

    await invoke(command, { id: selectedItem.value.id });
    selectedItem.value = null;
    await loadData();
  } catch (e) {
    console.error('Failed to delete item:', e);
  }
};

// Reset selection when tab changes
watch(
  () => props.activeTab,
  () => {
    selectedItem.value = null;
    loadData();
  }
);

onMounted(loadData);
</script>
