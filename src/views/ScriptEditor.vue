<template>
  <div class="editor-shell h-svh overflow-hidden">
    <div class="mx-auto flex h-full flex-col gap-[1px]">
      <EditorWindowTitlebar
        class="script-editor-titlebar"
        :title="draftScript?.data.name || '脚本编辑器'"
        :meta="formattedSaveTime ? `最近保存 ${formattedSaveTime}` : null"
        :status-label="hasValidationErrors ? '待修复' : dirty ? '未保存' : '已同步'"
        :status-tone="hasValidationErrors ? 'danger' : dirty ? 'warning' : 'success'"
        :request-close="handleRequestCloseWindow"
      >
        <template #prefix>
          <button class="app-icon-button group" type="button" title="返回" aria-label="返回" @click="handleNavigateBack">
            <AppIcon name="chevron-left" :size="16" class="text-(--app-text-soft) group-hover:text-(--app-accent) transition-colors" />
          </button>
        </template>

        <template #status-actions>
          <button
            v-if="canOpenCurrentRawEditor"
            class="app-icon-button group"
            type="button"
            title="查看 JSON"
            aria-label="查看 JSON"
            data-testid="editor-open-raw-json"
            @click="openCurrentRawEditor"
          >
            <AppIcon name="braces" :size="16" class="text-(--app-text-soft) group-hover:text-(--app-accent) transition-colors" />
          </button>
          <button class="app-icon-button group" type="button" title="显示日志" aria-label="显示日志" data-testid="editor-show-console" @click="showConsole">
            <AppIcon name="scroll-text" :size="16" class="text-(--app-text-soft) group-hover:text-(--app-accent) transition-colors" />
          </button>
        </template>

        <template #title-actions>
          <button class="app-icon-button group" type="button" title="编辑脚本信息" aria-label="编辑脚本信息" data-testid="editor-script-info" @click="infoDialogOpen = true">
            <AppIcon name="file-text" :size="16" class="text-(--app-text-soft) group-hover:text-(--app-accent) transition-colors" />
          </button>
          <button class="app-icon-button group" type="button" title="视觉测试" aria-label="视觉测试" data-testid="editor-open-vision-lab" :disabled="!draftScript" @click="handleOpenVisionLab">
            <AppIcon name="scan-search" :size="16" class="text-(--app-text-soft) group-hover:text-(--app-accent) transition-colors" />
          </button>
          <button class="app-icon-button group" type="button" title="开发者工具" aria-label="开发者工具" data-testid="editor-open-devtools" @click="openCurrentDevtools">
            <AppIcon name="bug" :size="16" class="text-(--app-text-soft) group-hover:text-(--app-accent) transition-colors" />
          </button>
          <button class="app-icon-button group" type="button" title="刷新页面" aria-label="刷新页面" data-testid="editor-reload-page" @click="reloadCurrentPage">
            <AppIcon name="refresh-cw" :size="16" class="text-(--app-text-soft) group-hover:text-(--app-accent) transition-colors" />
          </button>
        </template>

        <template #actions>
          <div class="flex flex-wrap items-center justify-end gap-2">
            <div class="max-w-[140px]">
              <AppSelect
                v-model="selectedPreviewDeviceId"
                :options="deviceSelectOptions"
                placeholder="请选择设备"
                test-id="editor-header-device"
              />
            </div>
            <button
              class="app-icon-button group"
              type="button"
              :title="selectedPreviewDeviceId ? '编辑设备' : '新建设备'"
              :aria-label="selectedPreviewDeviceId ? '编辑设备' : '新建设备'"
              :disabled="selectedPreviewDeviceBusy"
              @click="openDeviceEditor(selectedPreviewDeviceId)"
            >
              <AppIcon name="edit-3" :size="16" class="text-(--app-text-soft) group-hover:text-(--app-accent) transition-colors" />
            </button>
            <div class="max-w-[300px]">
              <AppSelect
                v-model="selectedRunTargetKey"
                :options="runTargetSelectOptions"
                :placeholder="runTargetSelectPlaceholder"
                searchable
                search-placeholder="按名称搜索"
                :max-menu-height="360"
                :show-description="true"
                test-id="editor-header-target-item"
              />
            </div>
            <button
              class="app-icon-button group"
              type="button"
              data-testid="editor-run"
              :title="runSelectionDisabledReason || '运行'"
              :aria-label="runSelectionDisabledReason || '运行'"
              :disabled="!canRunSelection"
              @click="handleRunSelection"
            >
              <AppIcon name="play" :size="16"/>
            </button>
            <button
              class="app-icon-button"
              type="button"
              data-testid="editor-save"
              :title="isSaving ? '保存中...' : '保存'"
              :aria-label="isSaving ? '保存中...' : '保存'"
              :disabled="!draftScript || isSaving || hasValidationErrors"
              @click="saveEditor"
            >
              <AppIcon :name="isSaving ? 'loader-circle' : 'save'" :size="16" :class="{ 'app-loading-spinner': isSaving }" />
            </button>
          </div>
        </template>
      </EditorWindowTitlebar>

      <div v-if="loadError" class="border border-red-500/16 bg-red-500/8 px-6 py-8 text-red-700">
        <h2 class="text-xl font-semibold">无法打开编辑器</h2>
        <p class="mt-3 max-w-2xl text-sm leading-6">{{ loadError }}</p>
      </div>

      <div
        v-else-if="isLoading"
        class="border border-(--app-border) bg-(--app-panel) px-6 py-10 text-sm text-(--app-text-soft)"
      >
        正在读取脚本和任务结构...
      </div>

      <div v-else class="flex min-h-0 flex-1 flex-col gap-[1px] bg-(--app-border) overflow-hidden">
        <div class="flex min-h-0 flex-1 gap-[1px]">
          <div class="relative shrink-0 flex flex-col min-h-0" :class="{ 'transition-[width] duration-200': !isResizingSidebar }" :style="{ width: leftSidebarCollapsed ? '64px' : `${sidebarWidth}px` }">
          <EditorTaskSidebar
            v-if="activeMode === 'task'"
            :tasks="draftTasks"
            :selected-task-id="selectedTaskId"
            :collapsed="leftSidebarCollapsed"
            @create="createTask"
            @select="selectTask"
            @duplicate="duplicateTask"
            @toggle-hidden="toggleTaskHidden"
            @remove="removeTask"
            @reorder="reorderTasks"
            @move-task="moveTaskByMenu"
          >
            <template #mode-switch>
              <EditorModeSwitch v-model="activeMode" :options="editorModeOptions" :collapsed="leftSidebarCollapsed" @toggle-collapsed="leftSidebarCollapsed = !leftSidebarCollapsed" />
            </template>
          </EditorTaskSidebar>

          <EditorCollectionSidebar
            v-else-if="activeMode === 'policy'"
            search-placeholder="按名称、备注或日志检索策略"
            :items="policyItems"
            :selected-id="selectedPolicyId"
            empty-title="没有可编辑的策略"
            empty-description="先创建策略，再在右侧配置命中条件和步骤。"
            create-test-id="editor-policy-create"
            item-test-id-prefix="editor-policy-item"
            :collapsed="leftSidebarCollapsed"
            @create="createPolicy"
            @select="selectedPolicyId = $event"
            @duplicate="duplicatePolicy"
            @remove="removePolicy"
            @reorder="reorderPolicies"
            @move-item="movePolicyByMenu"
          >
            <template #mode-switch>
              <EditorModeSwitch v-model="activeMode" :options="editorModeOptions" :collapsed="leftSidebarCollapsed" @toggle-collapsed="leftSidebarCollapsed = !leftSidebarCollapsed" />
            </template>
          </EditorCollectionSidebar>

          <EditorCollectionSidebar
            v-else-if="activeMode === 'policyGroup'"
            search-placeholder="按名称或备注检索策略组"
            :items="policyGroupItems"
            :selected-id="selectedPolicyGroupId"
            empty-title="没有可编辑的策略组"
            empty-description="先创建策略组，再在右侧维护策略关联。"
            create-test-id="editor-policy-group-create"
            item-test-id-prefix="editor-policy-group-item"
            :collapsed="leftSidebarCollapsed"
            @create="createPolicyGroup"
            @select="selectedPolicyGroupId = $event"
            @duplicate="duplicatePolicyGroup"
            @remove="removePolicyGroup"
            @reorder="reorderPolicyGroups"
            @move-item="movePolicyGroupByMenu"
          >
            <template #mode-switch>
              <EditorModeSwitch v-model="activeMode" :options="editorModeOptions" :collapsed="leftSidebarCollapsed" @toggle-collapsed="leftSidebarCollapsed = !leftSidebarCollapsed" />
            </template>
          </EditorCollectionSidebar>

          <EditorCollectionSidebar
            v-else
            search-placeholder="按名称或备注检索策略集"
            :items="policySetItems"
            :selected-id="selectedPolicySetId"
            empty-title="没有可编辑的策略集"
            empty-description="先创建策略集，再在右侧维护策略组关联。"
            create-test-id="editor-policy-set-create"
            item-test-id-prefix="editor-policy-set-item"
            :collapsed="leftSidebarCollapsed"
            @create="createPolicySet"
            @select="selectedPolicySetId = $event"
            @duplicate="duplicatePolicySet"
            @remove="removePolicySet"
            @reorder="reorderPolicySets"
            @move-item="movePolicySetByMenu"
          >
            <template #mode-switch>
              <EditorModeSwitch v-model="activeMode" :options="editorModeOptions" :collapsed="leftSidebarCollapsed" @toggle-collapsed="leftSidebarCollapsed = !leftSidebarCollapsed" />
            </template>
          </EditorCollectionSidebar>
            <div class="absolute -right-[3px] top-0 bottom-0 w-[5px] cursor-col-resize z-10 hover:bg-[rgba(70,110,255,0.96)] transition-colors" @mousedown.prevent="startSidebarResize" v-show="!leftSidebarCollapsed" />
          </div>

          <div class="flex min-h-0 flex-1 flex-col gap-[1px]">
            <div class="flex min-h-0 flex-1 gap-[1px]">
              <div class="relative shrink-0 flex flex-col min-h-0" :style="{ width: `${configPanelWidth}px` }">
              <EditorTaskConfigPanel
                v-if="activeMode === 'task'"
                :task="currentTask"
                :active-panel="activePanel"
                :task-name="taskName"
                :task-description="taskDescription"
                :task-row-type="taskRowType"
                :task-trigger-mode="taskTriggerMode"
                :task-hidden="taskHidden"
                :record-schedule="recordSchedule"
                :section-id="sectionId"
                :indent-level="indentLevel"
                :default-task-cycle-value="defaultTaskCycleValue"
                :default-task-cycle-mode="defaultTaskCycleMode"
                :default-task-cycle-day="defaultTaskCycleDay"
                :task-exec-max="taskExecMax"
                :show-enabled-toggle="showEnabledToggle"
                :default-enabled="defaultEnabled"
                :task-tone="taskTone"
                :title-options="titleTaskOptions"
                :input-entries="inputEntries"
                :input-error="inputError"
                :entry-reference-state="inputEntryReferenceState"
                :ui-schema="uiSchema"
                :selected-input-id="selectedInputId"
                :selected-ui-field-id="selectedUiFieldId"
                :restrict-sequence-templates="activeBranchPath.branch === 'sequence'"
                @update:active-panel="activePanel = $event"
                @update:task-name="taskName = $event"
                @update:task-description="taskDescription = $event"
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
                @update:task-exec-max="taskExecMax = $event"
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
                :input-entries="inputEntries"
                :selected-input-id="selectedInputId"
                :entry-reference-state="inputEntryReferenceState"
                :condition-count="currentPolicyConditionCount"
                :before-count="currentPolicy?.data.beforeAction.length ?? 0"
                :after-count="currentPolicy?.data.afterAction.length ?? 0"
                :restrict-sequence-templates="activePolicyBranchPath.branch === 'sequence'"
                @update:active-panel="activePolicyPanel = $event"
                @update:policy-name="updatePolicyTextField('name', $event)"
                @update:policy-note="updatePolicyTextField('note', $event)"
                @update:policy-log-print="updatePolicyTextField('logPrint', $event)"
                @update:number-field="updatePolicyNumberField"
                @update:boolean-field="updatePolicyBooleanField"
                @add-input="addInput"
                @select-input="selectedInputId = $event"
                @remove-input="removeInput"
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
              <div class="absolute -right-[3px] top-0 bottom-0 w-[5px] cursor-col-resize z-10 hover:bg-[rgba(70,110,255,0.96)] transition-colors" @mousedown.prevent="startConfigPanelResize" />
            </div>

            <div class="min-w-0 flex-1 flex flex-col min-h-0">
              <EditorTaskWorkspace
                v-if="activeMode === 'task'"
                :key="currentTask?.id ?? 'no-task'"
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
                :task-exec-max="taskExecMax"
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
                :label-index-options="imgDetLabelOptions"
                :label-select-placeholder="imgDetLabelSelectPlaceholder"
                :label-select-hint="imgDetLabelHint"
                :task-reference-options="taskReferenceOptions"
                :task-description-map="taskDescriptionMap"
                :policy-reference-options="policyReferenceOptions"
                :policy-note-map="policyNoteMap"
                :task-ui-variable-options="taskUiVariableOptions"
                :policy-group-reference-options="policyGroupReferenceOptions"
                :policy-group-note-map="policyGroupNoteMap"
                :policy-set-reference-options="policySetReferenceOptions"
                :policy-set-note-map="policySetNoteMap"
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
                @update:task-exec-max="taskExecMax = $event"
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
                :key="currentPolicy?.id ?? 'no-policy'"
                :policy="currentPolicy"
                :active-panel="activePolicyPanel"
                :steps="currentPolicySteps"
                :selected-step-path="selectedPolicyStepPath"
                :active-branch-path="activePolicyBranchPath"
                :input-entries="inputEntries"
                :selected-input-id="selectedInputId"
                :variable-options="policyVariableOptions"
                :catalog-variable-options="policyCatalogVariableOptions"
                :label-index-options="imgDetLabelOptions"
                :label-select-placeholder="imgDetLabelSelectPlaceholder"
                :label-select-hint="imgDetLabelHint"
                :task-reference-options="taskReferenceOptions"
                :task-description-map="taskDescriptionMap"
                :policy-reference-options="policyReferenceOptions"
                :policy-note-map="policyNoteMap"
                :task-ui-variable-options="taskUiVariableOptions"
                :policy-group-reference-options="policyGroupReferenceOptions"
                :policy-group-note-map="policyGroupNoteMap"
                :policy-set-reference-options="policySetReferenceOptions"
                :policy-set-note-map="policySetNoteMap"
                :create-reference="createReferenceResource"
                :jump-to-reference="jumpToReferenceResource"
                :create-variable="createVariableResource"
                :jump-to-variable="jumpToVariableResource"
                @update-input="updateInput"
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
                :key="`policy-group:${currentPolicyGroup?.id ?? 'none'}`"
                :has-selection="Boolean(currentPolicyGroup)"
                :selected-title="currentPolicyGroup?.data.name ?? null"
                assigned-title="已关联策略"
                unassigned-title="未关联策略"
                :assigned-items="assignedPolicies"
                :unassigned-items="unassignedPolicies"
                show-reverse-action
                reverse-action-label="逆序排列"
                @link="linkPolicyToGroup"
                @locate="locatePolicy"
                @unlink="unlinkPolicyFromGroup"
                @reorder="reorderGroupPolicies"
                @reverse="reverseGroupPolicies"
              />

              <EditorRelationWorkspace
                v-else
                :key="`policy-set:${currentPolicySet?.id ?? 'none'}`"
                :has-selection="Boolean(currentPolicySet)"
                :selected-title="currentPolicySet?.data.name ?? null"
                assigned-title="已关联策略组"
                unassigned-title="未关联策略组"
                :assigned-items="assignedGroups"
                :unassigned-items="unassignedGroups"
                @link="linkGroupToSet"
                @locate="locatePolicyGroup"
                @unlink="unlinkGroupFromSet"
                @reorder="reorderSetGroups"
              />
              </div>
            </div>
          </div>
        </div>

        <EditorConsolePanel
          v-if="isConsoleVisible"
          :entries="consoleEntries"
          :max-lines="MAX_CONSOLE_LINES"
          @clear="clearConsole"
          @close="hideConsole"
        />
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
      :busy="editingPreviewDeviceBusy"
      @close="deviceEditorOpen = false"
      @save="savePreviewDevice"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { onBeforeRouteLeave, useRoute, useRouter } from 'vue-router';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import EditorWindowTitlebar from '@/views/script-editor/EditorWindowTitlebar.vue';
