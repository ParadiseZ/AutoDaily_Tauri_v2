<template>
  <div class="editor-shell h-[100svh] overflow-hidden px-4 py-4 lg:px-6 lg:py-5">
    <div class="mx-auto flex h-full max-w-[1760px] flex-col gap-4">
      <header class="editor-toolbar rounded-[28px] border border-[var(--app-border)] px-5 py-4 lg:px-6">
        <div class="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
          <div class="flex flex-wrap items-center gap-3">
            <button class="app-button app-button-ghost group" type="button" @click="router.push('/scripts')">
              <AppIcon name="arrow-left" :size="16" class="text-[var(--app-text-soft)] group-hover:text-[var(--app-accent)] transition-colors" />
              返回
            </button>

            <div class="space-y-1">
              <div class="flex items-center gap-2 text-xs uppercase tracking-[0.18em] text-[var(--app-text-faint)]">
                <span>Task Editor</span>
                <span class="rounded-full border border-[var(--app-border)] bg-white/40 px-3 py-1">脚本开发工作台</span>
              </div>
              <div class="flex flex-wrap items-center gap-2">
                <h1 class="text-2xl font-semibold tracking-[-0.05em] text-[var(--app-text-strong)] lg:text-3xl">
                  {{ draftScript?.data.name || '脚本编辑器' }}
                </h1>
                <button class="app-button app-button-ghost group" type="button" data-testid="editor-script-info" @click="infoDialogOpen = true">
                  <AppIcon name="file-text" :size="16" class="text-[var(--app-text-soft)] group-hover:text-[var(--app-accent)] transition-colors" />
                  编辑脚本信息
                </button>
                <span
                  class="rounded-full px-3 py-1 text-xs font-medium"
                  :class="hasValidationErrors ? 'bg-red-500/12 text-red-700' : dirty ? 'bg-amber-500/12 text-amber-700' : 'bg-emerald-500/12 text-emerald-700'"
                >
                  {{ hasValidationErrors ? '待修复' : dirty ? '未保存' : '已同步' }}
                </span>
                <span v-if="formattedSaveTime" class="text-xs text-[var(--app-text-faint)]">最近保存 {{ formattedSaveTime }}</span>
              </div>
            </div>
          </div>

          <div class="flex flex-wrap items-center gap-2">
            <div class="min-w-[240px]">
              <AppSelect
                v-model="selectedPreviewDeviceId"
                :options="deviceSelectOptions"
                placeholder="请选择设备"
                test-id="editor-header-device"
              />
            </div>
            <button class="app-button app-button-ghost group" type="button" @click="openDeviceEditor(selectedPreviewDeviceId)">
              <AppIcon name="edit-3" :size="16" class="text-[var(--app-text-soft)] group-hover:text-[var(--app-accent)] transition-colors" />
              {{ selectedPreviewDeviceId ? '编辑' : '新建' }}
            </button>
            <div class="min-w-[280px]">
              <AppSelect
                v-model="activeTargetValue"
                :options="activeTargetSelectOptions"
                :placeholder="activeTargetSelectPlaceholder"
                test-id="editor-header-target-item"
              />
            </div>
            <button
              class="app-button app-button-ghost group"
              type="button"
              data-testid="editor-run"
              :title="runSelectionDisabledReason || undefined"
              :disabled="!canRunSelection"
              @click="handleRunSelection"
            >
              <AppIcon name="play" :size="16" class="text-[var(--app-text-soft)] group-hover:text-[var(--app-accent)] transition-colors" />
              运行
            </button>
            <button
              class="app-button app-button-primary shadow-lg shadow-[var(--app-accent-soft)]"
              type="button"
              data-testid="editor-save"
              :disabled="!draftScript || isSaving || hasValidationErrors"
              @click="saveEditor"
            >
              <AppIcon name="save" :size="16" />
              {{ isSaving ? '保存中...' : '保存脚本结构' }}
            </button>
          </div>
        </div>
      </header>

      <div v-if="loadError" class="rounded-[28px] border border-red-500/16 bg-red-500/8 px-6 py-8 text-red-700">
        <h2 class="text-xl font-semibold">无法打开编辑器</h2>
        <p class="mt-3 max-w-2xl text-sm leading-6">{{ loadError }}</p>
      </div>

      <div
        v-else-if="isLoading"
        class="rounded-[28px] border border-[var(--app-border)] bg-[var(--app-panel)] px-6 py-10 text-sm text-[var(--app-text-soft)]"
      >
        正在读取脚本和任务结构...
      </div>

      <div v-else class="flex min-h-0 flex-1 flex-col gap-4">
        <div class="grid min-h-0 flex-1 gap-4 xl:grid-cols-[340px_minmax(0,1fr)]">
          <EditorTaskSidebar
            v-if="activeMode === 'task'"
            :tasks="draftTasks"
            :selected-task-id="selectedTaskId"
            @create="createTask"
            @select="selectTask"
            @duplicate="duplicateTask"
            @toggle-hidden="toggleTaskHidden"
            @remove="removeTask"
            @reorder="reorderTasks"
          >
            <template #mode-switch>
              <EditorModeSwitch v-model="activeMode" :options="editorModeOptions" />
            </template>
          </EditorTaskSidebar>

          <EditorCollectionSidebar
            v-else-if="activeMode === 'policy'"
            eyebrow="Policy Mode"
            title="策略列表"
            create-label="新建策略"
            count-label="策略"
            search-placeholder="按名称或备注检索策略"
            :items="policyItems"
            :selected-id="selectedPolicyId"
            empty-title="没有可编辑的策略"
            empty-description="先创建策略，再在右侧配置命中条件和步骤。"
            create-test-id="editor-policy-create-sidebar"
            item-test-id-prefix="editor-policy-item"
            @create="createPolicy"
            @select="selectedPolicyId = $event"
            @remove="removePolicy"
            @reorder="reorderPolicies"
          >
            <template #mode-switch>
              <EditorModeSwitch v-model="activeMode" :options="editorModeOptions" />
            </template>
          </EditorCollectionSidebar>

          <EditorCollectionSidebar
            v-else-if="activeMode === 'policyGroup'"
            eyebrow="Policy Group"
            title="策略组列表"
            create-label="新建策略组"
            count-label="策略组"
            search-placeholder="按名称或备注检索策略组"
            :items="policyGroupItems"
            :selected-id="selectedPolicyGroupId"
            empty-title="没有可编辑的策略组"
            empty-description="先创建策略组，再在右侧维护策略关联。"
            create-test-id="editor-policy-group-create-sidebar"
            item-test-id-prefix="editor-policy-group-item"
            @create="createPolicyGroup"
            @select="selectedPolicyGroupId = $event"
            @remove="removePolicyGroup"
            @reorder="reorderPolicyGroups"
          >
            <template #mode-switch>
              <EditorModeSwitch v-model="activeMode" :options="editorModeOptions" />
            </template>
          </EditorCollectionSidebar>

          <EditorCollectionSidebar
            v-else
            eyebrow="Policy Set"
            title="策略集列表"
            create-label="新建策略集"
            count-label="策略集"
            search-placeholder="按名称或备注检索策略集"
            :items="policySetItems"
            :selected-id="selectedPolicySetId"
            empty-title="没有可编辑的策略集"
            empty-description="先创建策略集，再在右侧维护策略组关联。"
            create-test-id="editor-policy-set-create-sidebar"
            item-test-id-prefix="editor-policy-set-item"
            @create="createPolicySet"
            @select="selectedPolicySetId = $event"
            @remove="removePolicySet"
            @reorder="reorderPolicySets"
          >
            <template #mode-switch>
              <EditorModeSwitch v-model="activeMode" :options="editorModeOptions" />
            </template>
          </EditorCollectionSidebar>

          <div class="flex min-h-0 flex-1 flex-col gap-4">
            <section class="rounded-[22px] border border-[var(--app-border)] bg-[var(--app-panel)] px-4 py-3">
              <div class="flex flex-wrap items-center justify-end gap-2">
                <button
                  class="app-button app-button-ghost app-toolbar-button"
                  type="button"
                  :data-testid="`editor-${activeMode}-create`"
                  @click="createActiveItem"
                >
                  <AppIcon name="plus" :size="14" />
                  新建{{ activeModeLabel }}
                </button>
                <button
                  v-if="activeMode === 'task'"
                  class="app-button app-button-ghost app-toolbar-button"
                  type="button"
                  :disabled="!currentTask"
                  @click="duplicateActiveItem"
                >
                  <AppIcon name="copy" :size="14" />
                  复制
                </button>
                <button
                  class="app-button app-button-ghost app-toolbar-button"
                  type="button"
                  :disabled="!activeTargetValue"
                  @click="moveActiveItem(-1)"
                >
                  <AppIcon name="arrow-up" :size="14" />
                  上移
                </button>
                <button
                  class="app-button app-button-ghost app-toolbar-button"
                  type="button"
                  :disabled="!activeTargetValue"
                  @click="moveActiveItem(1)"
                >
                  <AppIcon name="arrow-down" :size="14" />
                  下移
                </button>
                <button
                  class="app-button app-button-danger app-toolbar-button"
                  type="button"
                  :disabled="!activeTargetValue"
                  @click="removeActiveItem"
                >
                  <AppIcon name="trash-2" :size="14" />
                  删除
                </button>
              </div>
            </section>

            <div class="grid min-h-0 flex-1 gap-4 xl:grid-cols-[360px_minmax(0,1fr)]">
          <EditorTaskConfigPanel
            v-if="activeMode === 'task'"
            :task="currentTask"
            :active-panel="activePanel"
            :task-name="taskName"
            :task-row-type="taskRowType"
            :task-trigger-mode="taskTriggerMode"
            :task-hidden="taskHidden"
            :record-schedule="recordSchedule"
            :section-id="sectionId"
            :indent-level="indentLevel"
            :default-task-cycle-value="defaultTaskCycleValue"
            :default-task-cycle-mode="defaultTaskCycleMode"
            :default-task-cycle-day="defaultTaskCycleDay"
            :show-enabled-toggle="showEnabledToggle"
            :default-enabled="defaultEnabled"
            :task-tone="taskTone"
            :title-options="titleTaskOptions"
            :input-entries="inputEntries"
            :input-error="inputError"
            :ui-schema="uiSchema"
            :selected-input-id="selectedInputId"
            :selected-ui-field-id="selectedUiFieldId"
            @update:active-panel="activePanel = $event"
            @update:task-name="taskName = $event"
            @update:task-row-type="taskRowType = $event"
            @update:task-trigger-mode="taskTriggerMode = $event"
            @update:task-hidden="taskHidden = $event"
            @update:record-schedule="recordSchedule = $event"
            @update:section-id="sectionId = $event"
            @update:indent-level="indentLevel = $event"
            @update:default-task-cycle-value="defaultTaskCycle = parseTaskCycleValue($event)"
            @update:default-task-cycle-day="
              defaultTaskCycle =
                defaultTaskCycleMode === 'weekDay'
                  ? { weekDay: Math.max(1, Math.min(7, $event)) }
                  : { monthDay: Math.max(1, Math.min(31, $event)) }
            "
            @update:show-enabled-toggle="showEnabledToggle = $event"
            @update:default-enabled="defaultEnabled = $event"
            @update:task-tone="taskTone = $event"
            @add-input="addInput"
            @select-input="selectedInputId = $event"
            @remove-input="removeInput"
            @add-ui-field="addUiField"
            @select-ui-field="selectedUiFieldId = $event"
            @remove-ui-field="removeUiField"
            @append-template-step="appendTemplateStep"
            @open-raw="openRawEditor"
          />

          <EditorPolicyConfigPanel
            v-else-if="activeMode === 'policy'"
            :policy="currentPolicy"
            :active-panel="activePolicyPanel"
            :policy-name="currentPolicy?.data.name || ''"
            :policy-note="currentPolicy?.data.note || ''"
            :policy-log-print="currentPolicy?.data.logPrint ?? null"
            @update:active-panel="activePolicyPanel = $event"
            @update:policy-name="updatePolicyTextField('name', $event)"
            @update:policy-note="updatePolicyTextField('note', $event)"
            @update:policy-log-print="updatePolicyTextField('logPrint', $event)"
            @update:number-field="updatePolicyNumberField"
            @update:boolean-field="updatePolicyBooleanField"
            @append-template-step="appendPolicyTemplateStep"
          />

          <EditorRelationConfigPanel
            v-else-if="activeMode === 'policyGroup'"
            :item="currentPolicyGroup"
            name-label="策略组名称"
            relation-title="策略组关联"
            relation-description="策略组只负责对策略分组，右侧上半区是已关联策略，下半区是未关联策略。"
            @update:name="updateRelationName('policyGroup', $event)"
            @update:note="updateRelationNote('policyGroup', $event)"
          />

          <EditorRelationConfigPanel
            v-else
            :item="currentPolicySet"
            name-label="策略集名称"
            relation-title="策略集关联"
            relation-description="策略集负责收拢多个策略组，右侧上半区是已关联策略组，下半区是未关联策略组。"
            @update:name="updateRelationName('policySet', $event)"
            @update:note="updateRelationNote('policySet', $event)"
          />

          <EditorTaskWorkspace
            v-if="activeMode === 'task'"
            :task="currentTask"
            :tasks="draftTasks"
            :active-panel="activePanel"
            :task-trigger-mode="taskTriggerMode"
            :record-schedule="recordSchedule"
            :section-id="sectionId"
            :indent-level="indentLevel"
            :default-task-cycle-value="defaultTaskCycleValue"
            :default-task-cycle-mode="defaultTaskCycleMode"
            :default-task-cycle-day="defaultTaskCycleDay"
            :show-enabled-toggle="showEnabledToggle"
            :default-enabled="defaultEnabled"
            :task-tone="taskTone"
            :title-options="titleTaskOptions"
            :steps="parsedSteps"
            :selected-step-path="selectedStepPath"
            :active-branch-path="activeBranchPath"
            :ui-schema="uiSchema"
            :selected-ui-field-id="selectedUiFieldId"
            :input-entries="inputEntries"
            :variable-options="variableOptions"
            :catalog-variable-options="catalogVariableOptions"
            :label-index-options="textDetLabelOptions"
            :label-select-placeholder="textDetLabelSelectPlaceholder"
            :label-select-hint="textDetLabelHint"
            :task-reference-options="taskReferenceOptions"
            :policy-reference-options="policyReferenceOptions"
            :policy-group-reference-options="policyGroupReferenceOptions"
            :policy-set-reference-options="policySetReferenceOptions"
            :create-reference="createReferenceResource"
            :jump-to-reference="jumpToReferenceResource"
            :create-variable="createVariableResource"
            :jump-to-variable="jumpToVariableResource"
            :selected-input-id="selectedInputId"
            @update:task-name="taskName = $event"
            @update-input="updateInput"
            @remove-input="removeInput"
            @select-input="selectedInputId = $event"
            @select-task="selectTask"
            @update:task-trigger-mode="taskTriggerMode = $event"
            @update:record-schedule="recordSchedule = $event"
            @update:section-id="sectionId = $event"
            @update:indent-level="indentLevel = $event"
            @update:default-task-cycle-value="defaultTaskCycle = parseTaskCycleValue($event)"
            @update:default-task-cycle-day="
              defaultTaskCycle =
                defaultTaskCycleMode === 'weekDay'
                  ? { weekDay: Math.max(1, Math.min(7, $event)) }
                  : { monthDay: Math.max(1, Math.min(31, $event)) }
            "
            @update:show-enabled-toggle="showEnabledToggle = $event"
            @update:default-enabled="defaultEnabled = $event"
            @update:task-tone="taskTone = $event"
            @select-ui-field="selectedUiFieldId = $event"
            @update-ui-field="updateUiField"
            @remove-ui-field="removeUiField"
            @select-step-path="selectStepPath"
            @navigate-branch="navigateBranch"
            @reorder-step="reorderSteps"
            @remove-step="removeStep"
            @update-step="updateStep"
            @open-raw="openRawEditor"
          />

          <EditorPolicyWorkspace
            v-else-if="activeMode === 'policy'"
            :policy="currentPolicy"
            :active-panel="activePolicyPanel"
            :steps="currentPolicySteps"
            :selected-step-path="selectedPolicyStepPath"
            :active-branch-path="activePolicyBranchPath"
            :variable-options="policyVariableOptions"
            :catalog-variable-options="policyCatalogVariableOptions"
            :label-index-options="textDetLabelOptions"
            :label-select-placeholder="textDetLabelSelectPlaceholder"
            :label-select-hint="textDetLabelHint"
            :task-reference-options="taskReferenceOptions"
            :policy-reference-options="policyReferenceOptions"
            :policy-group-reference-options="policyGroupReferenceOptions"
            :policy-set-reference-options="policySetReferenceOptions"
            :create-reference="createReferenceResource"
            :jump-to-reference="jumpToReferenceResource"
            :create-variable="createVariableResource"
            :jump-to-variable="jumpToVariableResource"
            @update:number-field="updatePolicyNumberField"
            @update:boolean-field="updatePolicyBooleanField"
            @update:condition="updatePolicyCondition"
            @select-step-path="selectPolicyStepPath"
            @navigate-branch="navigatePolicyBranch"
            @reorder-step="reorderPolicySteps"
            @remove-step="removePolicyStep"
            @update-step="updatePolicyStep"
          />

          <EditorRelationWorkspace
            v-else-if="activeMode === 'policyGroup'"
            title="策略组关联"
            :selected-title="currentPolicyGroup?.data.name || null"
            assigned-title="已关联策略"
            unassigned-title="未关联策略"
            :assigned-items="assignedPolicies"
            :unassigned-items="unassignedPolicies"
            show-reverse-action
            reverse-action-label="逆序排列"
            @link="linkPolicyToGroup"
            @unlink="unlinkPolicyFromGroup"
            @reorder="reorderGroupPolicies"
            @reverse="reverseGroupPolicies"
          />

          <EditorRelationWorkspace
            v-else
            title="策略集关联"
            :selected-title="currentPolicySet?.data.name || null"
            assigned-title="已关联策略组"
            unassigned-title="未关联策略组"
            :assigned-items="assignedGroups"
            :unassigned-items="unassignedGroups"
            @link="linkGroupToSet"
            @unlink="unlinkGroupFromSet"
            @reorder="reorderSetGroups"
          />
            </div>
          </div>
        </div>

        <EditorConsolePanel :lines="consoleLines" :max-lines="MAX_CONSOLE_LINES" @clear="clearConsole" />
      </div>
    </div>

    <ScriptInfoDialog
      :open="infoDialogOpen"
      mode="edit"
      :script="draftScript"
      :task-options="scriptRecoveryTaskOptions"
      @close="infoDialogOpen = false"
      @save="applyScriptInfo"
    />

    <EditorJsonDialog
      :open="rawDialogOpen"
      :title="rawDialogTitle"
      :description="rawDialogDescription"
      :model-value="rawDialogText"
      :error="rawDialogError"
      @close="rawDialogOpen = false"
      @apply="applyRawEditor"
      @format="formatRawEditor"
      @update:model-value="rawDialogText = $event"
    />

    <DeviceEditorDialog
      :open="deviceEditorOpen"
      :device="editingDevice"
      :cpu-count="deviceStore.cpuCount"
      @close="deviceEditorOpen = false"
      @save="savePreviewDevice"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import DeviceEditorDialog from '@/views/device-list/DeviceEditorDialog.vue';
