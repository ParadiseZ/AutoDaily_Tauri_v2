import type { PolicyGroupTable } from '@/types/bindings/PolicyGroupTable';
import type { PolicySetTable } from '@/types/bindings/PolicySetTable';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import { createSearchRule } from '@/views/script-editor/editorSearchRule';
import { cloneJson } from '@/views/script-editor/editorSchema';

export type EditorModeId = 'task' | 'policy' | 'policyGroup' | 'policySet';
export type PolicyEditorPanelId = 'basic' | 'condition' | 'before' | 'after';
export type RelationEditorPanelId = 'basic' | 'relations';

export interface EditorNamedItem {
  id: string;
  title: string;
  subtitle: string;
  badge?: string;
}

export const editorModeOptions: Array<{ id: EditorModeId; label: string }> = [
  { id: 'task', label: '任务' },
  { id: 'policy', label: '策略' },
  { id: 'policyGroup', label: '策略组' },
  { id: 'policySet', label: '策略集' },
];

export const policyPanelTabs: Array<{ id: PolicyEditorPanelId; label: string }> = [
  { id: 'basic', label: '信息' },
  { id: 'condition', label: '命中条件' },
  { id: 'before', label: '全局行为' },
  { id: 'after', label: '命中行为' },
];

export const relationPanelTabs: Array<{ id: RelationEditorPanelId; label: string }> = [
  { id: 'basic', label: '信息' },
  { id: 'relations', label: '关联' },
];

export const normalizePolicy = (policy: PolicyTable, index: number): PolicyTable => ({
  ...policy,
  orderIndex: index,
  data: {
    name: policy.data?.name || `策略 ${index + 1}`,
    note: policy.data?.note || '',
    logPrint: policy.data?.logPrint ?? null,
    curPos: policy.data?.curPos ?? 0,
    skipFlag: Boolean(policy.data?.skipFlag),
    execCur: policy.data?.execCur ?? 0,
    execMax: policy.data?.execMax ?? 1,
    beforeAction: Array.isArray(policy.data?.beforeAction) ? cloneJson(policy.data.beforeAction) : [],
    cond: policy.data?.cond ? cloneJson(policy.data.cond) : createSearchRule('group'),
    afterAction: Array.isArray(policy.data?.afterAction) ? cloneJson(policy.data.afterAction) : [],
  },
});

export const normalizePolicyGroup = (group: PolicyGroupTable, index: number): PolicyGroupTable => ({
  ...group,
  orderIndex: index,
  data: {
    name: group.data?.name || `策略组 ${index + 1}`,
    note: group.data?.note || '',
  },
});

export const normalizePolicySet = (set: PolicySetTable, index: number): PolicySetTable => ({
  ...set,
  orderIndex: index,
  data: {
    name: set.data?.name || `策略集 ${index + 1}`,
    note: set.data?.note || '',
  },
});

export const reorderCollection = <T,>(items: T[], fromIndex: number, toIndex: number) => {
  const next = [...items];
  const [moved] = next.splice(fromIndex, 1);
  next.splice(toIndex, 0, moved);
  return next;
};

export const createEmptyRelationMap = <T extends string>() => ({} as Record<string, T[]>);