import DeviceEditorDialog from '@/views/device-list/DeviceEditorDialog.vue';
import { buildDeviceTableFromForm } from '@/views/device-list/deviceEditorShared';
import { useScriptStore } from '@/store/script';
import { useDeviceStore } from '@/store/device';
import { useSettingsStore } from '@/store/settings';
import { deviceKey, getFromStore, scriptEditorViewStateKey, setToStore } from '@/store/store';
import { deviceService } from '@/services/deviceService';
import { runtimeService } from '@/services/runtimeService';
import { openCurrentDevtools, reloadCurrentPage } from '@/services/devtoolsService';
import { requestAppConfirm } from '@/services/appDialogService';
import { scriptService } from '@/services/scriptService';
import { taskService } from '@/services/taskService';
import type { DeviceFormState, ScriptTableRecord } from '@/types/app/domain';
import type { PolicyGroupTable } from '@/types/bindings/PolicyGroupTable';
import type { PolicySetTable } from '@/types/bindings/PolicySetTable';
import type { PolicyTable } from '@/types/bindings/PolicyTable';
import type { SearchRule } from '@/types/bindings/SearchRule';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import type { Step } from '@/types/bindings/Step';
import type { TaskCycle } from '@/types/bindings/TaskCycle';
import type { TaskRowType } from '@/types/bindings/TaskRowType';
import type { TaskTone } from '@/types/bindings/TaskTone';
import type { TaskTriggerMode } from '@/types/bindings/TaskTriggerMode';
import { showToast } from '@/utils/toast';
import { toErrorText } from '@/utils/api';
import { formatCaptureMethod, formatConnectLabel } from '@/utils/presenters';
import {
  createFullScriptRunTarget,
  createPolicyGroupRunTarget,
  createPolicyRunTarget,
  createPolicySetRunTarget,
  createTaskRunTarget,
} from '@/utils/runTarget';
import { validateDeviceRuntimePlatform } from '@/utils/runtimePolicy';
import {
  rewritePublishedDetectorModelPath,
  rewritePublishedRecognizerModelPath,
} from '@/utils/visionModelPresets';
import { openVisionLabWindow } from '@/utils/visionLabWindow';
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
import type { EditorReferenceKind, EditorReferenceOption, EditorTaskUiVariableOption } from '@/views/script-editor/editorReferences';
import type { EditorTaskMoveAction } from '@/views/script-editor/editorTaskMove';
import {
  createEmptyRelationMap,
  editorModeOptions,
  normalizePolicy,
  normalizePolicyGroup,
  normalizePolicySet,
  reorderCollection,
  type EditorCollectionMoveAction,
  type EditorModeId,
  type EditorNamedItem,
  type PolicyEditorPanelId,
  type RelationEditorPanelId,
} from '@/views/script-editor/editor-policy/editorPolicy';
import {
  createStepFromTemplate,
  isActionSequenceTemplateId,
} from '@/views/script-editor/editor-step/editorStepTemplates';
import {
  createStepList,
  TASK_CYCLE_VALUE,
  TASK_ROW_TYPE,
  TASK_TONE,
  TASK_TRIGGER_MODE,
} from '@/views/script-editor/editor-step/editorStepKinds';
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
  stableStringify,
  type EditorPanelId,
  type EditorUiSchema,
  type RawEditorSection,
  type UiFieldControl,
} from '@/views/script-editor/editorSchema';
import { parseTaskCycleValue } from '@/views/script-editor/editorTaskMeta';
import { createSearchRule } from '@/views/script-editor/editorSearchRule';
import { inputEntryToVariableOption } from '@/views/script-editor/editorInputVariableOptions';
import {
  buildPolicyDraft,
  buildPolicyGroupDraft,
  buildPolicySavePayload,
  buildPolicySetDraft,
  buildPolicySetSavePayload,
  buildPolicyGroupSavePayload,
  buildTaskDraft as createTaskDraft,
  buildTaskSavePayload,
  duplicatePolicyDraft,
  duplicatePolicyGroupDraft,
  duplicatePolicySetDraft,
  duplicateTaskDraft,
  normalizeTask as normalizeTaskDraft,
} from '@/views/script-editor/helpers/scriptEditorTaskDrafts';
import {
  buildVariableUsageMap,
  collectVariableReferencesFromSteps,
  renameInputKeyReferencesInUiData,
  renameVariableReferencesInSteps,
} from '@/views/script-editor/helpers/scriptEditorVariableRefs';
import {
  createInputEntry,
  listAllVariableOptions,
  listVariableOptions,
  parseInputEntries,
  syncInputVariableCatalog,
  type EditorInputType,
  type EditorInputEntry,
  type EditorVariableOption,
} from '@/views/script-editor/editorVariables';
import {
  buildPersistedEditorViewState,
  hydratePolicyStepEditorState,
  hydrateTaskEditorState,
  hydrateTaskStepEditorState,
  resolvePersistedEditorViewState,
  type ScriptEditorPersistedViewState,
} from '@/views/script-editor/helpers/scriptEditorViewState';
import {
  buildScriptEditorSnapshots,
  hasDirtyScriptEditorState,
  loadScriptEditorData,
  savePrimaryScriptEditorData,
} from '@/views/script-editor/helpers/scriptEditorPersistence';
import {
  applyTaskDescription,
  applyTaskDefaultEnabled,
  applyTaskDefaultTaskCycle,
  applyTaskExecMax,
  applyTaskHidden,
  applyTaskIndentLevel,
  applyTaskName,
  applyTaskRecordSchedule,
  applyTaskRowType,
  applyTaskSectionId,
  applyTaskShowEnabledToggle,
  applyTaskTone,
  applyTaskTriggerMode,
  applyTaskUiSchema,
  shouldSkipTaskMetaSync,
  syncTaskInputEntries,
} from '@/views/script-editor/helpers/scriptEditorTaskMeta';
import {
  appendConsoleEntry,
  attachScriptEditorRuntimeListeners,
  buildConsoleTimestamp,
  findStepPathById,
  MAX_CONSOLE_LINES,
  type EditorConsoleEntry,
  type EditorConsoleLevel,
  normalizeConsoleLevel,
  resolveNextPreviewDeviceId,
} from '@/views/script-editor/helpers/scriptEditorRuntime';
import {
  appendRelationId,
  removeRelationId,
  removeRelationIdFromAllOwners,
  removeRelationOwner,
  reorderRelationIds,
  reverseRelationIds,
} from '@/views/script-editor/helpers/scriptEditorRelations';
import {
  buildActiveRunTarget,
  buildRunTargetKey,
  buildRunTargetSelectOptions,
  extractYoloDetector,
  getImgDetLabelSelectPlaceholder,
  getRunSelectionDisabledReason,
  loadImgDetLabelState,
  resolveCurrentEditorRunTargetKey,
} from '@/views/script-editor/helpers/scriptEditorRunTarget';
import {
  buildRawDialogText,
  formatRawDialogText,
  getRawDialogDescription,
  getRawDialogTitle,
  parseRawDialogValue,
  resolveRawEditorSection,
} from '@/views/script-editor/helpers/scriptEditorRawDialog';
import {
  moveCollectionByMenuAction,
  moveTaskByMenuAction,
  reorderItemsById,
  selectNeighborIdAfterRemoval,
} from '@/views/script-editor/helpers/scriptEditorMoves';
import {
  buildVariableReferenceKey,
  createEditorStepId,
  createUniqueVariableStorageKey,
} from '@/views/script-editor/helpers/scriptEditorVariables';
import {
  buildAssignedRelationItems,
  buildPolicyGroupItems,
  buildPolicyGroupReferenceOptions,
  buildPolicyItems,
  buildPolicyReferenceOptions,
  buildPolicySetItems,
  buildPolicySetReferenceOptions,
  buildTaskReferenceOptions,
  buildTaskUiVariableOptions,
  buildUnassignedRelationItems,
} from '@/views/script-editor/helpers/scriptEditorOptions';
import { selectCurrentItem } from '@/views/script-editor/helpers/scriptEditorSelections';

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
const leftSidebarCollapsed = ref(false);
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
const consoleEntries = ref<EditorConsoleEntry[]>([]);
const isConsoleVisible = ref(true);
const selectedPreviewDeviceId = ref<string | null>(null);
const deviceEditorOpen = ref(false);
const editingDeviceId = ref<string | null>(null);
const imgDetLabelOptions = ref<Array<{ label: string; value: number; description?: string }>>([]);
const imgDetLabelHint = ref<string | null>('请先在脚本信息里设置图像检测模型的标签文件。');
const imgDetLabelLoading = ref(false);
const selectedRunTargetKey = ref<string | null>(null);

