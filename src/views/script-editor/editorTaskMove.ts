export type EditorTaskMoveAction =
  | {
      kind: 'current';
      position: 'top' | 'bottom';
    }
  | {
      kind: 'section';
      sectionId: string;
      position: 'top' | 'bottom';
    }
  | {
      kind: 'task';
      taskId: string;
      position: 'top' | 'bottom';
    };