import { useScriptStore } from '@/store/script';
import { useDeviceStore } from '@/store/device';
import { useSettingsStore } from '@/store/settings';
import { runtimeService } from '@/services/runtimeService';
import { scriptService } from '@/services/scriptService';
import { taskService } from '@/services/taskService';
import type { DeviceFormState, JsonValue, RunTarget, ScriptTableRecord } from '@/types/app/domain';
import type { ADBConnectConfig } from '@/types/bindings/ADBConnectConfig';
import type { DetectorType } from '@/types/bindings/DetectorType';
import type { DeviceTable } from '@/types/bindings/DeviceTable';
import type { PolicyGroupTable } from '@/types/bindings/PolicyGroupTable';
import type { PolicySetTable } from '@/types/bindings/PolicySetTable';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { SearchRule } from '@/types/bindings/SearchRule';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { Step } from '@/types/bindings/Step';
import type { TaskCycle } from '@/types/bindings/TaskCycle';
import type { TaskRowType } from '@/types/bindings/TaskRowType';
import type { TaskTone } from '@/types/bindings/TaskTone';
import type { TaskTriggerMode } from '@/types/bindings/TaskTriggerMode';
import type { YoloDet } from '@/types/bindings/YoloDet';
import { showToast } from '@/utils/toast';
import { formatCaptureMethod, formatConnectLabel } from '@/utils/presenters';
import { validateDeviceRuntimePlatform, validateRunTargetRecoveryForDevice } from '@/utils/runtimePolicy';
import ScriptInfoDialog from '@/views/script-list/ScriptInfoDialog.vue';
import EditorJsonDialog from '@/views/script-editor/EditorJsonDialog.vue';
import EditorConsolePanel from '@/views/script-editor/EditorConsolePanel.vue';
import EditorModeSwitch from '@/views/script-editor/EditorModeSwitch.vue';
import EditorTaskSidebar from '@/views/script-editor/EditorTaskSidebar.vue';
import EditorTaskConfigPanel from '@/views/script-editor/EditorTaskConfigPanel.vue';
import EditorTaskWorkspace from '@/views/script-editor/EditorTaskWorkspace.vue';
import EditorCollectionSidebar from '@/views/script-editor/editor-policy/EditorCollectionSidebar.vue';
import EditorPolicyConfigPanel from '@/views/script-editor/editor-policy/EditorPolicyConfigPanel.vue';
import EditorPolicyWorkspace from '@/views/script-editor/editor-policy/EditorPolicyWorkspace.vue';
import EditorRelationConfigPanel from '@/views/script-editor/editor-policy/EditorRelationConfigPanel.vue';
import EditorRelationWorkspace from '@/views/script-editor/editor-policy/EditorRelationWorkspace.vue';
import type { EditorReferenceKind, EditorReferenceOption } from '@/views/script-editor/editorReferences';
import {
  createEmptyRelationMap,
  editorModeOptions,
  normalizePolicy,
  normalizePolicyGroup,
  normalizePolicySet,
  reorderCollection,
  type EditorModeId,
  type EditorNamedItem,
  type PolicyEditorPanelId,
  type RelationEditorPanelId,
} from '@/views/script-editor/editor-policy/editorPolicy';
import { createStepFromTemplate } from '@/views/script-editor/editor-step/editorStepTemplates';
import {
  buildStepPath,
  cloneStepPath,
  createSiblingSelection,
  getBranchSteps,
  getParentBranchPath,
  getStepByPath,
  isSameBranchPath,
  ROOT_BRANCH_PATH,
  type StepBranchPath,
  type StepPath,
  updateBranchSteps,
  updateStepByPath,
} from '@/views/script-editor/editor-step/editorStepTree';
import {
  buildUiData,
  cloneJson,
  createUiField,
  createUiSchema,
  parseUiSchema,
  stableStringify,
  type EditorPanelId,
  type EditorUiSchema,
  type RawEditorSection,
  type UiFieldControl,
} from '@/views/script-editor/editorSchema';
import { parseTaskCycleValue } from '@/views/script-editor/editorTaskMeta';
import { createSearchRule } from '@/views/script-editor/editorSearchRule';
import {
  buildInputJson,
  createInputEntry,
  listAllVariableOptions,
  listVariableOptions,
  parseInputEntries,
  syncInputVariableCatalog,
  type EditorInputType,
  type EditorInputEntry,
  type EditorVariableOption,
} from '@/views/script-editor/editorVariables';

const route = useRoute();
const router = useRouter();
const scriptStore = useScriptStore();
const deviceStore = useDeviceStore();
const settingsStore = useSettingsStore();

const isLoading = ref(true);
const isSaving = ref(false);
const loadError = ref<string | null>(null);
const saveTime = ref<string | null>(null);

const infoDialogOpen = ref(false);
const rawDialogOpen = ref(false);
const rawDialogSection = ref<RawEditorSection>('steps');
const rawDialogText = ref('');
const rawDialogError = ref<string | null>(null);

const activeMode = ref<EditorModeId>('task');
const activePanel = ref<EditorPanelId>('basic');
const activePolicyPanel = ref<PolicyEditorPanelId>('basic');
const activePolicyGroupPanel = ref<RelationEditorPanelId>('basic');
const activePolicySetPanel = ref<RelationEditorPanelId>('basic');
const selectedTaskId = ref<string | null>(null);
const selectedInputId = ref<string | null>(null);
const selectedStepPath = ref<StepPath | null>(null);
const activeBranchPath = ref<StepBranchPath>(ROOT_BRANCH_PATH);
const selectedUiFieldId = ref<string | null>(null);
const selectedPolicyId = ref<string | null>(null);
const selectedPolicyGroupId = ref<string | null>(null);
const selectedPolicySetId = ref<string | null>(null);
const selectedPolicyStepPathBefore = ref<StepPath | null>(null);
const activePolicyBranchPathBefore = ref<StepBranchPath>(ROOT_BRANCH_PATH);
const selectedPolicyStepPathAfter = ref<StepPath | null>(null);
const activePolicyBranchPathAfter = ref<StepBranchPath>(ROOT_BRANCH_PATH);

const draftTasks = ref<ScriptTaskTable[]>([]);
const draftScript = ref<ScriptTableRecord | null>(null);
const draftPolicies = ref<PolicyTable[]>([]);
const draftPolicyGroups = ref<PolicyGroupTable[]>([]);
const draftPolicySets = ref<PolicySetTable[]>([]);
const groupPolicyIdsByGroupId = ref<Record<string, string[]>>(createEmptyRelationMap<string>());
const setGroupIdsBySetId = ref<Record<string, string[]>>(createEmptyRelationMap<string>());
const sourceTasksSnapshot = ref('');
const sourceScriptSnapshot = ref('');
const sourcePoliciesSnapshot = ref('');
const sourcePolicyGroupsSnapshot = ref('');
const sourcePolicySetsSnapshot = ref('');
const sourceGroupPoliciesSnapshot = ref('');
const sourceSetGroupsSnapshot = ref('');
const consoleLines = ref<string[]>([]);
const selectedPreviewDeviceId = ref<string | null>(null);
const deviceEditorOpen = ref(false);
const editingDeviceId = ref<string | null>(null);
const textDetLabelOptions = ref<Array<{ label: string; value: number; description?: string }>>([]);
const textDetLabelHint = ref<string | null>('请先在脚本信息里设置文字检测模型的标签文件。');
const textDetLabelLoading = ref(false);

const MAX_CONSOLE_LINES = 300;

const taskName = ref('');
const taskRowType = ref<TaskRowType>('task');
const taskTriggerMode = ref<TaskTriggerMode>('rootOnly');
const taskHidden = ref(false);
const recordSchedule = ref(true);
const sectionId = ref<string | null>(null);
const indentLevel = ref(1);
const defaultTaskCycle = ref<TaskCycle>('everyRun');
const showEnabledToggle = ref(true);
const defaultEnabled = ref(true);
const taskTone = ref<TaskTone>('normal');
const inputEntries = ref<EditorInputEntry[]>([]);
const inputError = ref<string | null>(null);
const uiSchema = ref<EditorUiSchema>(createUiSchema());

const hydratingTaskMeta = ref(false);
const hydratingTaskPanels = ref(false);

const scriptId = computed(() => (typeof route.query.scriptId === 'string' ? route.query.scriptId : ''));

