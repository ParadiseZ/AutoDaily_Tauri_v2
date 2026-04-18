<template>
  <SurfacePanel class="space-y-4">
    <div>
      <p class="text-sm font-semibold text-[var(--app-text-strong)]">任务与模板设置</p>
    </div>

    <template v-if="script">
      <div class="rounded-[18px] border border-[var(--app-border)] px-4 py-4">
        <p class="text-xs uppercase tracking-[0.14em] text-[var(--app-text-faint)]">设备关联</p>
        <div v-if="usageItems.length" class="mt-3 space-y-2">
          <div
            v-for="item in usageItems"
            :key="`${item.deviceId}-${item.assignmentId}`"
            class="flex items-center justify-between gap-3 rounded-[14px] border border-transparent px-2 py-2 text-sm transition-colors"
            :class="item.scopeKey === selectedScopeKey ? 'border-[var(--app-border)] bg-[var(--app-panel-muted)]' : ''"
          >
            <div class="min-w-0">
              <p class="truncate text-[var(--app-text-strong)]">{{ item.deviceName }}</p>
              <p class="truncate text-xs text-[var(--app-text-faint)]">{{ item.templateLabel }}</p>
            </div>
            <button
              v-if="item.timeTemplateId"
              class="app-button app-button-ghost h-8 px-3 text-xs"
              type="button"
              @click="selectedScopeKey = item.scopeKey"
            >
              设置
            </button>
            <span v-else class="text-xs text-[var(--app-text-faint)]">无模板</span>
          </div>
        </div>
        <p v-else class="mt-3 text-sm text-[var(--app-text-soft)]">未挂载到任何设备队列上。</p>
      </div>

      <div v-if="selectedScope" class="space-y-3">
        <div>
          <p class="text-xs uppercase tracking-[0.14em] text-[var(--app-text-faint)]">当前模板范围</p>
        </div>
        <ScriptTemplateValuePanel
          :script="script"
          :tasks="tasks"
          :scope="selectedScope"
        />
      </div>

      <div
        v-else-if="usageItems.some((item) => item.timeTemplateId === null)"
        class="rounded-[18px] border border-dashed border-[var(--app-border)] px-4 py-4 text-sm text-[var(--app-text-soft)]"
      >
        当前脚本虽然已挂到设备，但这些分配还没有选择时间模板，因此还没有独立的模板变量作用域。
      </div>

      <div class="space-y-3">
        <div v-if="loading" class="py-10 text-sm text-[var(--app-text-soft)]">正在读取脚本任务...</div>
        <div v-else-if="!tasks.length" class="rounded-[18px] border border-dashed border-[var(--app-border)] p-4 text-sm text-[var(--app-text-soft)]">
          无任务数据
        </div>
        <div v-else class="space-y-3">
          <div v-for="task in tasks" :key="task.id" class="rounded-[18px] border border-[var(--app-border)] px-4 py-4">
            <div class="flex items-center justify-between gap-3">
              <p class="text-sm font-semibold text-[var(--app-text-strong)]">{{ task.name }}</p>
              <StatusBadge :label="formatTaskRowTypeLabel(task.rowType)" :tone="task.rowType === 'title' ? 'neutral' : 'info'" />
            </div>

            <div class="mt-3 space-y-2">
              <div class="flex flex-wrap gap-2">
                <span class="rounded-full border border-[var(--app-border)] px-3 py-1 text-xs text-[var(--app-text-soft)]">
                  {{ formatTaskTriggerModeLabel(task.triggerMode) }}
                </span>
                <span class="rounded-full border border-[var(--app-border)] px-3 py-1 text-xs text-[var(--app-text-soft)]">
                  {{ formatTaskCycleLabel(task.defaultTaskCycle) }}
                </span>
                <span class="rounded-full border border-[var(--app-border)] px-3 py-1 text-xs text-[var(--app-text-soft)]">
                  {{ formatTaskToneLabel(task.taskTone) }}
                </span>
              </div>

              <div v-if="extractVariables(task.data.variables).length">
                <p class="text-xs uppercase tracking-[0.14em] text-[var(--app-text-faint)]">变量字段</p>
                <div class="mt-2 flex flex-wrap gap-2">
                  <span
                    v-for="entry in extractVariables(task.data.variables)"
                    :key="entry.key"
                    class="rounded-full border border-[var(--app-border)] px-3 py-1 text-xs text-[var(--app-text-soft)]"
                  >
                    {{ entry.key }} · {{ entry.preview }}
                  </span>
                </div>
              </div>

              <div v-if="extractVariables(task.data.uiData).length">
                <p class="text-xs uppercase tracking-[0.14em] text-[var(--app-text-faint)]">UI 提示字段</p>
                <div class="mt-2 flex flex-wrap gap-2">
                  <span
                    v-for="entry in extractVariables(task.data.uiData)"
                    :key="entry.key"
                    class="rounded-full border border-[var(--app-border)] px-3 py-1 text-xs text-[var(--app-text-soft)]"
                  >
                    {{ entry.key }} · {{ entry.preview }}
                  </span>
                </div>
              </div>

              <p
                v-if="!extractVariables(task.data.variables).length && !extractVariables(task.data.uiData).length"
                class="text-sm text-[var(--app-text-soft)]"
              >
                当前任务没有暴露可读的变量结构。后续如果要做“体力开关 / 次数”这类可配置项，建议统一从这里的变量定义生成表单。
              </p>
            </div>
          </div>
        </div>
      </div>
    </template>

    <EmptyState
      v-else
      title="选择一个脚本查看设置"
    />
  </SurfacePanel>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import StatusBadge from '@/components/shared/StatusBadge.vue';
