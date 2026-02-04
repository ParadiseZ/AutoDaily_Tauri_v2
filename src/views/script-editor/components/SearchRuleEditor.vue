<template>
  <div class="search-rule-editor space-y-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <label class="label-text font-bold text-xs uppercase opacity-60">命中规则组 (Search Rule Group)</label>
      </div>
    </div>

    <div class="bg-base-200/50 rounded-xl p-4 border border-base-300">
      <!-- Group Settings -->
      <div class="flex items-center gap-3 mb-4">
        <div class="join">
          <select class="select select-bordered select-sm join-item w-24" v-model="localRule.op">
            <option value="And">AND (且)</option>
            <option value="Or">OR (或)</option>
            <option value="Not">NOT (非)</option>
          </select>
          <select class="select select-bordered select-sm join-item w-32" v-model="localRule.scope">
            <option value="Global">Global (全屏)</option>
            <option value="Item">Item (单框内)</option>
          </select>
        </div>
        <div class="text-xs opacity-50 italic">
          {{ scopeDescription }}
        </div>
      </div>

      <!-- Rule Items -->
      <div class="space-y-2">
        <div
          v-for="(item, idx) in localRule.items"
          :key="idx"
          class="rule-item bg-base-100 rounded-lg p-2 flex items-center gap-3 border border-base-200"
        >
          <div class="flex-none p-1.5 rounded bg-base-200">
            <TypeIcon v-if="item.type === 'Keyword'" class="w-4 h-4 opacity-70" />
            <ZapIcon v-else-if="item.type === 'Regex'" class="w-4 h-4 text-warning" />
            <GridIcon v-else class="w-4 h-4 opacity-70" />
          </div>

          <div class="flex-1 min-w-0">
            <div v-if="item.type === 'Group'" class="text-xs font-mono py-1">
              [Nested Group: {{ item.op }} / {{ item.scope }}]
            </div>
            <input
              v-else
              type="text"
              v-model="item.pattern"
              class="input input-ghost input-sm w-full focus:bg-base-200 font-mono"
              :placeholder="item.type === 'Regex' ? '输入正则表达式...' : '输入模糊匹配关键字...'"
              @input="onUpdate"
            />
          </div>

          <div class="flex-none flex items-center gap-1">
            <button class="btn btn-ghost btn-xs btn-circle text-error" @click="removeItem(idx)">
              <TrashIcon class="w-3 h-3" />
            </button>
          </div>
        </div>

        <!-- Add Rule Type Selector -->
        <div class="flex gap-2 pt-2 border-t border-base-300 mt-4">
          <button class="btn btn-xs btn-outline btn-primary gap-1 flex-1" @click="addItem('Keyword')">
            <PlusIcon class="w-3 h-3" /> 字面值
          </button>
          <button class="btn btn-xs btn-outline btn-warning gap-1 flex-1" @click="addItem('Regex')">
            <PlusIcon class="w-3 h-3" /> 正则式
          </button>
          <button class="btn btn-xs btn-outline gap-1 flex-1" @click="addItem('Group')">
            <PlusIcon class="w-3 h-3" /> 子组
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
} from 'lucide-vue-next';

const props = defineProps({
  rule: {
    type: Object,
    required: true,
  },
});

const emit = defineEmits(['update']);

// Ensure we have a valid structure even if backend provides generic Group
const normalizeRule = (r) => {
  if (r.Group) return { type: 'Group', ...r.Group };
  if (r.Keyword) return { type: 'Keyword', pattern: r.Keyword.pattern };
  if (r.Regex) return { type: 'Regex', pattern: r.Regex.pattern };
  return r; // already normalized or other form
};

const localRule = ref(
  props.rule.type
    ? props.rule
    : {
        type: 'Group',
        op: 'And',
        scope: 'Global',
        items: (props.rule.items || []).map(normalizeRule),
      }
);

const scopeDescription = computed(() => {
  return localRule.value.scope === 'Global' ? '画面中任一部位匹配成功即可' : '所有子条件必须在同一个OCR文本框内满足';
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
</script>