const appendConsoleLine = (message: string) => {
  const stamp = new Date().toLocaleTimeString('zh-CN', {
    hour12: false,
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
  consoleLines.value = [...consoleLines.value, `[${stamp}] ${message}`].slice(-MAX_CONSOLE_LINES);
};

const clearConsole = () => {
  consoleLines.value = [];
};

const currentTask = computed<ScriptTaskTable | null>(() => {
  const tasks = draftTasks.value as ScriptTaskTable[];
  const selected = selectedTaskId.value;
  if (!selected) {
    return tasks[0] ?? null;
  }

  const matched = tasks.find((task) => task.id === selected) ?? null;
  return matched ?? tasks[0] ?? null;
});

const currentPolicy = computed<PolicyTable | null>(() => {
  const selected = selectedPolicyId.value;
  if (!selected) {
    return draftPolicies.value[0] ?? null;
  }
  return draftPolicies.value.find((item) => item.id === selected) ?? draftPolicies.value[0] ?? null;
});

const currentPolicyGroup = computed<PolicyGroupTable | null>(() => {
  const selected = selectedPolicyGroupId.value;
  if (!selected) {
    return draftPolicyGroups.value[0] ?? null;
  }
  return draftPolicyGroups.value.find((item) => item.id === selected) ?? draftPolicyGroups.value[0] ?? null;
});

const currentPolicySet = computed<PolicySetTable | null>(() => {
  const selected = selectedPolicySetId.value;
  if (!selected) {
    return draftPolicySets.value[0] ?? null;
  }
  return draftPolicySets.value.find((item) => item.id === selected) ?? draftPolicySets.value[0] ?? null;
});

const editingDevice = computed(() => deviceStore.devices.find((device) => device.id === editingDeviceId.value) ?? null);
const selectedPreviewDevice = computed(() => deviceStore.devices.find((device) => device.id === selectedPreviewDeviceId.value) ?? null);
const selectedPreviewDeviceRuntimeError = computed(() =>
  selectedPreviewDevice.value ? validateDeviceRuntimePlatform(selectedPreviewDevice.value) : null,
);
const deviceSelectOptions = computed(() =>
  deviceStore.devices.map((device) => ({
    label: device.data.deviceName,
    value: device.id,
    description: `${formatConnectLabel(device.data.adbConnect)} · ${formatCaptureMethod(device.data.capMethod)}`,
  })),
);

const extractYoloDetector = (model: DetectorType | null | undefined): YoloDet | null => {
  if (!model) {
    return null;
  }
  if ('Yolo11' in model) {
    return model.Yolo11;
  }
  if ('Yolo26' in model) {
    return model.Yolo26;
  }
  return null;
};

const textDetLabelPath = computed(() => extractYoloDetector(draftScript.value?.data.txtDetModel)?.labelPath?.trim() || '');
const textDetLabelSelectPlaceholder = computed(() => {
  if (textDetLabelLoading.value) {
    return '正在加载标签...';
  }
  if (textDetLabelOptions.value.length) {
    return '选择标签';
  }
  return '请先设置文字检测模型标签文件';
});

const activeModeLabel = computed(() => {
  switch (activeMode.value) {
    case 'policy':
      return '策略';
    case 'policyGroup':
      return '策略组';
    case 'policySet':
      return '策略集';
    default:
      return '任务';
  }
});

const activeModeFocusName = computed(() => {
  switch (activeMode.value) {
    case 'policy':
      return currentPolicy.value?.data.name || null;
    case 'policyGroup':
      return currentPolicyGroup.value?.data.name || null;
    case 'policySet':
      return currentPolicySet.value?.data.name || null;
    default:
      return currentTask.value?.name || null;
  }
});

const activeTargetSelectPlaceholder = computed(() => `选择${activeModeLabel.value}`);

const activeTargetSelectOptions = computed(() => {
  if (activeMode.value === 'policy') {
    return policyItems.value.map((item) => ({ label: item.title, value: item.id, description: item.subtitle }));
  }
  if (activeMode.value === 'policyGroup') {
    return policyGroupItems.value.map((item) => ({ label: item.title, value: item.id, description: item.subtitle }));
  }
  if (activeMode.value === 'policySet') {
    return policySetItems.value.map((item) => ({ label: item.title, value: item.id, description: item.subtitle }));
  }
  return draftTasks.value.map((task) => ({
    label: task.name,
    value: task.id,
    description: `${task.rowType === 'title' ? '标题行' : '任务行'} · ${task.index + 1}`,
  }));
});

const scriptRecoveryTaskOptions = computed(() =>
  draftTasks.value
    .filter((task) => task.rowType === 'task' && !task.isDeleted)
    .map((task) => ({
      label: task.name,
      value: task.id,
      description: `任务 ${task.index + 1}`,
    })),
);

const activeTargetValue = computed<string | null>({
  get: () => {
    if (activeMode.value === 'policy') return selectedPolicyId.value;
    if (activeMode.value === 'policyGroup') return selectedPolicyGroupId.value;
    if (activeMode.value === 'policySet') return selectedPolicySetId.value;
    return selectedTaskId.value;
  },
  set: (value) => {
    if (activeMode.value === 'policy') {
      selectedPolicyId.value = value;
      return;
    }
    if (activeMode.value === 'policyGroup') {
      selectedPolicyGroupId.value = value;
      return;
    }
    if (activeMode.value === 'policySet') {
      selectedPolicySetId.value = value;
      return;
    }
    selectedTaskId.value = value;
  },
});

const runSelectionDisabledReason = computed(() => {
  if (!selectedPreviewDeviceId.value || !activeTargetValue.value) {
    return '请先选择设备和目标对象。';
  }

  return selectedPreviewDeviceRuntimeError.value;
});

const canRunSelection = computed(() => !runSelectionDisabledReason.value);


const variableOptions = computed(() =>
  listVariableOptions(draftScript.value?.data.variableCatalog, currentTask.value?.id ?? null, parsedSteps.value),
);
const catalogVariableOptions = computed(() =>
  listVariableOptions(draftScript.value?.data.variableCatalog, currentTask.value?.id ?? null, parsedSteps.value, 'read', false),
);
const titleTaskOptions = computed(() => [
  {
    label: '未分组',
    value: null,
    description: '直接显示在顶层，不归属到任何标题行。',
  },
  ...draftTasks.value
    .filter((task) => task.rowType === 'title' && task.id !== currentTask.value?.id)
    .map((task) => ({
      label: task.name || '未命名标题',
      value: task.id,
      description: `标题行 · ${task.index + 1}`,
    })),
]);
const defaultTaskCycleValue = computed(() => {
  if (typeof defaultTaskCycle.value === 'string') {
    return defaultTaskCycle.value;
  }
  return 'weekDay' in defaultTaskCycle.value ? 'weekDay' : 'monthDay';
});
const defaultTaskCycleMode = computed<'named' | 'weekDay' | 'monthDay'>(() => {
  if (typeof defaultTaskCycle.value === 'string') {
    return 'named';
  }
  return 'weekDay' in defaultTaskCycle.value ? 'weekDay' : 'monthDay';
});
const defaultTaskCycleDay = computed(() => {
  if (typeof defaultTaskCycle.value === 'string') {
    return 1;
  }
  return 'weekDay' in defaultTaskCycle.value ? defaultTaskCycle.value.weekDay : defaultTaskCycle.value.monthDay;
});
const currentPolicyStepTarget = computed<'before' | 'after'>(() => (activePolicyPanel.value === 'before' ? 'before' : 'after'));
const currentPolicySteps = computed<Step[]>(() => {
  if (!currentPolicy.value) {
    return [];
  }
  return currentPolicyStepTarget.value === 'before' ? currentPolicy.value.data.beforeAction : currentPolicy.value.data.afterAction;
});
const selectedPolicyStepPath = computed<StepPath | null>(() =>
  currentPolicyStepTarget.value === 'before' ? selectedPolicyStepPathBefore.value : selectedPolicyStepPathAfter.value,
);
const activePolicyBranchPath = computed<StepBranchPath>(() =>
  currentPolicyStepTarget.value === 'before' ? activePolicyBranchPathBefore.value : activePolicyBranchPathAfter.value,
);
const policyVariableOptions = computed(() =>
  listAllVariableOptions(draftScript.value?.data.variableCatalog, currentPolicySteps.value),
);
const policyCatalogVariableOptions = computed(() =>
  listAllVariableOptions(draftScript.value?.data.variableCatalog, currentPolicySteps.value, 'read', false),
);

const parsedSteps = computed<Step[]>(() => (currentTask.value?.data.steps as Step[] | undefined) ?? []);
const hasValidationErrors = computed(() => Boolean(inputError.value));
const policyItems = computed<EditorNamedItem[]>(() =>
  draftPolicies.value.map((policy) => ({
    id: policy.id,
    title: policy.data.name,
    subtitle: `${policy.data.afterAction.length} 个命中步骤 · ${policy.data.beforeAction.length} 个全局步骤`,
    badge: String(policy.orderIndex + 1),
  })),
);
const policyGroupItems = computed<EditorNamedItem[]>(() =>
  draftPolicyGroups.value.map((group) => ({
    id: group.id,
    title: group.data.name,
    subtitle: `${(groupPolicyIdsByGroupId.value[group.id] ?? []).length} 个策略`,
    badge: String(group.orderIndex + 1),
  })),
);
const policySetItems = computed<EditorNamedItem[]>(() =>
  draftPolicySets.value.map((set) => ({
    id: set.id,
    title: set.data.name,
    subtitle: `${(setGroupIdsBySetId.value[set.id] ?? []).length} 个策略组`,
    badge: String(set.orderIndex + 1),
  })),
);
const describeTaskReferenceTriggerMode = (mode: TaskTriggerMode) => {
  switch (mode) {
    case 'linkOnly':
      return '仅跳转';
    case 'rootAndLink':
      return '循环 + 跳转';
    default:
      return '仅循环';
  }
};
const taskReferenceOptions = computed<EditorReferenceOption[]>(() =>
  draftTasks.value
    .filter((task) => task.rowType === 'task')
    .map((task) => ({
      label: task.name,
      value: task.id,
      description: `${describeTaskReferenceTriggerMode(task.triggerMode)} · ${task.defaultEnabled ? '默认启用' : '默认关闭'}`,
    })),
);
const policyReferenceOptions = computed<EditorReferenceOption[]>(() =>
  draftPolicies.value.map((policy) => ({
    label: policy.data.name,
    value: policy.id,
    description: `${policy.data.afterAction.length} 个命中步骤 · ${policy.data.beforeAction.length} 个全局步骤`,
  })),
);
const policyGroupReferenceOptions = computed<EditorReferenceOption[]>(() =>
  draftPolicyGroups.value.map((group) => ({
    label: group.data.name,
    value: group.id,
    description: `${(groupPolicyIdsByGroupId.value[group.id] ?? []).length} 个策略`,
  })),
);
const policySetReferenceOptions = computed<EditorReferenceOption[]>(() =>
  draftPolicySets.value.map((set) => ({
    label: set.data.name,
    value: set.id,
    description: `${(setGroupIdsBySetId.value[set.id] ?? []).length} 个策略组`,
  })),
);
const assignedPolicies = computed<EditorNamedItem[]>(() => {
  const assignedIds = currentPolicyGroup.value ? groupPolicyIdsByGroupId.value[currentPolicyGroup.value.id] ?? [] : [];
  return assignedIds
    .map((id) => draftPolicies.value.find((item) => item.id === id))
    .filter((item): item is PolicyTable => Boolean(item))
    .map((item) => ({
      id: item.id,
      title: item.data.name,
      subtitle: item.data.note || '未填写备注',
    }));
});
const unassignedPolicies = computed<EditorNamedItem[]>(() => {
  const assigned = new Set(currentPolicyGroup.value ? groupPolicyIdsByGroupId.value[currentPolicyGroup.value.id] ?? [] : []);
  return draftPolicies.value
    .filter((item) => !assigned.has(item.id))
    .map((item) => ({
      id: item.id,
      title: item.data.name,
      subtitle: item.data.note || '未填写备注',
    }));
});
const assignedGroups = computed<EditorNamedItem[]>(() => {
  const assignedIds = currentPolicySet.value ? setGroupIdsBySetId.value[currentPolicySet.value.id] ?? [] : [];
  return assignedIds
    .map((id) => draftPolicyGroups.value.find((item) => item.id === id))
    .filter((item): item is PolicyGroupTable => Boolean(item))
    .map((item) => ({
      id: item.id,
      title: item.data.name,
      subtitle: item.data.note || '未填写备注',
    }));
});
const unassignedGroups = computed<EditorNamedItem[]>(() => {
  const assigned = new Set(currentPolicySet.value ? setGroupIdsBySetId.value[currentPolicySet.value.id] ?? [] : []);
  return draftPolicyGroups.value
    .filter((item) => !assigned.has(item.id))
    .map((item) => ({
      id: item.id,
      title: item.data.name,
      subtitle: item.data.note || '未填写备注',
    }));
});

const dirty = computed(() => {
  if (!draftScript.value) {
    return false;
  }

  return (
    stableStringify(draftScript.value) !== sourceScriptSnapshot.value ||
    stableStringify(draftTasks.value) !== sourceTasksSnapshot.value ||
    stableStringify(draftPolicies.value) !== sourcePoliciesSnapshot.value ||
    stableStringify(draftPolicyGroups.value) !== sourcePolicyGroupsSnapshot.value ||
    stableStringify(draftPolicySets.value) !== sourcePolicySetsSnapshot.value ||
    stableStringify(groupPolicyIdsByGroupId.value) !== sourceGroupPoliciesSnapshot.value ||
    stableStringify(setGroupIdsBySetId.value) !== sourceSetGroupsSnapshot.value
  );
});

const formattedSaveTime = computed(() => {
  if (!saveTime.value) {
    return '';
  }

  return new Date(saveTime.value).toLocaleString('zh-TW', {
    hour12: false,
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
});

const rawDialogTitle = computed(() => {
  switch (rawDialogSection.value) {
    case 'inputs':
      return '输入变量 JSON';
    case 'ui':
      return 'UI Schema JSON';
    default:
      return '步骤 JSON';
  }
});

const rawDialogDescription = computed(() => {
  switch (rawDialogSection.value) {
    case 'inputs':
      return '这里是 input.* 的底层结构，作为调试入口保留。';
    case 'ui':
      return '这里是 UI schema 的底层结构，优先在可视化面板里编辑。';
    default:
      return '这里是任务步骤的底层结构，优先在可视化工作区里查看和调整。';
  }
});

const normalizeTask = (task: ScriptTaskTable, index: number): ScriptTaskTable => {
  const legacyTaskType = (task as ScriptTaskTable & { taskType?: 'main' | 'child' }).taskType;
  const rowType = task.rowType ?? 'task';
  const isTitle = rowType === 'title';
  return {
    ...task,
    scriptId: task.scriptId || scriptId.value,
    name: task.name || `任务 ${index + 1}`,
    rowType,
    triggerMode: task.triggerMode ?? (legacyTaskType === 'child' ? 'linkOnly' : 'rootOnly'),
    recordSchedule: isTitle ? false : task.recordSchedule ?? true,
    sectionId: isTitle ? null : task.sectionId ?? null,
    indentLevel: isTitle ? 0 : Math.max(0, Math.min(8, Number(task.indentLevel ?? 1))),
    defaultTaskCycle: task.defaultTaskCycle ?? 'everyRun',
    showEnabledToggle: isTitle ? false : task.showEnabledToggle ?? true,
    defaultEnabled: task.defaultEnabled ?? true,
    taskTone: isTitle ? 'normal' : task.taskTone ?? 'normal',
    isHidden: Boolean(task.isHidden),
    index,
    createdAt: task.createdAt || new Date().toISOString(),
    updatedAt: task.updatedAt || new Date().toISOString(),
    deletedAt: task.deletedAt ?? null,
    isDeleted: Boolean(task.isDeleted),
    data: {
      uiData: task.data?.uiData ?? {},
      variables: task.data?.variables ?? {},
      steps: Array.isArray(task.data?.steps) ? task.data.steps : [],
    },
  };
};

const buildTaskDraft = async (name?: string): Promise<ScriptTaskTable> => {
  const index = draftTasks.value.length;
  return normalizeTask(
    {
      id: await taskService.requestUuid(),
      scriptId: scriptId.value,
      name: name || `新任务 ${index + 1}`,
      rowType: 'task',
      triggerMode: 'rootOnly',
      recordSchedule: true,
      sectionId: draftTasks.value.filter((task) => task.rowType === 'title').at(-1)?.id ?? null,
      indentLevel: 1,
      defaultTaskCycle: 'everyRun',
      showEnabledToggle: true,
      defaultEnabled: true,
      taskTone: 'normal',
      isHidden: false,
      data: {
        uiData: {},
        variables: {},
        steps: [],
      },
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
      deletedAt: null,
      isDeleted: false,
      index,
    },
    index,
  );
};

const replaceTask = (taskId: string, updater: (task: ScriptTaskTable) => ScriptTaskTable) => {
  draftTasks.value = draftTasks.value.map((task, index) => {
    if (task.id !== taskId) {
      return normalizeTask(task, index);
    }

    return normalizeTask(updater(cloneJson(task)), index);
  });
};

const hydrateTaskEditors = () => {
  hydratingTaskMeta.value = true;
  hydratingTaskPanels.value = true;

  if (!currentTask.value) {
    taskName.value = '';
    taskRowType.value = 'task';
    taskTriggerMode.value = 'rootOnly';
    taskHidden.value = false;
    recordSchedule.value = true;
    sectionId.value = null;
    indentLevel.value = 1;
    defaultTaskCycle.value = 'everyRun';
    showEnabledToggle.value = true;
    defaultEnabled.value = true;
    taskTone.value = 'normal';
    inputEntries.value = [];
    inputError.value = null;
    selectedInputId.value = null;
    uiSchema.value = createUiSchema();
    selectedStepPath.value = null;
    activeBranchPath.value = ROOT_BRANCH_PATH;
    selectedUiFieldId.value = null;
  } else {
    taskName.value = currentTask.value.name;
    taskRowType.value = currentTask.value.rowType;
    taskTriggerMode.value = currentTask.value.triggerMode;
    taskHidden.value = currentTask.value.isHidden;
    recordSchedule.value = currentTask.value.recordSchedule;
    sectionId.value = currentTask.value.sectionId;
    indentLevel.value = currentTask.value.indentLevel;
    defaultTaskCycle.value = currentTask.value.defaultTaskCycle;
    showEnabledToggle.value = currentTask.value.showEnabledToggle;
    defaultEnabled.value = currentTask.value.defaultEnabled;
    taskTone.value = currentTask.value.taskTone;
    inputEntries.value = parseInputEntries(draftScript.value?.data.variableCatalog, currentTask.value.id, currentTask.value.data.variables ?? {});
    inputError.value = null;
    selectedInputId.value = inputEntries.value.find((entry) => entry.id === selectedInputId.value)?.id ?? inputEntries.value[0]?.id ?? null;
    uiSchema.value = parseUiSchema(currentTask.value.data.uiData ?? {});
    if (currentTask.value.rowType === 'title') {
      activePanel.value = 'basic';
    }
    if (!currentTask.value.data.steps.length) {
      selectedStepPath.value = null;
      activeBranchPath.value = ROOT_BRANCH_PATH;
    } else if (!selectedStepPath.value || !getStepByPath(currentTask.value.data.steps, selectedStepPath.value)) {
      selectedStepPath.value = buildStepPath(ROOT_BRANCH_PATH, 0);
      activeBranchPath.value = ROOT_BRANCH_PATH;
    }
    selectedUiFieldId.value =
      uiSchema.value.fields.find((field) => field.id === selectedUiFieldId.value)?.id ?? uiSchema.value.fields[0]?.id ?? null;
  }

  queueMicrotask(() => {
    hydratingTaskMeta.value = false;
    hydratingTaskPanels.value = false;
  });
};

const setCurrentTaskSteps = (steps: Step[]) => {
  if (!currentTask.value) {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.data.steps = steps;
    return task;
  });

  if (!steps.length) {
    selectedStepPath.value = null;
    activeBranchPath.value = ROOT_BRANCH_PATH;
  }
};

const replacePolicy = (policyId: string, updater: (policy: PolicyTable) => PolicyTable) => {
  draftPolicies.value = draftPolicies.value.map((policy, index) => {
    if (policy.id !== policyId) {
      return normalizePolicy(policy, index);
    }
    return normalizePolicy(updater(cloneJson(policy)), index);
  });
};

const replacePolicyGroup = (groupId: string, updater: (group: PolicyGroupTable) => PolicyGroupTable) => {
  draftPolicyGroups.value = draftPolicyGroups.value.map((group, index) => {
    if (group.id !== groupId) {
      return normalizePolicyGroup(group, index);
    }
    return normalizePolicyGroup(updater(cloneJson(group)), index);
  });
};

const replacePolicySet = (setId: string, updater: (set: PolicySetTable) => PolicySetTable) => {
  draftPolicySets.value = draftPolicySets.value.map((item, index) => {
    if (item.id !== setId) {
      return normalizePolicySet(item, index);
    }
    return normalizePolicySet(updater(cloneJson(item)), index);
  });
};

const buildPolicyDraft = async (name?: string): Promise<PolicyTable> =>
  normalizePolicy(
    {
      id: await taskService.requestUuid(),
      scriptId: scriptId.value,
      orderIndex: draftPolicies.value.length,
      data: {
        name: name || `策略 ${draftPolicies.value.length + 1}`,
        note: '',
        logPrint: null,
        curPos: 0,
        skipFlag: false,
        execCur: 0,
        execMax: 1,
        beforeAction: [],
        cond: createSearchRule('group'),
        afterAction: [],
      },
    },
    draftPolicies.value.length,
  );

const buildPolicyGroupDraft = async (name?: string): Promise<PolicyGroupTable> =>
  normalizePolicyGroup(
    {
      id: await taskService.requestUuid(),
      scriptId: scriptId.value,
      orderIndex: draftPolicyGroups.value.length,
      data: {
        name: name || `策略组 ${draftPolicyGroups.value.length + 1}`,
        note: '',
      },
    },
    draftPolicyGroups.value.length,
  );

const buildPolicySetDraft = async (name?: string): Promise<PolicySetTable> =>
  normalizePolicySet(
    {
      id: await taskService.requestUuid(),
      scriptId: scriptId.value,
      orderIndex: draftPolicySets.value.length,
      data: {
        name: name || `策略集 ${draftPolicySets.value.length + 1}`,
        note: '',
      },
    },
    draftPolicySets.value.length,
  );

const setCurrentPolicySteps = (steps: Step[]) => {
  if (!currentPolicy.value) {
    return;
  }

  replacePolicy(currentPolicy.value.id, (policy) => {
    if (currentPolicyStepTarget.value === 'before') {
      policy.data.beforeAction = steps;
    } else {
      policy.data.afterAction = steps;
    }
    return policy;
  });

  if (!steps.length) {
    if (currentPolicyStepTarget.value === 'before') {
      selectedPolicyStepPathBefore.value = null;
      activePolicyBranchPathBefore.value = ROOT_BRANCH_PATH;
    } else {
      selectedPolicyStepPathAfter.value = null;
      activePolicyBranchPathAfter.value = ROOT_BRANCH_PATH;
    }
  }
};

const createTask = async () => {
  const nextTask = await buildTaskDraft();
  draftTasks.value = [...draftTasks.value, nextTask].map((task, index) => normalizeTask(task, index));
  selectedTaskId.value = nextTask.id;
  activePanel.value = 'basic';
};

const selectTask = (taskId: string) => {
  selectedTaskId.value = taskId;
};

const duplicateTask = async (taskId: string) => {
  const source = draftTasks.value.find((task) => task.id === taskId);
  if (!source) {
    return;
  }

  const duplicate = normalizeTask(
    {
      ...cloneJson(source),
      id: await taskService.requestUuid(),
      name: `${source.name} 副本`,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    },
    draftTasks.value.length,
  );

  draftTasks.value = [...draftTasks.value, duplicate].map((task, index) => normalizeTask(task, index));
  selectedTaskId.value = duplicate.id;
};

const removeTask = (taskId: string) => {
  if (draftTasks.value.length <= 1) {
    showToast('至少保留一个任务，避免脚本变成空壳。', 'warning');
    return;
  }

  if (!window.confirm('确认要删除此任务吗？这将删除该任务下的所有数据')) {
    return;
  }

  draftTasks.value = draftTasks.value
    .filter((task) => task.id !== taskId)
    .map((task, index) => normalizeTask(task, index));

  if (selectedTaskId.value === taskId) {
    selectedTaskId.value = draftTasks.value[0]?.id ?? null;
  }
};

const toggleTaskHidden = (taskId: string) => {
  replaceTask(taskId, (task) => {
    task.isHidden = !task.isHidden;
    return task;
  });
};

const reorderTasks = (draggedTaskId: string, targetTaskId: string) => {
  const fromIndex = draftTasks.value.findIndex((task) => task.id === draggedTaskId);
  const toIndex = draftTasks.value.findIndex((task) => task.id === targetTaskId);
  if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) {
    return;
  }

  draftTasks.value = reorderCollection(draftTasks.value, fromIndex, toIndex).map((task, index) => normalizeTask(task, index));
};

const createPolicy = async () => {
  const nextPolicy = await buildPolicyDraft();
  draftPolicies.value = [...draftPolicies.value, nextPolicy].map((policy, index) => normalizePolicy(policy, index));
  selectedPolicyId.value = nextPolicy.id;
  activeMode.value = 'policy';
  activePolicyPanel.value = 'basic';
};

const createPolicyGroup = async () => {
  const nextGroup = await buildPolicyGroupDraft();
  draftPolicyGroups.value = [...draftPolicyGroups.value, nextGroup].map((group, index) => normalizePolicyGroup(group, index));
  selectedPolicyGroupId.value = nextGroup.id;
  activeMode.value = 'policyGroup';
  activePolicyGroupPanel.value = 'basic';
};

const createPolicySet = async () => {
  const nextSet = await buildPolicySetDraft();
  draftPolicySets.value = [...draftPolicySets.value, nextSet].map((set, index) => normalizePolicySet(set, index));
  selectedPolicySetId.value = nextSet.id;
  activeMode.value = 'policySet';
  activePolicySetPanel.value = 'basic';
};

const createReferenceResource = async (kind: EditorReferenceKind) => {
  if (kind === 'task') {
    const nextTask = await buildTaskDraft();
    draftTasks.value = [...draftTasks.value, nextTask].map((task, index) => normalizeTask(task, index));
    showToast(`已创建引用任务：${nextTask.name}`, 'success');
    return nextTask.id;
  }

  if (kind === 'policy') {
    const nextPolicy = await buildPolicyDraft();
    draftPolicies.value = [...draftPolicies.value, nextPolicy].map((policy, index) => normalizePolicy(policy, index));
    showToast(`已创建引用策略：${nextPolicy.data.name}`, 'success');
    return nextPolicy.id;
  }

  if (kind === 'policyGroup') {
    const nextGroup = await buildPolicyGroupDraft();
    draftPolicyGroups.value = [...draftPolicyGroups.value, nextGroup].map((group, index) => normalizePolicyGroup(group, index));
    showToast(`已创建引用策略组：${nextGroup.data.name}`, 'success');
    return nextGroup.id;
  }

  const nextSet = await buildPolicySetDraft();
  draftPolicySets.value = [...draftPolicySets.value, nextSet].map((set, index) => normalizePolicySet(set, index));
  showToast(`已创建引用策略集：${nextSet.data.name}`, 'success');
  return nextSet.id;
};

const jumpToReferenceResource = (kind: EditorReferenceKind, id: string) => {
  if (kind === 'task') {
    const matched = draftTasks.value.find((task) => task.id === id);
    if (!matched) {
      showToast('目标任务不存在，可能已被删除。', 'warning');
      return;
    }

    selectedTaskId.value = id;
    activeMode.value = 'task';
    activePanel.value = 'basic';
    return;
  }

  if (kind === 'policy') {
    const matched = draftPolicies.value.find((policy) => policy.id === id);
    if (!matched) {
      showToast('目标策略不存在，可能已被删除。', 'warning');
      return;
    }

    selectedPolicyId.value = id;
    activeMode.value = 'policy';
    activePolicyPanel.value = 'basic';
    return;
  }

  if (kind === 'policyGroup') {
    const matched = draftPolicyGroups.value.find((group) => group.id === id);
    if (!matched) {
      showToast('目标策略组不存在，可能已被删除。', 'warning');
      return;
    }

    selectedPolicyGroupId.value = id;
    activeMode.value = 'policyGroup';
    activePolicyGroupPanel.value = 'basic';
    return;
  }

  const matched = draftPolicySets.value.find((set) => set.id === id);
  if (!matched) {
    showToast('目标策略集不存在，可能已被删除。', 'warning');
    return;
  }

  selectedPolicySetId.value = id;
  activeMode.value = 'policySet';
  activePolicySetPanel.value = 'basic';
};

const buildVariableReferenceKey = (namespace: EditorInputEntry['namespace'], key: string) => {
  const trimmed = key.trim();
  if (!trimmed) {
    return '';
  }
  return `${namespace}.${trimmed}`;
};

const collectStepTreeIds = (step: Step, bucket = new Set<string>()) => {
  if (step.id) {
    bucket.add(step.id);
  }

  if (step.op === 'sequence') {
    step.steps.forEach((child) => collectStepTreeIds(child, bucket));
    return bucket;
  }

  if (step.op === 'dataHanding' && step.a.type === 'filter') {
    step.a.then_steps.forEach((child) => collectStepTreeIds(child, bucket));
    return bucket;
  }

  if (step.op === 'vision' && step.a.type === 'visionSearch') {
    step.a.then_steps.forEach((child) => collectStepTreeIds(child, bucket));
    return bucket;
  }

  if (step.op === 'flowControl') {
    if (step.a.type === 'if') {
      step.a.then.forEach((child) => collectStepTreeIds(child, bucket));
      (step.a.else_steps ?? []).forEach((child) => collectStepTreeIds(child, bucket));
      return bucket;
    }

    if (step.a.type === 'while' || step.a.type === 'forEach') {
      step.a.flow.forEach((child) => collectStepTreeIds(child, bucket));
    }
  }

  return bucket;
};

const collectVariableReferencesFromSteps = (steps: Step[], bucket = new Set<string>()) => {
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
    }

    if (step.op === 'dataHanding') {
      if ((step.a.type === 'setVar' || step.a.type === 'getVar') && step.a.name?.trim()) {
        bucket.add(step.a.name.trim());
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
    }

    if (step.op === 'vision' && step.a.type === 'visionSearch') {
      if (step.a.out_var?.trim()) {
        bucket.add(step.a.out_var.trim());
      }
      collectVariableReferencesFromSteps(step.a.then_steps, bucket);
      continue;
    }

    if (step.op === 'flowControl') {
      if (step.a.type === 'handlePolicySet' || step.a.type === 'handlePolicy') {
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

      if (step.a.type === 'while' || step.a.type === 'forEach') {
        if (step.a.type === 'forEach' && step.a.input_var?.trim()) {
          bucket.add(step.a.input_var.trim());
        }
        collectVariableReferencesFromSteps(step.a.flow, bucket);
      }
    }
  }

  return bucket;
};

const collectConditionVariableReferences = (condition: ConditionNode, bucket: Set<string>) => {
  if (condition.type === 'group') {
    condition.items.forEach((item: ConditionNode) => collectConditionVariableReferences(item, bucket));
    return;
  }

  if (condition.type === 'varCompare' && condition.var_name?.trim()) {
    bucket.add(condition.var_name.trim());
    return;
  }

  if (condition.type === 'policySetResult' && condition.result_var?.trim()) {
    bucket.add(condition.result_var.trim());
    return;
  }

  if (condition.type === 'policyCondition' && condition.input_var?.trim()) {
    bucket.add(condition.input_var.trim());
  }
};

type VariableCreateOptions = {
  preferredKey?: string;
  name?: string;
  select?: boolean;
  silent?: boolean;
  sourceStepId?: string | null;
};

const createEditorStepId = () =>
  globalThis.crypto?.randomUUID?.() ?? `step-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 8)}`;

const createUniqueVariableStorageKey = (namespace: EditorInputEntry['namespace'], preferredKey?: string) => {
  const existingKeys = new Set(inputEntries.value.map((entry) => entry.key.trim()).filter(Boolean));
  const trimmedPreferred = preferredKey?.trim().replace(/^(input|runtime|system)\./, '') ?? '';
  const defaultPrefix = namespace === 'runtime' ? 'runtimeVar' : 'newVar';
  const baseSeed = trimmedPreferred || `${defaultPrefix}${inputEntries.value.length + 1}`;

  if (!existingKeys.has(baseSeed)) {
    return baseSeed;
  }

  const matched = baseSeed.match(/^(.*?)(\d+)$/);
  const prefix = matched?.[1] || `${baseSeed}_`;
  let seed = matched ? Number(matched[2]) : 1;
  let nextKey = baseSeed;
  while (existingKeys.has(nextKey)) {
    seed += 1;
    nextKey = `${prefix}${seed}`;
  }
  return nextKey;
};

const createVariableEntry = (
  namespace: EditorInputEntry['namespace'],
  inputType: EditorInputType,
  options: VariableCreateOptions = {},
) => {
  const nextKey = createUniqueVariableStorageKey(namespace, options.preferredKey);
  const nextEntry: EditorInputEntry = {
    ...createInputEntry(inputType),
    key: nextKey,
    name: options.name?.trim() || nextKey,
    namespace,
    type: inputType,
    sourceStepId: options.sourceStepId ?? null,
  };
  inputEntries.value = [...inputEntries.value, nextEntry];
  if (options.select !== false) {
    selectedInputId.value = nextEntry.id;
  }
  if (!options.silent) {
    showToast(`已创建变量：${namespace}.${nextKey}`, 'success');
  }
  return `${namespace}.${nextKey}`;
};

const renameVariableReferencesInSteps = (steps: Step[], previousKey: string, nextKey: string): Step[] =>
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

      return nextStep;
    }

    if (nextStep.op === 'flowControl') {
      if (nextStep.a.type === 'if' || nextStep.a.type === 'while') {
        nextStep.a.con = renameConditionVariableReferences(nextStep.a.con, previousKey, nextKey);
      }

      if (nextStep.a.type === 'handlePolicySet' || nextStep.a.type === 'handlePolicy') {
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
        nextStep.a.else_steps = nextStep.a.else_steps
          ? renameVariableReferencesInSteps(nextStep.a.else_steps, previousKey, nextKey)
          : nextStep.a.else_steps;
        return nextStep;
      }

      if (nextStep.a.type === 'forEach') {
        if (nextStep.a.input_var === previousKey) {
          nextStep.a.input_var = nextKey;
        }
        nextStep.a.flow = renameVariableReferencesInSteps(nextStep.a.flow, previousKey, nextKey);
        return nextStep;
      }

      if (nextStep.a.type === 'while') {
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

const renameConditionVariableReferences = (condition: ConditionNode, previousKey: string, nextKey: string): ConditionNode => {
  const nextCondition = cloneJson(condition) as ConditionNode;

  if (nextCondition.type === 'group') {
    nextCondition.items = nextCondition.items.map((item: ConditionNode) => renameConditionVariableReferences(item, previousKey, nextKey));
    return nextCondition;
  }

  if (nextCondition.type === 'varCompare' && nextCondition.var_name === previousKey) {
    nextCondition.var_name = nextKey;
    return nextCondition;
  }

  if (nextCondition.type === 'policySetResult' && nextCondition.result_var === previousKey) {
    nextCondition.result_var = nextKey;
    return nextCondition;
  }

  if (nextCondition.type === 'policyCondition' && nextCondition.input_var === previousKey) {
    nextCondition.input_var = nextKey;
  }

  return nextCondition;
};

const syncVariableReferenceRename = (previousKey: string, nextKey: string) => {
  if (!previousKey || !nextKey || previousKey === nextKey) {
    return;
  }

  draftTasks.value = draftTasks.value.map((task, index) =>
    normalizeTask(
      {
        ...cloneJson(task),
        data: {
          ...cloneJson(task.data),
          steps: renameVariableReferencesInSteps(task.data.steps as Step[], previousKey, nextKey),
          uiData:
            task.data?.uiData && typeof task.data.uiData === 'object' && !Array.isArray(task.data.uiData)
              ? {
                  ...task.data.uiData,
                  fields: Array.isArray((task.data.uiData as { fields?: unknown[] }).fields)
                    ? ((task.data.uiData as { fields: Array<Record<string, unknown>> }).fields).map((field) => ({
                        ...field,
                        ...(field.inputKey === previousKey.replace(/^input\./, '') ? { inputKey: nextKey.replace(/^input\./, '') } : {}),
                      }))
                    : (task.data.uiData as { fields?: unknown[] }).fields,
                }
              : task.data.uiData,
        },
      },
      index,
    ),
  );

  draftPolicies.value = draftPolicies.value.map((policy, index) =>
    normalizePolicy(
      {
        ...cloneJson(policy),
        data: {
          ...cloneJson(policy.data),
          beforeAction: renameVariableReferencesInSteps(policy.data.beforeAction as Step[], previousKey, nextKey),
          afterAction: renameVariableReferencesInSteps(policy.data.afterAction as Step[], previousKey, nextKey),
        },
      },
      index,
    ),
  );
};

const createVariableResource = async (
  namespace: 'input' | 'runtime' = 'input',
  inputType: EditorInputType = namespace === 'runtime' ? 'json' : 'int',
  options: VariableCreateOptions = {},
) => {
  if (!currentTask.value) {
    showToast('当前没有选中任务，无法创建变量。', 'warning');
    return '';
  }

  return createVariableEntry(namespace, inputType, options);
};

const jumpToVariableResource = (option: EditorVariableOption) => {
  if (option.namespace === 'system') {
    showToast('系统变量由运行时注入，当前不可在编辑器中修改。', 'warning');
    return;
  }

  if (!option.ownerTaskId) {
    showToast('这个变量没有可定位的来源任务。', 'warning');
    return;
  }

  const matchedTask = draftTasks.value.find((task) => task.id === option.ownerTaskId);
  if (!matchedTask) {
    showToast('变量来源任务不存在，可能已被删除。', 'warning');
    return;
  }

  selectedTaskId.value = matchedTask.id;
  activeMode.value = 'task';
  if (option.sourceType === 'manual') {
    activePanel.value = 'inputs';
    selectedInputId.value = option.id;
    return;
  }

  activePanel.value = 'steps';
  showToast('已定位到变量来源任务的步骤工作区。', 'success');
};

const removePolicy = (policyId: string) => {
  draftPolicies.value = draftPolicies.value.filter((item) => item.id !== policyId).map((item, index) => normalizePolicy(item, index));
  groupPolicyIdsByGroupId.value = Object.fromEntries(
    Object.entries(groupPolicyIdsByGroupId.value).map(([groupId, policyIds]) => [groupId, policyIds.filter((id) => id !== policyId)]),
  );
  if (selectedPolicyId.value === policyId) {
    selectedPolicyId.value = draftPolicies.value[0]?.id ?? null;
  }
};

const removePolicyGroup = (groupId: string) => {
  draftPolicyGroups.value = draftPolicyGroups.value.filter((item) => item.id !== groupId).map((item, index) => normalizePolicyGroup(item, index));
  const nextGroupPolicies = { ...groupPolicyIdsByGroupId.value };
  delete nextGroupPolicies[groupId];
  groupPolicyIdsByGroupId.value = nextGroupPolicies;
  setGroupIdsBySetId.value = Object.fromEntries(
    Object.entries(setGroupIdsBySetId.value).map(([setId, groupIds]) => [setId, groupIds.filter((id) => id !== groupId)]),
  );
  if (selectedPolicyGroupId.value === groupId) {
    selectedPolicyGroupId.value = draftPolicyGroups.value[0]?.id ?? null;
  }
};

const removePolicySet = (setId: string) => {
  draftPolicySets.value = draftPolicySets.value.filter((item) => item.id !== setId).map((item, index) => normalizePolicySet(item, index));
  const nextSetGroups = { ...setGroupIdsBySetId.value };
  delete nextSetGroups[setId];
  setGroupIdsBySetId.value = nextSetGroups;
  if (selectedPolicySetId.value === setId) {
    selectedPolicySetId.value = draftPolicySets.value[0]?.id ?? null;
  }
};

const reorderPolicies = (draggedId: string, targetId: string) => {
  const fromIndex = draftPolicies.value.findIndex((item) => item.id === draggedId);
  const toIndex = draftPolicies.value.findIndex((item) => item.id === targetId);
  if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) return;
  draftPolicies.value = reorderCollection(draftPolicies.value, fromIndex, toIndex).map((item, index) => normalizePolicy(item, index));
};

const reorderPolicyGroups = (draggedId: string, targetId: string) => {
  const fromIndex = draftPolicyGroups.value.findIndex((item) => item.id === draggedId);
  const toIndex = draftPolicyGroups.value.findIndex((item) => item.id === targetId);
  if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) return;
  draftPolicyGroups.value = reorderCollection(draftPolicyGroups.value, fromIndex, toIndex).map((item, index) => normalizePolicyGroup(item, index));
};

const reorderPolicySets = (draggedId: string, targetId: string) => {
  const fromIndex = draftPolicySets.value.findIndex((item) => item.id === draggedId);
  const toIndex = draftPolicySets.value.findIndex((item) => item.id === targetId);
  if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) return;
  draftPolicySets.value = reorderCollection(draftPolicySets.value, fromIndex, toIndex).map((item, index) => normalizePolicySet(item, index));
};

