<template>
  <div class="editor-shell h-[100svh] overflow-hidden px-4 py-4 lg:px-6 lg:py-5">
    <div class="mx-auto flex h-full max-w-[1760px] flex-col gap-4">
      <header class="editor-toolbar rounded-[28px] border border-[var(--app-border)] px-5 py-4 lg:px-6">
        <div class="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
          <div class="flex flex-wrap items-center gap-3">
            <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="router.push('/scripts')">
              <ArrowLeft class="h-4 w-4" />
              返回
            </button>

            <div class="space-y-1">
              <div class="flex items-center gap-2 text-xs uppercase tracking-[0.18em] text-[var(--app-text-faint)]">
                <span>Task Editor</span>
                <span class="rounded-full border border-[var(--app-border)] bg-white/40 px-3 py-1">脚本开发工作台</span>
              </div>
              <h1 class="text-2xl font-semibold tracking-[-0.05em] text-[var(--app-text-strong)] lg:text-3xl">
                {{ draftScript?.data.name || '脚本编辑器' }}
              </h1>
            </div>
          </div>

          <div class="flex flex-wrap items-center gap-2">
            <span
              class="rounded-full px-3 py-1 text-xs font-medium"
              :class="hasValidationErrors ? 'bg-red-500/12 text-red-700' : dirty ? 'bg-amber-500/12 text-amber-700' : 'bg-emerald-500/12 text-emerald-700'"
            >
              {{ hasValidationErrors ? '待修复' : dirty ? '未保存' : '已同步' }}
            </span>
            <span v-if="formattedSaveTime" class="text-xs text-[var(--app-text-faint)]">最近保存 {{ formattedSaveTime }}</span>
            <button class="app-button app-button-ghost app-toolbar-button" type="button" data-testid="editor-script-info" @click="infoDialogOpen = true">
              编辑脚本信息
            </button>
            <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="reloadEditor">
              <RefreshCcw class="h-4 w-4" />
              重新载入
            </button>
            <button
              class="app-button app-button-primary"
              type="button"
              data-testid="editor-save"
              :disabled="!draftScript || isSaving || hasValidationErrors"
              @click="saveEditor"
            >
              <Save class="h-4 w-4" />
              {{ isSaving ? '保存中...' : '保存脚本结构' }}
            </button>
          </div>
        </div>
      </header>

      <div v-if="loadError" class="rounded-[28px] border border-red-500/16 bg-red-500/8 px-6 py-8 text-red-700">
        <h2 class="text-xl font-semibold">无法打开编辑器</h2>
        <p class="mt-3 max-w-2xl text-sm leading-6">{{ loadError }}</p>
      </div>

      <div
        v-else-if="isLoading"
        class="rounded-[28px] border border-[var(--app-border)] bg-[var(--app-panel)] px-6 py-10 text-sm text-[var(--app-text-soft)]"
      >
        正在读取脚本和任务结构...
      </div>

      <div v-else class="grid min-h-0 flex-1 gap-4 xl:grid-cols-[280px_360px_minmax(720px,1fr)]">
        <EditorTaskSidebar
          :tasks="draftTasks"
          :selected-task-id="selectedTaskId"
          @create="createTask"
          @select="selectTask"
          @duplicate="duplicateTask"
          @toggle-hidden="toggleTaskHidden"
          @remove="removeTask"
          @reorder="reorderTasks"
        />

        <EditorTaskConfigPanel
          :task="currentTask"
          :active-panel="activePanel"
          :task-name="taskName"
          :task-type="taskType"
          :task-hidden="taskHidden"
          :input-entries="inputEntries"
          :input-error="inputError"
          :ui-schema="uiSchema"
          :selected-input-id="selectedInputId"
          :selected-ui-field-id="selectedUiFieldId"
          @update:active-panel="activePanel = $event"
          @update:task-name="taskName = $event"
          @update:task-type="taskType = $event"
          @update:task-hidden="taskHidden = $event"
          @add-input="addInput"
          @select-input="selectedInputId = $event"
          @update-ui-layout="updateUiLayout"
          @add-ui-field="addUiField"
          @select-ui-field="selectedUiFieldId = $event"
          @remove-ui-field="removeUiField"
          @append-template-step="appendTemplateStep"
          @open-raw="openRawEditor"
        />

        <EditorTaskWorkspace
          :task="currentTask"
          :active-panel="activePanel"
          :steps="parsedSteps"
          :selected-step-path="selectedStepPath"
          :active-branch-path="activeBranchPath"
          :ui-schema="uiSchema"
          :selected-ui-field-id="selectedUiFieldId"
          :input-entries="inputEntries"
          :variable-options="variableOptions"
          :catalog-variable-options="catalogVariableOptions"
          :selected-input-id="selectedInputId"
          @update-input="updateInput"
          @remove-input="removeInput"
          @select-input="selectedInputId = $event"
          @select-ui-field="selectedUiFieldId = $event"
          @update-ui-field="updateUiField"
          @remove-ui-field="removeUiField"
          @select-step-path="selectStepPath"
          @navigate-branch="navigateBranch"
          @reorder-step="reorderSteps"
          @remove-step="removeStep"
          @update-step="updateStep"
          @open-raw="openRawEditor"
        />
      </div>
    </div>

    <ScriptInfoDialog
      :open="infoDialogOpen"
      mode="edit"
      :script="draftScript"
      @close="infoDialogOpen = false"
      @save="applyScriptInfo"
    />

    <EditorJsonDialog
      :open="rawDialogOpen"
      :title="rawDialogTitle"
      :description="rawDialogDescription"
      :model-value="rawDialogText"
      :error="rawDialogError"
      @close="rawDialogOpen = false"
      @apply="applyRawEditor"
      @format="formatRawEditor"
      @update:model-value="rawDialogText = $event"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { ArrowLeft, RefreshCcw, Save } from 'lucide-vue-next';
