import type { ScriptTableRecord } from '@/types/app/domain';
import type { PolicyGroupTable } from '@/types/bindings/PolicyGroupTable';
import type { PolicySetTable } from '@/types/bindings/PolicySetTable';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { Step } from '@/types/bindings/Step';
import type { TaskCycle } from '@/types/bindings/TaskCycle';
import type { TaskRowType } from '@/types/bindings/TaskRowType';
import type { TaskTone } from '@/types/bindings/TaskTone';
import type { TaskTriggerMode } from '@/types/bindings/TaskTriggerMode';
import { TASK_CYCLE_VALUE, TASK_ROW_TYPE, TASK_TONE, TASK_TRIGGER_MODE } from '@/views/script-editor/editor-step/editorStepKinds';
import { buildStepPath, cloneStepPath, getStepByPath, ROOT_BRANCH_PATH, type StepBranchPath, type StepPath } from '@/views/script-editor/editor-step/editorStepTree';
import { createUiSchema, parseUiSchema, type EditorPanelId, type EditorUiSchema } from '@/views/script-editor/editorSchema';
import type { PolicyEditorPanelId } from '@/views/script-editor/editor-policy/editorPolicy';
import type { EditorModeId } from '@/views/script-editor/editor-policy/editorPolicy';
import { parseInputEntries, type EditorInputEntry } from '@/views/script-editor/editorVariables';
import { cloneJson } from '@/views/script-editor/editorSchema';

export interface ScriptEditorPersistedViewState {
  activeMode?: EditorModeId;
  selectedTaskId?: string | null;
  selectedPolicyId?: string | null;
  selectedPolicyGroupId?: string | null;
  selectedPolicySetId?: string | null;
  activePanel?: EditorPanelId;
  activePolicyPanel?: PolicyEditorPanelId;
  selectedStepPath?: StepPath | null;
  activeBranchPath?: StepBranchPath;
  selectedPolicyStepPathBefore?: StepPath | null;
  activePolicyBranchPathBefore?: StepBranchPath;
  selectedPolicyStepPathAfter?: StepPath | null;
  activePolicyBranchPathAfter?: StepBranchPath;
}

export const isEditorModeId = (value: unknown): value is EditorModeId =>
  value === 'task' || value === 'policy' || value === 'policyGroup' || value === 'policySet';

export const isEditorPanelId = (value: unknown): value is EditorPanelId =>
  value === 'basic' || value === 'inputs' || value === 'ui' || value === 'steps';

export const isPolicyEditorPanelId = (value: unknown): value is PolicyEditorPanelId =>
  value === 'basic' || value === 'inputs' || value === 'before' || value === 'after';

export const isStepPathLike = (value: unknown): value is StepPath =>
  Array.isArray(value)
  && value.every(
    (segment) =>
      Boolean(segment)
      && typeof segment === 'object'
      && typeof (segment as { branch?: unknown }).branch === 'string'
      && typeof (segment as { index?: unknown }).index === 'number',
  );

export const isStepBranchPathLike = (value: unknown): value is StepBranchPath =>
  Boolean(value)
  && typeof value === 'object'
  && typeof (value as StepBranchPath).branch === 'string';

export const buildPersistedEditorViewState = ({
  activeMode,
  selectedTaskId,
  selectedPolicyId,
  selectedPolicyGroupId,
  selectedPolicySetId,
  activePanel,
  activePolicyPanel,
  selectedStepPath,
  activeBranchPath,
  selectedPolicyStepPathBefore,
  activePolicyBranchPathBefore,
  selectedPolicyStepPathAfter,
  activePolicyBranchPathAfter,
}: {
  activeMode: EditorModeId;
  selectedTaskId: string | null;
  selectedPolicyId: string | null;
  selectedPolicyGroupId: string | null;
  selectedPolicySetId: string | null;
  activePanel: EditorPanelId;
  activePolicyPanel: PolicyEditorPanelId;
  selectedStepPath: StepPath | null;
  activeBranchPath: StepBranchPath;
  selectedPolicyStepPathBefore: StepPath | null;
  activePolicyBranchPathBefore: StepBranchPath;
  selectedPolicyStepPathAfter: StepPath | null;
  activePolicyBranchPathAfter: StepBranchPath;
}): ScriptEditorPersistedViewState => ({
  activeMode,
  selectedTaskId,
  selectedPolicyId,
  selectedPolicyGroupId,
  selectedPolicySetId,
  activePanel,
  activePolicyPanel,
  selectedStepPath: activeMode === 'task' && activePanel === 'steps' ? selectedStepPath : null,
  activeBranchPath: activeMode === 'task' && activePanel === 'steps' ? activeBranchPath : ROOT_BRANCH_PATH,
  selectedPolicyStepPathBefore: activeMode === 'policy' && activePolicyPanel === 'before' ? selectedPolicyStepPathBefore : null,
  activePolicyBranchPathBefore: activeMode === 'policy' && activePolicyPanel === 'before' ? activePolicyBranchPathBefore : ROOT_BRANCH_PATH,
  selectedPolicyStepPathAfter: activeMode === 'policy' && activePolicyPanel === 'after' ? selectedPolicyStepPathAfter : null,
  activePolicyBranchPathAfter: activeMode === 'policy' && activePolicyPanel === 'after' ? activePolicyBranchPathAfter : ROOT_BRANCH_PATH,
});

