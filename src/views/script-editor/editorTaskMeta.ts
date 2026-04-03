import type { TaskCycle } from '@/types/bindings/TaskCycle';
import type { TaskRowType } from '@/types/bindings/TaskRowType';
import type { TaskTone } from '@/types/bindings/TaskTone';
import type { TaskTriggerMode } from '@/types/bindings/TaskTriggerMode';

export const taskRowTypeOptions: Array<{ label: string; value: TaskRowType; description: string }> = [
  { label: '任务行', value: 'task', description: '可执行任务，显示在普通用户任务列表中。' },
  { label: '标题行', value: 'title', description: '只用于任务分块显示，不直接执行。' },
];

export const taskTriggerModeOptions: Array<{ label: string; value: TaskTriggerMode; description: string }> = [
  { label: '一级循环', value: 'rootOnly', description: '只在一级循环里执行。' },
  { label: '仅跳转', value: 'linkOnly', description: '只能由其他任务通过 link 等方式进入。' },
  { label: '两者都可', value: 'rootAndLink', description: '既可在一级循环执行，也可被其他任务跳转进入。' },
];

export const taskToneOptions: Array<{ label: string; value: TaskTone; description: string }> = [
  { label: '普通', value: 'normal', description: '默认展示，不做风险强调。' },
  { label: '警告', value: 'warning', description: '提示用户注意资源消耗或道具使用。' },
  { label: '严重', value: 'danger', description: '强调高风险或重要消耗。' },
];

export const taskCycleOptions: Array<{ label: string; value: string; description: string }> = [
  { label: '每次运行', value: 'everyRun', description: '脚本每次运行都执行。' },
  { label: '每天', value: 'daily', description: '默认按每日周期执行。' },
  { label: '每周', value: 'weekly', description: '默认按每周周期执行。' },
  { label: '指定周几', value: 'weekDay', description: '按指定周几执行。' },
  { label: '每月', value: 'monthly', description: '默认按每月周期执行。' },
  { label: '指定日期', value: 'monthDay', description: '按指定日期执行。' },
];

export const formatTaskCycleValue = (value: TaskCycle): string => {
  if (typeof value === 'string') {
    return value;
  }

  if ('weekDay' in value) {
    return `weekDay:${value.weekDay}`;
  }

  return `monthDay:${value.monthDay}`;
};

export const parseTaskCycleValue = (value: string): TaskCycle => {
  if (value.startsWith('weekDay:')) {
    return {
      weekDay: Math.max(1, Math.min(7, Number(value.slice('weekDay:'.length)) || 1)),
    };
  }

  if (value === 'weekDay') {
    return {
      weekDay: defaultTaskCycleDraft.weekDay,
    };
  }

  if (value.startsWith('monthDay:')) {
    return {
      monthDay: Math.max(1, Math.min(31, Number(value.slice('monthDay:'.length)) || 1)),
    };
  }

  if (value === 'monthDay') {
    return {
      monthDay: defaultTaskCycleDraft.monthDay,
    };
  }

  if (value === 'daily' || value === 'weekly' || value === 'monthly' || value === 'everyRun') {
    return value;
  }

  return 'everyRun';
};

export const defaultTaskCycleDraft = {
  weekDay: 1,
  monthDay: 1,
};