import { useScriptStore } from '@/store/script';
import { taskService } from '@/services/taskService';
import type { JsonValue, ScriptTableRecord } from '@/types/app/domain';
import type { Step } from '@/types/bindings/Step';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import { showToast } from '@/utils/toast';
import ScriptInfoDialog from '@/views/script-list/ScriptInfoDialog.vue';
import EditorJsonDialog from '@/views/script-editor/EditorJsonDialog.vue';
import EditorTaskConfigPanel from '@/views/script-editor/EditorTaskConfigPanel.vue';
import EditorTaskSidebar from '@/views/script-editor/EditorTaskSidebar.vue';
import EditorTaskWorkspace from '@/views/script-editor/EditorTaskWorkspace.vue';
import { createStepFromTemplate } from '@/views/script-editor/editorStepTemplates';
import {
  buildStepPath,
  cloneStepPath,
  createSiblingSelection,
  getBranchSteps,
  getParentBranchPath,
  getStepByPath,
  isSameBranchPath,
  ROOT_BRANCH_PATH,
  type StepBranchPath,
  type StepPath,
  updateBranchSteps,
  updateStepByPath,
} from '@/views/script-editor/editorStepTree';
import {
  buildUiData,
  cloneJson,
  createUiField,
  createUiSchema,
  parseUiSchema,
  stableStringify,
  type EditorPanelId,
  type EditorUiSchema,
  type RawEditorSection,
  type UiFieldControl,
} from '@/views/script-editor/editorSchema';
import {
  buildInputJson,
  createInputEntry,
  listVariableOptions,
  parseInputEntries,
  syncInputVariableCatalog,
  type EditorInputEntry,
} from '@/views/script-editor/editorVariables';

const route = useRoute();
const router = useRouter();
const scriptStore = useScriptStore();

const isLoading = ref(true);
const isSaving = ref(false);
const loadError = ref<string | null>(null);
const saveTime = ref<string | null>(null);

const infoDialogOpen = ref(false);
const rawDialogOpen = ref(false);
const rawDialogSection = ref<RawEditorSection>('steps');
const rawDialogText = ref('');
const rawDialogError = ref<string | null>(null);

const activePanel = ref<EditorPanelId>('basic');
const selectedTaskId = ref<string | null>(null);
const selectedInputId = ref<string | null>(null);
const selectedStepPath = ref<StepPath | null>(null);
const activeBranchPath = ref<StepBranchPath>(ROOT_BRANCH_PATH);
const selectedUiFieldId = ref<string | null>(null);

