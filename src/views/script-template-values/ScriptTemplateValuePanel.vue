<template>
  <div class="flex h-full min-h-0 flex-col gap-4">
    <div v-if="loading" class="flex min-h-0 flex-1 items-center rounded-[18px] border border-[var(--app-border)] px-4 py-8 text-sm text-[var(--app-text-soft)]">
      正在读取模板变量...
    </div>

    <div v-else-if="loadError" class="flex min-h-0 flex-1 items-center rounded-[18px] border border-red-500/18 bg-red-500/8 px-4 py-4 text-sm text-red-700">
      {{ loadError }}
    </div>

    <template v-else>
      <div class="flex flex-wrap items-center justify-between gap-3 rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-3">
        <div class="flex flex-wrap items-center gap-2 text-xs text-[var(--app-text-faint)]">
          <span class="rounded-full border border-[var(--app-border)] px-3 py-1">{{ entries.length }} 个变量</span>
          <span v-if="recordUpdatedAt" class="rounded-full border border-[var(--app-border)] px-3 py-1">上次保存 {{ recordUpdatedAt }}</span>
          <span
            class="rounded-full px-3 py-1"
            :class="hasDirtyChanges ? 'bg-amber-500/12 text-amber-700' : 'bg-emerald-500/12 text-emerald-700'"
          >
            {{ hasDirtyChanges ? '未保存' : '已同步' }}
          </span>
        </div>

        <div class="flex flex-wrap gap-2">
          <button class="app-button app-button-ghost" type="button" :disabled="saving" @click="resetToDefaults">
            恢复默认
          </button>
          <button class="app-button app-button-primary" type="button" :disabled="saving" @click="saveValues">
            {{ saving ? '保存中...' : '保存模板变量' }}
          </button>
        </div>
      </div>

      <EditorTaskTablePreview
        v-if="previewTasks.length"
        :key="previewKey"
        class="min-h-0 flex-1"
        :tasks="previewTasks"
        :selected-task-id="selectedTaskId"
        :selected-task-ui-schema="selectedTaskUiSchema"
        :selected-task-input-entries="previewInputEntries"
        :shared-input-entries="previewInputEntries"
        :task-enabled-by-id="taskEnabledById"
        :task-cycle-by-id="taskCycleById"
        :selected-ui-field-id="null"
        :selected-task-cycle-value="'everyRun'"
        :selected-task-cycle-mode="'named'"
        :selected-task-cycle-day="1"
        :edit-all-tasks="true"
        :require-bound-input="true"
        :show-header="false"
        @select-task="selectedTaskId = $event"
        @update-input="handlePreviewInputUpdate"
        @update:task-enabled="handleTaskEnabledUpdate"
        @update:task-cycle="handleTaskCycleUpdate"
      />

      <div
        v-else
        class="flex min-h-0 flex-1 items-center rounded-[18px] border border-dashed border-[var(--app-border)] px-4 py-6 text-sm text-[var(--app-text-soft)]"
      >
        当前脚本没有可预览任务。
      </div>

      <div
        v-if="unboundEntries.length"
        class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4"
      >
        <div class="space-y-1">
          <p class="text-sm font-semibold text-[var(--app-text-strong)]">未挂到 UI 的变量</p>
          <p class="text-xs text-[var(--app-text-faint)]">这些变量当前没有出现在任务 UI 预览里，仍保留简化输入框。</p>
        </div>

        <div class="mt-4 space-y-3">
          <div v-for="entry in unboundEntries" :key="entry.id" class="rounded-[16px] border border-[var(--app-border)] bg-white/70 px-4 py-3 dark:bg-white/5">
            <div class="flex flex-wrap items-start justify-between gap-3">
              <div class="space-y-1">
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">{{ entry.name }}</p>
                <p class="text-xs text-[var(--app-text-faint)]">
                  {{ entry.ownerTaskName }} · {{ entry.displayKey }} · {{ getVariableValueTypeLabel(entry.valueType) }}
                </p>
              </div>
              <span class="rounded-full border border-[var(--app-border)] px-3 py-1 text-[11px] text-[var(--app-text-soft)]">
                默认 {{ entry.defaultPreview }}
              </span>
            </div>

            <label
              v-if="entry.valueType === 'bool'"
              class="mt-3 flex min-h-[44px] items-center gap-3 rounded-[14px] border border-[var(--app-border)] px-4 py-3 text-sm text-[var(--app-text-soft)]"
            >
              <input
                :checked="entry.booleanValue"
                type="checkbox"
                class="h-4 w-4"
                style="accent-color: var(--app-accent)"
                @change="updateBooleanValue(entry.id, ($event.target as HTMLInputElement).checked)"
              />
              <span>{{ entry.booleanValue ? '已开启' : '已关闭' }}</span>
            </label>

            <textarea
              v-else-if="entry.valueType === 'json' || entry.valueType === 'list' || entry.valueType === 'object'"
              :value="entry.stringValue"
              class="app-textarea mt-3 min-h-[120px]"
              spellcheck="false"
              @input="updateStringValue(entry.id, ($event.target as HTMLTextAreaElement).value)"
            />

            <input
              v-else
              :value="entry.stringValue"
              class="app-input mt-3"
              :type="entry.valueType === 'int' || entry.valueType === 'float' ? 'number' : 'text'"
              @input="updateStringValue(entry.id, ($event.target as HTMLInputElement).value)"
            />
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, shallowRef, watch } from 'vue';
import { scriptTemplateValueService } from '@/services/scriptTemplateValueService';
import { taskService } from '@/services/taskService';
import type { JsonValue, ScriptTableRecord, ScriptTimeTemplateValuesDto } from '@/types/app/domain';
import type { TaskCycle } from '@/types/bindings/TaskCycle';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import { filterUserVisibleTaskRows } from '@/utils/scriptTaskVisibility';
import { createUiSchema, parseUiSchema, stableStringify } from '@/views/script-editor/editorSchema';
import { getVariableValueTypeLabel } from '@/views/script-editor/editorVariables';
import EditorTaskTablePreview from '@/views/script-editor/EditorTaskTablePreview.vue';
import {
  buildTemplateEditorInputs,
  buildTemplateTaskSettingsPayload,
  buildTemplateVariablePayload,
  createTemplateTaskSettingEntries,
  createTemplateVariableEntries,
  type TemplateTaskSettingEntry,
  type TemplateVariableEntry,
  updateTemplateTaskCycleSetting,
  updateTemplateTaskSetting,
  updateTemplateEntryFromEditorInput,
} from '@/views/script-template-values/templateValueState';
import { showToast } from '@/utils/toast';

