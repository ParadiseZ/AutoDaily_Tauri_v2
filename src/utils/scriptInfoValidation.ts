import type { ScriptTableRecord } from '@/types/app/domain';

export interface ScriptInfoValidationIssue {
  field: string;
  label: string;
  message: string;
}

const isBlank = (value: string | null | undefined) => !value || !value.trim();

export const validateScriptInfo = (script: ScriptTableRecord | null | undefined): ScriptInfoValidationIssue[] => {
  if (!script) {
    return [];
  }

  const issues: ScriptInfoValidationIssue[] = [];

  if (isBlank(script.data.name)) {
    issues.push({ field: 'name', label: '脚本名称', message: '请填写脚本名称。' });
  }

  if (isBlank(script.data.description)) {
    issues.push({
      field: 'description',
      label: '描述',
      message: '请填写脚本描述，方便说明用途、前置条件和风险提示。',
    });
  }

  if (isBlank(script.data.verName)) {
    issues.push({ field: 'verName', label: '版本名称', message: '请填写版本名称。' });
  }

  return issues;
};

export const formatScriptInfoValidationMessage = (
  issues: ScriptInfoValidationIssue[],
  title = '请先补齐以下脚本信息：',
) => {
  if (!issues.length) {
    return title;
  }

  return `${title}\n${issues.map((issue) => `- ${issue.label}：${issue.message}`).join('\n')}`;
};
