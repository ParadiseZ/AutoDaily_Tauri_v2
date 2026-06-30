import { listen } from '@tauri-apps/api/event';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { Step } from '@/types/bindings/Step';
import { buildStepPath, type StepBranchPath, type StepPath, ROOT_BRANCH_PATH } from '@/views/script-editor/editor-step/editorStepTree';

const MAX_CONSOLE_LINES = 300;

export type EditorConsoleLevel = 'info' | 'warning' | 'error' | 'debug';

export interface EditorConsoleEntry {
  time: string;
  message: string;
  level: EditorConsoleLevel;
}

export const buildConsoleTimestamp = () =>
  new Date().toLocaleTimeString('zh-CN', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });

export const normalizeConsoleLevel = (value: unknown): EditorConsoleLevel => {
  const level = typeof value === 'string' ? value.trim().toLowerCase() : '';
  if (level === 'error') {
    return 'error';
  }
  if (level === 'warn' || level === 'warning') {
    return 'warning';
  }
  if (level === 'debug' || level === 'trace') {
    return 'debug';
  }
  return 'info';
};

export const appendConsoleEntry = (
  entries: EditorConsoleEntry[],
  message: string,
  level: EditorConsoleLevel = 'info',
  time = buildConsoleTimestamp(),
) => [...entries, { time, message, level }].slice(-MAX_CONSOLE_LINES);

const visitStepTree = (steps: Step[], visitor: (step: Step) => string | null): string | null => {
  for (const step of steps) {
    const matched = visitor(step);
    if (matched) {
      return matched;
    }

    if (step.op === 'sequence') {
      const nested = visitStepTree(step.steps, visitor);
      if (nested) {
        return nested;
      }
    }
  }

  return null;
};

const resolveTaskNameForConsole = (tasks: ScriptTaskTable[], taskId: string | null | undefined) => {
  if (!taskId) {
    return null;
  }
  return tasks.find((task) => task.id === taskId)?.name ?? null;
};

const resolveStepNameForConsole = (tasks: ScriptTaskTable[], policies: PolicyTable[], stepId: string | null | undefined) => {
  if (!stepId) {
    return null;
  }

  for (const task of tasks) {
    const matched = visitStepTree(task.data.steps as Step[], (step) =>
      step.id === stepId ? step.label?.trim() || '未命名步骤' : null,
    );
    if (matched) {
      return matched;
    }
  }

  for (const policy of policies) {
    const beforeMatched = visitStepTree(policy.data.beforeAction as Step[], (step) =>
      step.id === stepId ? step.label?.trim() || '未命名步骤' : null,
    );
    if (beforeMatched) {
      return beforeMatched;
    }

    const afterMatched = visitStepTree(policy.data.afterAction as Step[], (step) =>
      step.id === stepId ? step.label?.trim() || '未命名步骤' : null,
    );
    if (afterMatched) {
      return afterMatched;
    }
  }

  return null;
};

const buildProgressConsoleMessage = ({
  payload,
  scriptId,
  tasks,
  policies,
}: {
  payload: Record<string, unknown>;
  scriptId: string;
  tasks: ScriptTaskTable[];
  policies: PolicyTable[];
}) => {
  if (payload.phase !== 'executing') {
    return null;
  }

  const payloadScriptId = typeof payload.scriptId === 'string' ? payload.scriptId : null;
  if (!payloadScriptId || payloadScriptId !== scriptId) {
    return null;
  }

  const rawMessage = typeof payload.message === 'string' ? payload.message.trim() : '';
  if (rawMessage.startsWith('开始执行步骤:')) {
    return `步骤：${rawMessage.slice('开始执行步骤:'.length).trim()}`;
  }
  if (rawMessage.startsWith('开始执行任务:')) {
    return `任务：${rawMessage.slice('开始执行任务:'.length).trim()}`;
  }

  const stepName = resolveStepNameForConsole(tasks, policies, typeof payload.stepId === 'string' ? payload.stepId : null);
  if (stepName) {
    return `步骤：${stepName}`;
  }

  const taskName = resolveTaskNameForConsole(tasks, typeof payload.taskId === 'string' ? payload.taskId : null);
  if (taskName) {
    return `任务：${taskName}`;
  }

  return null;
};