const sidebarWidth = ref(300);
const configPanelWidth = ref(360);
const isResizingSidebar = ref(false);
const isResizingConfigPanel = ref(false);
let resizeStartX = 0;
let resizeStartWidth = 0;

const startSidebarResize = (e: MouseEvent) => {
  isResizingSidebar.value = true;
  resizeStartX = e.clientX;
  resizeStartWidth = sidebarWidth.value;
};

const startConfigPanelResize = (e: MouseEvent) => {
  isResizingConfigPanel.value = true;
  resizeStartX = e.clientX;
  resizeStartWidth = configPanelWidth.value;
};

const handleMouseMove = (e: MouseEvent) => {
  if (isResizingSidebar.value) {
    const delta = e.clientX - resizeStartX;
    sidebarWidth.value = Math.max(200, Math.min(800, resizeStartWidth + delta));
  } else if (isResizingConfigPanel.value) {
    const delta = e.clientX - resizeStartX;
    configPanelWidth.value = Math.max(250, Math.min(800, resizeStartWidth + delta));
  }
};

const stopResize = () => {
  isResizingSidebar.value = false;
  isResizingConfigPanel.value = false;
};
let detachChildLogListener: null | (() => void) = null;
let detachDeviceProgressListener: null | (() => void) = null;

const taskName = ref('');
const taskDescription = ref('');
const taskRowType = ref<TaskRowType>(TASK_ROW_TYPE.task);
const taskTriggerMode = ref<TaskTriggerMode>(TASK_TRIGGER_MODE.linkOnly);
const taskHidden = ref(false);
const recordSchedule = ref(true);
const sectionId = ref<string | null>(null);
const indentLevel = ref(1);
const defaultTaskCycle = ref<TaskCycle>(TASK_CYCLE_VALUE.everyRun);
const taskExecMax = ref(0);
const showEnabledToggle = ref(true);
const defaultEnabled = ref(true);
const taskTone = ref<TaskTone>(TASK_TONE.normal);
const inputEntries = ref<EditorInputEntry[]>([]);
const inputError = ref<string | null>(null);
const uiSchema = ref<EditorUiSchema>(createUiSchema());

const hydratingTaskMeta = ref(false);
const hydratingTaskPanels = ref(false);

const scriptId = computed(() => (typeof route.query.scriptId === 'string' ? route.query.scriptId : ''));
const editorViewStateStoreKey = computed(() => `${scriptEditorViewStateKey}:${scriptId.value}`);

const loadEditorViewState = async (): Promise<ScriptEditorPersistedViewState | null> => {
  if (!scriptId.value) {
    return null;
  }

  const stored = await getFromStore<ScriptEditorPersistedViewState>(editorViewStateStoreKey.value).catch(() => null);
  return stored && typeof stored === 'object' ? stored : null;
};

const persistEditorViewState = async () => {
  if (!scriptId.value || isLoading.value || loadError.value) {
    return;
  }

  const nextState = buildPersistedEditorViewState({
    activeMode: activeMode.value,
    selectedTaskId: selectedTaskId.value,
    selectedPolicyId: selectedPolicyId.value,
    selectedPolicyGroupId: selectedPolicyGroupId.value,
    selectedPolicySetId: selectedPolicySetId.value,
    activePanel: activePanel.value,
    activePolicyPanel: activePolicyPanel.value,
    selectedStepPath: selectedStepPath.value,
    activeBranchPath: activeBranchPath.value,
    selectedPolicyStepPathBefore: selectedPolicyStepPathBefore.value,
    activePolicyBranchPathBefore: activePolicyBranchPathBefore.value,
    selectedPolicyStepPathAfter: selectedPolicyStepPathAfter.value,
    activePolicyBranchPathAfter: activePolicyBranchPathAfter.value,
  });

  await setToStore(editorViewStateStoreKey.value, nextState);
};

const appendConsoleLine = (
  message: string,
  level: EditorConsoleLevel = 'info',
  time = buildConsoleTimestamp(),
) => {
  consoleEntries.value = appendConsoleEntry(consoleEntries.value, message, level, time);
};

const clearConsole = () => {
  consoleEntries.value = [];
};

const showConsole = () => {
  isConsoleVisible.value = true;
};

const hideConsole = () => {
  isConsoleVisible.value = false;
};

const currentTask = computed<ScriptTaskTable | null>(() => {
  return selectCurrentItem(draftTasks.value as ScriptTaskTable[], selectedTaskId.value);
});
const currentRawEditorSection = computed<RawEditorSection>(() => resolveRawEditorSection(activePanel.value));
const canOpenCurrentRawEditor = computed(() => activeMode.value === 'task' && Boolean(currentTask.value));

const currentPolicy = computed<PolicyTable | null>(() => {
  return selectCurrentItem(draftPolicies.value, selectedPolicyId.value);
});

const currentPolicyGroup = computed<PolicyGroupTable | null>(() => {
  return selectCurrentItem(draftPolicyGroups.value, selectedPolicyGroupId.value);
});

const currentPolicySet = computed<PolicySetTable | null>(() => {
  return selectCurrentItem(draftPolicySets.value, selectedPolicySetId.value);
});

const editingDevice = computed(() => deviceStore.devices.find((device) => device.id === editingDeviceId.value) ?? null);
const selectedPreviewDevice = computed(() => deviceStore.devices.find((device) => device.id === selectedPreviewDeviceId.value) ?? null);
const editingPreviewDeviceBusy = computed(() =>
  editingDevice.value ? deviceStore.isDeviceBusy(editingDevice.value.id) : false,
);
const selectedPreviewDeviceBusy = computed(() =>
  selectedPreviewDeviceId.value ? deviceStore.isDeviceBusy(selectedPreviewDeviceId.value) : false,
);
const selectedPreviewDeviceRuntimeError = computed(() =>
  selectedPreviewDevice.value ? validateDeviceRuntimePlatform(selectedPreviewDevice.value) : null,
);
const deviceSelectOptions = computed(() =>
  deviceStore.devices.map((device) => ({
    label: device.data.deviceName,
    value: device.id,
    description: `${formatConnectLabel(device.data)} · ${formatCaptureMethod(device.data.capMethod)}`,
  })),
);

const imgDetLabelPath = computed(() => extractYoloDetector(draftScript.value?.data.imgDetModel)?.labelPath?.trim() || '');
const imgDetLabelSelectPlaceholder = computed(() =>
  getImgDetLabelSelectPlaceholder(imgDetLabelLoading.value, imgDetLabelOptions.value.length),
);

const runTargetSelectPlaceholder = computed(() => '选择运行目标');

