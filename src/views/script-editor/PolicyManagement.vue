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
      <div class="flex items-center gap-2">
        <!-- Batch Mode Toggle -->
        <label class="label cursor-pointer gap-2">
          <span class="label-text text-xs">批量</span>
          <input type="checkbox" v-model="batchMode" class="checkbox checkbox-sm checkbox-primary" />
        </label>
        <!-- Delete Button (visible when item selected or batch mode) -->
        <button
          v-if="batchMode ? selectedItems.length > 0 : selectedItem"
          class="btn btn-error btn-sm btn-outline gap-1"
          @click="batchMode ? batchDeleteItems() : deleteItem()"
        >
          <TrashIcon class="w-4 h-4" />
          删除{{ batchMode ? `(${selectedItems.length})` : '' }}
        </button>
        <button class="btn btn-primary btn-sm gap-2" @click="addNewItem">
          <PlusIcon class="w-4 h-4" />
          新建{{ title }}
        </button>
      </div>
    </div>

    <!-- Main List/Content -->
    <div class="flex-1 flex overflow-hidden">
      <!-- Left: List -->
      <div class="w-40 border-r border-base-300 flex flex-col bg-base-100">
        <div v-if="loading" class="flex-1 flex items-center justify-center">
          <span class="loading loading-spinner loading-md"></span>
        </div>
        <div v-else class="flex-1 overflow-y-auto p-2">
          <div
            v-for="item in filteredItems"
            :key="item.id"
            class="p-2 rounded-lg cursor-pointer mb-1 transition-all flex items-center gap-2 group"
            :class="[
              batchMode && selectedItems.includes(item.id)
                ? 'bg-error/20 text-error ring-1 ring-error/30'
                : selectedItem?.id === item.id
                  ? 'bg-primary text-primary-content shadow-md'
                  : 'hover:bg-base-200 text-base-content/70',
            ]"
            @click="batchMode ? toggleBatchSelect(item.id) : (selectedItem = item)"
          >
            <div class="flex-1 min-w-0">
              <input
                v-if="selectedItem?.id === item.id && !batchMode"
                v-model="item.data.name"
                class="input input-sm w-full font-bold focus:outline-none focus:ring-2 focus:ring-white/50"
                :class="
                  selectedItem?.id === item.id
                    ? 'bg-primary-focus text-primary-content placeholder-primary-content/50 border-none'
                    : 'bg-transparent text-base-content'
                "
                placeholder="请输入策略名称..."
                @change="updateSelectedItemData(selectedItem?.data)"
                @click.stop
              />
              <div v-else class="font-bold truncate px-2 py-1">
                {{ item.data.name || '未命名' }}
                <span v-if="(item as any).isNew" class="badge badge-xs badge-secondary ml-1">草案</span>
              </div>
            </div>
          </div>

          <div v-if="filteredItems.length === 0" class="text-center py-10 opacity-40">未找到{{ title }}</div>
        </div>
      </div>

      <!-- Right: Details/Composition -->
      <div class="flex-1 bg-base-100 flex flex-col overflow-hidden">
        <div v-if="selectedItem" class="flex-1 flex flex-col p-6 overflow-y-auto">
          <!-- Composition View based on tab -->

          <div v-if="activeTab === 'policy_set'" class="flex-1 flex flex-col overflow-hidden">
            <PolicySetComposer ref="policySetComposerRef" :set-id="selectedItem.id" :script-id="scriptId" />
          </div>

          <div v-else-if="activeTab === 'policy_group'" class="flex-1 flex flex-col overflow-hidden">
            <PolicyGroupComposer ref="policyGroupComposerRef" :group-id="selectedItem.id" :script-id="scriptId" />
          </div>

          <div v-else-if="activeTab === 'policy'">
            <PolicyEditor
              ref="policyEditorRef"
              v-if="selectedItem"
              :policy="selectedItem.data as PolicyInfo"
              @save="updateSelectedItemData"
            />
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

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue';
import {
  Library,
  LayoutGrid,
  ListTodo,
  Plus as PlusIcon,
  Info as InfoIcon,
  Trash2 as TrashIcon,
} from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';
import PolicyEditor from './components/PolicyEditor.vue';
import PolicyGroupComposer from './components/PolicyGroupComposer.vue';
import PolicySetComposer from './components/PolicySetComposer.vue';
import type { PolicyTable, PolicyGroupTable, PolicySetTable, PolicyInfo } from '@/types/bindings';

type ItemTable = (PolicyTable | PolicyGroupTable | PolicySetTable) & { isNew?: boolean };

const props = defineProps({
  activeTab: {
    type: String,
    required: true,
  },
  scriptId: {
    type: String,
    required: true,
  },
  addLog: {
    type: Function,
    default: () => {},
  },
  logLevels: {
    type: Object,
    required: true,
  },
  getUuidV7: {
    type: Function,
    default: () => {},
  },
});

const searchQuery = ref('');
const selectedItem = ref<ItemTable | null>(null);
const items = ref<ItemTable[]>([]);
const loading = ref(false);
const batchMode = ref(false);
const selectedItems = ref<string[]>([]);
const policyEditorRef = ref<any>(null);
const policyGroupComposerRef = ref<any>(null);
const policySetComposerRef = ref<any>(null);

const toggleBatchSelect = (id: string) => {
  const idx = selectedItems.value.indexOf(id);
  if (idx > -1) {
    selectedItems.value.splice(idx, 1);
  } else {
    selectedItems.value.push(id);
  }
};

