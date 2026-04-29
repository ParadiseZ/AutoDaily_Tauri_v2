<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <div class="overflow-x-auto">
      <div class="editor-panel-tabs min-w-max">
        <button
          v-for="tab in policyPanelTabs"
          :key="tab.id"
          type="button"
          class="editor-panel-tab"
          :class="{ 'editor-panel-tab-active': activePanel === tab.id }"
          :data-testid="`editor-policy-tab-${tab.id}`"
          @click="$emit('update:active-panel', tab.id)"
        >
          {{ tab.label }}
        </button>
      </div>
    </div>

    <template v-if="policy">
      <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
        <div v-if="activePanel === 'basic'" class="space-y-4">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.14em] text-(--app-text-faint)">策略名称</span>
            <input :value="policyName" class="app-input" data-testid="editor-policy-name" @input="$emit('update:policy-name', ($event.target as HTMLInputElement).value)" />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.14em] text-(--app-text-faint)">备注</span>
            <textarea :value="policyNote" class="app-input min-h-[120px] resize-y" @input="$emit('update:policy-note', ($event.target as HTMLTextAreaElement).value)" />
          </label>

          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.14em] text-(--app-text-faint)">日志输出</span>
            <input :value="policyLogPrint ?? ''" class="app-input" placeholder="可选，用于调试输出" @input="$emit('update:policy-log-print', ($event.target as HTMLInputElement).value)" />
          </label>

          <div class="grid gap-3 sm:grid-cols-2">
            <label class="space-y-2">
              <span class="text-xs font-medium uppercase tracking-[0.14em] text-(--app-text-faint)">当前位置</span>
              <input :value="String(policy.data.curPos)" class="app-input" type="number" @input="$emit('update:number-field', 'curPos', ($event.target as HTMLInputElement).value)" />
              <span class="text-xs leading-5 text-(--app-text-faint)">用于选择第 N 个匹配目标；`999` 表示最后一个。</span>
            </label>

            <label class="space-y-2">
              <span class="text-xs font-medium uppercase tracking-[0.14em] text-(--app-text-faint)">最大执行次数</span>
              <input :value="String(policy.data.execMax)" class="app-input" type="number" @input="$emit('update:number-field', 'execMax', ($event.target as HTMLInputElement).value)" />
            </label>

            <label class="flex items-center gap-3 rounded-[18px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3 text-sm text-(--app-text-soft)">
              <input
                :checked="policy.data.skipFlag"
                class="h-4 w-4 accent-(--app-accent)"
                type="checkbox"
                @change="$emit('update:boolean-field', 'skipFlag', ($event.target as HTMLInputElement).checked)"
              />
              <span>命中后跳过后续执行</span>
            </label>
          </div>
        </div>

        <div v-else-if="activePanel === 'condition'" class="space-y-4">
          <div class="rounded-[18px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
            <p class="text-sm font-semibold text-(--app-text-strong)">命中条件</p>
            <p class="mt-2 text-xs leading-5 text-(--app-text-faint)">策略命中依赖搜索规则；根节点固定为逻辑组，规则和子组都在组内维护。</p>
          </div>
        </div>

        <div v-else class="space-y-4">
          <div class="rounded-[18px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
            <p class="text-sm font-semibold text-(--app-text-strong)">{{ activePanel === 'before' ? '全局行为模板' : '命中行为模板' }}</p>
            <p class="mt-2 text-xs leading-5 text-(--app-text-faint)">中间区域的模板会插入到当前选中的行为层级，右侧用于查看和编辑嵌套步骤。</p>
          </div>

          <div class="space-y-3">
            <details v-for="group in templateGroups" :key="group.name" class="rounded-[18px] border border-(--app-border) bg-(--app-panel-muted)" :open="true">
              <summary class="cursor-pointer list-none px-4 py-3 text-sm font-semibold text-(--app-text-strong)">
                <div class="flex items-center justify-between gap-3">
                  <span>{{ group.name }}</span>
                  <span class="text-xs text-(--app-text-faint)">{{ group.items.length }}</span>
                </div>
              </summary>

              <div class="grid gap-2 px-3 pb-3 sm:grid-cols-2">
                <button
                  v-for="template in group.items"
                  :key="template.id"
                  class="editor-template-tile"
                  :data-testid="`editor-policy-step-template-${template.id}`"
                  type="button"
                  @click="$emit('append-template-step', template.id)"
                >
                  <span class="editor-template-dot" />
                  <span class="truncate">{{ template.label }}</span>
                </button>
              </div>
            </details>
          </div>
        </div>
      </div>
    </template>
  </SurfacePanel>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import { policyPanelTabs, type PolicyEditorPanelId } from '@/views/script-editor/editor-policy/editorPolicy';
import { editorStepTemplates } from '@/views/script-editor/editor-step/editorStepTemplates';

defineProps<{
  policy: PolicyTable | null;
  activePanel: PolicyEditorPanelId;
  policyName: string;
  policyNote: string;
  policyLogPrint: string | null;
}>();

defineEmits<{
  'update:active-panel': [panel: PolicyEditorPanelId];
  'update:policy-name': [value: string];
  'update:policy-note': [value: string];
  'update:policy-log-print': [value: string];
  'update:number-field': [field: 'curPos' | 'execMax', value: string];
  'update:boolean-field': [field: 'skipFlag', value: boolean];
  'append-template-step': [templateId: string];
}>();

const templateGroups = computed(() => {
  const grouped = new Map<string, typeof editorStepTemplates>();
  for (const template of editorStepTemplates) {
    const bucket = grouped.get(template.group) ?? [];
    bucket.push(template);
    grouped.set(template.group, bucket);
  }

  const groupOrder = ['动作', '流程', '数据', '视觉', '状态', '容器', '兼容'];
  return Array.from(grouped.entries())
    .sort(([left], [right]) => {
      const leftIndex = groupOrder.indexOf(left);
      const rightIndex = groupOrder.indexOf(right);
      return (leftIndex === -1 ? Number.MAX_SAFE_INTEGER : leftIndex) - (rightIndex === -1 ? Number.MAX_SAFE_INTEGER : rightIndex);
    })
    .map(([name, items]) => ({ name, items }));
});
</script>

<style scoped>
.editor-panel-tabs {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  border-bottom: 1px solid var(--app-border);
}

.editor-panel-tab {
  position: relative;
  margin-bottom: -1px;
  border-bottom: 2px solid transparent;
  padding: 0.75rem 0.35rem 0.85rem;
  color: var(--app-text-faint);
  font-size: 0.93rem;
  font-weight: 600;
  transition: color 0.16s ease, border-color 0.16s ease;
}

.editor-panel-tab:hover {
  color: var(--app-text-soft);
}

.editor-panel-tab-active {
  border-bottom-color: var(--app-accent);
  color: var(--app-text-strong);
}

.editor-template-tile {
  display: flex;
  align-items: center;
  gap: 0.65rem;
  width: 100%;
  border-radius: 14px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.52);
  padding: 0.7rem 0.8rem;
  text-align: left;
  color: var(--app-text-strong);
  transition: border-color 0.16s ease, background 0.16s ease;
}

.editor-template-tile:hover {
  border-color: rgba(70, 110, 255, 0.22);
}

.editor-template-dot {
  width: 0.65rem;
  height: 0.65rem;
  border-radius: 999px;
  background: color-mix(in srgb, var(--app-accent) 60%, white);
  box-shadow: 0 0 0 4px color-mix(in srgb, var(--app-accent) 14%, transparent);
}
</style>
