import type { JsonValue } from '@/types/app/domain';
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { Step } from '@/types/bindings/Step';
import { describeStep } from '@/views/script-editor/editor-step/editorStepTemplates';
import { cloneJson, parseUiSchema } from '@/views/script-editor/editorSchema';
import { buildVariableReferenceKey } from '@/views/script-editor/helpers/scriptEditorVariables';

export type VariableUsage = { key: string; label: string };

const collectConditionVariableReferences = (condition: ConditionNode, bucket: Set<string>) => {
  if (condition.type === 'group') {
    condition.items.forEach((item) => collectConditionVariableReferences(item, bucket));
    return;
  }

  if (condition.type === 'varCompare' && condition.var_name?.trim()) {
    bucket.add(condition.var_name.trim());
    return;
  }

  if (condition.type === 'execNumCompare' && condition.value.type === 'variable' && condition.value.var_name.trim()) {
    bucket.add(condition.value.var_name.trim());
    return;
  }

  if (condition.type === 'visionCountCompare' && condition.input_var?.trim()) {
    bucket.add(condition.input_var.trim());
    return;
  }

  if (condition.type === 'policySetResult' && condition.result_var?.trim()) {
    bucket.add(condition.result_var.trim());
  }
};

export const collectVariableReferencesFromSteps = (steps: Step[], bucket = new Set<string>()) => {
  for (const step of steps) {
    if (step.op === 'sequence') {
      collectVariableReferencesFromSteps(step.steps, bucket);
      continue;
    }

    if (step.op === 'action') {
      if (step.a.ac === 'capture' && step.a.output_var?.trim()) {
        bucket.add(step.a.output_var.trim());
        continue;
      }

      if ((step.a.ac === 'click' || step.a.ac === 'swipe') && (step.a.mode === 'txt' || step.a.mode === 'labelIdx') && step.a.input_var?.trim()) {
        bucket.add(step.a.input_var.trim());
      }
      if ((step.a.ac === 'click' || step.a.ac === 'longClick') && (step.a.mode === 'point' || step.a.mode === 'percent') && step.a.p_expr?.trim()) {
        bucket.add(step.a.p_expr.trim());
      }
      if (step.a.ac === 'swipe' && (step.a.mode === 'point' || step.a.mode === 'percent')) {
        if (step.a.from_expr?.trim()) {
          bucket.add(step.a.from_expr.trim());
        }
        if (step.a.to_expr?.trim()) {
          bucket.add(step.a.to_expr.trim());
        }
      }
      if ((step.a.ac === 'click' || step.a.ac === 'longClick') && step.a.mode === 'txt' && step.a.txt_expr?.trim()) {
        bucket.add(step.a.txt_expr.trim());
      }
      if ((step.a.ac === 'click' || step.a.ac === 'longClick') && step.a.mode === 'labelIdx' && step.a.idx_expr?.trim()) {
        bucket.add(step.a.idx_expr.trim());
      }
      if (step.a.ac === 'swipe' && step.a.mode === 'txt') {
        if (step.a.from_expr?.trim()) {
          bucket.add(step.a.from_expr.trim());
        }
        if (step.a.to_expr?.trim()) {
          bucket.add(step.a.to_expr.trim());
        }
      }
    }

    if (step.op === 'dataHanding') {
      if ((step.a.type === 'setVar' || step.a.type === 'getVar') && step.a.name?.trim()) {
        bucket.add(step.a.name.trim());
        continue;
      }
      if (step.a.type === 'clearVars') {
        step.a.names.forEach((name) => {
          if (name.trim()) {
            bucket.add(name.trim());
          }
        });
        continue;
      }

      if (step.a.type === 'filter') {
        if (step.a.input_var?.trim()) {
          bucket.add(step.a.input_var.trim());
        }
        if (step.a.out_name?.trim()) {
          bucket.add(step.a.out_name.trim());
        }
        collectVariableReferencesFromSteps(step.a.then_steps, bucket);
        continue;
      }

      if (step.a.type === 'colorCompare') {
        if (step.a.input_var?.trim()) {
          bucket.add(step.a.input_var.trim());
        }
        if (step.a.out_var?.trim()) {
          bucket.add(step.a.out_var.trim());
        }
        continue;
      }
    }

    if (step.op === 'vision' && step.a.type === 'visionSearch') {
      if (step.a.det_res_var?.trim()) {
        bucket.add(step.a.det_res_var.trim());
      }
      if (step.a.ocr_res_var?.trim()) {
        bucket.add(step.a.ocr_res_var.trim());
      }
      if (step.a.out_var?.trim()) {
        bucket.add(step.a.out_var.trim());
      }
      if (step.a.out_det_var?.trim()) {
        bucket.add(step.a.out_det_var.trim());
      }
      if (step.a.out_ocr_var?.trim()) {
        bucket.add(step.a.out_ocr_var.trim());
      }
      collectVariableReferencesFromSteps(step.a.then_steps, bucket);
      continue;
    }

    if (step.op === 'flowControl') {
      if (step.a.type === 'waitMs') {
        if (step.a.input_var?.trim()) {
          bucket.add(step.a.input_var.trim());
        }
        if (step.a.runtime_var?.trim()) {
          bucket.add(step.a.runtime_var.trim());
        }
        continue;
      }
      if (step.a.type === 'searchPolicySetText') {
        if (step.a.ocr_input_var?.trim()) {
          bucket.add(step.a.ocr_input_var.trim());
        }
        if (step.a.out_var?.trim()) {
          bucket.add(step.a.out_var.trim());
        }
        continue;
      }
      if (step.a.type === 'handlePolicySet') {
        if (step.a.det_input_var?.trim()) {
          bucket.add(step.a.det_input_var.trim());
        }
        if (step.a.search_hits_var?.trim()) {
          bucket.add(step.a.search_hits_var.trim());
        }
        if (step.a.out_var?.trim()) {
          bucket.add(step.a.out_var.trim());
        }
        continue;
      }

      if (step.a.type === 'handlePolicy') {
        if (step.a.input_var?.trim()) {
          bucket.add(step.a.input_var.trim());
        }
        if (step.a.out_var?.trim()) {
          bucket.add(step.a.out_var.trim());
        }
        continue;
      }

      if (step.a.type === 'if' || step.a.type === 'while') {
        collectConditionVariableReferences(step.a.con, bucket);
      }

      if (step.a.type === 'if') {
        collectVariableReferencesFromSteps(step.a.then, bucket);
        collectVariableReferencesFromSteps(step.a.else_steps ?? [], bucket);
        continue;
      }

      if (step.a.type === 'while' || step.a.type === 'forEach' || step.a.type === 'repeat') {
        if (step.a.type === 'forEach' && step.a.input_var?.trim()) {
          bucket.add(step.a.input_var.trim());
        }
        collectVariableReferencesFromSteps(step.a.flow, bucket);
      }
    }
  }

  return bucket;
};