export const resolvePersistedEditorViewState = ({
  persistedViewState,
  draftTasks,
  draftPolicies,
  draftPolicyGroups,
  draftPolicySets,
}: {
  persistedViewState: ScriptEditorPersistedViewState | null;
  draftTasks: ScriptTaskTable[];
  draftPolicies: PolicyTable[];
  draftPolicyGroups: PolicyGroupTable[];
  draftPolicySets: PolicySetTable[];
}) => ({
  activeMode: isEditorModeId(persistedViewState?.activeMode) ? persistedViewState.activeMode : 'task',
  activePanel: isEditorPanelId(persistedViewState?.activePanel) ? persistedViewState.activePanel : 'basic',
  activePolicyPanel: isPolicyEditorPanelId(persistedViewState?.activePolicyPanel) ? persistedViewState.activePolicyPanel : 'basic',
  selectedTaskId:
    persistedViewState?.selectedTaskId && draftTasks.some((task) => task.id === persistedViewState.selectedTaskId)
      ? persistedViewState.selectedTaskId
      : draftTasks[0]?.id ?? null,
  selectedPolicyId:
    persistedViewState?.selectedPolicyId && draftPolicies.some((policy) => policy.id === persistedViewState.selectedPolicyId)
      ? persistedViewState.selectedPolicyId
      : draftPolicies[0]?.id ?? null,
  selectedPolicyGroupId:
    persistedViewState?.selectedPolicyGroupId && draftPolicyGroups.some((group) => group.id === persistedViewState.selectedPolicyGroupId)
      ? persistedViewState.selectedPolicyGroupId
      : draftPolicyGroups[0]?.id ?? null,
  selectedPolicySetId:
    persistedViewState?.selectedPolicySetId && draftPolicySets.some((set) => set.id === persistedViewState.selectedPolicySetId)
      ? persistedViewState.selectedPolicySetId
      : draftPolicySets[0]?.id ?? null,
  selectedStepPath: isStepPathLike(persistedViewState?.selectedStepPath) ? cloneStepPath(persistedViewState.selectedStepPath) : null,
  activeBranchPath: isStepBranchPathLike(persistedViewState?.activeBranchPath) ? cloneJson(persistedViewState.activeBranchPath) : ROOT_BRANCH_PATH,
  selectedPolicyStepPathBefore: isStepPathLike(persistedViewState?.selectedPolicyStepPathBefore)
    ? cloneStepPath(persistedViewState.selectedPolicyStepPathBefore)
    : null,
  activePolicyBranchPathBefore: isStepBranchPathLike(persistedViewState?.activePolicyBranchPathBefore)
    ? cloneJson(persistedViewState.activePolicyBranchPathBefore)
    : ROOT_BRANCH_PATH,
  selectedPolicyStepPathAfter: isStepPathLike(persistedViewState?.selectedPolicyStepPathAfter)
    ? cloneStepPath(persistedViewState.selectedPolicyStepPathAfter)
    : null,
  activePolicyBranchPathAfter: isStepBranchPathLike(persistedViewState?.activePolicyBranchPathAfter)
    ? cloneJson(persistedViewState.activePolicyBranchPathAfter)
    : ROOT_BRANCH_PATH,
});

