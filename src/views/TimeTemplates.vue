<template>
  <div class="flex min-h-full flex-col gap-6">
    <AppPageHeader title="时间模板">
      <template #actions>
        <button class="app-button app-button-primary" type="button" @click="openCreateTemplateDialog">
          <AppIcon name="plus" :size="16" class="stroke-current" />
          新增模板
        </button>
      </template>
    </AppPageHeader>

    <div class="grid flex-1 auto-rows-fr items-stretch gap-4 xl:min-h-[calc(100dvh-220px)] xl:grid-cols-[300px_360px_minmax(0,1fr)]">
      <SurfacePanel class="flex h-full min-h-0 flex-col gap-4">
        <div class="flex items-center justify-between gap-3">
          <div>
            <p class="text-sm font-semibold text-[var(--app-text-strong)]">模板定义</p>
            <p class="text-xs text-[var(--app-text-faint)]">这里管理时间窗口本身。</p>
          </div>
          <span class="rounded-full border border-[var(--app-border)] px-3 py-1 text-xs text-[var(--app-text-faint)]">
            {{ taskStore.timeTemplates.length }} 个
          </span>
        </div>

        <div
          v-if="!taskStore.timeTemplates.length"
          class="flex min-h-0 flex-1 items-center rounded-[18px] border border-dashed border-[var(--app-border)] px-4 py-6 text-sm text-[var(--app-text-soft)]"
        >
          还没有时间模板。
        </div>

        <div v-else class="min-h-0 flex-1 space-y-2 overflow-y-auto pr-1 custom-scrollbar">
          <div
            v-for="template in taskStore.timeTemplates"
            :key="template.id"
            class="template-list-item w-full text-left"
            :class="{ 'template-list-item-active': selectedTemplateId === template.id }"
            role="button"
            tabindex="0"
            @click="selectedTemplateId = template.id"
            @keydown.enter.prevent="selectedTemplateId = template.id"
            @keydown.space.prevent="selectedTemplateId = template.id"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ template.name }}</p>
                <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ formatTemplateWindow(template) }}</p>
                <p class="mt-2 text-[11px] text-[var(--app-text-soft)]">
                  {{ templateUsageSummary(template.id) }}
                </p>
              </div>

              <div class="flex shrink-0 gap-1">
                <button class="app-icon-button h-8 w-8" type="button" title="编辑模板" @click.stop="openEditTemplateDialog(template)">
                  <AppIcon name="edit-3" :size="14" />
                </button>
                <button class="app-icon-button h-8 w-8 text-red-600" type="button" title="删除模板" @click.stop="handleDeleteTemplate(template)">
                  <AppIcon name="trash-2" :size="14" />
                </button>
              </div>
            </div>
          </div>
        </div>
      </SurfacePanel>

      <SurfacePanel class="flex h-full min-h-0 flex-col gap-4">
        <div class="flex items-center justify-between gap-3">
          <div>
            <p class="text-sm font-semibold text-[var(--app-text-strong)]">模板作用域</p>
            <p class="text-xs text-[var(--app-text-faint)]">这里管理哪些设备队列项挂在当前模板下。</p>
          </div>
          <button class="app-button app-button-ghost" type="button" :disabled="!selectedTemplate" @click="bindingDialogOpen = true">
            追加作用域
          </button>
        </div>

        <div
          v-if="!selectedTemplate"
          class="flex min-h-0 flex-1 items-center rounded-[18px] border border-dashed border-[var(--app-border)] px-4 py-6 text-sm text-[var(--app-text-soft)]"
        >
          先从左侧选择一个模板。
        </div>

        <div
          v-else-if="!selectedScopes.length"
          class="flex min-h-0 flex-1 items-center rounded-[18px] border border-dashed border-[var(--app-border)] px-4 py-6 text-sm text-[var(--app-text-soft)]"
        >
          当前模板还没有挂到任何总队列项上。
        </div>

        <div v-else class="min-h-0 flex-1 space-y-2 overflow-y-auto pr-1 custom-scrollbar">
          <div
            v-for="scope in selectedScopes"
            :key="scope.scopeKey"
            class="template-scope-item w-full text-left"
            :class="{ 'template-scope-item-active': selectedScopeKey === scope.scopeKey }"
            role="button"
            tabindex="0"
            @click="selectedScopeKey = scope.scopeKey"
            @keydown.enter.prevent="selectedScopeKey = scope.scopeKey"
            @keydown.space.prevent="selectedScopeKey = scope.scopeKey"
          >
            <div class="flex items-start justify-between gap-3">
              <div class="min-w-0">
                <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ scope.scriptName }}</p>
                <p class="mt-1 truncate text-xs text-[var(--app-text-faint)]">{{ scope.deviceName }}</p>
              </div>
              <button
                class="app-icon-button h-8 w-8 text-red-600"
                type="button"
                title="移出当前模板"
                @click.stop="handleDetachScope(scope)"
              >
                <AppIcon name="x" :size="14" />
              </button>
            </div>
          </div>
        </div>
      </SurfacePanel>

      <SurfacePanel class="flex h-full min-h-0 flex-col overflow-hidden">
        <template v-if="selectedScope && selectedScopeScript">
          <ScriptTemplateValuePanel
            class="min-h-0 flex-1"
            :script="selectedScopeScript"
            :tasks="selectedScopeTasks"
            :scope="selectedScope.scope"
          />
        </template>
        <div v-else class="flex min-h-0 flex-1 items-center justify-center rounded-[18px] border border-dashed border-[var(--app-border)] text-sm text-[var(--app-text-soft)]">
          选择一个作用域后，这里显示模板变量编辑。
        </div>
      </SurfacePanel>
    </div>

    <AppDialog
      :open="templateDialogOpen"
      :title="editingTemplateId ? '编辑时间模板' : '新增时间模板'"
      description="时间模板只定义窗口本身，具体变量值在右侧作用域里配置。"
      width-class="max-w-lg"
      @close="closeTemplateDialog"
    >
      <div class="space-y-4">
        <label class="space-y-2 text-sm text-[var(--app-text-soft)]">
          <span>模板名称</span>
          <input v-model.trim="templateForm.name" class="app-input" type="text" placeholder="例如：早班 / 晚班" />
        </label>

        <div class="grid gap-3 md:grid-cols-2">
          <label class="space-y-2 text-sm text-[var(--app-text-soft)]">
            <span>开始时间</span>
            <input v-model="templateForm.startTime" class="app-input" type="time" />
          </label>
          <label class="space-y-2 text-sm text-[var(--app-text-soft)]">
            <span>结束时间</span>
            <input v-model="templateForm.endTime" class="app-input" type="time" />
          </label>
        </div>

        <div class="flex justify-end gap-2">
          <button class="app-button app-button-ghost" type="button" @click="closeTemplateDialog">取消</button>
          <button class="app-button app-button-primary" type="button" @click="handleSaveTemplate">保存</button>
        </div>
      </div>
    </AppDialog>

    <AppDialog
      :open="bindingDialogOpen"
      title="追加模板作用域"
      description="这里会直接向设备总队列追加一条脚本分配，并挂上当前模板。"
      width-class="max-w-lg"
      @close="closeBindingDialog"
    >
      <div class="space-y-4">
        <label class="space-y-2 text-sm text-[var(--app-text-soft)]">
          <span>设备</span>
          <AppSelect v-model="bindingForm.deviceId" :options="deviceOptions" placeholder="选择设备" />
        </label>

        <label class="space-y-2 text-sm text-[var(--app-text-soft)]">
          <span>脚本</span>
          <AppSelect v-model="bindingForm.scriptId" :options="bindingScriptOptions" placeholder="选择脚本" />
        </label>

        <div class="flex justify-end gap-2">
          <button class="app-button app-button-ghost" type="button" @click="closeBindingDialog">取消</button>
          <button class="app-button app-button-primary" type="button" :disabled="!selectedTemplate" @click="handleCreateScope">
            追加到总队列
          </button>
        </div>
      </div>
    </AppDialog>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue';
