<template>
  <div class="space-y-4">
    <div v-if="loading" class="rounded-[18px] border border-[var(--app-border)] px-4 py-8 text-sm text-[var(--app-text-soft)]">
      正在读取模板变量...
    </div>

    <div v-else-if="loadError" class="rounded-[18px] border border-red-500/18 bg-red-500/8 px-4 py-4 text-sm text-red-700">
      {{ loadError }}
    </div>

    <div
      v-else-if="!entries.length"
      class="rounded-[18px] border border-dashed border-[var(--app-border)] px-4 py-6 text-sm text-[var(--app-text-soft)]"
    >
      当前脚本还没有可持久化的 input 变量。
    </div>

    <template v-else>
      <div class="flex flex-wrap items-center justify-between gap-3 rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-3">
        <div class="flex flex-wrap items-center gap-2 text-xs text-[var(--app-text-faint)]">
          <span class="rounded-full border border-[var(--app-border)] px-3 py-1">{{ entries.length }} 个变量</span>
          <span v-if="recordUpdatedAt" class="rounded-full border border-[var(--app-border)] px-3 py-1">上次保存 {{ recordUpdatedAt }}</span>
          <span
            class="rounded-full px-3 py-1"
            :class="dirty ? 'bg-amber-500/12 text-amber-700' : 'bg-emerald-500/12 text-emerald-700'"
          >
            {{ dirty ? '未保存' : '已同步' }}
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
        :tasks="previewTasks"
        :selected-task-id="selectedTaskId"
        :selected-task-ui-schema="selectedTaskUiSchema"
        :selected-task-input-entries="previewInputEntries"
        :shared-input-entries="previewInputEntries"
        :selected-ui-field-id="null"
        :selected-task-cycle-value="'everyRun'"
        :selected-task-cycle-mode="'named'"
        :selected-task-cycle-day="1"
        :show-task-cycle="false"
        @select-task="selectedTaskId = $event"
        @update-input="handlePreviewInputUpdate"
      />

      <div
        v-else
        class="rounded-[18px] border border-dashed border-[var(--app-border)] px-4 py-6 text-sm text-[var(--app-text-soft)]"
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
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import { createUiSchema, parseUiSchema, stableStringify } from '@/views/script-editor/editorSchema';
import { getVariableValueTypeLabel } from '@/views/script-editor/editorVariables';
import EditorTaskTablePreview from '@/views/script-editor/EditorTaskTablePreview.vue';
import {
  buildTemplateEditorInputs,
  buildTemplateVariablePayload,
  createTemplateVariableEntries,
  type TemplateVariableEntry,
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
const loading = ref(false);
const saving = ref(false);
const loadError = ref<string | null>(null);
const record = shallowRef<ScriptTimeTemplateValuesDto | null>(null);
const loadedSnapshot = ref('');
const selectedTaskId = ref<string | null>(null);

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
const previewTasks = computed(() => props.tasks.filter((task) => !task.isDeleted));
const selectableTasks = computed(() => previewTasks.value.filter((task) => task.rowType === 'task'));
const previewInputEntries = computed(() => buildTemplateEditorInputs(entries.value));

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
  createTemplateVariableEntries(props.script, props.tasks, storedVariables);

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
    const valuesRoot = isRecord(nextRecord?.valuesJson) ? nextRecord.valuesJson : {};
    const storedVariables = isRecord(valuesRoot.variables) ? valuesRoot.variables : {};
    record.value = nextRecord;
    entries.value = buildEntriesFromStored(storedVariables);
    loadedSnapshot.value = stableStringify(entries.value);
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

const resetToDefaults = () => {
  entries.value = buildEntriesFromStored({});
};

const saveValues = async () => {
  saving.value = true;

  try {
    const variables = buildTemplateVariablePayload(entries.value);
    const currentRoot = isRecord(record.value?.valuesJson) ? record.value.valuesJson : {};
    const nextValuesJson = Object.assign({}, currentRoot, { variables }) as JsonValue;
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
    record.value = nextRecord;
    loadedSnapshot.value = stableStringify(entries.value);
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
  () => [props.script.id, props.scope.deviceId, props.scope.timeTemplateId, props.scope.accountId ?? ''].join('::'),
  () => {
    void loadValues();
  },
  { immediate: true },
);
</script>