const updatePolicyTextField = (field: 'name' | 'note' | 'logPrint', value: string) => {
  if (!currentPolicy.value) return;
  replacePolicy(currentPolicy.value.id, (policy) => {
    if (field === 'logPrint') {
      policy.data.logPrint = value.trim() ? value : null;
    } else if (field === 'note') {
      policy.data.note = value;
    } else {
      policy.data.name = value;
    }
    return policy;
  });
};

const updatePolicyNumberField = (field: 'curPos' | 'execCur' | 'execMax', value: string) => {
  if (!currentPolicy.value) return;
  replacePolicy(currentPolicy.value.id, (policy) => {
    policy.data[field] = Math.max(0, Number(value) || 0);
    return policy;
  });
};

const updatePolicyBooleanField = (field: 'skipFlag', value: boolean) => {
  if (!currentPolicy.value) return;
  replacePolicy(currentPolicy.value.id, (policy) => {
    policy.data[field] = value;
    return policy;
  });
};

const updatePolicyCondition = (value: SearchRule) => {
  if (!currentPolicy.value) return;
  replacePolicy(currentPolicy.value.id, (policy) => {
    policy.data.cond = cloneJson(value);
    return policy;
  });
};

const updateRelationName = (mode: 'policyGroup' | 'policySet', value: string) => {
  if (mode === 'policyGroup' && currentPolicyGroup.value) {
    replacePolicyGroup(currentPolicyGroup.value.id, (group) => {
      group.data.name = value;
      return group;
    });
  }
  if (mode === 'policySet' && currentPolicySet.value) {
    replacePolicySet(currentPolicySet.value.id, (set) => {
      set.data.name = value;
      return set;
    });
  }
};