const batchDeleteItems = async () => {
  if (selectedItems.value.length === 0) return;

  const idsToDelete = [...selectedItems.value];
  const deleteCount = idsToDelete.length;

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

    const newIds: string[] = [];
    const persistedIds: string[] = [];
    for (const id of idsToDelete) {
      const item = items.value.find((i) => i.id === id);
      if (item?.isNew) {
        newIds.push(id);
      } else {
        persistedIds.push(id);
      }
    }

    for (const id of persistedIds) {
      await invoke(command, { id });
    }

    if (newIds.length > 0) {
      items.value = items.value.filter((i) => !newIds.includes(i.id));
    }

    if (selectedItem.value && idsToDelete.includes(selectedItem.value.id)) {
      selectedItem.value = null;
    }
    selectedItems.value = [];

    if (persistedIds.length > 0) {
      const remainingNewItems = items.value.filter((i) => i.isNew);
      await loadData();
      items.value.push(...remainingNewItems);
    }

    props.addLog(`批量删除了 ${deleteCount} 个${title.value}`, props.logLevels.INFO);
  } catch (e) {
    props.addLog(`批量删除失败: ${e}`, props.logLevels.ERROR);
  }
};

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
    (item) =>
      item.data.name.toLowerCase().includes(q) ||
      ((item.data as any).note && (item.data as any).note.toLowerCase().includes(q))
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
      items.value = await invoke<ItemTable[]>(command, { scriptId: props.scriptId });
    }
  } catch (e) {
    props.addLog(`加载策略数据失败: ${e}`, props.logLevels.ERROR);
  } finally {
    loading.value = false;
  }
};

const addNewItem = async () => {
  const id = await props.getUuidV7();
  const newItemName = `新${title.value} ${items.value.length + 1}`;

  let itemData: any = {
    name: newItemName,
    note: '',
  };

  if (props.activeTab === 'policy') {
    itemData = {
      ...itemData,
      logPrint: '',
      curPos: 0,
      skipFlag: false,
      execCur: 0,
      execMax: 0,
      beforeAction: [],
      cond: { type: 'group', op: 'And', scope: 'Global', items: [] },
      afterAction: [],
    };
  } else if (props.activeTab === 'policy_group') {
    itemData = { ...itemData, policies: [] };
  } else if (props.activeTab === 'policy_set') {
    itemData = { ...itemData, groups: [] };
  }

  const newItem: ItemTable = {
    id,
    scriptId: props.scriptId,
    orderIndex: items.value.length + 1,
    data: itemData,
    isNew: true,
  } as ItemTable;

  items.value.push(newItem);
  selectedItem.value = newItem;
  props.addLog(`创建了新${title.value}草案`, props.logLevels.INFO);

  if (props.activeTab === 'policy') {
    await nextTick();
    const inputs = document.querySelectorAll('input[placeholder="请输入策略名称..."]');
    if (inputs && inputs.length > 0) {
      (inputs[inputs.length - 1] as HTMLInputElement).focus();
      (inputs[inputs.length - 1] as HTMLInputElement).select();
    }
  }
};

const updateSelectedItemData = async (newData: any) => {
  if (!selectedItem.value) return;
  const updatedItem = {
    ...selectedItem.value,
    data: { ...newData },
  };

  let command = '';
  let arg = '';
  switch (props.activeTab) {
    case 'policy_set':
      command = 'save_policy_set_cmd';
      arg = 'set';
      break;
    case 'policy_group':
      command = 'save_policy_group_cmd';
      arg = 'group';
      break;
    case 'policy':
      command = 'save_policy_cmd';
      arg = 'policy';
      break;
  }

  await invoke(command, { [arg]: updatedItem });
  await loadData();
  selectedItem.value = items.value.find((i) => i.id === updatedItem.id) || null;
};

const deleteItem = async () => {
  if (!selectedItem.value) return;
  const id = selectedItem.value.id;
  if (selectedItem.value.isNew) {
    items.value = items.value.filter((i) => i.id !== id);
    selectedItem.value = null;
    return;
  }

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

    await invoke(command, { id });
    selectedItem.value = null;
    await loadData();
  } catch (e) {
    props.addLog(`删除项目失败: ${e}`, props.logLevels.ERROR);
  }
};

const saveCurrentPolicy = async () => {
  if (!selectedItem.value || props.activeTab !== 'policy') return;
  const policyData = policyEditorRef.value?.getPolicyData?.();
  if (policyData) {
    await updateSelectedItemData(policyData);
  }
};

const saveComposers = async () => {
  try {
    // 如果当前选中的组/集是草案，先持久化它
    if (selectedItem.value?.isNew) {
      const updatedItem = { ...selectedItem.value };
      delete (updatedItem as any).isNew;

      let command = '';
      let arg = '';
      switch (props.activeTab) {
        case 'policy_set':
          command = 'save_policy_set_cmd';
          arg = 'set';
          break;
        case 'policy_group':
          command = 'save_policy_group_cmd';
          arg = 'group';
          break;
        case 'policy':
          command = 'save_policy_cmd';
          arg = 'policy';
          break;
      }
      if (command) {
        await invoke(command, { [arg]: updatedItem });
        selectedItem.value.isNew = false;
      }
    }

    if (policyGroupComposerRef.value?.saveGroupPolicies) {
      await policyGroupComposerRef.value.saveGroupPolicies();
    }
    if (policySetComposerRef.value?.saveSetGroups) {
      await policySetComposerRef.value.saveSetGroups();
    }
  } catch (e) {
    props.addLog(`保存编排关系失败: ${e}`, props.logLevels.ERROR);
  }
};

defineExpose({
  saveCurrentPolicy,
  saveComposers,
});

watch(batchMode, (newVal) => {
  if (!newVal) {
    selectedItems.value = [];
  }
});

watch(
  () => props.activeTab,
  () => {
    selectedItem.value = null;
    batchMode.value = false;
    selectedItems.value = [];
    loadData();
  }
);

onMounted(loadData);
</script>
