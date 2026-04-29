import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';

export const isUserVisibleScriptTask = (task: ScriptTaskTable) => !task.isDeleted && !task.isHidden;

export const isUserVisibleRunnableTask = (task: ScriptTaskTable) =>
  task.rowType === 'task' && isUserVisibleScriptTask(task);

export const filterUserVisibleTaskRows = (tasks: ScriptTaskTable[]) => {
  const visibleTaskRows = tasks.filter(isUserVisibleRunnableTask);
  const visibleSectionIds = new Set(
    visibleTaskRows
      .map((task) => task.sectionId)
      .filter((sectionId): sectionId is string => Boolean(sectionId)),
  );

  return tasks.filter((task) => {
    if (!isUserVisibleScriptTask(task)) {
      return false;
    }
    if (task.rowType === 'title') {
      return visibleSectionIds.has(task.id);
    }
    return task.rowType === 'task';
  });
};
