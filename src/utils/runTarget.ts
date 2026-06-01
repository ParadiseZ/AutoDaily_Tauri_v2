import type { RunTarget } from '@/types/bindings/RunTarget';

export type RunTargetKind =
  | 'deviceQueue'
  | 'fullScript'
  | 'task'
  | 'policyGroup'
  | 'policySet'
  | 'policy';

export interface RunTargetDetails {
  kind: RunTargetKind;
  scriptId?: string;
  taskId?: string;
  policyGroupId?: string;
  policySetId?: string;
  policyId?: string;
}

export const createDeviceQueueRunTarget = (): RunTarget => 'deviceQueue';

export const createFullScriptRunTarget = (scriptId: string): RunTarget => ({
  fullScript: {
    script_id: scriptId,
  },
});

export const createTaskRunTarget = (scriptId: string, taskId: string): RunTarget => ({
  task: {
    script_id: scriptId,
    task_id: taskId,
  },
});

export const createPolicyGroupRunTarget = (scriptId: string, policyGroupId: string): RunTarget => ({
  policyGroup: {
    script_id: scriptId,
    policy_group_id: policyGroupId,
  },
});

export const createPolicySetRunTarget = (scriptId: string, policySetId: string): RunTarget => ({
  policySet: {
    script_id: scriptId,
    policy_set_id: policySetId,
  },
});

export const createPolicyRunTarget = (scriptId: string, policyId: string): RunTarget => ({
  policy: {
    script_id: scriptId,
    policy_id: policyId,
  },
});

export const getRunTargetDetails = (target: RunTarget): RunTargetDetails => {
  if (target === 'deviceQueue') {
    return { kind: 'deviceQueue' };
  }
  if ('fullScript' in target) {
    return {
      kind: 'fullScript',
      scriptId: target.fullScript.script_id,
    };
  }
  if ('task' in target) {
    return {
      kind: 'task',
      scriptId: target.task.script_id,
      taskId: target.task.task_id,
    };
  }
  if ('policyGroup' in target) {
    return {
      kind: 'policyGroup',
      scriptId: target.policyGroup.script_id,
      policyGroupId: target.policyGroup.policy_group_id,
    };
  }
  if ('policySet' in target) {
    return {
      kind: 'policySet',
      scriptId: target.policySet.script_id,
      policySetId: target.policySet.policy_set_id,
    };
  }
  return {
    kind: 'policy',
    scriptId: target.policy.script_id,
    policyId: target.policy.policy_id,
  };
};