const props = defineProps<{
  script: ScriptTableRecord;
  tasks: ScriptTaskTable[];
  scope: {
    deviceId: string;
    deviceName: string;
    timeTemplateId: string;
    templateLabel: string;
    accountId?: string | null;
  };
}>();

const entries = shallowRef<TemplateVariableEntry[]>([]);
const taskSettings = shallowRef<TemplateTaskSettingEntry[]>([]);
const loading = ref(false);
const saving = ref(false);
const loadError = ref<string | null>(null);
const record = shallowRef<ScriptTimeTemplateValuesDto | null>(null);
const loadedSnapshot = ref('');
const taskSettingsSnapshot = ref('');
const selectedTaskId = ref<string | null>(null);
const previewVersion = ref(0);

const recordUpdatedAt = computed(() => {
  if (!record.value?.updatedAt) {
    return null;
  }
  return new Date(record.value.updatedAt).toLocaleString('zh-CN', {
    hour12: false,
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
});

const dirty = computed(() => stableStringify(entries.value) !== loadedSnapshot.value);
const taskSettingsDirty = computed(() => stableStringify(taskSettings.value) !== taskSettingsSnapshot.value);
const hasDirtyChanges = computed(() => dirty.value || taskSettingsDirty.value);
const previewKey = computed(() => `${props.script.id}:${props.scope.deviceId}:${props.scope.timeTemplateId}:${previewVersion.value}`);
const previewTasks = computed(() => filterUserVisibleTaskRows(props.tasks));
const selectableTasks = computed(() => previewTasks.value.filter((task) => task.rowType === 'task'));
const previewInputEntries = computed(() => buildTemplateEditorInputs(entries.value));
const taskEnabledById = computed(() =>
  Object.fromEntries(taskSettings.value.map((entry) => [entry.taskId, entry.enabled])),
);
const taskCycleById = computed(() =>
  Object.fromEntries(taskSettings.value.map((entry) => [entry.taskId, entry.taskCycle])),
);

const referencedVariableKeys = computed(() => {
  const ids = new Set<string>();
  const keys = new Set<string>();

  for (const task of previewTasks.value) {
    const uiSchema = parseUiSchema(task.data.uiData ?? {});
    for (const field of uiSchema.fields) {
      if (field.variableId?.trim()) {
        ids.add(field.variableId.trim());
      }
      if (field.inputKey?.trim()) {
        keys.add(field.inputKey.trim());
      }
      if (field.key?.trim()) {
        keys.add(field.key.trim());
      }
    }
  }

  return { ids, keys };
});

const unboundEntries = computed(() =>
  entries.value.filter(
    (entry) =>
      !referencedVariableKeys.value.ids.has(entry.id) &&
      !referencedVariableKeys.value.keys.has(entry.key) &&
      !referencedVariableKeys.value.keys.has(entry.displayKey),
  ),
);

const selectedTask = computed(() =>
  selectableTasks.value.find((task) => task.id === selectedTaskId.value) ?? selectableTasks.value[0] ?? null,
);
const selectedTaskUiSchema = computed(() =>
  selectedTask.value ? parseUiSchema(selectedTask.value.data.uiData ?? {}) : createUiSchema(),
);

const isRecord = (value: unknown): value is Record<string, JsonValue> =>
  Boolean(value) && !Array.isArray(value) && typeof value === 'object';

const buildEntriesFromStored = (storedVariables: JsonValue) =>
  createTemplateVariableEntries(props.script, previewTasks.value, storedVariables);

const extractStoredVariables = (recordValue: ScriptTimeTemplateValuesDto | null) => {
  const valuesRoot = isRecord(recordValue?.valuesJson) ? recordValue.valuesJson : {};
  return isRecord(valuesRoot.variables) ? valuesRoot.variables : {};
};

const extractStoredTaskSettings = (recordValue: ScriptTimeTemplateValuesDto | null) => {
  const valuesRoot = isRecord(recordValue?.valuesJson) ? recordValue.valuesJson : {};
  return isRecord(valuesRoot.taskSettings) ? valuesRoot.taskSettings : {};
};

const rebuildTaskSettings = (storedTaskSettings: JsonValue, resetSnapshot: boolean) => {
  const currentByTaskId = new Map(taskSettings.value.map((entry) => [entry.taskId, entry]));
  const nextSettings = createTemplateTaskSettingEntries(previewTasks.value, storedTaskSettings).map((entry) =>
    !resetSnapshot && currentByTaskId.has(entry.taskId)
      ? {
          ...entry,
          enabled: currentByTaskId.get(entry.taskId)?.enabled ?? entry.enabled,
          taskCycle: currentByTaskId.get(entry.taskId)?.taskCycle ?? entry.taskCycle,
        }
      : entry,
  );

  taskSettings.value = nextSettings;
  if (resetSnapshot) {
    taskSettingsSnapshot.value = stableStringify(nextSettings);
  }
};

const loadValues = async () => {
  loading.value = true;
  loadError.value = null;

  try {
    const nextRecord = await scriptTemplateValueService.get(
      props.scope.deviceId,
      props.script.id,
      props.scope.timeTemplateId,
      props.scope.accountId ?? null,
    );
    const storedVariables = extractStoredVariables(nextRecord);
    const storedTaskSettings = extractStoredTaskSettings(nextRecord);
    record.value = nextRecord;
    entries.value = buildEntriesFromStored(storedVariables);
    rebuildTaskSettings(storedTaskSettings, true);
    loadedSnapshot.value = stableStringify(entries.value);
    previewVersion.value += 1;
  } catch (error) {
    loadError.value = error instanceof Error ? error.message : '读取模板变量失败';
  } finally {
    loading.value = false;
  }
};

const updateStringValue = (entryId: string, value: string) => {
  entries.value = updateTemplateEntryFromEditorInput(entries.value, entryId, 'stringValue', value);
};

const updateBooleanValue = (entryId: string, value: boolean) => {
  entries.value = updateTemplateEntryFromEditorInput(entries.value, entryId, 'booleanValue', value);
};

const handlePreviewInputUpdate = (
  entryId: string,
  field: 'stringValue' | 'booleanValue',
  value: string | boolean,
) => {
  entries.value = updateTemplateEntryFromEditorInput(entries.value, entryId, field, value);
};

const handleTaskEnabledUpdate = (taskId: string, enabled: boolean) => {
  const task = props.tasks.find((item) => item.id === taskId);
  if (!taskSettings.value.some((entry) => entry.taskId === taskId)) {
    taskSettings.value = [
      ...taskSettings.value,
      {
        taskId,
        enabled,
        defaultEnabled: task?.defaultEnabled ?? true,
        taskCycle: task?.defaultTaskCycle ?? 'everyRun',
        defaultTaskCycle: task?.defaultTaskCycle ?? 'everyRun',
      },
    ];
    return;
  }

  taskSettings.value = updateTemplateTaskSetting(taskSettings.value, taskId, enabled);
};

const handleTaskCycleUpdate = (taskId: string, taskCycle: TaskCycle) => {
  const task = props.tasks.find((item) => item.id === taskId);
  if (!taskSettings.value.some((entry) => entry.taskId === taskId)) {
    taskSettings.value = [
      ...taskSettings.value,
      {
        taskId,
        enabled: task?.defaultEnabled ?? true,
        defaultEnabled: task?.defaultEnabled ?? true,
        taskCycle,
        defaultTaskCycle: task?.defaultTaskCycle ?? 'everyRun',
      },
    ];
    return;
  }

  taskSettings.value = updateTemplateTaskCycleSetting(taskSettings.value, taskId, taskCycle);
};

const resetToDefaults = () => {
  entries.value = buildEntriesFromStored({});
  taskSettings.value = createTemplateTaskSettingEntries(previewTasks.value, {});
  previewVersion.value += 1;
};

const mergeVisibleVariables = (currentVariables: JsonValue, visibleVariables: Record<string, JsonValue>) => {
  const current = isRecord(currentVariables) ? currentVariables : {};
  const visibleKeys = new Set(
    entries.value.flatMap((entry) => [entry.id, entry.key, entry.displayKey, `input.${entry.displayKey}`]),
  );
  return {
    ...Object.fromEntries(Object.entries(current).filter(([key]) => !visibleKeys.has(key))),
    ...visibleVariables,
  };
};

const mergeVisibleTaskSettings = (currentSettings: JsonValue, visibleSettings: Record<string, JsonValue>) => {
  const current = isRecord(currentSettings) ? currentSettings : {};
  const visibleTaskIds = new Set(taskSettings.value.map((entry) => entry.taskId));
  return {
    ...Object.fromEntries(Object.entries(current).filter(([taskId]) => !visibleTaskIds.has(taskId))),
    ...visibleSettings,
  };
};

const saveValues = async () => {
  saving.value = true;

  try {
    const variables = buildTemplateVariablePayload(entries.value);
    const taskSettingsPayload = buildTemplateTaskSettingsPayload(taskSettings.value);
    const currentRoot = isRecord(record.value?.valuesJson) ? record.value.valuesJson : {};
    const nextValuesJson = Object.assign({}, currentRoot, {
      variables: mergeVisibleVariables(currentRoot.variables, variables),
      taskSettings: mergeVisibleTaskSettings(currentRoot.taskSettings, taskSettingsPayload),
    }) as JsonValue;
    const now = new Date().toISOString();
    const nextRecord: ScriptTimeTemplateValuesDto = {
      id: record.value?.id ?? (await taskService.requestUuid()),
      deviceId: props.scope.deviceId,
      scriptId: props.script.id,
      timeTemplateId: props.scope.timeTemplateId,
      accountId: props.scope.accountId ?? null,
      valuesJson: nextValuesJson,
      createdAt: record.value?.createdAt ?? now,
      updatedAt: now,
    };

    await scriptTemplateValueService.save(nextRecord);
    const savedRecord = await scriptTemplateValueService.get(
      props.scope.deviceId,
      props.script.id,
      props.scope.timeTemplateId,
      props.scope.accountId ?? null,
    );
    const savedVariables = extractStoredVariables(savedRecord);
    const savedTaskSettings = extractStoredTaskSettings(savedRecord);
    const expectedSnapshot = stableStringify(variables);
    const actualSnapshot = stableStringify(
      Object.fromEntries(Object.keys(variables).map((key) => [key, savedVariables[key]])),
    );
    const expectedTaskSettingsSnapshot = stableStringify(taskSettingsPayload);
    const actualTaskSettingsSnapshot = stableStringify(
      Object.fromEntries(Object.keys(taskSettingsPayload).map((key) => [key, savedTaskSettings[key]])),
    );
    if (actualSnapshot !== expectedSnapshot || actualTaskSettingsSnapshot !== expectedTaskSettingsSnapshot) {
      throw new Error('保存后读取到的模板变量仍是旧值，请检查当前作用域是否存在重复记录。');
    }
    record.value = savedRecord;
    entries.value = buildEntriesFromStored(savedVariables);
    taskSettings.value = createTemplateTaskSettingEntries(previewTasks.value, savedTaskSettings);
    loadedSnapshot.value = stableStringify(entries.value);
    taskSettingsSnapshot.value = stableStringify(taskSettings.value);
    previewVersion.value += 1;
    showToast('模板变量已保存', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '模板变量保存失败', 'error');
  } finally {
    saving.value = false;
  }
};

watch(
  selectableTasks,
  (tasks) => {
    if (tasks.some((task) => task.id === selectedTaskId.value)) {
      return;
    }
    selectedTaskId.value = tasks[0]?.id ?? null;
  },
  { immediate: true },
);

watch(
  previewTasks,
  () => {
    const wasVariableDirty = dirty.value;
    const wasDirty = taskSettingsDirty.value;
    if (!wasVariableDirty) {
      entries.value = buildEntriesFromStored(extractStoredVariables(record.value));
      loadedSnapshot.value = stableStringify(entries.value);
    }
    rebuildTaskSettings(extractStoredTaskSettings(record.value), false);
    if (!wasDirty) {
      taskSettingsSnapshot.value = stableStringify(taskSettings.value);
    }
  },
);

watch(
  () => [props.script.id, props.scope.deviceId, props.scope.timeTemplateId, props.scope.accountId ?? ''].join('::'),
  () => {
    void loadValues();
  },
  { immediate: true },
);
</script>
