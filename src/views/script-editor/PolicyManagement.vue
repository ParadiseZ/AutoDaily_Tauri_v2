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
          <div class="input-group">
            <input
              type="text"
              v-model="searchQuery"
              :placeholder="`搜索${title}...`"
              class="input input-bordered w-full h-10"
            />
          </div>
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
        <div class="flex-1 overflow-y-auto p-2">
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
            <!-- Policy Set specific content -->
            <div class="alert alert-info shadow-sm mb-4">
              <InfoIcon class="w-5 h-5 shrink-0" />
              <span>这里可以编排策略组。</span>
            </div>
            <!-- TODO: Implement Group Selection and Reordering -->
          </div>

          <div v-else-if="activeTab === 'policy_group'">
            <!-- Policy Group specific content -->
            <div class="alert alert-info shadow-sm mb-4">
              <InfoIcon class="w-5 h-5 shrink-0" />
              <span>这里可以编排策略。</span>
            </div>
            <!-- TODO: Implement Policy Selection and Reordering -->
          </div>

          <div v-else-if="activeTab === 'policy'">
            <!-- Policy specific content -->
            <div class="bg-base-200 p-4 rounded-xl">
              <h4 class="font-bold mb-2">命中条件</h4>
              <div v-if="selectedItem.data.conditions && selectedItem.data.conditions.length > 0">
                <div v-for="(cond, idx) in selectedItem.data.conditions" :key="idx" class="badge badge-outline mr-2">
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
import { ref, computed, watch } from 'vue';
import { Library, LayoutGrid, ListTodo, Plus as PlusIcon, Info as InfoIcon } from 'lucide-vue-next';

const props = defineProps({
  activeTab: {
    type: String,
    required: true,
  },
});

const searchQuery = ref('');
const selectedItem = ref(null);
const items = ref([]); // This will be fetched from backend

const title = computed(() => {
  switch (props.activeTab) {
    case 'policy_set':
      return '策略集合';
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
  return items.value.filter(
    (item) =>
      item.data.name.toLowerCase().includes(searchQuery.value.toLowerCase()) ||
      (item.data.note && item.data.note.toLowerCase().includes(searchQuery.value.toLowerCase()))
  );
});

// Reset selection when tab changes
watch(
  () => props.activeTab,
  () => {
    selectedItem.value = null;
    loadData();
  }
);

const loadData = async () => {
  // TODO: Implement actual backend fetching based on activeTab
  console.log('Loading data for', props.activeTab);
  // Mock data for now
  items.value = [
    {
      id: '1',
      data: { name: `示例${title.value} 1`, note: '这是一个备注说明', conditions: ['Condition A', 'Condition B'] },
    },
    { id: '2', data: { name: `示例${title.value} 2`, note: '另一个详细的备注', conditions: [] } },
  ];
};

const addNewItem = () => {
  console.log('Adding new', props.activeTab);
  // TODO: Implement modal for adding
};

const editItem = () => {
  console.log('Editing', selectedItem.value);
};

const deleteItem = () => {
  console.log('Deleting', selectedItem.value);
};

loadData();
</script>