const updateRelationNote = (mode: 'policyGroup' | 'policySet', value: string) => {
  if (mode === 'policyGroup' && currentPolicyGroup.value) {
    replacePolicyGroup(currentPolicyGroup.value.id, (group) => {
      group.data.note = value;
      return group;
    });
  }
  if (mode === 'policySet' && currentPolicySet.value) {
    replacePolicySet(currentPolicySet.value.id, (set) => {
      set.data.note = value;
      return set;
    });
  }
};

const loadTextDetLabels = async (path: string) => {
  const trimmedPath = path.trim();
  if (!trimmedPath) {
    textDetLabelOptions.value = [];
    textDetLabelHint.value = '当前脚本未设置文字检测模型的标签文件，请先在“编辑脚本信息 > 模型信息 > 文字检测”里配置标签路径。';
    appendConsoleLine('文字检测标签文件未配置。');
    return;
  }

  textDetLabelLoading.value = true;

  try {
    const labels = await scriptService.getYoloLabels(trimmedPath);
    textDetLabelOptions.value = labels.map((item) => ({
      label: `${item.index}: ${item.label}`,
      value: item.index,
      description: `idx ${item.index}`,
    }));
    textDetLabelHint.value = labels.length ? null : '标签文件已读取，但未解析出任何 names 标签。';
    appendConsoleLine(`已加载文字检测标签 ${labels.length} 项：${trimmedPath}`);
  } catch (error) {
    console.error(error);
    textDetLabelOptions.value = [];
    textDetLabelHint.value = error instanceof Error ? `标签文件读取失败：${error.message}` : '标签文件读取失败，请检查路径和格式。';
    appendConsoleLine(`文字检测标签加载失败：${error instanceof Error ? error.message : '未知错误'}`);
  } finally {
    textDetLabelLoading.value = false;
  }
};

