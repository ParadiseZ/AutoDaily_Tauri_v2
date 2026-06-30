import type { JsonValue } from '@/types/app/domain';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { Step } from '@/types/bindings/Step';
import { stableStringify, type EditorPanelId, type RawEditorSection } from '@/views/script-editor/editorSchema';

export const resolveRawEditorSection = (activePanel: EditorPanelId): RawEditorSection => {
  if (activePanel === 'steps') return 'steps';
  if (activePanel === 'ui') return 'ui';
  return 'inputs';
};

export const getRawDialogTitle = (section: RawEditorSection) => {
  switch (section) {
    case 'inputs':
      return '输入变量 JSON';
    case 'ui':
      return 'UI Schema JSON';
    default:
      return '步骤 JSON';
  }
};

export const getRawDialogDescription = (section: RawEditorSection) => {
  switch (section) {
    case 'inputs':
      return '这里是 input.* 的底层结构，作为调试入口保留。';
    case 'ui':
      return '这里是 UI schema 的底层结构，优先在可视化面板里编辑。';
    default:
      return '这里是任务步骤的底层结构，优先在可视化工作区里查看和调整。';
  }
};

export const buildRawDialogText = (section: RawEditorSection, task: ScriptTaskTable) =>
  stableStringify(
    section === 'inputs'
      ? task.data.variables ?? {}
      : section === 'ui'
        ? task.data.uiData ?? {}
        : task.data.steps ?? [],
  );

export const formatRawDialogText = (text: string) => stableStringify(JSON.parse(text) as JsonValue);

export const parseRawDialogValue = (section: RawEditorSection, text: string) => {
  const parsed = JSON.parse(text) as JsonValue;
  if (section === 'steps' && !Array.isArray(parsed)) {
    throw new Error('步骤 JSON 顶层必须是数组。');
  }
  return parsed as JsonValue | Step[];
};