const pushVariableUsage = (bucket: VariableUsage[], key: string | null | undefined, label: string) => {
  const trimmed = key?.trim();
  if (!trimmed) {
    return;
  }
  bucket.push({ key: trimmed, label });
};

const collectVariableUsagesFromCondition = (condition: ConditionNode, scopeLabel: string, bucket: VariableUsage[]) => {
  if (condition.type === 'group') {
    condition.items.forEach((item) => collectVariableUsagesFromCondition(item, scopeLabel, bucket));
    return;
  }
  if (condition.type === 'varCompare') {
    pushVariableUsage(bucket, condition.var_name, `${scopeLabel}的条件`);
    return;
  }
  if (condition.type === 'execNumCompare' && condition.value.type === 'variable') {
    pushVariableUsage(bucket, condition.value.var_name, `${scopeLabel}的执行次数条件`);
    return;
  }
  if (condition.type === 'visionCountCompare') {
    pushVariableUsage(bucket, condition.input_var, `${scopeLabel}的条件`);
    return;
  }
  if (condition.type === 'policySetResult') {
    pushVariableUsage(bucket, condition.result_var, `${scopeLabel}的条件`);
  }
};

export const collectVariableUsagesFromSteps = (steps: Step[], scopeLabel: string, bucket: VariableUsage[]) => {
  for (const step of steps) {
    const stepLabel = `${scopeLabel}的步骤「${describeStep(step)}」`;

    if (step.op === 'sequence') {
      collectVariableUsagesFromSteps(step.steps, scopeLabel, bucket);
      continue;
    }

    if (step.op === 'action') {
      if (step.a.ac === 'capture') {
        pushVariableUsage(bucket, step.a.output_var, stepLabel);
        continue;
      }
      if ((step.a.ac === 'click' || step.a.ac === 'swipe') && (step.a.mode === 'txt' || step.a.mode === 'labelIdx')) {
        pushVariableUsage(bucket, step.a.input_var, stepLabel);
      }
      if ((step.a.ac === 'click' || step.a.ac === 'longClick') && (step.a.mode === 'point' || step.a.mode === 'percent')) {
        pushVariableUsage(bucket, step.a.p_expr, stepLabel);
      }
      if (step.a.ac === 'swipe' && (step.a.mode === 'point' || step.a.mode === 'percent')) {
        pushVariableUsage(bucket, step.a.from_expr, stepLabel);
        pushVariableUsage(bucket, step.a.to_expr, stepLabel);
      }
      if ((step.a.ac === 'click' || step.a.ac === 'longClick') && step.a.mode === 'txt') {
        pushVariableUsage(bucket, step.a.txt_expr, stepLabel);
      }
      if ((step.a.ac === 'click' || step.a.ac === 'longClick') && step.a.mode === 'labelIdx') {
        pushVariableUsage(bucket, step.a.idx_expr, stepLabel);
      }
      if (step.a.ac === 'swipe' && step.a.mode === 'txt') {
        pushVariableUsage(bucket, step.a.from_expr, stepLabel);
        pushVariableUsage(bucket, step.a.to_expr, stepLabel);
      }
    }

    if (step.op === 'dataHanding') {
      if (step.a.type === 'setVar' || step.a.type === 'getVar') {
        pushVariableUsage(bucket, step.a.name, stepLabel);
        continue;
      }
      if (step.a.type === 'clearVars') {
        step.a.names.forEach((name) => pushVariableUsage(bucket, name, stepLabel));
        continue;
      }
      if (step.a.type === 'filter') {
        pushVariableUsage(bucket, step.a.input_var, stepLabel);
        pushVariableUsage(bucket, step.a.out_name, stepLabel);
        collectVariableUsagesFromSteps(step.a.then_steps, scopeLabel, bucket);
        continue;
      }
      if (step.a.type === 'colorCompare') {
        pushVariableUsage(bucket, step.a.input_var, stepLabel);
        pushVariableUsage(bucket, step.a.out_var, stepLabel);
        continue;
      }
    }

    if (step.op === 'vision' && step.a.type === 'visionSearch') {
      pushVariableUsage(bucket, step.a.det_res_var, stepLabel);
      pushVariableUsage(bucket, step.a.ocr_res_var, stepLabel);
      pushVariableUsage(bucket, step.a.out_var, stepLabel);
      pushVariableUsage(bucket, step.a.out_det_var, stepLabel);
      pushVariableUsage(bucket, step.a.out_ocr_var, stepLabel);
      collectVariableUsagesFromSteps(step.a.then_steps, scopeLabel, bucket);
      continue;
    }

    if (step.op === 'flowControl') {
      if (step.a.type === 'waitMs') {
        pushVariableUsage(bucket, step.a.input_var, stepLabel);
        pushVariableUsage(bucket, step.a.runtime_var, stepLabel);
        continue;
      }
      if (step.a.type === 'searchPolicySetText') {
        pushVariableUsage(bucket, step.a.ocr_input_var, `${stepLabel}的OCR输入`);
        pushVariableUsage(bucket, step.a.out_var, stepLabel);
        continue;
      }
      if (step.a.type === 'handlePolicySet') {
        pushVariableUsage(bucket, step.a.det_input_var, `${stepLabel}的检测输入`);
        pushVariableUsage(bucket, step.a.search_hits_var, `${stepLabel}的搜索命中输入`);
        pushVariableUsage(bucket, step.a.out_var, stepLabel);
        continue;
      }
      if (step.a.type === 'handlePolicy') {
        pushVariableUsage(bucket, step.a.input_var, stepLabel);
        pushVariableUsage(bucket, step.a.out_var, stepLabel);
        continue;
      }
      if (step.a.type === 'if' || step.a.type === 'while') {
        collectVariableUsagesFromCondition(step.a.con, stepLabel, bucket);
      }
      if (step.a.type === 'if') {
        collectVariableUsagesFromSteps(step.a.then, scopeLabel, bucket);
        collectVariableUsagesFromSteps(step.a.else_steps ?? [], scopeLabel, bucket);
        continue;
      }
      if (step.a.type === 'forEach') {
        pushVariableUsage(bucket, step.a.input_var, `${stepLabel}的结果集`);
        pushVariableUsage(bucket, step.a.item_var, `${stepLabel}的元素变量`);
        pushVariableUsage(bucket, step.a.index_var, `${stepLabel}的索引变量`);
      }
      if (step.a.type === 'repeat') {
        pushVariableUsage(bucket, step.a.count_expr, `${stepLabel}的循环次数`);
        pushVariableUsage(bucket, step.a.index_var, `${stepLabel}的索引变量`);
      }
      if (step.a.type === 'while' || step.a.type === 'forEach' || step.a.type === 'repeat') {
        collectVariableUsagesFromSteps(step.a.flow, scopeLabel, bucket);
      }
    }
  }
};