import AppDialog from '@/components/shared/AppDialog.vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import { taskService } from '@/services/taskService';
import { useDeviceStore } from '@/store/device';
import { useScriptStore } from '@/store/script';
import { useTaskStore } from '@/store/task';
import type { AssignmentRecord } from '@/types/app/domain';
import type { TimeTemplate } from '@/types/bindings/TimeTemplate';
import { formatPlatformLabel, formatTemplateWindow } from '@/utils/presenters';
import { showToast } from '@/utils/toast';
import ScriptTemplateValuePanel from '@/views/script-template-values/ScriptTemplateValuePanel.vue';

const deviceStore = useDeviceStore();
const scriptStore = useScriptStore();
const taskStore = useTaskStore();

const selectedTemplateId = ref<string | null>(null);
const selectedScopeKey = ref<string | null>(null);
const templateDialogOpen = ref(false);
const bindingDialogOpen = ref(false);
const editingTemplateId = ref<string | null>(null);

const templateForm = reactive({
  name: '',
  startTime: '',
  endTime: '',
});

const bindingForm = reactive({
  deviceId: '',
  scriptId: '',
});

const selectedTemplate = computed(() =>
  taskStore.timeTemplates.find((template) => template.id === selectedTemplateId.value) ?? null,
);

const allScopes = computed(() =>
  deviceStore.devices.flatMap((device) =>
    (taskStore.assignmentsByDevice[device.id] ?? []).map((assignment) => {
      const script = scriptStore.sortedScripts.find((item) => item.id === assignment.scriptId) ?? null;
      return {
        assignment,
        deviceId: device.id,
        deviceName: device.data.deviceName,
        scriptName: script?.data.name ?? '未知脚本',
        timeTemplateId: assignment.timeTemplateId,
        scopeKey: `${device.id}::${assignment.id}`,
      };
    }),
  ),
);