const buildAdbConnect = (form: DeviceFormState): ADBConnectConfig | null => {
  const serverConfig = {
    adbPath: settingsStore.preferences.adbPath || null,
    serverConnect: `${settingsStore.preferences.adbServerHost}:${settingsStore.preferences.adbServerPort}`,
  };

  if (form.connectMethod === 'directTcp') {
    return {
      directTcp: form.connectAddress || null,
    };
  }

  if (form.connectMethod === 'serverConnectByIp') {
    return {
      serverConnectByIp: {
        adbConfig: serverConfig,
        clientConnect: form.connectAddress || null,
      },
    };
  }

  return {
    serverConnectByName: {
      adbConfig: serverConfig,
      deviceName: form.connectDeviceName || null,
    },
  };
};

const buildDeviceTable = async (form: DeviceFormState): Promise<DeviceTable> => ({
  id: form.id ?? (await taskService.requestUuid()),
  data: {
    deviceName: form.deviceName,
    platform: form.platform,
    exePath: form.exePath || null,
    exeArgs: form.exeArgs || null,
    cores: form.cores,
    logLevel: form.logLevel,
    logToFile: form.logToFile,
    adbConnect: buildAdbConnect(form),
    capMethod: form.capMethodType === 'adb' ? 'adb' : { window: form.capMethodValue || form.deviceName },
    imageCompression: form.capMethodType === 'adb' ? 'AdbOriginal' : 'WindowOriginal',
    enable: form.enable,
    autoStart: form.autoStart,
    executionPolicy: {
      actionWaitMs: Math.max(0, Number(form.actionWaitMs) || 0),
      progressTimeoutEnabled: form.progressTimeoutEnabled,
      progressTimeoutMs: Math.max(1000, Number(form.progressTimeoutMs) || 30000),
      timeoutAction: form.timeoutAction,
      timeoutNotifyChannels: [...form.timeoutNotifyChannels],
    },
  },
});

const openDeviceEditor = (deviceId: string | null) => {
  editingDeviceId.value = deviceId;
  deviceEditorOpen.value = true;
};

const savePreviewDevice = async (form: DeviceFormState) => {
  try {
    const device = await buildDeviceTable(form);
    await deviceStore.saveDevice(device);
    deviceEditorOpen.value = false;
    selectedPreviewDeviceId.value = device.id;
    appendConsoleLine(`设备已保存：${device.data.deviceName}`);
    showToast('设备已保存', 'success');
  } catch (error) {
    appendConsoleLine(`设备保存失败：${error instanceof Error ? error.message : '未知错误'}`);
    showToast(error instanceof Error ? error.message : '设备保存失败', 'error');
  }
};

const buildActiveRunTarget = (): RunTarget | null => {
  if (!scriptId.value || !activeTargetValue.value) {
    return null;
  }

  if (activeMode.value === 'policyGroup') {
    return {
      type: 'policyGroup',
      scriptId: scriptId.value,
      policyGroupId: activeTargetValue.value,
    };
  }

  if (activeMode.value === 'policySet') {
    return {
      type: 'policySet',
      scriptId: scriptId.value,
      policySetId: activeTargetValue.value,
    };
  }

  if (activeMode.value === 'task') {
    return {
      type: 'task',
      scriptId: scriptId.value,
      taskId: activeTargetValue.value,
    };
  }

  return null;
};

const handleRunSelection = async () => {
  if (!selectedPreviewDevice.value || !selectedPreviewDeviceId.value || !activeTargetValue.value) {
    showToast('请先选择设备和目标对象。', 'warning');
    return;
  }

  if (selectedPreviewDeviceRuntimeError.value) {
    appendConsoleLine(`运行前校验失败：${selectedPreviewDeviceRuntimeError.value}`);
    showToast(selectedPreviewDeviceRuntimeError.value, 'warning');
    return;
  }

  if (activeMode.value === 'policy') {
    showToast('策略单项运行尚未接入，当前仅支持任务、策略组、策略集。', 'warning');
    return;
  }

  if (activeMode.value === 'policyGroup' || activeMode.value === 'policySet') {
    const message = activeMode.value === 'policyGroup'
      ? '策略组运行目标的执行计划尚未接入，当前版本仅支持任务与整脚本运行。'
      : '策略集运行目标的执行计划尚未接入，当前版本仅支持任务与整脚本运行。';
    appendConsoleLine(`运行前校验失败：${message}`);
    showToast(message, 'warning');
    return;
  }

  const runTarget = buildActiveRunTarget();
  if (!runTarget) {
    showToast('当前目标对象无法转换为运行目标。', 'error');
    return;
  }

  if (draftScript.value) {
    const recoveryError = validateRunTargetRecoveryForDevice(
      selectedPreviewDevice.value,
      draftScript.value,
      draftTasks.value,
    );
    if (recoveryError) {
      appendConsoleLine(`运行前校验失败：${recoveryError}`);
      showToast(recoveryError, 'warning');
      return;
    }
  }

  if (dirty.value) {
    appendConsoleLine('运行前检测到未保存改动，先保存当前脚本结构。');
    await saveEditor();
    if (dirty.value) {
      appendConsoleLine('运行已取消：脚本草稿仍未保存。');
      return;
    }
  }

  appendConsoleLine(`请求运行：设备=${selectedPreviewDevice.value.data.deviceName}，目标=${activeModeLabel.value} ${activeModeFocusName.value || activeTargetValue.value}`);

  try {
    const result = await runtimeService.runScriptTarget(selectedPreviewDeviceId.value, runTarget);
    await deviceStore.refreshRunningDevices();
    appendConsoleLine(result);
    showToast('运行命令已发送', 'success');
  } catch (error) {
    const message = error instanceof Error ? error.message : '运行命令发送失败';
    appendConsoleLine(`运行失败：${message}`);
    showToast(message, 'error');
  }
};

const createActiveItem = () => {
  if (activeMode.value === 'policy') {
    void createPolicy();
    return;
  }
  if (activeMode.value === 'policyGroup') {
    void createPolicyGroup();
    return;
  }
  if (activeMode.value === 'policySet') {
    void createPolicySet();
    return;
  }
  void createTask();
};

const duplicateActiveItem = () => {
  if (!currentTask.value) {
    return;
  }
  duplicateTask(currentTask.value.id);
};

const removeActiveItem = () => {
  if (activeMode.value === 'policy' && currentPolicy.value) {
    removePolicy(currentPolicy.value.id);
    return;
  }
  if (activeMode.value === 'policyGroup' && currentPolicyGroup.value) {
    removePolicyGroup(currentPolicyGroup.value.id);
    return;
  }
  if (activeMode.value === 'policySet' && currentPolicySet.value) {
    removePolicySet(currentPolicySet.value.id);
    return;
  }
  if (currentTask.value) {
    removeTask(currentTask.value.id);
  }
};

const moveActiveItem = (direction: -1 | 1) => {
  if (activeMode.value === 'policy' && currentPolicy.value) {
    const fromIndex = draftPolicies.value.findIndex((item) => item.id === currentPolicy.value?.id);
    const toIndex = fromIndex + direction;
    if (fromIndex >= 0 && toIndex >= 0 && toIndex < draftPolicies.value.length) {
      reorderPolicies(draftPolicies.value[fromIndex].id, draftPolicies.value[toIndex].id);
    }
    return;
  }
  if (activeMode.value === 'policyGroup' && currentPolicyGroup.value) {
    const fromIndex = draftPolicyGroups.value.findIndex((item) => item.id === currentPolicyGroup.value?.id);
    const toIndex = fromIndex + direction;
    if (fromIndex >= 0 && toIndex >= 0 && toIndex < draftPolicyGroups.value.length) {
      reorderPolicyGroups(draftPolicyGroups.value[fromIndex].id, draftPolicyGroups.value[toIndex].id);
    }
    return;
  }
  if (activeMode.value === 'policySet' && currentPolicySet.value) {
    const fromIndex = draftPolicySets.value.findIndex((item) => item.id === currentPolicySet.value?.id);
    const toIndex = fromIndex + direction;
    if (fromIndex >= 0 && toIndex >= 0 && toIndex < draftPolicySets.value.length) {
      reorderPolicySets(draftPolicySets.value[fromIndex].id, draftPolicySets.value[toIndex].id);
    }
    return;
  }
  if (currentTask.value) {
    const fromIndex = draftTasks.value.findIndex((item) => item.id === currentTask.value?.id);
    const toIndex = fromIndex + direction;
    if (fromIndex >= 0 && toIndex >= 0 && toIndex < draftTasks.value.length) {
      reorderTasks(draftTasks.value[fromIndex].id, draftTasks.value[toIndex].id);
    }
  }
};

const linkPolicyToGroup = (policyId: string) => {
  if (!currentPolicyGroup.value) return;
  const groupId = currentPolicyGroup.value.id;
  const assigned = groupPolicyIdsByGroupId.value[groupId] ?? [];
  if (assigned.includes(policyId)) return;
  groupPolicyIdsByGroupId.value = {
    ...groupPolicyIdsByGroupId.value,
    [groupId]: [...assigned, policyId],
  };
};

const unlinkPolicyFromGroup = (policyId: string) => {
  if (!currentPolicyGroup.value) return;
  const groupId = currentPolicyGroup.value.id;
  groupPolicyIdsByGroupId.value = {
    ...groupPolicyIdsByGroupId.value,
    [groupId]: (groupPolicyIdsByGroupId.value[groupId] ?? []).filter((id) => id !== policyId),
  };
};

const reorderGroupPolicies = (draggedId: string, targetId: string) => {
  if (!currentPolicyGroup.value) return;
  const groupId = currentPolicyGroup.value.id;
  const currentIds = groupPolicyIdsByGroupId.value[groupId] ?? [];
  const fromIndex = currentIds.indexOf(draggedId);
  const toIndex = currentIds.indexOf(targetId);
  if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) return;
  groupPolicyIdsByGroupId.value = {
    ...groupPolicyIdsByGroupId.value,
    [groupId]: reorderCollection(currentIds, fromIndex, toIndex),
  };
};

const reverseGroupPolicies = () => {
  if (!currentPolicyGroup.value) return;
  const groupId = currentPolicyGroup.value.id;
  groupPolicyIdsByGroupId.value = {
    ...groupPolicyIdsByGroupId.value,
    [groupId]: [...(groupPolicyIdsByGroupId.value[groupId] ?? [])].reverse(),
  };
};

const linkGroupToSet = (groupId: string) => {
  if (!currentPolicySet.value) return;
  const setId = currentPolicySet.value.id;
  const assigned = setGroupIdsBySetId.value[setId] ?? [];
  if (assigned.includes(groupId)) return;
  setGroupIdsBySetId.value = {
    ...setGroupIdsBySetId.value,
    [setId]: [...assigned, groupId],
  };
};

const unlinkGroupFromSet = (groupId: string) => {
  if (!currentPolicySet.value) return;
  const setId = currentPolicySet.value.id;
  setGroupIdsBySetId.value = {
    ...setGroupIdsBySetId.value,
    [setId]: (setGroupIdsBySetId.value[setId] ?? []).filter((id) => id !== groupId),
  };
};

const reorderSetGroups = (draggedId: string, targetId: string) => {
  if (!currentPolicySet.value) return;
  const setId = currentPolicySet.value.id;
  const currentIds = setGroupIdsBySetId.value[setId] ?? [];
  const fromIndex = currentIds.indexOf(draggedId);
  const toIndex = currentIds.indexOf(targetId);
  if (fromIndex < 0 || toIndex < 0 || fromIndex === toIndex) return;
  setGroupIdsBySetId.value = {
    ...setGroupIdsBySetId.value,
    [setId]: reorderCollection(currentIds, fromIndex, toIndex),
  };
};

const addInput = () => {
  const nextEntry = createInputEntry('int');
  inputEntries.value = [...inputEntries.value, nextEntry];
  selectedInputId.value = nextEntry.id;
};

const updateInput = (
  entryId: string,
  field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue',
  value: string | boolean,
) => {
  const currentEntry = inputEntries.value.find((entry) => entry.id === entryId) ?? null;
  inputEntries.value = inputEntries.value.map((entry) => {
    if (entry.id !== entryId) {
      return entry;
    }

    const next = { ...entry };
    if (field === 'type') {
      next.type = value as EditorInputEntry['type'];
      next.stringValue = next.type === 'string' ? '' : next.type === 'json' ? '{}' : '0';
      next.booleanValue = false;
      return next;
    }

    if (field === 'namespace') {
      next.namespace = String(value) as EditorInputEntry['namespace'];
      return next;
    }

    if (field === 'booleanValue') {
      next.booleanValue = Boolean(value);
      return next;
    }

    if (field === 'key') {
      const nextKey = String(value);
      const currentName = entry.name.trim();
      const currentKey = entry.key.trim();
      next.key = nextKey;
      if (!currentName || currentName === currentKey) {
        next.name = nextKey;
      }
      return next;
    }

    next[field] = String(value) as never;
    return next;
  });

  if (currentEntry && (field === 'key' || field === 'namespace')) {
    const updatedEntry = inputEntries.value.find((entry) => entry.id === entryId) ?? null;
    if (updatedEntry) {
      syncVariableReferenceRename(
        buildVariableReferenceKey(currentEntry.namespace, currentEntry.key),
        buildVariableReferenceKey(updatedEntry.namespace, updatedEntry.key),
      );
    }
  }
};

