<template>
  <nav class="editor-step-breadcrumb" aria-label="步骤层级">
    <template v-for="(item, index) in items" :key="item.key">
      <span v-if="index > 0" class="editor-step-breadcrumb-separator">/</span>
      <button
        class="editor-step-breadcrumb-link"
        :class="{ 'editor-step-breadcrumb-link-active': index === items.length - 1 }"
        type="button"
        :disabled="index === items.length - 1"
        @click="handleClick(item)"
      >
        {{ item.label }}
      </button>
    </template>
  </nav>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Step } from '@/types/bindings/Step';
import {
  getParentBranchPath,
  getStepByPath,
  isSameBranchPath,
  isSameStepPath,
  ROOT_BRANCH_PATH,
  type StepBranchKind,
  type StepBranchPath,
  type StepPath,
} from '@/views/script-editor/editor-step/editorStepTree';

interface StepBreadcrumbItem {
  key: string;
  label: string;
  type: 'branch' | 'step';
  branchPath?: StepBranchPath;
  stepPath?: StepPath;
}

const props = defineProps<{
  steps: Step[];
  activeBranchPath: StepBranchPath;
  selectedStepPath: StepPath | null;
}>();

const emit = defineEmits<{
  'navigate-branch': [branchPath: StepBranchPath];
  'select-step-path': [path: StepPath];
}>();

const branchLabelMap: Record<StepBranchKind, string> = {
  root: '顶层步骤',
  sequence: '顺序步骤',
  then: 'Then',
  else: 'Else',
  flow: '循环体',
  visionThen: '命中后执行',
  filterThen: '过滤命中后',
};

const stepLabel = (path: StepPath) => {
  const lastSegment = path[path.length - 1];
  const step = getStepByPath(props.steps, path);
  return step?.label?.trim() || `步骤 ${lastSegment.index + 1}`;
};

const selectedStepInActiveBranch = computed(() => {
  if (!props.selectedStepPath) return null;
  return isSameBranchPath(getParentBranchPath(props.selectedStepPath), props.activeBranchPath) ? props.selectedStepPath : null;
});

const items = computed<StepBreadcrumbItem[]>(() => {
  const result: StepBreadcrumbItem[] = [
    {
      key: 'branch-root',
      label: branchLabelMap.root,
      type: 'branch',
      branchPath: ROOT_BRANCH_PATH,
    },
  ];

  const parentPath = props.activeBranchPath.parentStepPath ?? [];
  parentPath.forEach((_, index) => {
    const path = parentPath.slice(0, index + 1);
    result.push({
      key: `step-${index}`,
      label: stepLabel(path),
      type: 'step',
      stepPath: path,
    });
  });

  if (props.activeBranchPath.branch !== 'root') {
    result.push({
      key: `branch-${props.activeBranchPath.branch}`,
      label: branchLabelMap[props.activeBranchPath.branch],
      type: 'branch',
      branchPath: props.activeBranchPath,
    });
  }

  if (selectedStepInActiveBranch.value && !isSameStepPath(selectedStepInActiveBranch.value, props.activeBranchPath.parentStepPath)) {
    result.push({
      key: `step-selected-${selectedStepInActiveBranch.value.length}`,
      label: stepLabel(selectedStepInActiveBranch.value),
      type: 'step',
      stepPath: selectedStepInActiveBranch.value,
    });
  }

  return result;
});

const handleClick = (item: StepBreadcrumbItem) => {
  if (item.type === 'step' && item.stepPath) {
    emit('select-step-path', item.stepPath);
    return;
  }

  if (item.branchPath) {
    if (isSameBranchPath(item.branchPath, props.activeBranchPath) && item.branchPath.parentStepPath?.length) {
      emit('select-step-path', item.branchPath.parentStepPath);
      return;
    }
    emit('navigate-branch', item.branchPath);
  }
};
</script>

<style scoped>
.editor-step-breadcrumb {
  display: flex;
  align-items: center;
  gap: 0.45rem;
  min-width: 0;
  flex-wrap: wrap;
}

.editor-step-breadcrumb-separator {
  color: var(--app-text-faint);
  font-size: 0.78rem;
}

.editor-step-breadcrumb-link {
  min-width: 0;
  padding: 0;
  border: none;
  background: transparent;
  color: var(--app-text-soft);
  font-size: 0.82rem;
  line-height: 1.4;
  transition: color 0.16s ease;
}

.editor-step-breadcrumb-link:hover:enabled {
  color: var(--app-text-strong);
}

.editor-step-breadcrumb-link-active {
  color: var(--app-text-strong);
  font-weight: 600;
  cursor: default;
}
</style>
