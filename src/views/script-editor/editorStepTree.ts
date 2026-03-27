import type { Step } from '@/types/bindings/Step';
import { DATA_TYPE, FLOW_TYPE, STEP_OP, VISION_TYPE } from '@/views/script-editor/editorStepKinds';

export type StepBranchKind = 'root' | 'sequence' | 'then' | 'else' | 'flow' | 'visionThen' | 'filterThen';

export interface StepPathSegment {
  branch: StepBranchKind;
  index: number;
}

export type StepPath = StepPathSegment[];

export interface StepBranchPath {
  parentStepPath: StepPath | null;
  branch: StepBranchKind;
}

export const ROOT_BRANCH_PATH: StepBranchPath = {
  parentStepPath: null,
  branch: 'root',
};

export const cloneStepPath = (path: StepPath | null) => (path ? path.map((segment) => ({ ...segment })) : null);

export const isSameStepPath = (left: StepPath | null, right: StepPath | null) =>
  JSON.stringify(left ?? null) === JSON.stringify(right ?? null);

export const isSameBranchPath = (left: StepBranchPath, right: StepBranchPath) =>
  left.branch === right.branch && isSameStepPath(left.parentStepPath, right.parentStepPath);

export const buildStepPath = (branchPath: StepBranchPath, index: number): StepPath => [
  ...(cloneStepPath(branchPath.parentStepPath) ?? []),
  { branch: branchPath.branch, index },
];

export const getStepByPath = (rootSteps: Step[], path: StepPath | null): Step | null => {
  if (!path?.length) return null;

  let currentSteps = rootSteps;
  let currentStep: Step | null = null;

  for (let index = 0; index < path.length; index += 1) {
    const segment = path[index];
    currentStep = currentSteps[segment.index] ?? null;
    if (!currentStep) return null;

    const nextSegment = path[index + 1];
    if (nextSegment) {
      currentSteps = getBranchStepsFromStep(currentStep, nextSegment.branch);
    }
  }

  return currentStep;
};

export const getBranchSteps = (rootSteps: Step[], branchPath: StepBranchPath): Step[] => {
  if (branchPath.branch === 'root' || !branchPath.parentStepPath?.length) {
    return rootSteps;
  }

  const parentStep = getStepByPath(rootSteps, branchPath.parentStepPath);
  if (!parentStep) return [];
  return getBranchStepsFromStep(parentStep, branchPath.branch);
};

export const updateStepByPath = (rootSteps: Step[], path: StepPath, updater: (step: Step) => Step): Step[] => {
  if (!path.length) return rootSteps;

  const [current, ...rest] = path;
  return rootSteps.map((step, index) => {
    if (index !== current.index) return step;
    if (!rest.length) return updater(step);
    return setBranchStepsOnStep(step, rest[0].branch, updateStepByPath(getBranchStepsFromStep(step, rest[0].branch), rest, updater));
  });
};

export const updateBranchSteps = (rootSteps: Step[], branchPath: StepBranchPath, updater: (steps: Step[]) => Step[]): Step[] => {
  if (branchPath.branch === 'root' || !branchPath.parentStepPath?.length) {
    return updater(rootSteps);
  }

  return updateStepByPath(rootSteps, branchPath.parentStepPath, (step) =>
    setBranchStepsOnStep(step, branchPath.branch, updater(getBranchStepsFromStep(step, branchPath.branch))),
  );
};

export const createSiblingSelection = (branchPath: StepBranchPath, nextLength: number, preferredIndex: number): StepPath | null => {
  if (nextLength <= 0) return null;
  const safeIndex = Math.max(0, Math.min(preferredIndex, nextLength - 1));
  return buildStepPath(branchPath, safeIndex);
};

export const getParentBranchPath = (path: StepPath | null): StepBranchPath => {
  if (!path?.length) return ROOT_BRANCH_PATH;
  const last = path[path.length - 1];
  const parentPath = path.slice(0, -1);
  return {
    parentStepPath: parentPath.length ? parentPath : null,
    branch: last.branch,
  };
};

const getBranchStepsFromStep = (step: Step, branch: StepBranchKind): Step[] => {
  switch (branch) {
    case 'sequence':
      return step.op === STEP_OP.sequence ? step.steps : [];
    case 'then':
      return step.op === STEP_OP.flowControl && step.a.type === FLOW_TYPE.if ? step.a.then : [];
    case 'else':
      return step.op === STEP_OP.flowControl && step.a.type === FLOW_TYPE.if ? (step.a.else_steps ?? []) : [];
    case 'flow':
      return step.op === STEP_OP.flowControl && (step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.for) ? step.a.flow : [];
    case 'visionThen':
      return step.op === STEP_OP.vision && step.a.type === VISION_TYPE.visionSearch ? step.a.then_steps : [];
    case 'filterThen':
      return step.op === STEP_OP.dataHanding && step.a.type === DATA_TYPE.filter ? step.a.then_steps : [];
    default:
      return [];
  }
};

const setBranchStepsOnStep = (step: Step, branch: StepBranchKind, steps: Step[]): Step => {
  switch (branch) {
    case 'sequence':
      return step.op === STEP_OP.sequence ? { ...step, steps } : step;
    case 'then':
      return step.op === STEP_OP.flowControl && step.a.type === FLOW_TYPE.if ? { ...step, a: { ...step.a, then: steps } } : step;
    case 'else':
      return step.op === STEP_OP.flowControl && step.a.type === FLOW_TYPE.if ? { ...step, a: { ...step.a, else_steps: steps } } : step;
    case 'flow':
      return step.op === STEP_OP.flowControl && (step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.for)
        ? { ...step, a: { ...step.a, flow: steps } }
        : step;
    case 'visionThen':
      return step.op === STEP_OP.vision && step.a.type === VISION_TYPE.visionSearch ? { ...step, a: { ...step.a, then_steps: steps } } : step;
    case 'filterThen':
      return step.op === STEP_OP.dataHanding && step.a.type === DATA_TYPE.filter
        ? { ...step, a: { ...step.a, then_steps: steps } }
        : step;
    default:
      return step;
  }
};