const selectedScopes = computed(() =>
  allScopes.value
    .filter((scope) => scope.timeTemplateId === selectedTemplateId.value)
    .map((scope) => ({
      ...scope,
      scope: {
        deviceId: scope.deviceId,
        deviceName: scope.deviceName,
        timeTemplateId: String(scope.timeTemplateId),
        templateLabel: formatTemplateWindow(selectedTemplate.value),
        accountId: null,
      },
    })),
);

const selectedScope = computed(() =>
  selectedScopes.value.find((scope) => scope.scopeKey === selectedScopeKey.value) ?? selectedScopes.value[0] ?? null,
);

const selectedScopeScript = computed(() =>
  selectedScope.value
    ? scriptStore.sortedScripts.find((item) => item.id === selectedScope.value.assignment.scriptId) ?? null
    : null,
);

const selectedScopeTasks = computed(() =>
  selectedScope.value ? scriptStore.tasksByScriptId[selectedScope.value.assignment.scriptId] ?? [] : [],
);

const deviceOptions = computed(() =>
  deviceStore.devices.map((device) => ({
    label: device.data.deviceName,
    value: device.id,
    description: formatPlatformLabel(device.data.platform),
  })),
);

const bindingScriptOptions = computed(() => {
  if (!bindingForm.deviceId) {
    return [];
  }

  const device = deviceStore.devices.find((item) => item.id === bindingForm.deviceId);
  const platform = device?.data.platform ?? 'android';
  return scriptStore.sortedScripts
    .filter((script) => (script.data.platform ?? 'android') === platform)
    .map((script) => ({
      label: script.data.name,
      value: script.id,
      description: script.data.description || formatPlatformLabel(script.data.platform),
    }));
});

const templateUsageSummary = (templateId: string) => {
  const usages = allScopes.value.filter((scope) => scope.timeTemplateId === templateId);
  const deviceIds = new Set(usages.map((scope) => scope.deviceId));
  const scriptIds = new Set(usages.map((scope) => scope.assignment.scriptId));
  return `${usages.length} 个作用域 · ${deviceIds.size} 台设备 · ${scriptIds.size} 个脚本`;
};

const loadPageData = async () => {
  await Promise.all([deviceStore.refreshAll(), scriptStore.loadScripts()]);
  await taskStore.hydrateForDevices(deviceStore.devices.map((device) => device.id));
};

const resetTemplateForm = () => {
  templateForm.name = '';
  templateForm.startTime = '';
  templateForm.endTime = '';
};

const openCreateTemplateDialog = () => {
  editingTemplateId.value = null;
  resetTemplateForm();
  templateDialogOpen.value = true;
};

const openEditTemplateDialog = (template: TimeTemplate) => {
  editingTemplateId.value = template.id;
  templateForm.name = template.name;
  templateForm.startTime = template.startTime ?? '';
  templateForm.endTime = template.endTime ?? '';
  templateDialogOpen.value = true;
};