import type { AssignmentRecord, JsonValue, ScriptTableRecord } from '@/types/app/domain';
import type { DeviceTable } from '@/types/bindings/DeviceTable';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { TimeTemplate } from '@/types/bindings/TimeTemplate';
import ScriptTemplateValuePanel from '@/views/script-template-values/ScriptTemplateValuePanel.vue';
import {
  formatTaskCycleLabel,
  formatTaskRowTypeLabel,
  formatTaskToneLabel,
  formatTaskTriggerModeLabel,
  formatTemplateWindow,
} from '@/utils/presenters';

const props = defineProps<{
  script: ScriptTableRecord | null;
  tasks: ScriptTaskTable[];
  loading: boolean;
  devices: DeviceTable[];
  assignmentsByDevice: Record<string, AssignmentRecord[]>;
  timeTemplates: TimeTemplate[];
}>();

const templateMap = computed(() =>
  Object.fromEntries(props.timeTemplates.map((template) => [template.id, template])),
);

const selectedScopeKey = ref<string | null>(null);

const usageItems = computed(() => {
  if (!props.script) {
    return [];
  }

  return props.devices.flatMap((device) =>
    (props.assignmentsByDevice[device.id] ?? [])
      .filter((assignment) => assignment.scriptId === props.script?.id)
      .map((assignment) => ({
        assignmentId: assignment.id,
        deviceId: device.id,
        deviceName: device.data.deviceName,
        timeTemplateId: assignment.timeTemplateId,
        accountId: null,
        scopeKey: [device.id, assignment.timeTemplateId ?? '', ''].join('::'),
        templateLabel: formatTemplateWindow(
          assignment.timeTemplateId ? templateMap.value[assignment.timeTemplateId] : null,
        ),
      })),
  );
});

const selectedScope = computed<{
  deviceId: string;
  deviceName: string;
  timeTemplateId: string;
  templateLabel: string;
  accountId: null;
} | null>(() => {
  const item = usageItems.value.find((candidate) => candidate.timeTemplateId && candidate.scopeKey === selectedScopeKey.value);
  if (!item || !item.timeTemplateId) {
    return null;
  }

  return {
    deviceId: item.deviceId,
    deviceName: item.deviceName,
    timeTemplateId: item.timeTemplateId,
    templateLabel: item.templateLabel,
    accountId: item.accountId,
  };
});

watch(
  usageItems,
  (items) => {
    const hasCurrent = items.some((item) => item.scopeKey === selectedScopeKey.value && item.timeTemplateId);
    if (hasCurrent) {
      return;
    }
    selectedScopeKey.value = items.find((item) => item.timeTemplateId)?.scopeKey ?? null;
  },
  { immediate: true },
);

const extractVariables = (value: JsonValue) => {
  if (!value || Array.isArray(value) || typeof value !== 'object') {
    return [];
  }

  return Object.entries(value)
    .slice(0, 8)
    .map(([key, itemValue]) => ({
      key,
      preview:
        typeof itemValue === 'boolean'
          ? itemValue
            ? '开'
            : '关'
          : typeof itemValue === 'number' || typeof itemValue === 'string'
            ? String(itemValue)
            : Array.isArray(itemValue)
              ? `${itemValue.length} 项`
              : '对象',
    }));
};
</script>