export const hydrateTaskEditorState = ({
  currentTask,
  draftScript,
  selectedInputId,
  selectedUiFieldId,
  selectedStepPath,
  activeBranchPath,
}: {
  currentTask: ScriptTaskTable | null;
  draftScript: ScriptTableRecord | null;
  selectedInputId: string | null;
  selectedUiFieldId: string | null;
  selectedStepPath: StepPath | null;
  activeBranchPath: StepBranchPath;
}): {
  taskName: string;
  taskDescription: string;
  taskRowType: TaskRowType;
  taskTriggerMode: TaskTriggerMode;
  taskHidden: boolean;
  recordSchedule: boolean;
  sectionId: string | null;
  indentLevel: number;
  defaultTaskCycle: TaskCycle;
  taskExecMax: number;
  showEnabledToggle: boolean;
  defaultEnabled: boolean;
  taskTone: TaskTone;
  inputEntries: EditorInputEntry[];
  inputError: string | null;
  uiSchema: EditorUiSchema;
  selectedInputId: string | null;
  selectedStepPath: StepPath | null;
  activeBranchPath: StepBranchPath;
  selectedUiFieldId: string | null;
  forceBasicPanel: boolean;
} => {
  const inputEntries = parseInputEntries(draftScript?.data.variableCatalog, null, {});
  const nextSelectedInputId = inputEntries.find((entry) => entry.id === selectedInputId)?.id ?? inputEntries[0]?.id ?? null;

  if (!currentTask) {
    return {
      taskName: '',
      taskDescription: '',
      taskRowType: TASK_ROW_TYPE.task,
      taskTriggerMode: TASK_TRIGGER_MODE.linkOnly,
      taskHidden: false,
      recordSchedule: true,
      sectionId: null,
      indentLevel: 1,
      defaultTaskCycle: TASK_CYCLE_VALUE.everyRun,
      taskExecMax: 0,
      showEnabledToggle: true,
      defaultEnabled: true,
      taskTone: TASK_TONE.normal,
      inputEntries,
      inputError: null,
      uiSchema: createUiSchema(),
      selectedInputId: nextSelectedInputId,
      selectedStepPath: null,
      activeBranchPath: ROOT_BRANCH_PATH,
      selectedUiFieldId: null,
      forceBasicPanel: false,
    };
  }

  const uiSchema = parseUiSchema(currentTask.data.uiData ?? {});
  const nextSelectedStepPath = !currentTask.data.steps.length
    ? null
    : !selectedStepPath || !getStepByPath(currentTask.data.steps as Step[], selectedStepPath)
      ? buildStepPath(ROOT_BRANCH_PATH, 0)
      : selectedStepPath;
  const nextActiveBranchPath = nextSelectedStepPath === selectedStepPath && nextSelectedStepPath ? activeBranchPath : ROOT_BRANCH_PATH;

  return {
    taskName: currentTask.name,
    taskDescription: currentTask.description || '',
    taskRowType: currentTask.rowType,
    taskTriggerMode: currentTask.triggerMode,
    taskHidden: currentTask.isHidden,
    recordSchedule: currentTask.recordSchedule,
    sectionId: currentTask.sectionId,
    indentLevel: currentTask.indentLevel,
    defaultTaskCycle: currentTask.defaultTaskCycle,
    taskExecMax: currentTask.execMax,
    showEnabledToggle: currentTask.showEnabledToggle,
    defaultEnabled: currentTask.defaultEnabled,
    taskTone: currentTask.taskTone,
    inputEntries,
    inputError: null,
    uiSchema,
    selectedInputId: nextSelectedInputId,
    selectedStepPath: nextSelectedStepPath,
    activeBranchPath: nextActiveBranchPath,
    selectedUiFieldId: uiSchema.fields.find((field) => field.id === selectedUiFieldId)?.id ?? uiSchema.fields[0]?.id ?? null,
    forceBasicPanel: currentTask.rowType === TASK_ROW_TYPE.title,
  };
};

export const hydratePolicyStepEditorState = ({
  currentPolicy,
  selectedPolicyStepPathBefore,
  selectedPolicyStepPathAfter,
  activePolicyBranchPathBefore,
  activePolicyBranchPathAfter,
}: {
  currentPolicy: PolicyTable | null;
  selectedPolicyStepPathBefore: StepPath | null;
  selectedPolicyStepPathAfter: StepPath | null;
  activePolicyBranchPathBefore: StepBranchPath;
  activePolicyBranchPathAfter: StepBranchPath;
}) => {
  if (!currentPolicy) {
    return {
      selectedPolicyStepPathBefore: null,
      activePolicyBranchPathBefore: ROOT_BRANCH_PATH,
      selectedPolicyStepPathAfter: null,
      activePolicyBranchPathAfter: ROOT_BRANCH_PATH,
    };
  }

  const nextBeforePath = !currentPolicy.data.beforeAction.length
    ? null
    : !selectedPolicyStepPathBefore || !getStepByPath(currentPolicy.data.beforeAction as Step[], selectedPolicyStepPathBefore)
      ? buildStepPath(ROOT_BRANCH_PATH, 0)
      : selectedPolicyStepPathBefore;

  const nextAfterPath = !currentPolicy.data.afterAction.length
    ? null
    : !selectedPolicyStepPathAfter || !getStepByPath(currentPolicy.data.afterAction as Step[], selectedPolicyStepPathAfter)
      ? buildStepPath(ROOT_BRANCH_PATH, 0)
      : selectedPolicyStepPathAfter;

  return {
    selectedPolicyStepPathBefore: nextBeforePath,
    activePolicyBranchPathBefore:
      nextBeforePath === selectedPolicyStepPathBefore && nextBeforePath ? activePolicyBranchPathBefore : ROOT_BRANCH_PATH,
    selectedPolicyStepPathAfter: nextAfterPath,
    activePolicyBranchPathAfter:
      nextAfterPath === selectedPolicyStepPathAfter && nextAfterPath ? activePolicyBranchPathAfter : ROOT_BRANCH_PATH,
  };
};