const draftTasks = ref<ScriptTaskTable[]>([]);
const draftScript = ref<ScriptTableRecord | null>(null);
const sourceTasksSnapshot = ref('');
const sourceScriptSnapshot = ref('');

const taskName = ref('');
const taskType = ref<'main' | 'child'>('main');
const taskHidden = ref(false);
const inputEntries = ref<EditorInputEntry[]>([]);
const inputError = ref<string | null>(null);
const uiSchema = ref<EditorUiSchema>(createUiSchema());

const hydratingTaskMeta = ref(false);
const hydratingTaskPanels = ref(false);

const scriptId = computed(() => (typeof route.query.scriptId === 'string' ? route.query.scriptId : ''));

const currentTask = computed<ScriptTaskTable | null>(() => {
  const tasks = draftTasks.value as ScriptTaskTable[];
  const selected = selectedTaskId.value;
  if (!selected) {
    return tasks[0] ?? null;
  }

  const matched = tasks.find((task) => task.id === selected) ?? null;
  return matched ?? tasks[0] ?? null;
});

const variableOptions = computed(() =>
  listVariableOptions(draftScript.value?.data.variableCatalog, currentTask.value?.id ?? null, parsedSteps.value),
);
const catalogVariableOptions = computed(() =>
  listVariableOptions(draftScript.value?.data.variableCatalog, currentTask.value?.id ?? null, parsedSteps.value, 'read', false),
);

const parsedSteps = computed<Step[]>(() => (currentTask.value?.data.steps as Step[] | undefined) ?? []);
const hasValidationErrors = computed(() => Boolean(inputError.value));

const dirty = computed(() => {
  if (!draftScript.value) {
    return false;
  }

  return stableStringify(draftScript.value) !== sourceScriptSnapshot.value || stableStringify(draftTasks.value) !== sourceTasksSnapshot.value;
});