export const buildVariableUsageMap = (tasks: ScriptTaskTable[], policies: PolicyTable[]) => {
  const bucket: VariableUsage[] = [];

  for (const task of tasks) {
    collectVariableUsagesFromSteps(task.data.steps as Step[], `任务「${task.name}」`, bucket);
    const taskUiSchema = parseUiSchema(task.data.uiData ?? {});
    for (const field of taskUiSchema.fields) {
      if (field.inputKey?.trim()) {
        bucket.push({
          key: buildVariableReferenceKey('input', field.inputKey),
          label: `任务「${task.name}」的界面字段「${field.label || field.inputKey}」`,
        });
      }
    }
  }

  for (const policy of policies) {
    collectVariableUsagesFromSteps(policy.data.beforeAction as Step[], `策略「${policy.data.name}」的全局行为`, bucket);
    collectVariableUsagesFromSteps(policy.data.afterAction as Step[], `策略「${policy.data.name}」的命中行为`, bucket);
  }

  return bucket.reduce<Record<string, string[]>>((map, usage) => {
    if (!map[usage.key]) {
      map[usage.key] = [];
    }
    if (!map[usage.key].includes(usage.label)) {
      map[usage.key].push(usage.label);
    }
    return map;
  }, {});
};

const renameConditionVariableReferences = (condition: ConditionNode, previousKey: string, nextKey: string): ConditionNode => {
  const nextCondition = cloneJson(condition) as ConditionNode;

  if (nextCondition.type === 'group') {
    nextCondition.items = nextCondition.items.map((item) => renameConditionVariableReferences(item, previousKey, nextKey));
    return nextCondition;
  }

  if (nextCondition.type === 'varCompare' && nextCondition.var_name === previousKey) {
    nextCondition.var_name = nextKey;
    return nextCondition;
  }

  if (
    nextCondition.type === 'execNumCompare'
    && nextCondition.value.type === 'variable'
    && nextCondition.value.var_name === previousKey
  ) {
    nextCondition.value.var_name = nextKey;
    return nextCondition;
  }

  if (nextCondition.type === 'visionCountCompare' && nextCondition.input_var === previousKey) {
    nextCondition.input_var = nextKey;
    return nextCondition;
  }

  if (nextCondition.type === 'policySetResult' && nextCondition.result_var === previousKey) {
    nextCondition.result_var = nextKey;
  }

  return nextCondition;
};

