<template>
  <div class="search-rule-editor space-y-5">
    <div class="bg-base-200/40 rounded-4xl p-6 border border-base-300 shadow-inner">
      <!-- Group Settings -->
      <div class="flex flex-wrap items-center gap-4 mb-2">
        <div class="join bg-base-100 p-1 rounded-2xl shadow-sm border border-base-300">
          <select class="select select-ghost select-sm join-item w-20 text-xs font-bold" v-model="localRule.op">
            <option value="And">AND</option>
            <option value="Or">OR</option>
            <option value="Not">NOT</option>
          </select>
          <div class="divider divider-horizontal mx-0 w-px opacity-10"></div>
          <select class="select select-ghost select-sm join-item w-24 text-xs font-bold" v-model="localRule.scope">
            <option value="Global">全屏</option>
            <option value="Item">单目标</option>
          </select>
        </div>
        <div class="flex items-center gap-3">
          <div class="badge badge-xm badge-outline opacity-40 rounded-md font-mono">
            {{ localRule.items?.length || 0 }} 项
          </div>
        </div>
      </div>

      <!-- Rule Items -->
      <div class="space-y-3">
        <div v-for="(item, idx) in localRule.items" :key="idx">
          <!-- Keyword / Regex Item -->
          <div
            v-if="item.type !== 'Group'"
            class="rule-item bg-base-100 rounded-2xl p-3 flex items-center gap-4 border border-base-300 hover:border-primary/30 hover:shadow-md transition-all group duration-300"
          >
            <div
              class="flex-none w-10 h-10 rounded-xl flex items-center justify-center transition-colors shadow-sm"
              :class="item.type === 'Regex' ? 'bg-amber-100 text-amber-600' : 'bg-base-200 text-base-content/40'"
            >
              <TypeIcon v-if="item.type === 'Keyword'" class="w-5 h-5" />
              <ZapIcon v-else class="w-5 h-5" />
            </div>

            <div class="flex-1 min-w-0">
              <input
                type="text"
                v-model="(item as any).pattern"
                class="input input-ghost w-full focus:bg-base-200 font-mono text-sm tracking-tight placeholder:italic"
                :placeholder="item.type === 'Regex' ? 'e.g. ^\\d{3}-\\d{3}-\\d{4}$' : '关键字...'"
              />
            </div>

            <div class="flex-none opacity-0 group-hover:opacity-100 transition-opacity flex items-center gap-2 pr-1">
              <button class="btn btn-ghost btn-sm btn-circle text-error hover:bg-error/10" @click="removeItem(idx)">
                <TrashIcon class="w-4 h-4" />
              </button>
            </div>
          </div>

          <!-- Nested Group Item -->
          <div
            v-else
            class="nested-group bg-base-100 rounded-2xl border border-base-300 overflow-hidden transition-all duration-300"
            :class="{ 'border-primary/30': expandedGroups[idx] }"
          >
            <!-- Group Header (Clickable to toggle) -->
            <div
              class="flex items-center gap-3 p-3 cursor-pointer select-none group hover:bg-base-200/50 transition-colors"
              @click="toggleGroup(idx)"
            >
              <div
                class="flex-none w-10 h-10 rounded-xl bg-linear-to-br from-slate-200 to-slate-300 flex items-center justify-center shadow-sm"
              >
                <GridIcon class="w-5 h-5 text-slate-600" />
              </div>

              <div class="flex-1 min-w-0">
                <div class="text-[14px] font-black uppercase tracking-widest text-base-content/50">
                  子逻辑组 ({{ (item as any).op }} / {{ (item as any).scope }}) 【{{
                    (item as any).items?.length || 0
                  }}
                  项】
                </div>
              </div>

              <div class="flex items-center gap-2">
                <button
                  class="btn btn-ghost btn-sm btn-circle text-error hover:bg-error/10 opacity-0 group-hover:opacity-100 transition-opacity"
                  @click.stop="removeItem(idx)"
                >
                  <TrashIcon class="w-4 h-4" />
                </button>
                <ChevronDownIcon
                  class="w-4 h-4 transition-transform duration-300"
                  :class="{ 'rotate-180': expandedGroups[idx] }"
                />
              </div>
            </div>

            <!-- Expanded Group Content (Recursive) -->
            <div
              v-if="expandedGroups[idx]"
              class="p-4 pt-0 border-t border-base-200 animate-in slide-in-from-top-2 duration-300"
            >
              <SearchRuleEditor :rule="item as any" @update="updateNestedGroup(idx, $event)" />
            </div>
          </div>
        </div>

        <!-- Add Rule Type Selector -->
        <div class="grid grid-cols-3 gap-3 pt-1 border-t border-base-300/50 mt-6">
          <button
            class="btn btn-sm bg-base-100 border-base-300 hover:border-primary hover:text-primary rounded-xl gap-2 shadow-sm font-bold"
            @click="addItem('Keyword')"
          >
            <PlusIcon class="w-3.5 h-3.5" /> <span class="text-[10px]">标签/文字</span>
          </button>
          <button
            class="btn btn-sm bg-base-100 border-base-300 hover:border-amber-500 hover:text-amber-600 rounded-xl gap-2 shadow-sm font-bold"
            @click="addItem('Regex')"
          >
            <PlusIcon class="w-3.5 h-3.5" /> <span class="text-[10px]">正则表达式</span>
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

<script setup lang="ts">
import { ref, watch, defineAsyncComponent } from 'vue';
import {
  Plus as PlusIcon,
  Trash2 as TrashIcon,
  Type as TypeIcon,
  Zap as ZapIcon,
  LayoutGrid as GridIcon,
  ChevronDown as ChevronDownIcon,
} from 'lucide-vue-next';
import type { SearchRule } from '@/types/bindings';

const SearchRuleEditor = defineAsyncComponent(() => import('./SearchRuleEditor.vue'));

const props = defineProps<{
  rule: SearchRule;
}>();

const emit = defineEmits<{
  (e: 'update', rule: SearchRule): void;
}>();

const parseInputRule = (r: any): any => {
  if (!r) return { type: 'Group', op: 'And', scope: 'Global', items: [] };
  if (r.type) return r;
  if (r.Group) return { type: 'Group', ...r.Group };
  if (r.Keyword) return { type: 'Keyword', pattern: r.Keyword.pattern };
  if (r.Regex) return { type: 'Regex', pattern: r.Regex.pattern };
  return { type: 'Group', op: 'And', scope: 'Global', items: [] };
};

const localRule = ref(parseInputRule(props.rule));
const expandedGroups = ref<Record<number, boolean>>({});

const toggleGroup = (idx: number) => {
  expandedGroups.value[idx] = !expandedGroups.value[idx];
};

const updateNestedGroup = (idx: number, newGroupData: SearchRule) => {
  localRule.value.items[idx] = newGroupData;
  onUpdate();
};

const addItem = (type: string) => {
  if (type === 'Group') {
    localRule.value.items.push({ type: 'Group', op: 'And', scope: 'Global', items: [] });
  } else {
    localRule.value.items.push({ type, pattern: '' });
  }
  onUpdate();
};

const removeItem = (idx: number) => {
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
    if (JSON.stringify(parsed) !== JSON.stringify(localRule.value)) {
      localRule.value = parsed;
    }
  },
  { deep: true }
);
</script>