const formattedSaveTime = computed(() => {
  if (!saveTime.value) {
    return '';
  }

  return new Date(saveTime.value).toLocaleString('zh-TW', {
    hour12: false,
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
});

const rawDialogTitle = computed(() => {
  switch (rawDialogSection.value) {
    case 'inputs':
      return '输入变量 JSON';
    case 'ui':
      return 'UI Schema JSON';
    default:
      return '步骤 JSON';
  }
});

const rawDialogDescription = computed(() => {
  switch (rawDialogSection.value) {
    case 'inputs':
      return '这里是 input.* 的底层结构，作为调试入口保留。';
    case 'ui':
      return '这里是 UI schema 的底层结构，优先在可视化面板里编辑。';
    default:
      return '这里是任务步骤的底层结构，优先在可视化工作区里查看和调整。';
  }
});

const normalizeTask = (task: ScriptTaskTable, index: number): ScriptTaskTable => ({
  ...task,
  scriptId: task.scriptId || scriptId.value,
  name: task.name || `任务 ${index + 1}`,
  taskType: task.taskType ?? 'main',
  isHidden: Boolean(task.isHidden),
  index,
  createdAt: task.createdAt || new Date().toISOString(),
  updatedAt: task.updatedAt || new Date().toISOString(),
  deletedAt: task.deletedAt ?? null,
  isDeleted: Boolean(task.isDeleted),
  data: {
    uiData: task.data?.uiData ?? {},
    variables: task.data?.variables ?? {},
    steps: Array.isArray(task.data?.steps) ? task.data.steps : [],
  },
});

const buildTaskDraft = async (name?: string): Promise<ScriptTaskTable> => {
  const index = draftTasks.value.length;
  return normalizeTask(
    {
      id: await taskService.requestUuid(),
      scriptId: scriptId.value,
      name: name || `新任务 ${index + 1}`,
      isHidden: false,
      taskType: 'main',
      data: {
        uiData: {},
        variables: {},
        steps: [],
      },
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      deletedAt: null,
      isDeleted: false,
      index,
    },
    index,
  );
};

const replaceTask = (taskId: string, updater: (task: ScriptTaskTable) => ScriptTaskTable) => {
  draftTasks.value = draftTasks.value.map((task, index) => {
    if (task.id !== taskId) {
      return normalizeTask(task, index);
    }

    return normalizeTask(updater(cloneJson(task)), index);
  });
};

const hydrateTaskEditors = () => {
  hydratingTaskMeta.value = true;
  hydratingTaskPanels.value = true;

  if (!currentTask.value) {
    taskName.value = '';
    taskType.value = 'main';
    taskHidden.value = false;
    inputEntries.value = [];
    inputError.value = null;
    selectedInputId.value = null;
    uiSchema.value = createUiSchema();
    selectedStepPath.value = null;
    activeBranchPath.value = ROOT_BRANCH_PATH;
    selectedUiFieldId.value = null;
  } else {
    taskName.value = currentTask.value.name;
    taskType.value = currentTask.value.taskType;
    taskHidden.value = currentTask.value.isHidden;
    inputEntries.value = parseInputEntries(draftScript.value?.data.variableCatalog, currentTask.value.id, currentTask.value.data.variables ?? {});
    inputError.value = null;
    selectedInputId.value = inputEntries.value.find((entry) => entry.id === selectedInputId.value)?.id ?? inputEntries.value[0]?.id ?? null;
    uiSchema.value = parseUiSchema(currentTask.value.data.uiData ?? {});
    if (!currentTask.value.data.steps.length) {
      selectedStepPath.value = null;
      activeBranchPath.value = ROOT_BRANCH_PATH;
    } else if (!selectedStepPath.value || !getStepByPath(currentTask.value.data.steps, selectedStepPath.value)) {
      selectedStepPath.value = buildStepPath(ROOT_BRANCH_PATH, 0);
      activeBranchPath.value = ROOT_BRANCH_PATH;
    }
    selectedUiFieldId.value =
      uiSchema.value.fields.find((field) => field.id === selectedUiFieldId.value)?.id ?? uiSchema.value.fields[0]?.id ?? null;
  }

  queueMicrotask(() => {
    hydratingTaskMeta.value = false;
    hydratingTaskPanels.value = false;
  });
};

const setCurrentTaskSteps = (steps: Step[]) => {
  if (!currentTask.value) {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.data.steps = steps;
    return task;
  });

  if (!steps.length) {
    selectedStepPath.value = null;
    activeBranchPath.value = ROOT_BRANCH_PATH;
  }
};

const reorderCollection = <T,>(items: T[], fromIndex: number, toIndex: number) => {
  const next = [...items];
  const [moved] = next.splice(fromIndex, 1);
  next.splice(toIndex, 0, moved);
  return next;
};

const createTask = async () => {
  const nextTask = await buildTaskDraft();
  draftTasks.value = [...draftTasks.value, nextTask].map((task, index) => normalizeTask(task, index));
  selectedTaskId.value = nextTask.id;
  activePanel.value = 'basic';
};

const selectTask = (taskId: string) => {
  selectedTaskId.value = taskId;
};

const duplicateTask = async (taskId: string) => {
  const source = draftTasks.value.find((task) => task.id === taskId);
  if (!source) {
    return;
  }

  const duplicate = normalizeTask(
    {
      ...cloneJson(source),
      id: await taskService.requestUuid(),
      name: `${source.name} 副本`,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    },
    draftTasks.value.length,
  );

  draftTasks.value = [...draftTasks.value, duplicate].map((task, index) => normalizeTask(task, index));
  selectedTaskId.value = duplicate.id;
};

const removeTask = (taskId: string) => {
  if (draftTasks.value.length <= 1) {
    showToast('至少保留一个任务，避免脚本变成空壳。', 'warning');
    return;
  }

  draftTasks.value = draftTasks.value
    .filter((task) => task.id !== taskId)
    .map((task, index) => normalizeTask(task, index));

  if (selectedTaskId.value === taskId) {
    selectedTaskId.value = draftTasks.value[0]?.id ?? null;
  }
};

const toggleTaskHidden = (taskId: string) => {
  replaceTask(taskId, (task) => {
    task.isHidden = !task.isHidden;
    return task;
  });
};

const reorderTasks = (draggedTaskId: string, targetTaskId: string) => {
  const fromIndex = draftTasks.value.findIndex((task) => task.id === draggedTaskId);
  const toIndex = draftTasks.value.findIndex((task) => task.id === targetTaskId);
  if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) {
    return;
  }

  draftTasks.value = reorderCollection(draftTasks.value, fromIndex, toIndex).map((task, index) => normalizeTask(task, index));
};