const closeTemplateDialog = () => {
  templateDialogOpen.value = false;
  resetTemplateForm();
};

const closeBindingDialog = () => {
  bindingDialogOpen.value = false;
  bindingForm.deviceId = '';
  bindingForm.scriptId = '';
};

const handleSaveTemplate = async () => {
  if (!templateForm.name.trim()) {
    showToast('请先填写模板名称', 'warning');
    return;
  }

  try {
    const nextTemplate: TimeTemplate = {
      id: editingTemplateId.value ?? (await taskService.requestUuid()),
      name: templateForm.name.trim(),
      startTime: templateForm.startTime || null,
      endTime: templateForm.endTime || null,
    };
    await taskStore.saveTimeTemplate(nextTemplate);
    selectedTemplateId.value = nextTemplate.id;
    closeTemplateDialog();
    showToast('时间模板已保存', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '保存时间模板失败', 'error');
  }
};

const handleDeleteTemplate = async (template: TimeTemplate) => {
  const usageCount = allScopes.value.filter((scope) => scope.timeTemplateId === template.id).length;
  if (usageCount > 0) {
    showToast('当前模板仍被总队列使用，请先处理中栏里的作用域。', 'warning');
    return;
  }

  try {
    await taskStore.deleteTimeTemplate(template.id);
    if (selectedTemplateId.value === template.id) {
      selectedTemplateId.value = taskStore.timeTemplates[0]?.id ?? null;
    }
    showToast('时间模板已删除', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '删除时间模板失败', 'error');
  }
};

const handleCreateScope = async () => {
  if (!selectedTemplate.value) {
    return;
  }
  if (!bindingForm.deviceId || !bindingForm.scriptId) {
    showToast('请先选择设备和脚本', 'warning');
    return;
  }

  try {
    const assignment = await taskStore.createAssignment(bindingForm.deviceId, bindingForm.scriptId, selectedTemplate.value.id);
    selectedScopeKey.value = `${assignment.deviceId}::${assignment.id}`;
    closeBindingDialog();
    showToast('作用域已追加到总队列', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '追加作用域失败', 'error');
  }
};

const handleDetachScope = async (scope: { assignment: AssignmentRecord }) => {
  try {
    await taskStore.detachAssignmentTemplate(scope.assignment);
    showToast('已移出当前模板，总队列项仍然保留。', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '移出模板失败', 'error');
  }
};

watch(
  () => bindingForm.deviceId,
  () => {
    if (bindingScriptOptions.value.some((option) => option.value === bindingForm.scriptId)) {
      return;
    }
    bindingForm.scriptId = '';
  },
);

watch(
  () => taskStore.timeTemplates,
  (templates) => {
    if (!templates.length) {
      selectedTemplateId.value = null;
      return;
    }
    if (templates.some((template) => template.id === selectedTemplateId.value)) {
      return;
    }
    selectedTemplateId.value = templates[0]?.id ?? null;
  },
  { immediate: true, deep: true },
);

watch(
  selectedScopes,
  (scopes) => {
    if (scopes.some((scope) => scope.scopeKey === selectedScopeKey.value)) {
      return;
    }
    selectedScopeKey.value = scopes[0]?.scopeKey ?? null;
  },
  { immediate: true },
);

watch(
  () => selectedScope.value?.assignment.scriptId ?? null,
  async (scriptId) => {
    if (!scriptId || scriptStore.tasksByScriptId[scriptId] || scriptStore.taskLoading[scriptId]) {
      return;
    }
    await scriptStore.loadScriptTasks(scriptId).catch(() => []);
  },
  { immediate: true },
);

onMounted(() => {
  void loadPageData();
});
</script>

<style scoped>
.template-list-item,
.template-scope-item {
  border-radius: 18px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.7);
  padding: 0.95rem 1rem;
  transition: border-color 0.16s ease, background 0.16s ease, box-shadow 0.16s ease;
}

.template-list-item:hover,
.template-scope-item:hover {
  border-color: color-mix(in srgb, var(--app-accent) 28%, var(--app-border));
}

.template-list-item-active,
.template-scope-item-active {
  border-color: color-mix(in srgb, var(--app-accent) 34%, var(--app-border));
  background: color-mix(in srgb, var(--app-accent-soft) 58%, white);
  box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--app-accent) 16%, transparent);
}
</style>