const removeInput = (entryId: string) => {
  inputEntries.value = inputEntries.value.filter((entry) => entry.id !== entryId);
  if (selectedInputId.value === entryId) {
    selectedInputId.value = inputEntries.value[0]?.id ?? null;
  }
};

const addUiField = (control: UiFieldControl) => {
  const field = createUiField(control);
  uiSchema.value = {
    ...uiSchema.value,
    fields: [...uiSchema.value.fields, field],
  };
  selectedUiFieldId.value = field.id;
};

const updateUiField = (
  fieldId: string,
  key: 'label' | 'key' | 'editable' | 'checkboxStyle' | 'variableId' | 'inputKey' | 'description' | 'placeholder' | 'optionsText' | 'min' | 'max' | 'step' | 'numericMode',
  value: string | boolean,
) => {
  uiSchema.value = {
    ...uiSchema.value,
    fields: uiSchema.value.fields.map((field) => {
      if (field.id !== fieldId) {
        return field;
      }

      if (key === 'min' || key === 'max' || key === 'step') {
        const parsed = Number(value);
        return {
          ...field,
          [key]: Number.isFinite(parsed) ? parsed : key === 'step' ? 1 : 0,
        };
      }

      return { ...field, [key]: value };
    }),
  };
};

const removeUiField = (fieldId: string) => {
  uiSchema.value = {
    ...uiSchema.value,
    fields: uiSchema.value.fields.filter((field) => field.id !== fieldId),
  };
  if (selectedUiFieldId.value === fieldId) {
    selectedUiFieldId.value = uiSchema.value.fields[0]?.id ?? null;
  }
};

const bindTemplateVariableDefaults = async (templateId: string, step: Step) => {
  const nextStep = cloneJson(step);
  if (!currentTask.value) {
    return nextStep;
  }
  if (!nextStep.id) {
    nextStep.id = createEditorStepId();
  }

  if (templateId === 'capture' && nextStep.op === 'action' && nextStep.a.ac === 'capture') {
    nextStep.a.output_var = await createVariableResource('runtime', 'image', {
      preferredKey: 'captureResult',
      name: '截图结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'set-var' && nextStep.op === 'dataHanding' && nextStep.a.type === 'setVar') {
    nextStep.a.name = await createVariableResource('input', 'int', {
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'get-var' && nextStep.op === 'dataHanding' && nextStep.a.type === 'getVar') {
    nextStep.a.name = await createVariableResource('input', 'int', {
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'filter-var' && nextStep.op === 'dataHanding' && nextStep.a.type === 'filter') {
    nextStep.a.input_var = await createVariableResource('input', 'json', {
      preferredKey: 'items',
      name: '输入列表',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    nextStep.a.out_name = await createVariableResource('runtime', 'json', {
      preferredKey: 'filteredItems',
      name: '过滤结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'vision-search' && nextStep.op === 'vision' && nextStep.a.type === 'visionSearch') {
    nextStep.a.out_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'visionHit',
      name: '视觉命中',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'click-text' && nextStep.op === 'action' && nextStep.a.ac === 'click' && nextStep.a.mode === 'txt') {
    nextStep.a.input_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'ocrResults',
      name: 'OCR结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'swipe-text' && nextStep.op === 'action' && nextStep.a.ac === 'swipe' && nextStep.a.mode === 'txt') {
    nextStep.a.input_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'ocrResults',
      name: 'OCR结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'click-label' && nextStep.op === 'action' && nextStep.a.ac === 'click' && nextStep.a.mode === 'labelIdx') {
    nextStep.a.input_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'detResults',
      name: '检测结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'swipe-label' && nextStep.op === 'action' && nextStep.a.ac === 'swipe' && nextStep.a.mode === 'labelIdx') {
    nextStep.a.input_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'detResults',
      name: '检测结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'handle-policy-set' && nextStep.op === 'flowControl' && nextStep.a.type === 'handlePolicySet') {
    nextStep.a.input_var = await createVariableResource('runtime', 'image', {
      preferredKey: 'policySetImage',
      name: '策略集输入图像',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    nextStep.a.out_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'policySetResult',
      name: '策略集结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'handle-policy' && nextStep.op === 'flowControl' && nextStep.a.type === 'handlePolicy') {
    nextStep.a.input_var = await createVariableResource('runtime', 'image', {
      preferredKey: 'policyImage',
      name: '策略输入图像',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    nextStep.a.out_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'policyResult',
      name: '策略结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  return nextStep;
};

const appendTemplateStep = async (templateId: string) => {
  const templateStep = createStepFromTemplate(templateId);
  const step = templateStep ? await bindTemplateVariableDefaults(templateId, templateStep) : null;
  if (!step) {
    return;
  }

  const nextSteps = updateBranchSteps(
    parsedSteps.value,
    activeBranchPath.value,
    (steps) => [...steps, step],
  );
  setCurrentTaskSteps(nextSteps);
  selectedStepPath.value = buildStepPath(activeBranchPath.value, getBranchSteps(nextSteps, activeBranchPath.value).length - 1);
  activePanel.value = 'steps';
};

const reorderSteps = (fromIndex: number, toIndex: number) => {
  if (fromIndex === toIndex) {
    return;
  }

  const nextSteps = updateBranchSteps(parsedSteps.value, activeBranchPath.value, (steps) => reorderCollection(steps, fromIndex, toIndex));
  setCurrentTaskSteps(nextSteps);
  selectedStepPath.value = buildStepPath(activeBranchPath.value, toIndex);
};

const removeStep = (index: number) => {
  const removedStep = getBranchSteps(parsedSteps.value, activeBranchPath.value)[index] ?? null;
  const nextSteps = updateBranchSteps(parsedSteps.value, activeBranchPath.value, (steps) => steps.filter((_, stepIndex) => stepIndex !== index));
  if (removedStep) {
    const removedSourceStepIds = collectStepTreeIds(removedStep);
    if (removedSourceStepIds.size) {
      const remainingReferences = collectVariableReferencesFromSteps(nextSteps);
      for (const field of uiSchema.value.fields) {
        if (field.inputKey?.trim()) {
          remainingReferences.add(buildVariableReferenceKey('input', field.inputKey));
        }
      }

      const removableEntryIds = new Set(
        inputEntries.value
          .filter((entry) => entry.sourceStepId && removedSourceStepIds.has(entry.sourceStepId))
          .filter((entry) => !remainingReferences.has(buildVariableReferenceKey(entry.namespace, entry.key)))
          .map((entry) => entry.id),
      );

      if (removableEntryIds.size) {
        inputEntries.value = inputEntries.value.filter((entry) => !removableEntryIds.has(entry.id));
        if (selectedInputId.value && removableEntryIds.has(selectedInputId.value)) {
          selectedInputId.value = inputEntries.value[0]?.id ?? null;
        }
      }
    }
  }
  setCurrentTaskSteps(nextSteps);
  selectedStepPath.value = createSiblingSelection(activeBranchPath.value, getBranchSteps(nextSteps, activeBranchPath.value).length, index);
};

const updateStep = (index: number, nextStep: Step) => {
  const nextSteps = updateStepByPath(parsedSteps.value, buildStepPath(activeBranchPath.value, index), () => nextStep);
  setCurrentTaskSteps(nextSteps);
  selectedStepPath.value = buildStepPath(activeBranchPath.value, index);
};

const appendPolicyTemplateStep = (templateId: string) => {
  const step = createStepFromTemplate(templateId);
  if (!step) {
    return;
  }
  const nextSteps = updateBranchSteps(currentPolicySteps.value, activePolicyBranchPath.value, (steps) => [...steps, step]);
  setCurrentPolicySteps(nextSteps);
  const nextPath = buildStepPath(activePolicyBranchPath.value, getBranchSteps(nextSteps, activePolicyBranchPath.value).length - 1);
  if (currentPolicyStepTarget.value === 'before') {
    selectedPolicyStepPathBefore.value = nextPath;
  } else {
    selectedPolicyStepPathAfter.value = nextPath;
  }
};

const selectPolicyStepPath = (path: StepPath) => {
  if (currentPolicyStepTarget.value === 'before') {
    selectedPolicyStepPathBefore.value = cloneStepPath(path);
    activePolicyBranchPathBefore.value = getParentBranchPath(path);
    return;
  }
  selectedPolicyStepPathAfter.value = cloneStepPath(path);
  activePolicyBranchPathAfter.value = getParentBranchPath(path);
};

const navigatePolicyBranch = (branchPath: StepBranchPath) => {
  const nextBranch = {
    branch: branchPath.branch,
    parentStepPath: cloneStepPath(branchPath.parentStepPath),
  };
  if (currentPolicyStepTarget.value === 'before') {
    activePolicyBranchPathBefore.value = nextBranch;
    if (
      selectedPolicyStepPathBefore.value &&
      isSameBranchPath(getParentBranchPath(selectedPolicyStepPathBefore.value), activePolicyBranchPathBefore.value)
    ) {
      return;
    }
    const steps = getBranchSteps(currentPolicySteps.value, activePolicyBranchPathBefore.value);
    selectedPolicyStepPathBefore.value = steps.length ? buildStepPath(activePolicyBranchPathBefore.value, 0) : null;
    return;
  }

  activePolicyBranchPathAfter.value = nextBranch;
  if (
    selectedPolicyStepPathAfter.value &&
    isSameBranchPath(getParentBranchPath(selectedPolicyStepPathAfter.value), activePolicyBranchPathAfter.value)
  ) {
    return;
  }
  const steps = getBranchSteps(currentPolicySteps.value, activePolicyBranchPathAfter.value);
  selectedPolicyStepPathAfter.value = steps.length ? buildStepPath(activePolicyBranchPathAfter.value, 0) : null;
};

const reorderPolicySteps = (fromIndex: number, toIndex: number) => {
  if (fromIndex === toIndex) return;
  const nextSteps = updateBranchSteps(currentPolicySteps.value, activePolicyBranchPath.value, (steps) => reorderCollection(steps, fromIndex, toIndex));
  setCurrentPolicySteps(nextSteps);
  const nextPath = buildStepPath(activePolicyBranchPath.value, toIndex);
  if (currentPolicyStepTarget.value === 'before') {
    selectedPolicyStepPathBefore.value = nextPath;
  } else {
    selectedPolicyStepPathAfter.value = nextPath;
  }
};

const removePolicyStep = (index: number) => {
  const nextSteps = updateBranchSteps(currentPolicySteps.value, activePolicyBranchPath.value, (steps) => steps.filter((_, stepIndex) => stepIndex !== index));
  setCurrentPolicySteps(nextSteps);
  const nextSelection = createSiblingSelection(activePolicyBranchPath.value, getBranchSteps(nextSteps, activePolicyBranchPath.value).length, index);
  if (currentPolicyStepTarget.value === 'before') {
    selectedPolicyStepPathBefore.value = nextSelection;
  } else {
    selectedPolicyStepPathAfter.value = nextSelection;
  }
};

const updatePolicyStep = (index: number, nextStep: Step) => {
  const nextSteps = updateStepByPath(currentPolicySteps.value, buildStepPath(activePolicyBranchPath.value, index), () => nextStep);
  setCurrentPolicySteps(nextSteps);
  const nextPath = buildStepPath(activePolicyBranchPath.value, index);
  if (currentPolicyStepTarget.value === 'before') {
    selectedPolicyStepPathBefore.value = nextPath;
  } else {
    selectedPolicyStepPathAfter.value = nextPath;
  }
};

const selectStepPath = (path: StepPath) => {
  selectedStepPath.value = cloneStepPath(path);
  activeBranchPath.value = getParentBranchPath(path);
};

const navigateBranch = (branchPath: StepBranchPath) => {
  activeBranchPath.value = {
    branch: branchPath.branch,
    parentStepPath: cloneStepPath(branchPath.parentStepPath),
  };

  if (selectedStepPath.value && isSameBranchPath(getParentBranchPath(selectedStepPath.value), activeBranchPath.value)) {
    return;
  }

  const steps = getBranchSteps(parsedSteps.value, activeBranchPath.value);
  selectedStepPath.value = steps.length ? buildStepPath(activeBranchPath.value, 0) : null;
};

const openRawEditor = (section: RawEditorSection) => {
  if (!currentTask.value) {
    return;
  }

  rawDialogSection.value = section;
  rawDialogError.value = null;
  rawDialogText.value = stableStringify(
    section === 'inputs'
      ? currentTask.value.data.variables ?? {}
      : section === 'ui'
        ? currentTask.value.data.uiData ?? {}
        : currentTask.value.data.steps ?? [],
  );
  rawDialogOpen.value = true;
};

const formatRawEditor = () => {
  try {
    rawDialogText.value = stableStringify(JSON.parse(rawDialogText.value) as JsonValue);
    rawDialogError.value = null;
  } catch (error) {
    rawDialogError.value = error instanceof Error ? error.message : 'JSON 结构无效';
  }
};

const applyRawEditor = () => {
  if (!currentTask.value) {
    return;
  }

  try {
    const parsed = JSON.parse(rawDialogText.value) as JsonValue;
    if (rawDialogSection.value === 'steps' && !Array.isArray(parsed)) {
      throw new Error('步骤 JSON 顶层必须是数组。');
    }

    replaceTask(currentTask.value.id, (task) => {
      if (rawDialogSection.value === 'inputs') {
        task.data.variables = parsed;
      } else if (rawDialogSection.value === 'ui') {
        task.data.uiData = parsed;
      } else {
        task.data.steps = parsed as Step[];
      }
      return task;
    });

    hydrateTaskEditors();
    rawDialogError.value = null;
    rawDialogOpen.value = false;
  } catch (error) {
    rawDialogError.value = error instanceof Error ? error.message : 'JSON 结构无效';
  }
};

const applyScriptInfo = (script: ScriptTableRecord) => {
  draftScript.value = cloneJson(script);
  infoDialogOpen.value = false;
  showToast('脚本信息已写入当前草稿，顶部保存后生效。', 'success');
};

const buildSavePayload = () =>
  draftTasks.value.map((task, index) =>
    normalizeTask(
      {
        ...task,
        scriptId: scriptId.value,
      },
      index,
    ),
  );

const buildPolicyPayload = () =>
  draftPolicies.value.map((policy, index) =>
    normalizePolicy(
      {
        ...policy,
        scriptId: scriptId.value,
      },
      index,
    ),
  );

const buildPolicyGroupPayload = () =>
  draftPolicyGroups.value.map((group, index) =>
    normalizePolicyGroup(
      {
        ...group,
        scriptId: scriptId.value,
      },
      index,
    ),
  );

const buildPolicySetPayload = () =>
  draftPolicySets.value.map((set, index) =>
    normalizePolicySet(
      {
        ...set,
        scriptId: scriptId.value,
      },
      index,
    ),
  );

const saveEditor = async () => {
  if (!draftScript.value) {
    return;
  }

  if (hasValidationErrors.value) {
    showToast('请先修复输入变量里的错误，再执行保存。', 'error');
    return;
  }

  isSaving.value = true;

  try {
    const nextSaveTime = new Date().toISOString();
    const tasks = buildSavePayload().map((task) => ({
      ...task,
      updatedAt: nextSaveTime,
    }));
    const policies = buildPolicyPayload();
    const policyGroups = buildPolicyGroupPayload();
    const policySets = buildPolicySetPayload();
    const script = {
      ...draftScript.value,
      data: {
        ...draftScript.value.data,
        updateTime: nextSaveTime,
      },
    };
    const sourcePolicyIds = new Set(((JSON.parse(sourcePoliciesSnapshot.value || '[]') as PolicyTable[]) ?? []).map((item) => item.id));
    const sourcePolicyGroupIds = new Set(
      ((JSON.parse(sourcePolicyGroupsSnapshot.value || '[]') as PolicyGroupTable[]) ?? []).map((item) => item.id),
    );
    const sourcePolicySetIds = new Set(((JSON.parse(sourcePolicySetsSnapshot.value || '[]') as PolicySetTable[]) ?? []).map((item) => item.id));
    const nextPolicyIds = new Set(policies.map((item) => item.id));
    const nextPolicyGroupIds = new Set(policyGroups.map((item) => item.id));
    const nextPolicySetIds = new Set(policySets.map((item) => item.id));

    await Promise.all([
      scriptStore.saveScriptTasks(script.id, tasks),
      ...policies.map((policy) => scriptService.savePolicy(policy)),
      ...policyGroups.map((group) => scriptService.savePolicyGroup(group)),
      ...policySets.map((set) => scriptService.savePolicySet(set)),
    ]);

    await Promise.all([
      ...Array.from(sourcePolicyIds).filter((id) => !nextPolicyIds.has(id)).map((id) => scriptService.removePolicy(id)),
      ...Array.from(sourcePolicyGroupIds).filter((id) => !nextPolicyGroupIds.has(id)).map((id) => scriptService.removePolicyGroup(id)),
      ...Array.from(sourcePolicySetIds).filter((id) => !nextPolicySetIds.has(id)).map((id) => scriptService.removePolicySet(id)),
    ]);

    await Promise.all([
      ...policyGroups.map((group) => scriptService.updateGroupPolicies(group.id, groupPolicyIdsByGroupId.value[group.id] ?? [])),
      ...policySets.map((set) => scriptService.updateSetGroups(set.id, setGroupIdsBySetId.value[set.id] ?? [])),
    ]);

    await scriptStore.saveScript(script);

    draftTasks.value = tasks;
    draftPolicies.value = policies;
    draftPolicyGroups.value = policyGroups;
    draftPolicySets.value = policySets;
    draftScript.value = script;
    sourceTasksSnapshot.value = stableStringify(tasks);
    sourcePoliciesSnapshot.value = stableStringify(policies);
    sourcePolicyGroupsSnapshot.value = stableStringify(policyGroups);
    sourcePolicySetsSnapshot.value = stableStringify(policySets);
    sourceGroupPoliciesSnapshot.value = stableStringify(groupPolicyIdsByGroupId.value);
    sourceSetGroupsSnapshot.value = stableStringify(setGroupIdsBySetId.value);
    sourceScriptSnapshot.value = stableStringify(script);
    saveTime.value = nextSaveTime;
    appendConsoleLine(`脚本结构已保存：${script.data.name || script.id}`);
    showToast('脚本编辑结果已保存', 'success');
  } catch (error) {
    console.error(error);
    appendConsoleLine(`脚本保存失败：${error instanceof Error ? error.message : '未知错误'}`);
    showToast(error instanceof Error ? error.message : '保存失败', 'error');
  } finally {
    isSaving.value = false;
  }
};

const loadEditor = async () => {
  isLoading.value = true;
  loadError.value = null;

  try {
    if (!scriptId.value) {
      throw new Error('缺少 scriptId 参数，无法确定要打开哪个脚本。');
    }

    if (!scriptStore.scripts.length) {
      await scriptStore.loadScripts();
    }

    const sourceScript = (scriptStore.scripts as ScriptTableRecord[]).find((item) => item.id === scriptId.value) ?? null;
    if (!sourceScript) {
      throw new Error('当前脚本不存在，可能已被删除或尚未加载到本地列表。');
    }

    draftScript.value = cloneJson(sourceScript);
    sourceScriptSnapshot.value = stableStringify(draftScript.value);

    const [loadedTasks, loadedPolicies, loadedPolicyGroups, loadedPolicySets] = await Promise.all([
      scriptStore.loadScriptTasks(sourceScript.id),
      scriptService.listPolicies(sourceScript.id),
      scriptService.listPolicyGroups(sourceScript.id),
      scriptService.listPolicySets(sourceScript.id),
    ]);
    if (loadedTasks.length) {
      draftTasks.value = loadedTasks.map((task, index) => normalizeTask(task, index));
      sourceTasksSnapshot.value = stableStringify(draftTasks.value);
    } else {
      draftTasks.value = [await buildTaskDraft('主任务 1')];
      sourceTasksSnapshot.value = stableStringify([]);
    }

    draftPolicies.value = loadedPolicies.map((policy, index) => normalizePolicy(policy, index));
    draftPolicyGroups.value = loadedPolicyGroups.map((group, index) => normalizePolicyGroup(group, index));
    draftPolicySets.value = loadedPolicySets.map((set, index) => normalizePolicySet(set, index));

    const [groupRelations, setRelations] = await Promise.all([
      Promise.all(
        draftPolicyGroups.value.map(async (group) => [group.id, await scriptService.getGroupPolicies(group.id)] as const),
      ),
      Promise.all(
        draftPolicySets.value.map(async (set) => [set.id, await scriptService.getSetGroups(set.id)] as const),
      ),
    ]);
    groupPolicyIdsByGroupId.value = Object.fromEntries(groupRelations);
    setGroupIdsBySetId.value = Object.fromEntries(setRelations);
    sourcePoliciesSnapshot.value = stableStringify(draftPolicies.value);
    sourcePolicyGroupsSnapshot.value = stableStringify(draftPolicyGroups.value);
    sourcePolicySetsSnapshot.value = stableStringify(draftPolicySets.value);
    sourceGroupPoliciesSnapshot.value = stableStringify(groupPolicyIdsByGroupId.value);
    sourceSetGroupsSnapshot.value = stableStringify(setGroupIdsBySetId.value);

    selectedTaskId.value = draftTasks.value[0]?.id ?? null;
    selectedPolicyId.value = draftPolicies.value[0]?.id ?? null;
    selectedPolicyGroupId.value = draftPolicyGroups.value[0]?.id ?? null;
    selectedPolicySetId.value = draftPolicySets.value[0]?.id ?? null;
    activeBranchPath.value = ROOT_BRANCH_PATH;
    activePolicyBranchPathBefore.value = ROOT_BRANCH_PATH;
    activePolicyBranchPathAfter.value = ROOT_BRANCH_PATH;
    saveTime.value = sourceScript.data.updateTime || null;
    hydrateTaskEditors();
    appendConsoleLine(`已载入脚本：${sourceScript.data.name}`);
  } catch (error) {
    console.error(error);
    loadError.value = error instanceof Error ? error.message : '脚本编辑器初始化失败';
    appendConsoleLine(`编辑器载入失败：${loadError.value}`);
  } finally {
    isLoading.value = false;
  }
};

const reloadEditor = async () => {
  await loadEditor();
};

const handleKeydown = (event: KeyboardEvent) => {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
    event.preventDefault();
    if (!isSaving.value) {
      void saveEditor();
    }
  }
};

const hydratePolicyStepEditors = () => {
  if (!currentPolicy.value) {
    selectedPolicyStepPathBefore.value = null;
    selectedPolicyStepPathAfter.value = null;
    activePolicyBranchPathBefore.value = ROOT_BRANCH_PATH;
    activePolicyBranchPathAfter.value = ROOT_BRANCH_PATH;
    return;
  }

  if (!currentPolicy.value.data.beforeAction.length) {
    selectedPolicyStepPathBefore.value = null;
    activePolicyBranchPathBefore.value = ROOT_BRANCH_PATH;
  } else if (
    !selectedPolicyStepPathBefore.value ||
    !getStepByPath(currentPolicy.value.data.beforeAction, selectedPolicyStepPathBefore.value)
  ) {
    selectedPolicyStepPathBefore.value = buildStepPath(ROOT_BRANCH_PATH, 0);
    activePolicyBranchPathBefore.value = ROOT_BRANCH_PATH;
  }

  if (!currentPolicy.value.data.afterAction.length) {
    selectedPolicyStepPathAfter.value = null;
    activePolicyBranchPathAfter.value = ROOT_BRANCH_PATH;
  } else if (
    !selectedPolicyStepPathAfter.value ||
    !getStepByPath(currentPolicy.value.data.afterAction, selectedPolicyStepPathAfter.value)
  ) {
    selectedPolicyStepPathAfter.value = buildStepPath(ROOT_BRANCH_PATH, 0);
    activePolicyBranchPathAfter.value = ROOT_BRANCH_PATH;
  }
};

watch(
  () => currentTask.value?.id,
  () => {
    hydrateTaskEditors();
  },
  { immediate: true },
);

watch(
  () => currentPolicy.value?.id,
  () => {
    hydratePolicyStepEditors();
  },
  { immediate: true },
);

watch(
  textDetLabelPath,
  (path) => {
    void loadTextDetLabels(path);
  },
  { immediate: true },
);

watch(
  () => deviceStore.devices.map((device) => device.id).join('|'),
  () => {
    if (selectedPreviewDeviceId.value && deviceStore.devices.some((device) => device.id === selectedPreviewDeviceId.value)) {
      return;
    }
    selectedPreviewDeviceId.value = deviceStore.devices[0]?.id ?? null;
  },
  { immediate: true },
);

watch(activeMode, (value) => {
  if (value === 'task') {
    activePanel.value = activePanel.value || 'basic';
    return;
  }
  if (value === 'policy') {
    selectedPolicyId.value = selectedPolicyId.value ?? draftPolicies.value[0]?.id ?? null;
    return;
  }
  if (value === 'policyGroup') {
    selectedPolicyGroupId.value = selectedPolicyGroupId.value ?? draftPolicyGroups.value[0]?.id ?? null;
    return;
  }
  selectedPolicySetId.value = selectedPolicySetId.value ?? draftPolicySets.value[0]?.id ?? null;
});

watch(taskName, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value) {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.name = value.trim() || '未命名任务';
    return task;
  });
});

watch(taskRowType, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value) {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.rowType = value;
    if (value === 'title') {
      activePanel.value = 'basic';
      task.recordSchedule = false;
      task.sectionId = null;
      task.indentLevel = 0;
      task.showEnabledToggle = false;
      task.taskTone = 'normal';
    }
    return task;
  });
});

watch(taskTriggerMode, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value || taskRowType.value === 'title') {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.triggerMode = value;
    return task;
  });
});

watch(taskHidden, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value) {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.isHidden = value;
    return task;
  });
});

watch(recordSchedule, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value || taskRowType.value === 'title') {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.recordSchedule = value;
    return task;
  });
});

