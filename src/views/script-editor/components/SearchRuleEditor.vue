<template>
  <div class="search-rule-editor space-y-5">
    <div class="flex items-center justify-between px-1">
      <div class="flex items-center gap-3">
        <label class="text-[11px] font-black tracking-widest text-primary uppercase opacity-60"
          >规则编排 (Rule Logic)</label
        >
        <div class="badge badge-sm badge-outline opacity-40 rounded-md font-mono">
          {{ localRule.items?.length || 0 }} Rules
        </div>
      </div>
    </div>

    <div class="bg-base-200/40 rounded-[2rem] p-6 border border-base-300 shadow-inner">
      <!-- Group Settings -->
      <div class="flex flex-wrap items-center gap-4 mb-6">
        <div class="join bg-base-100 p-1 rounded-2xl shadow-sm border border-base-300">
          <select class="select select-ghost select-sm join-item w-28 text-xs font-bold" v-model="localRule.op">
            <option value="And">AND (且)</option>
            <option value="Or">OR (或)</option>
            <option value="Not">NOT (非)</option>
          </select>
          <div class="divider divider-horizontal mx-0 w-px opacity-10"></div>
          <select class="select select-ghost select-sm join-item w-32 text-xs font-bold" v-model="localRule.scope">
            <option value="Global">Global (全屏)</option>
            <option value="Item">Item (单框内)</option>
          </select>
        </div>
        <div
          class="text-[10px] bg-primary/5 text-primary/60 px-3 py-2 rounded-xl font-medium flex items-center gap-2 border border-primary/10"
        >
          <InfoIcon class="w-3 h-3" />
          {{ scopeDescription }}
        </div>
      </div>

      <!-- Rule Items -->
      <div class="space-y-3">
        <div
          v-for="(item, idx) in localRule.items"
          :key="idx"
          class="rule-item bg-base-100 rounded-2xl p-3 flex items-center gap-4 border border-base-300 hover:border-primary/30 hover:shadow-md transition-all group duration-300"
        >
          <div
            class="flex-none w-10 h-10 rounded-xl flex items-center justify-center transition-colors shadow-sm"
            :class="item.type === 'Regex' ? 'bg-amber-100 text-amber-600' : 'bg-base-200 text-base-content/40'"
          >
            <TypeIcon v-if="item.type === 'Keyword'" class="w-5 h-5" />
            <ZapIcon v-else-if="item.type === 'Regex'" class="w-5 h-5" />
            <GridIcon v-else class="w-5 h-5" />
          </div>

          <div class="flex-1 min-w-0">
            <div
              v-if="item.type === 'Group'"
              class="text-[10px] font-black py-2 px-3 bg-base-200 text-base-content/50 rounded-xl flex justify-between items-center border border-base-300"
            >
              <span class="tracking-widest uppercase">Nested Group ({{ item.op }} / {{ item.scope }})</span>
              <span
                class="opacity-100 text-primary font-mono bg-white px-2 py-0.5 rounded-lg shadow-sm border border-base-100"
                >{{ item.items?.length || 0 }} 项规则</span
              >
            </div>
            <input
              v-else
              type="text"
              v-model="item.pattern"
              class="input input-ghost w-full focus:bg-base-200 font-mono text-sm tracking-tight placeholder:italic placeholder:opacity-20"
              :placeholder="item.type === 'Regex' ? 'e.g. ^\\d{3}-\\d{3}-\\d{4}$' : '输入关键字...'"
            />
          </div>

          <div class="flex-none opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-2 pr-1">
            <button class="btn btn-ghost btn-sm btn-circle text-error hover:bg-error/10" @click="removeItem(idx)">
              <TrashIcon class="w-4 h-4" />
            </button>
          </div>
        </div>

        <!-- Add Rule Type Selector -->
        <div class="grid grid-cols-3 gap-3 pt-4 border-t border-base-300/50 mt-6">
          <button
            class="btn btn-sm bg-base-100 border-base-300 hover:border-primary hover:text-primary rounded-xl gap-2 shadow-sm font-bold"
            @click="addItem('Keyword')"
          >
            <PlusIcon class="w-3.5 h-3.5" /> <span class="text-[10px]">添加字面值</span>
          </button>
          <button
            class="btn btn-sm bg-base-100 border-base-300 hover:border-amber-500 hover:text-amber-600 rounded-xl gap-2 shadow-sm font-bold"
            @click="addItem('Regex')"
          >
            <PlusIcon class="w-3.5 h-3.5" /> <span class="text-[10px]">添加正则式</span>
          </button>
          <button
            class="btn btn-sm bg-base-100 border-base-300 hover:border-base-content rounded-xl gap-2 shadow-sm font-bold"
            @click="addItem('Group')"
          >
            <PlusIcon class="w-3.5 h-3.5" /> <span class="text-[10px]">创建子逻辑组</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch } from 'vue';
import {
  Plus as PlusIcon,
  Trash2 as TrashIcon,
  Type as TypeIcon,
  Zap as ZapIcon,
  LayoutGrid as GridIcon,
  Info as InfoIcon,
} from 'lucide-vue-next';

const props = defineProps({
  rule: {
    type: Object,
    required: true,
  },
});

const emit = defineEmits(['update']);

// Internal normalization helper
const parseInputRule = (r) => {
  if (!r) return { type: 'Group', op: 'And', scope: 'Global', items: [] };
  if (r.type) return r; // already normalized

  if (r.Group) return { type: 'Group', ...r.Group };
  if (r.Keyword) return { type: 'Keyword', pattern: r.Keyword.pattern };
  if (r.Regex) return { type: 'Regex', pattern: r.Regex.pattern };

  return { type: 'Group', op: 'And', scope: 'Global', items: [] };
};

const localRule = ref(parseInputRule(props.rule));

const scopeDescription = computed(() => {
  return localRule.value.scope === 'Global'
    ? '屏幕内任一可见文本区块匹配成功即可。'
    : '所有子条件必须在属于同一个 OCR 边框的文本内同时满足。';
});

const addItem = (type) => {
  if (type === 'Group') {
    localRule.value.items.push({ type: 'Group', op: 'And', scope: 'Global', items: [] });
  } else {
    localRule.value.items.push({ type, pattern: '' });
  }
  onUpdate();
};

const removeItem = (idx) => {
  localRule.value.items.splice(idx, 1);
  onUpdate();
};

const onUpdate = () => {
  emit('update', localRule.value);
};

watch(localRule, onUpdate, { deep: true });

watch(
  () => props.rule,
  (newVal) => {
    const parsed = parseInputRule(newVal);
    // Simple check to avoid loop if identical
    if (JSON.stringify(parsed) !== JSON.stringify(localRule.value)) {
      localRule.value = parsed;
    }
  },
  { deep: true }
);
</script>
