import { reorderCollection } from '@/views/script-editor/editor-policy/editorPolicy';

export type ScriptEditorRelationMap = Record<string, string[]>;

export const appendRelationId = (
  relationMap: ScriptEditorRelationMap,
  ownerId: string,
  itemId: string,
): ScriptEditorRelationMap => {
  const assigned = relationMap[ownerId] ?? [];
  if (assigned.includes(itemId)) {
    return relationMap;
  }
  return {
    ...relationMap,
    [ownerId]: [...assigned, itemId],
  };
};

export const removeRelationId = (
  relationMap: ScriptEditorRelationMap,
  ownerId: string,
  itemId: string,
): ScriptEditorRelationMap => ({
  ...relationMap,
  [ownerId]: (relationMap[ownerId] ?? []).filter((id) => id !== itemId),
});

export const reorderRelationIds = (
  relationMap: ScriptEditorRelationMap,
  ownerId: string,
  draggedId: string,
  targetId: string,
): ScriptEditorRelationMap => {
  const currentIds = relationMap[ownerId] ?? [];
  const fromIndex = currentIds.indexOf(draggedId);
  const toIndex = currentIds.indexOf(targetId);
  if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) {
    return relationMap;
  }
  return {
    ...relationMap,
    [ownerId]: reorderCollection(currentIds, fromIndex, toIndex),
  };
};

export const reverseRelationIds = (
  relationMap: ScriptEditorRelationMap,
  ownerId: string,
): ScriptEditorRelationMap => ({
  ...relationMap,
  [ownerId]: [...(relationMap[ownerId] ?? [])].reverse(),
});

export const removeRelationOwner = (
  relationMap: ScriptEditorRelationMap,
  ownerId: string,
): ScriptEditorRelationMap => {
  const nextRelationMap = { ...relationMap };
  delete nextRelationMap[ownerId];
  return nextRelationMap;
};

export const removeRelationIdFromAllOwners = (
  relationMap: ScriptEditorRelationMap,
  itemId: string,
): ScriptEditorRelationMap =>
  Object.fromEntries(
    Object.entries(relationMap).map(([ownerId, assignedIds]) => [ownerId, assignedIds.filter((id) => id !== itemId)]),
  );
