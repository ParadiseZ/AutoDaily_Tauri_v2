import type { DetectorType } from '@/types/bindings/DetectorType';
import type { RunTarget } from '@/types/bindings/RunTarget';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { YoloDet } from '@/types/bindings/YoloDet';
import type { EditorNamedItem } from '@/views/script-editor/editor-policy/editorPolicy';
import { TASK_ROW_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';

type RunTargetKind = 'fullScript' | 'task' | 'policy' | 'policyGroup' | 'policySet';

export const extractYoloDetector = (model: DetectorType | null | undefined): YoloDet | null => {
  if (!model) {
    return null;
  }
  if ('Yolo11' in model) {
    return model.Yolo11;
  }
  if ('Yolo26' in model) {
    return model.Yolo26;
  }
  return null;
};

export const getImgDetLabelSelectPlaceholder = (loading: boolean, optionCount: number) => {
  if (loading) {
    return '正在加载标签...';
  }
  if (optionCount > 0) {
    return '选择标签';
  }
  return '请先设置图像检测模型标签文件';
};

export const loadImgDetLabelState = async ({
  path,
  getYoloLabels,
}: {
  path: string;
  getYoloLabels: (path: string) => Promise<Array<{ index: number; label: string }>>;
}) => {
  const trimmedPath = path.trim();
  if (!trimmedPath) {
    return {
      options: [] as Array<{ label: string; value: number; description?: string }>,
      hint: '当前脚本未设置图像检测模型的标签文件，请先在“编辑脚本信息 > 模型信息 > 目标检测”里配置标签路径。',
      logMessage: '图像检测标签文件未配置。',
      logLevel: 'warning' as const,
    };
  }

  const labels = await getYoloLabels(trimmedPath);
  return {
    options: labels.map((item) => ({
      label: `${item.index}: ${item.label}`,
      value: item.index,
      description: `idx ${item.index}`,
    })),
    hint: labels.length ? null : '标签文件已读取，但未解析出任何 names 标签。',
    logMessage: `已加载图像检测标签 ${labels.length} 项：${trimmedPath}`,
    logLevel: 'info' as const,
  };
};

export const buildRunTargetKey = (kind: RunTargetKind, id?: string | null) => `${kind}:${id ?? ''}`;

export const buildRunTargetSelectOptions = ({
  scriptName,
  scriptId,
  tasks,
  policyItems,
  policySetItems,
}: {
  scriptName: string | null | undefined;
  scriptId: string;
  tasks: ScriptTaskTable[];
  policyItems: EditorNamedItem[];
  policySetItems: EditorNamedItem[];
}) => [
  {
    label: `脚本 · ${scriptName || '(空)'}`,
    value: buildRunTargetKey('fullScript', scriptId),
  },
  ...tasks
    .filter((task) => task.rowType === TASK_ROW_TYPE.task)
    .map((task) => ({
      label: `任务 · ${task.name}`,
      value: buildRunTargetKey('task', task.id),
    })),
  ...policyItems.map((item) => ({
    label: `策略 · ${item.title}`,
    value: buildRunTargetKey('policy', item.id),
  })),
  ...policySetItems.map((item) => ({
    label: `策略集 · ${item.title}`,
    value: buildRunTargetKey('policySet', item.id),
  })),
];

export const resolveCurrentEditorRunTargetKey = ({
  scriptId,
  activeMode,
  selectedTaskId,
  selectedPolicyId,
  selectedPolicyGroupId,
  selectedPolicySetId,
}: {
  scriptId: string;
  activeMode: 'task' | 'policy' | 'policyGroup' | 'policySet';
  selectedTaskId: string | null;
  selectedPolicyId: string | null;
  selectedPolicyGroupId: string | null;
  selectedPolicySetId: string | null;
}) => {
  if (!scriptId) {
    return null;
  }
  if (activeMode === 'policy') {
    return selectedPolicyId ? buildRunTargetKey('policy', selectedPolicyId) : null;
  }
  if (activeMode === 'policyGroup') {
    return selectedPolicyGroupId ? buildRunTargetKey('policyGroup', selectedPolicyGroupId) : null;
  }
  if (activeMode === 'policySet') {
    return selectedPolicySetId ? buildRunTargetKey('policySet', selectedPolicySetId) : null;
  }
  return selectedTaskId ? buildRunTargetKey('task', selectedTaskId) : buildRunTargetKey('fullScript', scriptId);
};

export const getRunSelectionDisabledReason = (
  selectedPreviewDeviceId: string | null,
  selectedRunTargetKey: string | null,
  runtimeError: string | null,
) => {
  if (!selectedPreviewDeviceId || !selectedRunTargetKey) {
    return '请先选择设备和目标对象。';
  }
  return runtimeError;
};

export const buildActiveRunTarget = ({
  scriptId,
  selectedRunTargetKey,
  createFullScriptRunTarget,
  createPolicyRunTarget,
  createPolicyGroupRunTarget,
  createPolicySetRunTarget,
  createTaskRunTarget,
}: {
  scriptId: string;
  selectedRunTargetKey: string | null;
  createFullScriptRunTarget: (scriptId: string) => RunTarget;
  createPolicyRunTarget: (scriptId: string, targetId: string) => RunTarget;
  createPolicyGroupRunTarget: (scriptId: string, targetId: string) => RunTarget;
  createPolicySetRunTarget: (scriptId: string, targetId: string) => RunTarget;
  createTaskRunTarget: (scriptId: string, targetId: string) => RunTarget;
}) => {
  if (!scriptId || !selectedRunTargetKey) {
    return null;
  }

  const [kind, rawId] = selectedRunTargetKey.split(':', 2);
  const targetId = rawId?.trim() || '';

  if (kind === 'fullScript') {
    return createFullScriptRunTarget(scriptId);
  }
  if (!targetId) {
    return null;
  }
  if (kind === 'policy') {
    return createPolicyRunTarget(scriptId, targetId);
  }
  if (kind === 'policyGroup') {
    return createPolicyGroupRunTarget(scriptId, targetId);
  }
  if (kind === 'policySet') {
    return createPolicySetRunTarget(scriptId, targetId);
  }
  if (kind === 'task') {
    return createTaskRunTarget(scriptId, targetId);
  }
  return null;
};