const addInput = () => {
  const nextEntry = createInputEntry('int');
  inputEntries.value = [...inputEntries.value, nextEntry];
  selectedInputId.value = nextEntry.id;
};

const updateInput = (
  entryId: string,
  field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue',
  value: string | boolean,
) => {
  inputEntries.value = inputEntries.value.map((entry) => {
    if (entry.id !== entryId) {
      return entry;
    }

    const next = { ...entry };
    if (field === 'type') {
      next.type = value as EditorInputEntry['type'];
      next.stringValue = next.type === 'string' ? '' : next.type === 'json' ? '{}' : '0';
      next.booleanValue = false;
      return next;
    }

    if (field === 'namespace') {
      next.namespace = String(value) as EditorInputEntry['namespace'];
      return next;
    }

    if (field === 'booleanValue') {
      next.booleanValue = Boolean(value);
      return next;
    }

    next[field] = String(value) as never;
    return next;
  });
};

const removeInput = (entryId: string) => {
  inputEntries.value = inputEntries.value.filter((entry) => entry.id !== entryId);
  if (selectedInputId.value === entryId) {
    selectedInputId.value = inputEntries.value[0]?.id ?? null;
  }
};

const updateUiLayout = (value: 'horizontal' | 'vertical') => {
  uiSchema.value = {
    ...uiSchema.value,
    layout: value,
  };
};

const addUiField = (control: UiFieldControl) => {
  const field = createUiField(control);
  uiSchema.value = {
    ...uiSchema.value,
    fields: [...uiSchema.value.fields, field],
  };
  selectedUiFieldId.value = field.id;
};

const updateUiField = (
  fieldId: string,
  key: 'label' | 'key' | 'variableId' | 'inputKey' | 'description' | 'placeholder' | 'optionsText',
  value: string,
) => {
  uiSchema.value = {
    ...uiSchema.value,
    fields: uiSchema.value.fields.map((field) => (field.id === fieldId ? { ...field, [key]: value } : field)),
  };
};

const removeUiField = (fieldId: string) => {
  uiSchema.value = {
    ...uiSchema.value,
    fields: uiSchema.value.fields.filter((field) => field.id !== fieldId),
  };
  if (selectedUiFieldId.value === fieldId) {
    selectedUiFieldId.value = uiSchema.value.fields[0]?.id ?? null;
  }
};

const appendTemplateStep = (templateId: string) => {
  const step = createStepFromTemplate(templateId);
  if (!step) {
    return;
  }

  const nextSteps = updateBranchSteps(
    parsedSteps.value,
    activeBranchPath.value,
    (steps) => [...steps, step],
  );
  setCurrentTaskSteps(nextSteps);
  selectedStepPath.value = buildStepPath(activeBranchPath.value, getBranchSteps(nextSteps, activeBranchPath.value).length - 1);
  activePanel.value = 'steps';
};

const reorderSteps = (fromIndex: number, toIndex: number) => {
  if (fromIndex === toIndex) {
    return;
  }

  const nextSteps = updateBranchSteps(parsedSteps.value, activeBranchPath.value, (steps) => reorderCollection(steps, fromIndex, toIndex));
  setCurrentTaskSteps(nextSteps);
  selectedStepPath.value = buildStepPath(activeBranchPath.value, toIndex);
};

const removeStep = (index: number) => {
  const nextSteps = updateBranchSteps(parsedSteps.value, activeBranchPath.value, (steps) => steps.filter((_, stepIndex) => stepIndex !== index));
  setCurrentTaskSteps(nextSteps);
  selectedStepPath.value = createSiblingSelection(activeBranchPath.value, getBranchSteps(nextSteps, activeBranchPath.value).length, index);
};

