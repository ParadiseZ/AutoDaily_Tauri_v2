export type EditorReferenceKind = 'task' | 'policy' | 'policyGroup' | 'policySet';

export interface EditorReferenceOption {
  label: string;
  value: string;
  description?: string;
}

export interface EditorTaskUiVariableOption {
  taskId: string;
  taskLabel: string;
  variableId: string;
  label: string;
  description?: string;
  options: string[];
}

const getReferenceKindLabel = (kind: EditorReferenceKind) => {
  switch (kind) {
    case 'policyGroup':
      return '策略组';
    case 'policySet':
      return '策略集';
    case 'policy':
      return '策略';
    default:
      return '任务';
  }
};

export const withResolvedReferenceOption = (
  options: EditorReferenceOption[],
  currentId: string | null | undefined,
  kind: EditorReferenceKind,
) => {
  const trimmedId = currentId?.trim() ?? '';
  if (!trimmedId || options.some((option) => option.value === trimmedId)) {
    return options;
  }

  return [
    {
      label: `未解析${getReferenceKindLabel(kind)}`,
      value: trimmedId,
      description: `保留历史引用 ${trimmedId}`,
    },
    ...options,
  ];
};
