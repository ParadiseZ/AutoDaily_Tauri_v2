export type EditorReferenceKind = 'task' | 'policy';

export interface EditorReferenceOption {
  label: string;
  value: string;
  description?: string;
}

const getReferenceKindLabel = (kind: EditorReferenceKind) => (kind === 'task' ? '任务' : '策略');

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