const updateStep = (index: number, nextStep: Step) => {
  const nextSteps = updateStepByPath(parsedSteps.value, buildStepPath(activeBranchPath.value, index), () => nextStep);
  setCurrentTaskSteps(nextSteps);
  selectedStepPath.value = buildStepPath(activeBranchPath.value, index);
};

const selectStepPath = (path: StepPath) => {
  selectedStepPath.value = cloneStepPath(path);
  activeBranchPath.value = getParentBranchPath(path);
};

const navigateBranch = (branchPath: StepBranchPath) => {
  activeBranchPath.value = {
    branch: branchPath.branch,
    parentStepPath: cloneStepPath(branchPath.parentStepPath),
  };

  if (selectedStepPath.value && isSameBranchPath(getParentBranchPath(selectedStepPath.value), activeBranchPath.value)) {
    return;
  }

  const steps = getBranchSteps(parsedSteps.value, activeBranchPath.value);
  selectedStepPath.value = steps.length ? buildStepPath(activeBranchPath.value, 0) : null;
};

const openRawEditor = (section: RawEditorSection) => {
  if (!currentTask.value) {
    return;
  }

  rawDialogSection.value = section;
  rawDialogError.value = null;
  rawDialogText.value = stableStringify(
    section === 'inputs'
      ? currentTask.value.data.variables ?? {}
      : section === 'ui'
        ? currentTask.value.data.uiData ?? {}
        : currentTask.value.data.steps ?? [],
  );
  rawDialogOpen.value = true;
};

const formatRawEditor = () => {
  try {
    rawDialogText.value = stableStringify(JSON.parse(rawDialogText.value) as JsonValue);
    rawDialogError.value = null;
  } catch (error) {
    rawDialogError.value = error instanceof Error ? error.message : 'JSON 结构无效';
  }
};

const applyRawEditor = () => {
  if (!currentTask.value) {
    return;
  }

  try {
    const parsed = JSON.parse(rawDialogText.value) as JsonValue;
    if (rawDialogSection.value === 'steps' && !Array.isArray(parsed)) {
      throw new Error('步骤 JSON 顶层必须是数组。');
    }

    replaceTask(currentTask.value.id, (task) => {
      if (rawDialogSection.value === 'inputs') {
        task.data.variables = parsed;
      } else if (rawDialogSection.value === 'ui') {
        task.data.uiData = parsed;
      } else {
        task.data.steps = parsed as Step[];
      }
      return task;
    });

    hydrateTaskEditors();
    rawDialogError.value = null;
    rawDialogOpen.value = false;
  } catch (error) {
    rawDialogError.value = error instanceof Error ? error.message : 'JSON 结构无效';
  }
};

const applyScriptInfo = (script: ScriptTableRecord) => {
  draftScript.value = cloneJson(script);
  infoDialogOpen.value = false;
  showToast('脚本信息已写入当前草稿，顶部保存后生效。', 'success');
};

const buildSavePayload = () =>
  draftTasks.value.map((task, index) =>
    normalizeTask(
      {
        ...task,
        scriptId: scriptId.value,
      },
      index,
    ),
  );

const saveEditor = async () => {
  if (!draftScript.value) {
    return;
  }

  if (hasValidationErrors.value) {
    showToast('请先修复输入变量里的错误，再执行保存。', 'error');
    return;
  }

  isSaving.value = true;

  try {
    const nextSaveTime = new Date().toISOString();
    const tasks = buildSavePayload().map((task) => ({
      ...task,
      updatedAt: nextSaveTime,
    }));
    const script = {
      ...draftScript.value,
      data: {
        ...draftScript.value.data,
        updateTime: nextSaveTime,
      },
    };

    await scriptStore.saveScriptTasks(script.id, tasks);
    await scriptStore.saveScript(script);

    draftTasks.value = tasks;
    draftScript.value = script;
    sourceTasksSnapshot.value = stableStringify(tasks);
    sourceScriptSnapshot.value = stableStringify(script);
    saveTime.value = nextSaveTime;
    showToast('脚本编辑结果已保存', 'success');
  } catch (error) {
    console.error(error);
    showToast(error instanceof Error ? error.message : '保存失败', 'error');
  } finally {
    isSaving.value = false;
  }
};

