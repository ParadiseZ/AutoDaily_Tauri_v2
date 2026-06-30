import type { EditorInputEntry } from '@/views/script-editor/editorVariables';

export const buildVariableReferenceKey = (namespace: EditorInputEntry['namespace'], key: string) => {
  const trimmed = key.trim();
  if (!trimmed) {
    return '';
  }
  return `${namespace}.${trimmed}`;
};

export const createEditorStepId = () =>
  globalThis.crypto?.randomUUID?.() ?? `step-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;

export const createUniqueVariableStorageKey = (
  entries: EditorInputEntry[],
  namespace: EditorInputEntry['namespace'],
  preferredKey?: string,
) => {
  const existingKeys = new Set(entries.map((entry) => entry.key.trim()).filter(Boolean));
  const trimmedPreferred = preferredKey?.trim().replace(/^(input|runtime|system)\./, '') ?? '';
  const defaultPrefix = namespace === 'runtime' ? 'runtimeVar' : 'newVar';
  const baseSeed = trimmedPreferred || `${defaultPrefix}${entries.length + 1}`;

  if (!existingKeys.has(baseSeed)) {
    return baseSeed;
  }

  const matched = baseSeed.match(/^(.*?)(\d+)$/);
  const prefix = matched?.[1] || `${baseSeed}_`;
  let seed = matched ? Number(matched[2]) : 1;
  let nextKey = baseSeed;
  while (existingKeys.has(nextKey)) {
    seed += 1;
    nextKey = `${prefix}${seed}`;
  }
  return nextKey;
};