watch(sectionId, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value || taskRowType.value === 'title') {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.sectionId = value;
    return task;
  });
});

watch(indentLevel, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value || taskRowType.value === 'title') {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.indentLevel = Math.max(0, Math.min(8, Number(value || 0)));
    return task;
  });
});

watch(defaultTaskCycle, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value || taskRowType.value === 'title') {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.defaultTaskCycle = value;
    return task;
  });
});

watch(showEnabledToggle, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value || taskRowType.value === 'title') {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.showEnabledToggle = value;
    return task;
  });
});

watch(defaultEnabled, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value || taskRowType.value === 'title') {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.defaultEnabled = value;
    return task;
  });
});

watch(taskTone, (value) => {
  if (!currentTask.value || hydratingTaskMeta.value || taskRowType.value === 'title') {
    return;
  }

  replaceTask(currentTask.value.id, (task) => {
    task.taskTone = value;
    return task;
  });
});

watch(
  inputEntries,
  (entries) => {
    if (!currentTask.value || hydratingTaskPanels.value) {
      return;
    }

    try {
      const nextVariables = buildInputJson(entries);
      const nextCatalog = syncInputVariableCatalog(draftScript.value?.data.variableCatalog, currentTask.value.id, entries);
      inputError.value = null;
      replaceTask(currentTask.value.id, (task) => {
        task.data.variables = nextVariables;
        return task;
      });
      if (draftScript.value) {
        draftScript.value = {
          ...draftScript.value,
          data: {
            ...draftScript.value.data,
            variableCatalog: nextCatalog,
          },
        };
      }
    } catch (error) {
      inputError.value = error instanceof Error ? error.message : '输入变量结构无效';
    }
  },
  { deep: true },
);

watch(
  uiSchema,
  (value) => {
    if (!currentTask.value || hydratingTaskPanels.value) {
      return;
    }

    replaceTask(currentTask.value.id, (task) => {
      task.data.uiData = buildUiData(value);
      return task;
    });
  },
  { deep: true },
);

watch(
  () => scriptId.value,
  async () => {
    await loadEditor();
  },
  { immediate: true },
);

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
  void deviceStore.initIpcListeners();
  void Promise.all([deviceStore.refreshAll(), settingsStore.loadPreferences()]);
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown);
});
</script>

<style scoped>
.editor-shell {
  background:
    radial-gradient(circle at 10% 12%, rgba(70, 110, 255, 0.12), transparent 24%),
    radial-gradient(circle at 88% 14%, rgba(87, 196, 255, 0.15), transparent 22%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.22), rgba(255, 255, 255, 0)),
    transparent;
}

.editor-toolbar {
  background:
    radial-gradient(circle at 16% 20%, rgba(255, 255, 255, 0.42), transparent 30%),
    linear-gradient(135deg, rgba(255, 255, 255, 0.62), rgba(245, 248, 255, 0.34)),
    var(--app-panel);
  box-shadow: var(--app-shadow-soft);
  backdrop-filter: blur(16px);
}
</style>