const runTargetSelectOptions = computed(() =>
  //@ts-ignore
  buildRunTargetSelectOptions({
    scriptName: draftScript.value?.data.name,
    scriptId: scriptId.value,
    tasks: draftTasks.value,
    policyItems: policyItems.value,
    policySetItems: policySetItems.value,
  }),
);

const currentEditorRunTargetKey = computed(() =>
  resolveCurrentEditorRunTargetKey({
    scriptId: scriptId.value,
    activeMode: activeMode.value,
    selectedTaskId: selectedTaskId.value,
    selectedPolicyId: selectedPolicyId.value,
    selectedPolicyGroupId: selectedPolicyGroupId.value,
    selectedPolicySetId: selectedPolicySetId.value,
  }),
);

const selectedRunTargetOption = computed(
  () => runTargetSelectOptions.value.find((option) => option.value === selectedRunTargetKey.value) ?? null,
);

const scriptRecoveryTaskOptions = computed(() =>
    //@ts-ignore
  draftTasks.value
    .filter((task) => task.rowType === TASK_ROW_TYPE.task && !task.isDeleted)
    .map((task) => ({
      label: task.name,
      value: task.id,
      description: `任务 ${task.index + 1}`,
    })),
);

const runSelectionDisabledReason = computed(() =>
  getRunSelectionDisabledReason(
    selectedPreviewDeviceId.value,
    selectedRunTargetKey.value,
    selectedPreviewDeviceRuntimeError.value,
  ),
);

const canRunSelection = computed(() => !runSelectionDisabledReason.value);

