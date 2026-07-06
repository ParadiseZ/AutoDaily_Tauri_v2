import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { EditorCollectionMoveAction } from '@/views/script-editor/editor-policy/editorPolicy';
import { reorderCollection } from '@/views/script-editor/editor-policy/editorPolicy';
import { TASK_ROW_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';
import type { EditorTaskMoveAction } from '@/views/script-editor/editorTaskMove';

export const reorderItemsById = <T extends { id: string }>(
  items: T[],
  draggedId: string,
  targetId: string,
  normalize: (item: T, index: number) => T,
) => {
  const fromIndex = items.findIndex((item) => item.id === draggedId);
  const toIndex = items.findIndex((item) => item.id === targetId);
  if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) {
    return items;
  }
  return reorderCollection(items, fromIndex, toIndex).map((item, index) => normalize(item, index));
};

export const moveTaskByMenuAction = (
  tasks: ScriptTaskTable[],
  taskId: string,
  action: EditorTaskMoveAction,
  normalize: (task: ScriptTaskTable, index: number) => ScriptTaskTable,
) => {
  const fromIndex = tasks.findIndex((task) => task.id === taskId);
  if (fromIndex < 0) {
    return tasks;
  }

  const nextTasks = [...tasks];
  const sourceTask = nextTasks[fromIndex];
  if (!sourceTask) {
    return tasks;
  }

  if (action.kind === 'current') {
    if (sourceTask.rowType === TASK_ROW_TYPE.title) {
      const blockIds = new Set([
        sourceTask.id,
        ...nextTasks.filter((task) => task.rowType === TASK_ROW_TYPE.task && task.sectionId === sourceTask.id).map((task) => task.id),
      ]);
      const block = nextTasks.filter((task) => blockIds.has(task.id));
      const remainder = nextTasks.filter((task) => !blockIds.has(task.id));
      return (action.position === 'top' ? [...block, ...remainder] : [...remainder, ...block]).map((task, index) =>
        normalize(task, index),
      );
    }

    const [movedTask] = nextTasks.splice(fromIndex, 1);
    if (!movedTask) {
      return tasks;
    }
    movedTask.sectionId = null;
    nextTasks.splice(action.position === 'top' ? 0 : nextTasks.length, 0, movedTask);
    return nextTasks.map((task, index) => normalize(task, index));
  }

  const [movedTask] = nextTasks.splice(fromIndex, 1);
  if (!movedTask) {
    return tasks;
  }

  const insertBySection = (sectionId: string, position: 'top' | 'bottom') => {
    const sectionTaskIndexes = nextTasks.reduce<number[]>((indexes, task, index) => {
      if (task.rowType === TASK_ROW_TYPE.task && task.sectionId === sectionId) {
        indexes.push(index);
      }
      return indexes;
    }, []);
    const titleIndex = nextTasks.findIndex((task) => task.id === sectionId);
    const anchorIndex = titleIndex >= 0 ? titleIndex + 1 : nextTasks.length;
    const insertIndex =
      position === 'top'
        ? sectionTaskIndexes[0] ?? anchorIndex
        : (sectionTaskIndexes[sectionTaskIndexes.length - 1] ?? titleIndex) + 1;

    movedTask.sectionId = sectionId;
    nextTasks.splice(insertIndex, 0, movedTask);
  };

  if (action.kind === 'section') {
    insertBySection(action.sectionId, action.position);
  } else {
    const targetIndex = nextTasks.findIndex((task) => task.id === action.taskId);
    const targetTask = targetIndex >= 0 ? nextTasks[targetIndex] : null;
    if (!targetTask) {
      nextTasks.splice(fromIndex, 0, movedTask);
      return tasks;
    }

    movedTask.sectionId = targetTask.sectionId ?? null;
    nextTasks.splice(action.position === 'top' ? targetIndex : targetIndex + 1, 0, movedTask);
  }

  return nextTasks.map((task, index) => normalize(task, index));
};

export const moveCollectionByMenuAction = <T extends { id: string }>(
  items: T[],
  itemId: string,
  action: EditorCollectionMoveAction,
  normalize: (item: T, index: number) => T,
) => {
  const fromIndex = items.findIndex((item) => item.id === itemId);
  if (fromIndex < 0) {
    return items;
  }

  const nextItems = [...items];
  const [movedItem] = nextItems.splice(fromIndex, 1);
  if (!movedItem) {
    return items;
  }

  if (action.kind === 'current') {
    nextItems.splice(action.position === 'top' ? 0 : nextItems.length, 0, movedItem);
    return nextItems.map((item, index) => normalize(item, index));
  }

  const targetIndex = nextItems.findIndex((item) => item.id === action.itemId);
  if (targetIndex < 0) {
    return items;
  }

  nextItems.splice(action.position === 'top' ? targetIndex : targetIndex + 1, 0, movedItem);
  return nextItems.map((item, index) => normalize(item, index));
};

export const selectNeighborIdAfterRemoval = <T extends { id: string }>(items: T[], removedId: string) => {
  const removedIndex = items.findIndex((item) => item.id === removedId);
  if (removedIndex < 0) {
    return null;
  }

  const nextItems = items.filter((item) => item.id !== removedId);
  if (nextItems.length === 0) {
    return null;
  }

  return nextItems[Math.min(removedIndex, nextItems.length - 1)]?.id ?? null;
};