const loadEditor = async () => {
  isLoading.value = true;
  loadError.value = null;

  try {
    if (!scriptId.value) {
      throw new Error('缺少 scriptId 参数，无法确定要打开哪个脚本。');
    }

    if (!scriptStore.scripts.length) {
      await scriptStore.loadScripts();
    }

    const sourceScript = (scriptStore.scripts as ScriptTableRecord[]).find((item) => item.id === scriptId.value) ?? null;
    if (!sourceScript) {
      throw new Error('当前脚本不存在，可能已被删除或尚未加载到本地列表。');
    }

    draftScript.value = cloneJson(sourceScript);
    sourceScriptSnapshot.value = stableStringify(draftScript.value);

    const loadedTasks = await scriptStore.loadScriptTasks(sourceScript.id);
    if (loadedTasks.length) {
      draftTasks.value = loadedTasks.map((task, index) => normalizeTask(task, index));
      sourceTasksSnapshot.value = stableStringify(draftTasks.value);
    } else {
      draftTasks.value = [await buildTaskDraft('主任务 1')];
      sourceTasksSnapshot.value = stableStringify([]);
    }

    selectedTaskId.value = draftTasks.value[0]?.id ?? null;
    activeBranchPath.value = ROOT_BRANCH_PATH;
    saveTime.value = sourceScript.data.updateTime || null;
    hydrateTaskEditors();
  } catch (error) {
    console.error(error);
    loadError.value = error instanceof Error ? error.message : '脚本编辑器初始化失败';
  } finally {
    isLoading.value = false;
  }
};

const reloadEditor = async () => {
  await loadEditor();
};

const handleKeydown = (event: KeyboardEvent) => {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
    event.preventDefault();
    if (!isSaving.value) {
      void saveEditor();
    }
  }
};

watch(
  () => currentTask.value?.id,
  () => {
    hydrateTaskEditors();
  },
  { immediate: true },
);

watch(taskName, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value) {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.name = value.trim() || '未命名任务';
    return task;
  });
});

watch(taskType, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value) {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.taskType = value;
    return task;
  });
});

watch(taskHidden, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value) {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.isHidden = value;
    return task;
  });
});

watch(
  inputEntries,
  (entries) => {
    if (!currentTask.value || hydratingTaskPanels.value) {
      return;
    }

    try {
      const nextVariables = buildInputJson(entries);
      const nextCatalog = syncInputVariableCatalog(draftScript.value?.data.variableCatalog, currentTask.value.id, entries);
      inputError.value = null;
      replaceTask(currentTask.value.id, (task) => {
        task.data.variables = nextVariables;
        return task;
      });
      if (draftScript.value) {
        draftScript.value = {
          ...draftScript.value,
          data: {
            ...draftScript.value.data,
            variableCatalog: nextCatalog,
          },
        };
      }
    } catch (error) {
      inputError.value = error instanceof Error ? error.message : '输入变量结构无效';
    }
  },
  { deep: true },
);

watch(
  uiSchema,
  (value) => {
    if (!currentTask.value || hydratingTaskPanels.value) {
      return;
    }

    replaceTask(currentTask.value.id, (task) => {
      task.data.uiData = buildUiData(value);
      return task;
    });
  },
  { deep: true },
);

watch(
  () => scriptId.value,
  async () => {
    await loadEditor();
  },
  { immediate: true },
);

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown);
});
</script>

<style scoped>
.editor-shell {
  background:
    radial-gradient(circle at 10% 12%, rgba(70, 110, 255, 0.12), transparent 24%),
    radial-gradient(circle at 88% 14%, rgba(87, 196, 255, 0.15), transparent 22%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.22), rgba(255, 255, 255, 0)),
    transparent;
}

.editor-toolbar {
  background:
    radial-gradient(circle at 16% 20%, rgba(255, 255, 255, 0.42), transparent 30%),
    linear-gradient(135deg, rgba(255, 255, 255, 0.62), rgba(245, 248, 255, 0.34)),
    var(--app-panel);
  box-shadow: var(--app-shadow-soft);
  backdrop-filter: blur(16px);
}
</style>