const variableOptions = computed(() => {
  const options = listVariableOptions(draftScript.value?.data.variableCatalog, currentTask.value?.id ?? null, parsedSteps.value);
  if (!currentTask.value) {
    return options;
  }
  const merged = new Map(options.map((option) => [option.id, option]));
  for (const entry of inputEntries.value) {
    if (!entry.key.trim()) {
      continue;
    }
    merged.set(entry.id, inputEntryToVariableOption(entry, currentTask.value.id));
  }
  return Array.from(merged.values());
});
const catalogVariableOptions = computed(() =>
  listVariableOptions(draftScript.value?.data.variableCatalog, currentTask.value?.id ?? null, parsedSteps.value, 'read', false),
);
const titleTaskOptions = computed(() => [
  {
    label: '未分组',
    value: null,
    description: '直接显示在顶层，不归属到任何标题行。',
  },
  //@ts-ignore
  ...draftTasks.value
    .filter((task) => task.rowType === TASK_ROW_TYPE.title && task.id !== currentTask.value?.id)
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
const currentPolicyAllSteps = computed<Step[]>(() => {
  if (!currentPolicy.value) {
    return [];
  }
  return [
    ...(currentPolicy.value.data.beforeAction as Step[]),
    ...(currentPolicy.value.data.afterAction as Step[]),
  ];
});
const currentPolicySteps = computed<Step[]>(() => {
  if (!currentPolicy.value) {
    return [];
  }
  return currentPolicyStepTarget.value === 'before' ? currentPolicy.value.data.beforeAction : currentPolicy.value.data.afterAction;
});
const currentPolicyConditionCount = computed(() => {
  const condition = currentPolicy.value?.data.cond ?? createSearchRule('group');
  return condition.type === 'group' ? condition.items.length : 1;
});
const selectedPolicyStepPath = computed<StepPath | null>(() =>
  currentPolicyStepTarget.value === 'before' ? selectedPolicyStepPathBefore.value : selectedPolicyStepPathAfter.value,
);
const activePolicyBranchPath = computed<StepBranchPath>(() =>
  currentPolicyStepTarget.value === 'before' ? activePolicyBranchPathBefore.value : activePolicyBranchPathAfter.value,
);
const policyVariableOptions = computed(() =>
  listAllVariableOptions(draftScript.value?.data.variableCatalog, currentPolicyAllSteps.value),
);
const policyCatalogVariableOptions = computed(() =>
  listAllVariableOptions(draftScript.value?.data.variableCatalog, currentPolicyAllSteps.value, 'read', false),
);
const variableUsageMap = computed(() => buildVariableUsageMap(draftTasks.value, draftPolicies.value));
const inputEntryReferenceState = computed<Record<string, { referenced: boolean }>>(() =>
  inputEntries.value.reduce<Record<string, { referenced: boolean }>>((map, entry) => {
    map[entry.id] = {
      referenced: Boolean(variableUsageMap.value[buildVariableReferenceKey(entry.namespace, entry.key)]?.length),
    };
    return map;
  }, {}),
);

const parsedSteps = computed<Step[]>(() => (currentTask.value?.data.steps as Step[] | undefined) ?? []);
const hasValidationErrors = computed(() => Boolean(inputError.value));
const policyItems = computed<EditorNamedItem[]>(() => buildPolicyItems(draftPolicies.value));
const policyGroupItems = computed<EditorNamedItem[]>(() => buildPolicyGroupItems(draftPolicyGroups.value));
const policySetItems = computed<EditorNamedItem[]>(() => buildPolicySetItems(draftPolicySets.value));
const taskReferenceOptions = computed<EditorReferenceOption[]>(() => buildTaskReferenceOptions(draftTasks.value));
const taskUiVariableOptions = computed<EditorTaskUiVariableOption[]>(() =>
  buildTaskUiVariableOptions(draftTasks.value, draftScript.value?.data.variableCatalog),
);
const taskDescriptionMap = computed<Record<string, string>>(() =>
  Object.fromEntries(draftTasks.value.map((task) => [task.id, task.description?.trim() || '未填写说明'])),
);
const policyReferenceOptions = computed<EditorReferenceOption[]>(() => buildPolicyReferenceOptions(draftPolicies.value));
const policyNoteMap = computed<Record<string, string>>(() =>
  Object.fromEntries(draftPolicies.value.map((policy) => [policy.id, policy.data.note?.trim() || '未填写备注'])),
);
const policyGroupReferenceOptions = computed<EditorReferenceOption[]>(() =>
  buildPolicyGroupReferenceOptions(draftPolicyGroups.value, groupPolicyIdsByGroupId.value),
);
const policyGroupNoteMap = computed<Record<string, string>>(() =>
  Object.fromEntries(draftPolicyGroups.value.map((group) => [group.id, group.data.note?.trim() || '未填写备注'])),
);
const policySetReferenceOptions = computed<EditorReferenceOption[]>(() =>
  buildPolicySetReferenceOptions(draftPolicySets.value, setGroupIdsBySetId.value),
);
const policySetNoteMap = computed<Record<string, string>>(() =>
  Object.fromEntries(draftPolicySets.value.map((set) => [set.id, set.data.note?.trim() || '未填写备注'])),
);
const assignedPolicies = computed<EditorNamedItem[]>(() =>
  buildAssignedRelationItems(
    draftPolicies.value,
    currentPolicyGroup.value ? groupPolicyIdsByGroupId.value[currentPolicyGroup.value.id] ?? [] : [],
  ),
);
const unassignedPolicies = computed<EditorNamedItem[]>(() =>
  buildUnassignedRelationItems(
    draftPolicies.value,
    currentPolicyGroup.value ? groupPolicyIdsByGroupId.value[currentPolicyGroup.value.id] ?? [] : [],
  ),
);
const assignedGroups = computed<EditorNamedItem[]>(() =>
  buildAssignedRelationItems(
    draftPolicyGroups.value,
    currentPolicySet.value ? setGroupIdsBySetId.value[currentPolicySet.value.id] ?? [] : [],
  ),
);
const unassignedGroups = computed<EditorNamedItem[]>(() =>
  buildUnassignedRelationItems(
    draftPolicyGroups.value,
    currentPolicySet.value ? setGroupIdsBySetId.value[currentPolicySet.value.id] ?? [] : [],
  ),
);

const dirty = computed(() => {
  if (!draftScript.value) {
    return false;
  }

  return hasDirtyScriptEditorState({
    script: draftScript.value,
    tasks: draftTasks.value,
    policies: draftPolicies.value,
    policyGroups: draftPolicyGroups.value,
    policySets: draftPolicySets.value,
    groupPolicyIdsByGroupId: groupPolicyIdsByGroupId.value,
    setGroupIdsBySetId: setGroupIdsBySetId.value,
    snapshots: {
      script: sourceScriptSnapshot.value,
      tasks: sourceTasksSnapshot.value,
      policies: sourcePoliciesSnapshot.value,
      policyGroups: sourcePolicyGroupsSnapshot.value,
      policySets: sourcePolicySetsSnapshot.value,
      groupPolicies: sourceGroupPoliciesSnapshot.value,
      setGroups: sourceSetGroupsSnapshot.value,
    },
    stableStringify,
  });
});

const formattedSaveTime = computed(() => {
  if (!saveTime.value) {
    return '';
  }

  return new Date(saveTime.value).toLocaleString('zh-CN', {
    hour12: false,
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  });
});
const bypassDirtyExitGuard = ref(false);

const confirmDiscardUnsavedChanges = async () => {
  if (!dirty.value) {
    return true;
  }
  return requestAppConfirm({
    title: '当前内容未保存',
    message: '确认退出？未保存的改动将会丢失。',
    confirmText: '确认',
    cancelText: '取消',
    tone: 'warning',
  });
};

const handleNavigateBack = async () => {
  const approved = await confirmDiscardUnsavedChanges();
  if (!approved) {
    return;
  }
  bypassDirtyExitGuard.value = true;
  await router.push('/scripts');
};

const handleRequestCloseWindow = async () => confirmDiscardUnsavedChanges();

const rawDialogTitle = computed(() => getRawDialogTitle(rawDialogSection.value));

const rawDialogDescription = computed(() => getRawDialogDescription(rawDialogSection.value));

const normalizeTask = (task: ScriptTaskTable, index: number) => normalizeTaskDraft(task, index, scriptId.value);

const buildTaskDraft = (name?: string) =>
  //@ts-ignore
  createTaskDraft({
    name,
    requestUuid: taskService.requestUuid,
    scriptId: scriptId.value,
    tasks: draftTasks.value,
  });

const replaceTask = (taskId: string, updater: (task: ScriptTaskTable) => ScriptTaskTable) => {
  draftTasks.value = draftTasks.value.map((task, index) => {
    if (task.id !== taskId) {
      return normalizeTask(task, index);
    }

    return normalizeTask(updater(cloneJson(task)), index);
  });
};

const replaceCurrentTask = (updater: (task: ScriptTaskTable) => ScriptTaskTable) => {
  const task = currentTask.value;
  if (!task) {
    return;
  }
  replaceTask(task.id, updater);
};

const hydrateTaskEditors = () => {
  hydratingTaskMeta.value = true;
  hydratingTaskPanels.value = true;
  const hydrated = hydrateTaskEditorState({
    currentTask: currentTask.value,
    draftScript: draftScript.value,
    selectedInputId: selectedInputId.value,
    selectedUiFieldId: selectedUiFieldId.value,
    selectedStepPath: selectedStepPath.value,
    activeBranchPath: activeBranchPath.value,
  });

  taskName.value = hydrated.taskName;
  taskDescription.value = hydrated.taskDescription;
  taskRowType.value = hydrated.taskRowType;
  taskTriggerMode.value = hydrated.taskTriggerMode;
  taskHidden.value = hydrated.taskHidden;
  recordSchedule.value = hydrated.recordSchedule;
  sectionId.value = hydrated.sectionId;
  indentLevel.value = hydrated.indentLevel;
  defaultTaskCycle.value = hydrated.defaultTaskCycle;
  taskExecMax.value = hydrated.taskExecMax;
  showEnabledToggle.value = hydrated.showEnabledToggle;
  defaultEnabled.value = hydrated.defaultEnabled;
  taskTone.value = hydrated.taskTone;
  inputEntries.value = hydrated.inputEntries;
  inputError.value = hydrated.inputError;
  selectedInputId.value = hydrated.selectedInputId;
  uiSchema.value = hydrated.uiSchema;
  selectedStepPath.value = hydrated.selectedStepPath;
  activeBranchPath.value = hydrated.activeBranchPath;
  selectedUiFieldId.value = hydrated.selectedUiFieldId;
  if (hydrated.forceBasicPanel) {
    activePanel.value = 'basic';
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

  replaceCurrentTask((task) => {
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

  const duplicate = await duplicateTaskDraft({
    task: source,
    requestUuid: taskService.requestUuid,
    scriptId: scriptId.value,
    tasks: draftTasks.value,
  });

  draftTasks.value = [...draftTasks.value, duplicate].map((task, index) => normalizeTask(task, index));
  selectedTaskId.value = duplicate.id;
};

const removeTask = async (taskId: string) => {
  if (draftTasks.value.length <= 1) {
    showToast('至少保留一个任务，避免脚本变成空壳。', 'warning');
    return;
  }

  const nextSelectedTaskId =
    selectedTaskId.value === taskId ? selectNeighborIdAfterRemoval(draftTasks.value, taskId) : selectedTaskId.value;

  const approved = await requestAppConfirm({
    title: '删除任务',
    message: '确认要删除此任务吗？这将删除该任务下的所有数据',
    confirmText: '删除',
    tone: 'danger',
  });
  if (!approved) {
    return;
  }

  draftTasks.value = draftTasks.value
    .filter((task) => task.id !== taskId)
    .map((task, index) => normalizeTask(task, index));

  if (selectedTaskId.value === taskId) {
    selectedTaskId.value = nextSelectedTaskId;
  }
};

const toggleTaskHidden = (taskId: string) => {
  replaceTask(taskId, (task) => {
    task.isHidden = !task.isHidden;
    return task;
  });
};

const reorderTasks = (draggedTaskId: string, targetTaskId: string) => {
  draftTasks.value = reorderItemsById(draftTasks.value, draggedTaskId, targetTaskId, normalizeTask);
};

const moveTaskByMenu = (taskId: string, action: EditorTaskMoveAction) => {
  draftTasks.value = moveTaskByMenuAction(draftTasks.value, taskId, action, normalizeTask);
};

const createPolicy = async () => {
  const nextPolicy = await buildPolicyDraft({
    requestUuid: taskService.requestUuid,
    scriptId: scriptId.value,
    policies: draftPolicies.value,
  });
  draftPolicies.value = [...draftPolicies.value, nextPolicy].map((policy, index) => normalizePolicy(policy, index));
  selectedPolicyId.value = nextPolicy.id;
  activeMode.value = 'policy';
  activePolicyPanel.value = 'basic';
};

const duplicatePolicy = async (policyId: string) => {
  const source = draftPolicies.value.find((policy) => policy.id === policyId);
  if (!source) {
    return;
  }

  const duplicate = await duplicatePolicyDraft({
    policy: source,
    requestUuid: taskService.requestUuid,
    scriptId: scriptId.value,
    policies: draftPolicies.value,
  });

  draftPolicies.value = [...draftPolicies.value, duplicate].map((policy, index) => normalizePolicy(policy, index));
  selectedPolicyId.value = duplicate.id;
  activeMode.value = 'policy';
  activePolicyPanel.value = 'basic';
};

const createPolicyGroup = async () => {
  const nextGroup = await buildPolicyGroupDraft({
    requestUuid: taskService.requestUuid,
    scriptId: scriptId.value,
    groups: draftPolicyGroups.value,
  });
  draftPolicyGroups.value = [...draftPolicyGroups.value, nextGroup].map((group, index) => normalizePolicyGroup(group, index));
  selectedPolicyGroupId.value = nextGroup.id;
  activeMode.value = 'policyGroup';
  activePolicyGroupPanel.value = 'basic';
};

const duplicatePolicyGroup = async (groupId: string) => {
  const source = draftPolicyGroups.value.find((group) => group.id === groupId);
  if (!source) {
    return;
  }

  const duplicate = await duplicatePolicyGroupDraft({
    group: source,
    requestUuid: taskService.requestUuid,
    scriptId: scriptId.value,
    groups: draftPolicyGroups.value,
    relatedPolicyIds: groupPolicyIdsByGroupId.value[groupId] ?? [],
  });

  draftPolicyGroups.value = [...draftPolicyGroups.value, duplicate.item].map((group, index) => normalizePolicyGroup(group, index));
  groupPolicyIdsByGroupId.value = {
    ...groupPolicyIdsByGroupId.value,
    [duplicate.item.id]: duplicate.relatedPolicyIds,
  };
  selectedPolicyGroupId.value = duplicate.item.id;
  activeMode.value = 'policyGroup';
  activePolicyGroupPanel.value = 'basic';
};

const createPolicySet = async () => {
  const nextSet = await buildPolicySetDraft({
    requestUuid: taskService.requestUuid,
    scriptId: scriptId.value,
    sets: draftPolicySets.value,
  });
  draftPolicySets.value = [...draftPolicySets.value, nextSet].map((set, index) => normalizePolicySet(set, index));
  selectedPolicySetId.value = nextSet.id;
  activeMode.value = 'policySet';
  activePolicySetPanel.value = 'basic';
};

const duplicatePolicySet = async (setId: string) => {
  const source = draftPolicySets.value.find((item) => item.id === setId);
  if (!source) {
    return;
  }

  const duplicate = await duplicatePolicySetDraft({
    set: source,
    requestUuid: taskService.requestUuid,
    scriptId: scriptId.value,
    sets: draftPolicySets.value,
    relatedGroupIds: setGroupIdsBySetId.value[setId] ?? [],
  });

  draftPolicySets.value = [...draftPolicySets.value, duplicate.item].map((item, index) => normalizePolicySet(item, index));
  setGroupIdsBySetId.value = {
    ...setGroupIdsBySetId.value,
    [duplicate.item.id]: duplicate.relatedGroupIds,
  };
  selectedPolicySetId.value = duplicate.item.id;
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
    const nextPolicy = await buildPolicyDraft({
      requestUuid: taskService.requestUuid,
      scriptId: scriptId.value,
      policies: draftPolicies.value,
    });
    draftPolicies.value = [...draftPolicies.value, nextPolicy].map((policy, index) => normalizePolicy(policy, index));
    showToast(`已创建引用策略：${nextPolicy.data.name}`, 'success');
    return nextPolicy.id;
  }

  if (kind === 'policyGroup') {
    const nextGroup = await buildPolicyGroupDraft({
      requestUuid: taskService.requestUuid,
      scriptId: scriptId.value,
      groups: draftPolicyGroups.value,
    });
    draftPolicyGroups.value = [...draftPolicyGroups.value, nextGroup].map((group, index) => normalizePolicyGroup(group, index));
    showToast(`已创建引用策略组：${nextGroup.data.name}`, 'success');
    return nextGroup.id;
  }

  const nextSet = await buildPolicySetDraft({
    requestUuid: taskService.requestUuid,
    scriptId: scriptId.value,
    sets: draftPolicySets.value,
  });
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

type VariableCreateOptions = {
  preferredKey?: string;
  name?: string;
  select?: boolean;
  silent?: boolean;
  sourceStepId?: string | null;
  focusEditor?: boolean;
};

const focusVariableEditor = () => {
  if (activeMode.value === 'policy') {
    activePolicyPanel.value = 'inputs';
    return;
  }
  activePanel.value = 'inputs';
};

const createVariableEntry = (
  namespace: EditorInputEntry['namespace'],
  inputType: EditorInputType,
  options: VariableCreateOptions = {},
) => {
  const trimmedPreferred = options.preferredKey?.trim().replace(/^(input|runtime|system)\./, '') ?? '';
  if (trimmedPreferred) {
    const existingEntry = inputEntries.value.find((entry) => entry.namespace === namespace && entry.key.trim() === trimmedPreferred) ?? null;
    if (existingEntry) {
      if (options.select !== false) {
        selectedInputId.value = existingEntry.id;
      }
      if (options.focusEditor) {
        focusVariableEditor();
      }
      return buildVariableReferenceKey(namespace, trimmedPreferred);
    }
  }

  const nextKey = createUniqueVariableStorageKey(inputEntries.value, namespace, options.preferredKey);
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
  if (options.focusEditor) {
    focusVariableEditor();
  }
  if (!options.silent) {
    showToast(`已创建变量：${namespace}.${nextKey}`, 'success');
  }
  return `${namespace}.${nextKey}`;
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
          uiData: renameInputKeyReferencesInUiData(task.data.uiData, previousKey, nextKey),
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
  return createVariableEntry(namespace, inputType, options);
};

const jumpToVariableResource = (option: EditorVariableOption) => {
  if (option.namespace === 'system') {
    showToast('系统变量由运行时注入，当前不可在编辑器中修改。', 'warning');
    return;
  }

  if (option.sourceType === 'manual') {
    if (activeMode.value === 'policy') {
      activePolicyPanel.value = 'inputs';
      selectedInputId.value = option.id;
      return;
    }

    activeMode.value = 'task';
    activePanel.value = 'inputs';
    selectedInputId.value = option.id;
    return;
  }

  if (option.sourceStepId && activeMode.value === 'policy' && currentPolicy.value) {
    const beforePath = findStepPathById(currentPolicy.value.data.beforeAction as Step[], option.sourceStepId);
    if (beforePath) {
      activePolicyPanel.value = 'before';
      selectedPolicyStepPathBefore.value = beforePath;
      activePolicyBranchPathBefore.value = getParentBranchPath(beforePath);
      return;
    }

    const afterPath = findStepPathById(currentPolicy.value.data.afterAction as Step[], option.sourceStepId);
    if (afterPath) {
      activePolicyPanel.value = 'after';
      selectedPolicyStepPathAfter.value = afterPath;
      activePolicyBranchPathAfter.value = getParentBranchPath(afterPath);
      return;
    }
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

  if (option.sourceStepId) {
    const path = findStepPathById(matchedTask.data.steps as Step[], option.sourceStepId);
    if (path) {
      activePanel.value = 'steps';
      selectedStepPath.value = path;
      activeBranchPath.value = getParentBranchPath(path);
      showToast('已定位到变量来源任务的步骤工作区。', 'success');
      return;
    }
  }

  activePanel.value = 'steps';
  showToast('已定位到变量来源任务的步骤工作区。', 'success');
};

const removePolicy = (policyId: string) => {
  const nextSelectedPolicyId =
    selectedPolicyId.value === policyId ? selectNeighborIdAfterRemoval(draftPolicies.value, policyId) : selectedPolicyId.value;
  draftPolicies.value = draftPolicies.value.filter((item) => item.id !== policyId).map((item, index) => normalizePolicy(item, index));
  groupPolicyIdsByGroupId.value = removeRelationIdFromAllOwners(groupPolicyIdsByGroupId.value, policyId);
  if (selectedPolicyId.value === policyId) {
    selectedPolicyId.value = nextSelectedPolicyId;
  }
};

const removePolicyGroup = (groupId: string) => {
  const nextSelectedPolicyGroupId =
    selectedPolicyGroupId.value === groupId
      ? selectNeighborIdAfterRemoval(draftPolicyGroups.value, groupId)
      : selectedPolicyGroupId.value;
  draftPolicyGroups.value = draftPolicyGroups.value.filter((item) => item.id !== groupId).map((item, index) => normalizePolicyGroup(item, index));
  groupPolicyIdsByGroupId.value = removeRelationOwner(groupPolicyIdsByGroupId.value, groupId);
  setGroupIdsBySetId.value = removeRelationIdFromAllOwners(setGroupIdsBySetId.value, groupId);
  if (selectedPolicyGroupId.value === groupId) {
    selectedPolicyGroupId.value = nextSelectedPolicyGroupId;
  }
};

const removePolicySet = (setId: string) => {
  const nextSelectedPolicySetId =
    selectedPolicySetId.value === setId ? selectNeighborIdAfterRemoval(draftPolicySets.value, setId) : selectedPolicySetId.value;
  draftPolicySets.value = draftPolicySets.value.filter((item) => item.id !== setId).map((item, index) => normalizePolicySet(item, index));
  setGroupIdsBySetId.value = removeRelationOwner(setGroupIdsBySetId.value, setId);
  if (selectedPolicySetId.value === setId) {
    selectedPolicySetId.value = nextSelectedPolicySetId;
  }
};

const reorderPolicies = (draggedId: string, targetId: string) => {
  draftPolicies.value = reorderItemsById(draftPolicies.value, draggedId, targetId, normalizePolicy);
};

const movePolicyByMenu = (policyId: string, action: EditorCollectionMoveAction) => {
  draftPolicies.value = moveCollectionByMenuAction(draftPolicies.value, policyId, action, normalizePolicy);
};

const reorderPolicyGroups = (draggedId: string, targetId: string) => {
  draftPolicyGroups.value = reorderItemsById(draftPolicyGroups.value, draggedId, targetId, normalizePolicyGroup);
};

const movePolicyGroupByMenu = (groupId: string, action: EditorCollectionMoveAction) => {
  draftPolicyGroups.value = moveCollectionByMenuAction(draftPolicyGroups.value, groupId, action, normalizePolicyGroup);
};

const reorderPolicySets = (draggedId: string, targetId: string) => {
  draftPolicySets.value = reorderItemsById(draftPolicySets.value, draggedId, targetId, normalizePolicySet);
};

const movePolicySetByMenu = (setId: string, action: EditorCollectionMoveAction) => {
  draftPolicySets.value = moveCollectionByMenuAction(draftPolicySets.value, setId, action, normalizePolicySet);
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

const updatePolicyNumberField = (field: 'curPos' | 'execMax', value: string) => {
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

const loadImgDetLabels = async (path: string) => {
  imgDetLabelLoading.value = true;

  try {
    const nextState = await loadImgDetLabelState({
      path,
      getYoloLabels: scriptService.getYoloLabels,
    });
    imgDetLabelOptions.value = nextState.options;
    imgDetLabelHint.value = nextState.hint;
    appendConsoleLine(nextState.logMessage, nextState.logLevel);
  } catch (error) {
    console.error(error);
    imgDetLabelOptions.value = [];
    imgDetLabelHint.value = error instanceof Error ? `标签文件读取失败：${error.message}` : '标签文件读取失败，请检查路径和格式。';
    appendConsoleLine(`图像检测标签加载失败：${error instanceof Error ? error.message : '未知错误'}`, 'error');
  } finally {
    imgDetLabelLoading.value = false;
  }
};

const openDeviceEditor = (deviceId: string | null) => {
  if (deviceId && deviceStore.isDeviceBusy(deviceId)) {
    return;
  }
  editingDeviceId.value = deviceId;
  deviceEditorOpen.value = true;
};

const savePreviewDevice = async (form: DeviceFormState) => {
  try {
    const device = await buildDeviceTableFromForm(form, settingsStore.preferences);
    await deviceStore.saveDevice(device);
    deviceEditorOpen.value = false;
    selectedPreviewDeviceId.value = device.id;
    appendConsoleLine(`设备已保存：${device.data.deviceName}`);
    showToast('设备已保存', 'success');
  } catch (error) {
    const message = toErrorText(error).trim() || '设备保存失败';
    console.error('[device save] 设备保存失败', error);
    appendConsoleLine(`设备保存失败：${message}`, 'error');
    showToast(message, 'error');
  }
};

const handleRunSelection = async () => {
  if (!selectedPreviewDevice.value || !selectedPreviewDeviceId.value || !selectedRunTargetKey.value) {
    showToast('请先选择设备和目标对象。', 'warning');
    return;
  }

  if (selectedPreviewDeviceRuntimeError.value) {
    appendConsoleLine(`运行前校验失败：${selectedPreviewDeviceRuntimeError.value}`, 'warning');
    showToast(selectedPreviewDeviceRuntimeError.value, 'warning');
    return;
  }

  const runTarget = buildActiveRunTarget({
    scriptId: scriptId.value,
    selectedRunTargetKey: selectedRunTargetKey.value,
    createFullScriptRunTarget,
    createPolicyRunTarget,
    createPolicyGroupRunTarget,
    createPolicySetRunTarget,
    createTaskRunTarget,
  });
  if (!runTarget) {
    showToast('当前目标对象无法转换为运行目标。', 'error');
    return;
  }

  if (dirty.value) {
    appendConsoleLine('运行前检测到未保存改动，先保存当前脚本结构。', 'warning');
    await saveEditor();
    if (dirty.value) {
      appendConsoleLine('运行已取消：脚本草稿仍未保存。', 'warning');
      return;
    }
  }

  appendConsoleLine(
    `请求运行：设备=${selectedPreviewDevice.value.data.deviceName}，目标=${selectedRunTargetOption.value?.label || selectedRunTargetKey.value}`,
  );

  try {
    await deviceService.updateChildLogLevel(selectedPreviewDeviceId.value, 'Debug');
    const result = await runtimeService.runScriptTarget(selectedPreviewDeviceId.value, runTarget);
    await deviceStore.refreshRunningDevices();
    appendConsoleLine(result);
    showToast('运行命令已发送', 'success');
  } catch (error) {
    const message = toErrorText(error).trim() || '运行命令发送失败';
    appendConsoleLine(`运行失败：${message}`, 'error');
    showToast(message, 'error');
  }
};

const linkPolicyToGroup = (policyId: string) => {
  if (!currentPolicyGroup.value) return;
  groupPolicyIdsByGroupId.value = appendRelationId(groupPolicyIdsByGroupId.value, currentPolicyGroup.value.id, policyId);
};

const unlinkPolicyFromGroup = (policyId: string) => {
  if (!currentPolicyGroup.value) return;
  groupPolicyIdsByGroupId.value = removeRelationId(groupPolicyIdsByGroupId.value, currentPolicyGroup.value.id, policyId);
};

const reorderGroupPolicies = (draggedId: string, targetId: string) => {
  if (!currentPolicyGroup.value) return;
  groupPolicyIdsByGroupId.value = reorderRelationIds(
    groupPolicyIdsByGroupId.value,
    currentPolicyGroup.value.id,
    draggedId,
    targetId,
  );
};

const reverseGroupPolicies = () => {
  if (!currentPolicyGroup.value) return;
  groupPolicyIdsByGroupId.value = reverseRelationIds(groupPolicyIdsByGroupId.value, currentPolicyGroup.value.id);
};

const locatePolicy = (policyId: string) => {
  activeMode.value = 'policy';
  selectedPolicyId.value = policyId;
};

const linkGroupToSet = (groupId: string) => {
  if (!currentPolicySet.value) return;
  setGroupIdsBySetId.value = appendRelationId(setGroupIdsBySetId.value, currentPolicySet.value.id, groupId);
};

const unlinkGroupFromSet = (groupId: string) => {
  if (!currentPolicySet.value) return;
  setGroupIdsBySetId.value = removeRelationId(setGroupIdsBySetId.value, currentPolicySet.value.id, groupId);
};

const reorderSetGroups = (draggedId: string, targetId: string) => {
  if (!currentPolicySet.value) return;
  setGroupIdsBySetId.value = reorderRelationIds(setGroupIdsBySetId.value, currentPolicySet.value.id, draggedId, targetId);
};

const locatePolicyGroup = (groupId: string) => {
  activeMode.value = 'policyGroup';
  selectedPolicyGroupId.value = groupId;
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
  const targetEntry = inputEntries.value.find((entry) => entry.id === entryId) ?? null;
  if (!targetEntry) {
    return;
  }

  const references = variableUsageMap.value[buildVariableReferenceKey(targetEntry.namespace, targetEntry.key)] ?? [];
  if (references.length) {
    void requestAppConfirm({
      title: '无法删除变量',
      message: `该变量被${references[0]}引用，无法删除。若要删除请先删除该步骤。`,
      confirmText: '知道了',
      cancelText: '关闭',
      tone: 'warning',
    });
    return;
  }

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
      preferredKey: 'newVar',
      name: '新变量',
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

  if (templateId === 'color-compare' && nextStep.op === 'dataHanding' && nextStep.a.type === 'colorCompare') {
    nextStep.a.input_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'ocrResults',
      name: 'OCR结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    nextStep.a.out_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'colorMatchedResults',
      name: '颜色筛选结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'vision-search' && nextStep.op === 'vision' && nextStep.a.type === 'visionSearch') {
    nextStep.a.out_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'searchHits',
      name: '搜索命中',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'vision-detect' && nextStep.op === 'vision' && nextStep.a.type === 'detect') {
    nextStep.a.input_var = await createVariableResource('runtime', 'image', {
      preferredKey: 'captureResult',
      name: '截图结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    nextStep.a.out_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'detResults',
      name: '检测结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'vision-ocr' && nextStep.op === 'vision' && nextStep.a.type === 'ocr') {
    nextStep.a.input_var = await createVariableResource('runtime', 'image', {
      preferredKey: 'captureResult',
      name: '截图结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    nextStep.a.out_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'ocrResults',
      name: 'OCR结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'click-text' && nextStep.op === 'action' && nextStep.a.ac === 'click' && nextStep.a.mode === 'txt') {
    nextStep.a.input_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'searchHits',
      name: '搜索命中',
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

  if (templateId === 'search-policy-set-text' && nextStep.op === 'flowControl' && nextStep.a.type === 'searchPolicySetText') {
    nextStep.a.ocr_input_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'ocrResults',
      name: 'OCR结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    nextStep.a.out_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'searchHits',
      name: '搜索命中',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    return nextStep;
  }

  if (templateId === 'handle-policy-set' && nextStep.op === 'flowControl' && nextStep.a.type === 'handlePolicySet') {
    nextStep.a.det_input_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'detResults',
      name: '检测结果',
      select: false,
      silent: true,
      sourceStepId: nextStep.id,
    });
    nextStep.a.search_hits_var = await createVariableResource('runtime', 'json', {
      preferredKey: 'searchHits',
      name: '搜索命中',
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
      preferredKey: 'captureResult',
      name: '截图结果',
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

const actionSequenceTemplateBlockedMessage = '动作序列里只允许可直接下发的设备动作和显式等待。';

const appendTemplateStep = async (templateId: string) => {
  if (activeBranchPath.value.branch === 'sequence' && !isActionSequenceTemplateId(templateId)) {
    showToast(actionSequenceTemplateBlockedMessage, 'warning');
    return;
  }

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
  const nextSteps = updateBranchSteps(parsedSteps.value, activeBranchPath.value, (steps) => steps.filter((_, stepIndex) => stepIndex !== index));
  setCurrentTaskSteps(nextSteps);
  selectedStepPath.value = createSiblingSelection(activeBranchPath.value, getBranchSteps(nextSteps, activeBranchPath.value).length, index);
};

const updateStep = (index: number, nextStep: Step) => {
  const nextSteps = updateStepByPath(parsedSteps.value, buildStepPath(activeBranchPath.value, index), () => nextStep);
  setCurrentTaskSteps(nextSteps);
  selectedStepPath.value = buildStepPath(activeBranchPath.value, index);
};

const appendPolicyTemplateStep = async (templateId: string) => {
  if (activePolicyBranchPath.value.branch === 'sequence' && !isActionSequenceTemplateId(templateId)) {
    showToast(actionSequenceTemplateBlockedMessage, 'warning');
    return;
  }

  const templateStep = createStepFromTemplate(templateId);
  const step = templateStep ? await bindTemplateVariableDefaults(templateId, templateStep) : null;
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
  rawDialogText.value = buildRawDialogText(section, currentTask.value);
  rawDialogOpen.value = true;
};

const openCurrentRawEditor = () => {
  if (!canOpenCurrentRawEditor.value) {
    return;
  }
  openRawEditor(currentRawEditorSection.value);
};

const formatRawEditor = () => {
  try {
    rawDialogText.value = formatRawDialogText(rawDialogText.value);
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
    const parsed = parseRawDialogValue(rawDialogSection.value, rawDialogText.value);

    replaceCurrentTask((task) => {
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

const handleOpenVisionLab = async () => {
  if (!draftScript.value) {
    showToast('当前没有可用的脚本草稿', 'warning');
    return;
  }

  try {
    const scriptId = draftScript.value.id;
    const isPublished = draftScript.value.data.scriptType === 'published';
    await openVisionLabWindow({
      source: 'editor',
      scriptId,
      scriptName: draftScript.value.data.name || null,
      selectedDeviceId: selectedPreviewDeviceId.value,
      imgDetModel: isPublished
        ? rewritePublishedDetectorModelPath(cloneJson(draftScript.value.data.imgDetModel), scriptId, 'img_det_model.onnx')
        : cloneJson(draftScript.value.data.imgDetModel),
      txtDetModel: isPublished
        ? rewritePublishedDetectorModelPath(cloneJson(draftScript.value.data.txtDetModel), scriptId, 'txt_det_model.onnx')
        : cloneJson(draftScript.value.data.txtDetModel),
      txtRecModel: isPublished
        ? rewritePublishedRecognizerModelPath(cloneJson(draftScript.value.data.txtRecModel), scriptId)
        : cloneJson(draftScript.value.data.txtRecModel),
      createdAt: new Date().toISOString(),
    });
  } catch (error) {
    showToast(error instanceof Error ? error.message : '打开视觉测试窗口失败', 'error');
  }
};

const buildSavePayload = () => buildTaskSavePayload(draftTasks.value, scriptId.value);

const buildPolicyPayload = () => buildPolicySavePayload(draftPolicies.value, scriptId.value);

const buildPolicyGroupPayload = () => buildPolicyGroupSavePayload(draftPolicyGroups.value, scriptId.value);

const buildPolicySetPayload = () => buildPolicySetSavePayload(draftPolicySets.value, scriptId.value);

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
    await savePrimaryScriptEditorData({
      script,
      tasks,
      policies,
      policyGroups,
      policySets,
      groupPolicyIdsByGroupId: groupPolicyIdsByGroupId.value,
      setGroupIdsBySetId: setGroupIdsBySetId.value,
      saveScriptEditorBundle: scriptService.saveEditorBundle,
    });
    await scriptStore.loadScripts();
    scriptStore.selectScript(script.id);

    draftTasks.value = tasks;
    draftPolicies.value = policies;
    draftPolicyGroups.value = policyGroups;
    draftPolicySets.value = policySets;
    draftScript.value = script;
    const snapshots = buildScriptEditorSnapshots({
      script,
      tasks,
      policies,
      policyGroups,
      policySets,
      groupPolicyIdsByGroupId: groupPolicyIdsByGroupId.value,
      setGroupIdsBySetId: setGroupIdsBySetId.value,
      stableStringify,
    });
    sourceTasksSnapshot.value = snapshots.tasks;
    sourcePoliciesSnapshot.value = snapshots.policies;
    sourcePolicyGroupsSnapshot.value = snapshots.policyGroups;
    sourcePolicySetsSnapshot.value = snapshots.policySets;
    sourceGroupPoliciesSnapshot.value = snapshots.groupPolicies;
    sourceSetGroupsSnapshot.value = snapshots.setGroups;
    sourceScriptSnapshot.value = snapshots.script;
    saveTime.value = nextSaveTime;
    appendConsoleLine(`脚本结构已保存：${script.data.name || script.id}`);
    showToast('脚本编辑结果已保存', 'success');
  } catch (error) {
    let msg = `脚本保存失败,${error instanceof Error ? error.message : '未知错误'}`;
    showToast(msg, 'error',5000);
    appendConsoleLine(msg, 'error');
    console.log(error);
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

    const loaded = await loadScriptEditorData({
      sourceScript,
      loadScriptTasks: scriptStore.loadScriptTasks,
      listPolicies: scriptService.listPolicies,
      listPolicyGroups: scriptService.listPolicyGroups,
      listPolicySets: scriptService.listPolicySets,
      getGroupPolicies: scriptService.getGroupPolicies,
      getSetGroups: scriptService.getSetGroups,
      normalizeTask,
      buildTaskDraft,
      normalizePolicy,
      normalizePolicyGroup,
      normalizePolicySet,
      cloneScript: cloneJson,
      stableStringify,
    });
    draftScript.value = loaded.draftScript;
    draftTasks.value = loaded.draftTasks;
    draftPolicies.value = loaded.draftPolicies;
    draftPolicyGroups.value = loaded.draftPolicyGroups;
    draftPolicySets.value = loaded.draftPolicySets;
    groupPolicyIdsByGroupId.value = loaded.groupPolicyIdsByGroupId;
    setGroupIdsBySetId.value = loaded.setGroupIdsBySetId;
    sourceScriptSnapshot.value = loaded.snapshots.script;
    sourceTasksSnapshot.value = loaded.snapshots.tasks;
    sourcePoliciesSnapshot.value = loaded.snapshots.policies;
    sourcePolicyGroupsSnapshot.value = loaded.snapshots.policyGroups;
    sourcePolicySetsSnapshot.value = loaded.snapshots.policySets;
    sourceGroupPoliciesSnapshot.value = loaded.snapshots.groupPolicies;
    sourceSetGroupsSnapshot.value = loaded.snapshots.setGroups;

    const persistedViewState = await loadEditorViewState();
    const restoredViewState = resolvePersistedEditorViewState({
      persistedViewState,
      draftTasks: draftTasks.value,
      draftPolicies: draftPolicies.value,
      draftPolicyGroups: draftPolicyGroups.value,
      draftPolicySets: draftPolicySets.value,
    });
    activeMode.value = restoredViewState.activeMode;
    activePanel.value = restoredViewState.activePanel;
    activePolicyPanel.value = restoredViewState.activePolicyPanel;
    selectedTaskId.value = restoredViewState.selectedTaskId;
    selectedPolicyId.value = restoredViewState.selectedPolicyId;
    selectedPolicyGroupId.value = restoredViewState.selectedPolicyGroupId;
    selectedPolicySetId.value = restoredViewState.selectedPolicySetId;
    selectedStepPath.value = restoredViewState.selectedStepPath;
    activeBranchPath.value = restoredViewState.activeBranchPath;
    selectedPolicyStepPathBefore.value = restoredViewState.selectedPolicyStepPathBefore;
    activePolicyBranchPathBefore.value = restoredViewState.activePolicyBranchPathBefore;
    selectedPolicyStepPathAfter.value = restoredViewState.selectedPolicyStepPathAfter;
    activePolicyBranchPathAfter.value = restoredViewState.activePolicyBranchPathAfter;
    saveTime.value = loaded.saveTime;
    hydrateTaskEditors();
    appendConsoleLine(`已载入脚本：${sourceScript.data.name}`);
  } catch (error) {
    console.error(error);
    loadError.value = error instanceof Error ? error.message : '脚本编辑器初始化失败';
    appendConsoleLine(`编辑器载入失败：${loadError.value}`, 'error');
  } finally {
    isLoading.value = false;
  }
};

const handleKeydown = (event: KeyboardEvent) => {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
    event.preventDefault();
    if (!isSaving.value) {
      void saveEditor();
    }
  }
};

const hydrateTaskStepEditors = () => {
  const hydrated = hydrateTaskStepEditorState({
    currentTask: currentTask.value,
    selectedStepPath: selectedStepPath.value,
    activeBranchPath: activeBranchPath.value,
  });
  selectedStepPath.value = hydrated.selectedStepPath;
  activeBranchPath.value = hydrated.activeBranchPath;
};

const hydratePolicyStepEditors = () => {
  const hydrated = hydratePolicyStepEditorState({
    currentPolicy: currentPolicy.value,
    selectedPolicyStepPathBefore: selectedPolicyStepPathBefore.value,
    selectedPolicyStepPathAfter: selectedPolicyStepPathAfter.value,
    activePolicyBranchPathBefore: activePolicyBranchPathBefore.value,
    activePolicyBranchPathAfter: activePolicyBranchPathAfter.value,
  });
  selectedPolicyStepPathBefore.value = hydrated.selectedPolicyStepPathBefore;
  activePolicyBranchPathBefore.value = hydrated.activePolicyBranchPathBefore;
  selectedPolicyStepPathAfter.value = hydrated.selectedPolicyStepPathAfter;
  activePolicyBranchPathAfter.value = hydrated.activePolicyBranchPathAfter;
};

watch(
  () => currentTask.value?.id,
  () => {
    hydrateTaskEditors();
  },
  { immediate: true },
);

watch(
  () => currentTask.value?.id,
  () => {
    hydrateTaskStepEditors();
  },
  { immediate: true, flush: 'sync' },
);

watch(
  () => currentPolicy.value?.id,
  () => {
    hydratePolicyStepEditors();
  },
  { immediate: true, flush: 'sync' },
);

watch(activePolicyPanel, (panel) => {
  if (panel === 'inputs' && !inputEntries.value.find((entry) => entry.id === selectedInputId.value)) {
    selectedInputId.value = inputEntries.value[0]?.id ?? null;
  }
});

watch(
  [imgDetLabelPath, isLoading],
  ([path, loading]) => {
    if (loading) {
      return;
    }
    void loadImgDetLabels(path);
  },
  { immediate: true },
);

watch(
  () => deviceStore.devices.map((device) => device.id).join('|'),
  async () => {
    const storedDeviceId = await getFromStore<string>(deviceKey).catch(() => null);
    selectedPreviewDeviceId.value = resolveNextPreviewDeviceId({
      currentSelectedDeviceId: selectedPreviewDeviceId.value,
      storedDeviceId: storedDeviceId ?? null,
      availableDeviceIds: deviceStore.devices.map((device) => device.id),
    });
  },
  { immediate: true },
);

watch(
  selectedPreviewDeviceId,
  (value) => {
    if (!value) {
      return;
    }
    void setToStore(deviceKey, value);
  },
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

watch(
  [
    activeMode,
    activePanel,
    activePolicyPanel,
    selectedTaskId,
    selectedPolicyId,
    selectedPolicyGroupId,
    selectedPolicySetId,
    selectedStepPath,
    activeBranchPath,
    selectedPolicyStepPathBefore,
    activePolicyBranchPathBefore,
    selectedPolicyStepPathAfter,
    activePolicyBranchPathAfter,
  ],
  () => {
    void persistEditorViewState();
  },
  { deep: true },
);

watch(taskName, (value) => {
  if (shouldSkipTaskMetaSync({ hasCurrentTask: Boolean(currentTask.value), hydrating: hydratingTaskMeta.value })) {
    return;
  }

  replaceCurrentTask((task) => {
    return applyTaskName(task, value);
  });
});

watch(taskDescription, (value) => {
  if (shouldSkipTaskMetaSync({ hasCurrentTask: Boolean(currentTask.value), hydrating: hydratingTaskMeta.value })) {
    return;
  }

  replaceCurrentTask((task) => {
    return applyTaskDescription(task, value);
  });
});

watch(taskRowType, (value) => {
  if (shouldSkipTaskMetaSync({ hasCurrentTask: Boolean(currentTask.value), hydrating: hydratingTaskMeta.value })) {
    return;
  }

  replaceCurrentTask((task) => {
    const next = applyTaskRowType(task, value);
    if (next.forceBasicPanel) {
      activePanel.value = 'basic';
    }
    return next.task;
  });
});

watch(taskTriggerMode, (value) => {
  if (shouldSkipTaskMetaSync({
    hasCurrentTask: Boolean(currentTask.value),
    hydrating: hydratingTaskMeta.value,
    rowType: taskRowType.value,
    allowTitle: false,
  })) {
    return;
  }

  replaceCurrentTask((task) => {
    return applyTaskTriggerMode(task, value);
  });
});

watch(taskHidden, (value) => {
  if (shouldSkipTaskMetaSync({ hasCurrentTask: Boolean(currentTask.value), hydrating: hydratingTaskMeta.value })) {
    return;
  }

  replaceCurrentTask((task) => {
    return applyTaskHidden(task, value);
  });
});

watch(recordSchedule, (value) => {
  if (shouldSkipTaskMetaSync({
    hasCurrentTask: Boolean(currentTask.value),
    hydrating: hydratingTaskMeta.value,
    rowType: taskRowType.value,
    allowTitle: false,
  })) {
    return;
  }

  replaceCurrentTask((task) => {
    return applyTaskRecordSchedule(task, value);
  });
});

watch(sectionId, (value) => {
  if (shouldSkipTaskMetaSync({
    hasCurrentTask: Boolean(currentTask.value),
    hydrating: hydratingTaskMeta.value,
    rowType: taskRowType.value,
    allowTitle: false,
  })) {
    return;
  }

  replaceCurrentTask((task) => {
    return applyTaskSectionId(task, value);
  });
});

watch(indentLevel, (value) => {
  if (shouldSkipTaskMetaSync({
    hasCurrentTask: Boolean(currentTask.value),
    hydrating: hydratingTaskMeta.value,
    rowType: taskRowType.value,
    allowTitle: false,
  })) {
    return;
  }

  replaceCurrentTask((task) => {
    return applyTaskIndentLevel(task, value);
  });
});

watch(defaultTaskCycle, (value) => {
  if (shouldSkipTaskMetaSync({
    hasCurrentTask: Boolean(currentTask.value),
    hydrating: hydratingTaskMeta.value,
    rowType: taskRowType.value,
    allowTitle: false,
  })) {
    return;
  }

  replaceCurrentTask((task) => {
    return applyTaskDefaultTaskCycle(task, value);
  });
});

watch(taskExecMax, (value) => {
  if (shouldSkipTaskMetaSync({
    hasCurrentTask: Boolean(currentTask.value),
    hydrating: hydratingTaskMeta.value,
    rowType: taskRowType.value,
    allowTitle: false,
  })) {
    return;
  }

  replaceCurrentTask((task) => {
    return applyTaskExecMax(task, value);
  });
});

watch(showEnabledToggle, (value) => {
  if (shouldSkipTaskMetaSync({
    hasCurrentTask: Boolean(currentTask.value),
    hydrating: hydratingTaskMeta.value,
    rowType: taskRowType.value,
    allowTitle: false,
  })) {
    return;
  }

  replaceCurrentTask((task) => {
    return applyTaskShowEnabledToggle(task, value);
  });
});

watch(defaultEnabled, (value) => {
  if (shouldSkipTaskMetaSync({
    hasCurrentTask: Boolean(currentTask.value),
    hydrating: hydratingTaskMeta.value,
    rowType: taskRowType.value,
    allowTitle: false,
  })) {
    return;
  }

  replaceCurrentTask((task) => {
    return applyTaskDefaultEnabled(task, value);
  });
});

watch(taskTone, (value) => {
  if (shouldSkipTaskMetaSync({
    hasCurrentTask: Boolean(currentTask.value),
    hydrating: hydratingTaskMeta.value,
    rowType: taskRowType.value,
    allowTitle: false,
  })) {
    return;
  }

  replaceCurrentTask((task) => {
    return applyTaskTone(task, value);
  });
});

watch(
  inputEntries,
  (entries) => {
    if (hydratingTaskPanels.value) {
      return;
    }

    try {
      const nextState = syncTaskInputEntries({
        draftScript: draftScript.value,
        currentTask: currentTask.value,
        uiSchema: uiSchema.value,
        entries,
      });
      inputError.value = null;
      draftScript.value = nextState.draftScript;
      uiSchema.value = nextState.uiSchema;
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

    replaceCurrentTask((task) => {
      return applyTaskUiSchema(task, value);
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

onBeforeRouteLeave(async () => {
  if (bypassDirtyExitGuard.value) {
    bypassDirtyExitGuard.value = false;
    return true;
  }
  return confirmDiscardUnsavedChanges();
});

watch(
  [currentEditorRunTargetKey, runTargetSelectOptions],
  () => {
    const hasCurrent = runTargetSelectOptions.value.some((option) => option.value === selectedRunTargetKey.value);
    if (hasCurrent) {
      return;
    }
    selectedRunTargetKey.value = currentEditorRunTargetKey.value;
  },
  { immediate: true },
);

onMounted(() => {
  window.addEventListener('keydown', handleKeydown);
  window.addEventListener('mousemove', handleMouseMove);
  window.addEventListener('mouseup', stopResize);
  void deviceStore.initIpcListeners();
  void Promise.all([deviceStore.refreshAll(), settingsStore.loadPreferences()]);
  void attachScriptEditorRuntimeListeners({
    getSelectedPreviewDeviceId: () => selectedPreviewDeviceId.value,
    getScriptId: () => scriptId.value,
    getDraftTasks: () => draftTasks.value,
    getDraftPolicies: () => draftPolicies.value,
    appendConsoleLine,
  }).then((listeners) => {
    detachChildLogListener = listeners.detachChildLogListener;
    detachDeviceProgressListener = listeners.detachDeviceProgressListener;
  });
});

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeydown);
  window.removeEventListener('mousemove', handleMouseMove);
  window.removeEventListener('mouseup', stopResize);
  detachChildLogListener?.();
  detachChildLogListener = null;
  detachDeviceProgressListener?.();
  detachDeviceProgressListener = null;
});
</script>

<style scoped src="./script-editor/ScriptEditor.css"></style>