export const renameVariableReferencesInSteps = (steps: Step[], previousKey: string, nextKey: string): Step[] =>
  steps.map((step) => {
    const nextStep = cloneJson(step);

    if (nextStep.op === 'sequence') {
      nextStep.steps = renameVariableReferencesInSteps(nextStep.steps, previousKey, nextKey);
      return nextStep;
    }

    if (nextStep.op === 'dataHanding') {
      if (nextStep.a.type === 'setVar' || nextStep.a.type === 'getVar') {
        if (nextStep.a.name === previousKey) {
          nextStep.a.name = nextKey;
        }
        return nextStep;
      }
      if (nextStep.a.type === 'clearVars') {
        nextStep.a.names = nextStep.a.names.map((name) => (name === previousKey ? nextKey : name));
        return nextStep;
      }

      if (nextStep.a.type === 'filter') {
        if (nextStep.a.input_var === previousKey) {
          nextStep.a.input_var = nextKey;
        }
        if (nextStep.a.out_name === previousKey) {
          nextStep.a.out_name = nextKey;
        }
        nextStep.a.then_steps = renameVariableReferencesInSteps(nextStep.a.then_steps, previousKey, nextKey);
        return nextStep;
      }

      if (nextStep.a.type === 'colorCompare') {
        if (nextStep.a.input_var === previousKey) {
          nextStep.a.input_var = nextKey;
        }
        if (nextStep.a.out_var === previousKey) {
          nextStep.a.out_var = nextKey;
        }
        return nextStep;
      }

      return nextStep;
    }

    if (nextStep.op === 'flowControl') {
      if (nextStep.a.type === 'if' || nextStep.a.type === 'while') {
        nextStep.a.con = renameConditionVariableReferences(nextStep.a.con, previousKey, nextKey);
      }

      if (nextStep.a.type === 'handlePolicySet') {
        if (nextStep.a.det_input_var === previousKey) {
          nextStep.a.det_input_var = nextKey;
        }
        if (nextStep.a.search_hits_var === previousKey) {
          nextStep.a.search_hits_var = nextKey;
        }
        if (nextStep.a.out_var === previousKey) {
          nextStep.a.out_var = nextKey;
        }
        return nextStep;
      }

      if (nextStep.a.type === 'searchPolicySetText') {
        if (nextStep.a.ocr_input_var === previousKey) {
          nextStep.a.ocr_input_var = nextKey;
        }
        if (nextStep.a.out_var === previousKey) {
          nextStep.a.out_var = nextKey;
        }
        return nextStep;
      }

      if (nextStep.a.type === 'handlePolicy') {
        if (nextStep.a.input_var === previousKey) {
          nextStep.a.input_var = nextKey;
        }
        if (nextStep.a.out_var === previousKey) {
          nextStep.a.out_var = nextKey;
        }
        return nextStep;
      }

      if (nextStep.a.type === 'if') {
        nextStep.a.then = renameVariableReferencesInSteps(nextStep.a.then, previousKey, nextKey);
        nextStep.a.else_steps = nextStep.a.else_steps ? renameVariableReferencesInSteps(nextStep.a.else_steps, previousKey, nextKey) : nextStep.a.else_steps;
        return nextStep;
      }

      if (nextStep.a.type === 'forEach') {
        if (nextStep.a.input_var === previousKey) {
          nextStep.a.input_var = nextKey;
        }
        nextStep.a.flow = renameVariableReferencesInSteps(nextStep.a.flow, previousKey, nextKey);
        return nextStep;
      }

      if (nextStep.a.type === 'while' || nextStep.a.type === 'repeat') {
        nextStep.a.flow = renameVariableReferencesInSteps(nextStep.a.flow, previousKey, nextKey);
        return nextStep;
      }

      return nextStep;
    }

    if (nextStep.op === 'action') {
      if (nextStep.a.ac === 'capture') {
        if (nextStep.a.output_var === previousKey) {
          nextStep.a.output_var = nextKey;
        }
        return nextStep;
      }

      if ((nextStep.a.ac === 'click' || nextStep.a.ac === 'swipe') && (nextStep.a.mode === 'txt' || nextStep.a.mode === 'labelIdx') && nextStep.a.input_var === previousKey) {
        nextStep.a.input_var = nextKey;
      }
      if (
        (nextStep.a.ac === 'click' || nextStep.a.ac === 'longClick') &&
        (nextStep.a.mode === 'point' || nextStep.a.mode === 'percent') &&
        nextStep.a.p_expr === previousKey
      ) {
        nextStep.a.p_expr = nextKey;
      }
      if (nextStep.a.ac === 'swipe' && (nextStep.a.mode === 'point' || nextStep.a.mode === 'percent')) {
        if (nextStep.a.from_expr === previousKey) {
          nextStep.a.from_expr = nextKey;
        }
        if (nextStep.a.to_expr === previousKey) {
          nextStep.a.to_expr = nextKey;
        }
      }
      if ((nextStep.a.ac === 'click' || nextStep.a.ac === 'longClick') && nextStep.a.mode === 'txt' && nextStep.a.txt_expr === previousKey) {
        nextStep.a.txt_expr = nextKey;
      }
      if ((nextStep.a.ac === 'click' || nextStep.a.ac === 'longClick') && nextStep.a.mode === 'labelIdx' && nextStep.a.idx_expr === previousKey) {
        nextStep.a.idx_expr = nextKey;
      }
      if (nextStep.a.ac === 'swipe' && nextStep.a.mode === 'txt') {
        if (nextStep.a.from_expr === previousKey) {
          nextStep.a.from_expr = nextKey;
        }
        if (nextStep.a.to_expr === previousKey) {
          nextStep.a.to_expr = nextKey;
        }
      }
      return nextStep;
    }

    if (nextStep.op === 'vision' && nextStep.a.type === 'visionSearch') {
      if (nextStep.a.out_var === previousKey) {
        nextStep.a.out_var = nextKey;
      }
      nextStep.a.then_steps = renameVariableReferencesInSteps(nextStep.a.then_steps, previousKey, nextKey);
    }

    return nextStep;
  });

export const renameInputKeyReferencesInUiData = (uiData: JsonValue, previousKey: string, nextKey: string): JsonValue => {
  if (!uiData || typeof uiData !== 'object' || Array.isArray(uiData)) {
    return uiData;
  }

  const previousInputKey = previousKey.replace(/^input\./, '');
  const nextInputKey = nextKey.replace(/^input\./, '');
  const fields = Array.isArray((uiData as { fields?: unknown[] }).fields) ? (uiData as { fields: Array<Record<string, unknown>> }).fields : null;

  if (!fields) {
    return uiData;
  }

  return {
    ...uiData,
    fields: fields.map((field) => ({
      ...field,
      ...(field.inputKey === previousInputKey ? { inputKey: nextInputKey } : {}),
    })),
  };
};
