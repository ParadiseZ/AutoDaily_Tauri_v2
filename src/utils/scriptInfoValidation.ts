import type { ScriptTableRecord } from '@/types/app/domain';

export interface ScriptInfoValidationIssue {
  field: string;
  label: string;
  message: string;
}

const isBlank = (value: string | null | undefined) => !value || !value.trim();

const getDetectorBaseModelPath = (model: ScriptTableRecord['data']['imgDetModel'] | ScriptTableRecord['data']['txtDetModel']) => {
  if (!model) {
    return null;
  }
  if ('Yolo11' in model) {
    return model.Yolo11.baseModel.modelPath;
  }
  if ('Yolo26' in model) {
    return model.Yolo26.baseModel.modelPath;
  }
  if ('PaddleDbNet' in model) {
    return model.PaddleDbNet.baseModel.modelPath;
  }
  return null;
};

const getDetectorLabelPath = (model: ScriptTableRecord['data']['imgDetModel'] | ScriptTableRecord['data']['txtDetModel']) => {
  if (!model) {
    return null;
  }
  if ('Yolo11' in model) {
    return model.Yolo11.labelPath;
  }
  if ('Yolo26' in model) {
    return model.Yolo26.labelPath;
  }
  return null;
};

const getRecognizerBaseModelPath = (model: ScriptTableRecord['data']['txtRecModel']) => {
  if (!model) {
    return null;
  }
  if ('PaddleCrnn' in model) {
    return model.PaddleCrnn.baseModel.modelPath;
  }
  return null;
};

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
  if(isBlank(script.data.runtimeType)) {
    script.data.runtimeType = 'rhai';
  }
  if (script.data.runtimeType == 'rhai') {
    if (!script.data.imgDetModel) {
      issues.push({ field: 'imgDetModel', label: '图像检测模型', message: 'Rhai 运行时必须配置图像检测模型。' });
    } else {
      if (isBlank(getDetectorBaseModelPath(script.data.imgDetModel))) {
        issues.push({ field: 'imgDetModel.modelPath', label: '图像检测模型路径', message: 'Rhai 运行时必须填写图像检测模型路径。' });
      }
      if (isBlank(getDetectorLabelPath(script.data.imgDetModel))) {
        issues.push({ field: 'imgDetModel.labelPath', label: '图像检测标签路径', message: 'Rhai 运行时必须填写图像检测标签路径。' });
      }
    }

    if (!script.data.txtDetModel) {
      issues.push({ field: 'txtDetModel', label: '文字检测模型', message: 'Rhai 运行时必须配置文字检测模型。' });
    } else {
      if (isBlank(getDetectorBaseModelPath(script.data.txtDetModel))) {
        issues.push({ field: 'txtDetModel.modelPath', label: '文字检测模型路径', message: 'Rhai 运行时必须填写文字检测模型路径。' });
      }
      if (!('PaddleDbNet' in script.data.txtDetModel) && isBlank(getDetectorLabelPath(script.data.txtDetModel))) {
        issues.push({ field: 'txtDetModel.labelPath', label: '文字检测标签路径', message: 'Rhai 运行时必须填写文字检测标签路径。' });
      }
    }

    if (!script.data.txtRecModel) {
      issues.push({ field: 'txtRecModel', label: '文字识别模型', message: 'Rhai 运行时必须配置文字识别模型。' });
    } else if (isBlank(getRecognizerBaseModelPath(script.data.txtRecModel)) && script.data.txtRecModel.PaddleCrnn.baseModel.modelSource !== 'BuiltIn') {
      issues.push({ field: 'txtRecModel.modelPath', label: '文字识别模型路径', message: 'Rhai 运行时必须填写文字识别模型路径。' });
    }
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