export const resolveNextPreviewDeviceId = ({
  currentSelectedDeviceId,
  storedDeviceId,
  availableDeviceIds,
}: {
  currentSelectedDeviceId: string | null;
  storedDeviceId: string | null;
  availableDeviceIds: string[];
}) => {
  if (!currentSelectedDeviceId && storedDeviceId && availableDeviceIds.includes(storedDeviceId)) {
    return storedDeviceId;
  }
  if (currentSelectedDeviceId && availableDeviceIds.includes(currentSelectedDeviceId)) {
    return currentSelectedDeviceId;
  }
  return availableDeviceIds[0] ?? null;
};

export const attachScriptEditorRuntimeListeners = async ({
  getSelectedPreviewDeviceId,
  getScriptId,
  getDraftTasks,
  getDraftPolicies,
  appendConsoleLine,
}: {
  getSelectedPreviewDeviceId: () => string | null;
  getScriptId: () => string;
  getDraftTasks: () => ScriptTaskTable[];
  getDraftPolicies: () => PolicyTable[];
  appendConsoleLine: (message: string, level?: EditorConsoleLevel, time?: string) => void;
}) => {
  const detachChildLogListener = await listen('child-log', (event) => {
    const payload = event.payload as Record<string, unknown> | null;
    if (!payload || typeof payload.deviceId !== 'string' || typeof payload.message !== 'string') {
      return;
    }
    if (!getSelectedPreviewDeviceId() || payload.deviceId !== getSelectedPreviewDeviceId()) {
      return;
    }
    const rawLevel = typeof payload.level === 'string' ? payload.level : 'Info';
    const time = typeof payload.time === 'string' ? payload.time : buildConsoleTimestamp();
    appendConsoleLine(`[child:${rawLevel}] ${payload.message}`, normalizeConsoleLevel(rawLevel), time);
  });

  const detachDeviceProgressListener = await listen('device-progress', (event) => {
    const payload = event.payload as Record<string, unknown> | null;
    if (!payload || typeof payload.deviceId !== 'string') {
      return;
    }
    if (!getSelectedPreviewDeviceId() || payload.deviceId !== getSelectedPreviewDeviceId()) {
      return;
    }

    const message = buildProgressConsoleMessage({
      payload,
      scriptId: getScriptId(),
      tasks: getDraftTasks(),
      policies: getDraftPolicies(),
    });
    if (!message) {
      return;
    }

    appendConsoleLine(message, 'debug', buildConsoleTimestamp());
  });

  return {
    detachChildLogListener,
    detachDeviceProgressListener,
  };
};

export const findStepPathById = (steps: Step[], stepId: string, branch: StepBranchPath = ROOT_BRANCH_PATH): StepPath | null => {
  for (let index = 0; index < steps.length; index += 1) {
    const step = steps[index];
    const path = buildStepPath(branch, index);
    if (step.id === stepId) {
      return path;
    }

    if (step.op === 'sequence') {
      const nested = findStepPathById(step.steps, stepId, { parentStepPath: path, branch: 'sequence' });
      if (nested) {
        return nested;
      }
      continue;
    }

    if (step.op === 'flowControl') {
      if (step.a.type === 'if') {
        const thenPath = findStepPathById(step.a.then, stepId, { parentStepPath: path, branch: 'then' });
        if (thenPath) {
          return thenPath;
        }
        const elsePath = findStepPathById(step.a.else_steps ?? [], stepId, { parentStepPath: path, branch: 'else' });
        if (elsePath) {
          return elsePath;
        }
        continue;
      }

      if (step.a.type === 'while' || step.a.type === 'forEach' || step.a.type === 'repeat') {
        const flowPath = findStepPathById(step.a.flow, stepId, { parentStepPath: path, branch: 'flow' });
        if (flowPath) {
          return flowPath;
        }
      }
      continue;
    }

    if (step.op === 'vision' && (step.a.type === 'visionSearch' || step.a.type === 'countCompare')) {
      const visionPath = findStepPathById(step.a.then_steps, stepId, { parentStepPath: path, branch: 'visionThen' });
      if (visionPath) {
        return visionPath;
      }
      continue;
    }

    if (step.op === 'dataHanding' && (step.a.type === 'filter' || step.a.type === 'colorCompare' || step.a.type === 'relativeFilter')) {
      const filterPath = findStepPathById(step.a.then_steps ?? [], stepId, { parentStepPath: path, branch: 'filterThen' });
      if (filterPath) {
        return filterPath;
      }
    }
  }

  return null;
};
