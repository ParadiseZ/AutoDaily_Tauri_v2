<template>
  <SurfacePanel padding="sm" class="flex h-full min-h-0 flex-col gap-4 overflow-hidden">
    <template v-if="task">
      <div class="flex items-start justify-between gap-3">
        <div class="space-y-1">
          <p class="text-xs uppercase tracking-[0.18em] text-[var(--app-text-faint)]">Workspace</p>
          <h2 class="text-xl font-semibold text-[var(--app-text-strong)]">{{ workspaceTitle }}</h2>
        </div>
        <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('open-raw', rawSection)">
          查看底层结构
        </button>
      </div>

      <div v-if="activePanel === 'inputs'" class="grid min-h-0 gap-4 xl:grid-cols-[minmax(0,1fr)_280px]">
        <div class="min-h-0 space-y-3 overflow-y-auto pr-1 custom-scrollbar">
          <article
            v-for="(entry, index) in inputEntries"
            :key="entry.id"
            class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4"
          >
            <div class="grid gap-3">
              <div class="grid gap-3 md:grid-cols-[minmax(0,1fr)_160px]">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">变量键</span>
                  <input
                    :value="entry.key"
                    class="app-input"
                    placeholder="activitySweepCount"
                    :data-testid="index === 0 ? 'editor-input-key-0' : undefined"
                    @input="$emit('update-input', entry.id, 'key', ($event.target as HTMLInputElement).value)"
                  />
                </label>

                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">类型</span>
                  <AppSelect
                    :model-value="entry.type"
                    :options="inputTypeOptions"
                    placeholder="选择类型"
                    :test-id="index === 0 ? 'editor-input-type-0' : undefined"
                    @update:model-value="$emit('update-input', entry.id, 'type', String($event))"
                  />
                </label>
              </div>

              <label v-if="entry.type === 'boolean'" class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
                <input
                  :checked="entry.booleanValue"
                  type="checkbox"
                  class="h-4 w-4"
                  :data-testid="index === 0 ? 'editor-input-bool-0' : undefined"
                  style="accent-color: var(--app-accent)"
                  @change="$emit('update-input', entry.id, 'booleanValue', ($event.target as HTMLInputElement).checked)"
                />
                <span class="text-sm text-[var(--app-text-soft)]">默认启用</span>
              </label>

              <label v-else class="space-y-2">
                <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">默认值</span>
                <textarea
                  v-if="entry.type === 'json'"
                  :value="entry.stringValue"
                  class="app-textarea min-h-[120px]"
                  spellcheck="false"
                  @input="$emit('update-input', entry.id, 'stringValue', ($event.target as HTMLTextAreaElement).value)"
                />
                <input
                  v-else
                  :value="entry.stringValue"
                  class="app-input"
                  :type="entry.type === 'number' ? 'number' : 'text'"
                  :data-testid="index === 0 ? 'editor-input-value-0' : undefined"
                  @input="$emit('update-input', entry.id, 'stringValue', ($event.target as HTMLInputElement).value)"
                />
              </label>
            </div>

            <div class="mt-3 flex justify-end">
              <button class="app-button app-button-danger app-toolbar-button" type="button" @click="$emit('remove-input', entry.id)">
                删除
              </button>
            </div>
          </article>

          <EmptyState
            v-if="!inputEntries.length"
            title="还没有输入变量"
            description="中间点“添加输入”后，这里会直接显示可编辑的持久化变量。"
          />
        </div>

        <div class="space-y-3">
          <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <p class="text-sm font-semibold text-[var(--app-text-strong)]">当前输入</p>
            <div class="mt-3 space-y-3">
              <div
                v-for="entry in inputEntries"
                :key="entry.id"
                class="flex items-center justify-between gap-3 rounded-[14px] border border-[var(--app-border)] bg-white/30 px-3 py-3"
              >
                <span class="truncate text-[var(--app-text-strong)]">{{ entry.key || '未命名输入' }}</span>
                <span class="text-[var(--app-text-faint)]">{{ getInputTypeLabel(entry.type) }}</span>
              </div>
            </div>

            <EmptyState
              v-if="!inputEntries.length"
              title="还没有输入变量"
              description="先定义 input.*，UI 和步骤才能绑定到稳定的持久化值。"
            />
          </div>
        </div>
      </div>

      <div v-else-if="activePanel === 'ui'" class="grid min-h-0 gap-4 xl:grid-rows-[auto_minmax(0,1fr)]">
        <div
          class="rounded-[22px] border border-[var(--app-border)] bg-[linear-gradient(160deg,rgba(255,255,255,0.92),rgba(243,247,255,0.9))] px-5 py-5 shadow-[var(--app-shadow-soft)]"
        >
          <div class="space-y-3">
            <div class="flex flex-wrap items-center gap-3">
              <label class="editor-ui-chip editor-ui-chip-static">
                <input type="checkbox" checked disabled />
                <span>启用</span>
              </label>
              <span class="editor-ui-task-name">{{ task.name }}</span>
              <button
                v-if="uiSchema.layout === 'vertical'"
                class="app-button app-button-ghost app-toolbar-button"
                type="button"
                @click="uiPreviewExpanded = !uiPreviewExpanded"
              >
                {{ uiPreviewExpanded ? '收起' : '展开' }}
              </button>
            </div>

            <div
              v-if="uiSchema.layout === 'horizontal' || uiPreviewExpanded"
              :class="uiSchema.layout === 'vertical' ? 'grid gap-3' : 'flex flex-wrap items-center gap-3'"
            >
              <button
                v-for="field in uiSchema.fields"
                :key="field.id"
                class="editor-ui-chip"
                :class="{ 'editor-ui-chip-active': selectedUiFieldId === field.id, 'editor-ui-chip-vertical': uiSchema.layout === 'vertical' }"
                type="button"
                @click="$emit('select-ui-field', field.id)"
              >
                <template v-if="field.control === 'checkbox'">
                  <input type="checkbox" :checked="Boolean(resolvePreviewValue(field))" disabled />
                  <span>{{ field.label || '未命名字段' }}</span>
                </template>

                <template v-else-if="field.control === 'number'">
                  <span v-if="field.label" class="text-[var(--app-text-soft)]">{{ field.label }}</span>
                  <span class="editor-ui-inline-value">{{ String(resolvePreviewValue(field) ?? 0) }}</span>
                  <span v-if="field.description" class="text-[var(--app-text-soft)]">{{ field.description }}</span>
                </template>

                <template v-else-if="field.control === 'select'">
                  <span v-if="field.label" class="text-[var(--app-text-soft)]">{{ field.label }}</span>
                  <span class="editor-ui-inline-value">{{ resolvePreviewValue(field) || firstOption(field) || '请选择' }}</span>
                  <span v-if="field.description" class="text-[var(--app-text-soft)]">{{ field.description }}</span>
                </template>

                <template v-else-if="field.control === 'radio'">
                  <span v-if="field.label" class="text-[var(--app-text-soft)]">{{ field.label }}</span>
                  <span class="editor-ui-inline-options">
                    <span
                      v-for="option in field.optionsText.split('\n').map((item) => item.trim()).filter(Boolean)"
                      :key="option"
                      class="editor-ui-inline-pill"
                      :class="{ 'editor-ui-inline-pill-active': resolvePreviewValue(field) === option }"
                    >
                      {{ option }}
                    </span>
                  </span>
                </template>

                <template v-else>
                  <span v-if="field.inputKey" class="editor-ui-inline-value">{{ String((resolvePreviewValue(field) ?? field.placeholder) || '') }}</span>
                  <span v-else class="text-[var(--app-text-soft)]">{{ field.label || field.placeholder || '说明文本' }}</span>
                </template>
              </button>
            </div>
          </div>

          <EmptyState
            v-if="!uiSchema.fields.length"
            title="还没有可预览的字段"
            description="在中间插入 checkbox、radio、number 等模板后，这里会按排布方向实时预览。"
          />
        </div>

        <div class="grid min-h-0 gap-4 xl:grid-cols-[minmax(0,1fr)_300px]">
          <div class="min-h-0 overflow-y-auto pr-1 custom-scrollbar">
            <div v-if="selectedUiField" class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
              <div class="flex items-start justify-between gap-3">
                <div>
                  <p class="text-sm font-semibold text-[var(--app-text-strong)]">字段详情</p>
                  <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ getUiControlLabel(selectedUiField.control) }}</p>
                </div>
                <button class="app-button app-button-danger app-toolbar-button" type="button" @click="$emit('remove-ui-field', selectedUiField.id)">
                  删除字段
                </button>
              </div>

              <div class="mt-4 grid gap-3">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">字段名</span>
                  <input
                    :value="selectedUiField.label"
                    class="app-input"
                    :data-testid="selectedUiFieldIndex === 0 ? 'editor-ui-field-label-0' : undefined"
                    @input="$emit('update-ui-field', selectedUiField.id, 'label', ($event.target as HTMLInputElement).value)"
                  />
                </label>

                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">字段键</span>
                  <input
                    :value="selectedUiField.key"
                    class="app-input"
                    @input="$emit('update-ui-field', selectedUiField.id, 'key', ($event.target as HTMLInputElement).value)"
                  />
                </label>

                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">绑定输入</span>
                  <AppSelect
                    :model-value="selectedUiField.inputKey || null"
                    :options="bindOptions"
                    placeholder="未绑定"
                    :test-id="selectedUiFieldIndex === 0 ? 'editor-ui-field-bind-0' : undefined"
                    @update:model-value="$emit('update-ui-field', selectedUiField.id, 'inputKey', String($event ?? ''))"
                  />
                </label>

                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">说明</span>
                  <input
                    :value="selectedUiField.description"
                    class="app-input"
                    @input="$emit('update-ui-field', selectedUiField.id, 'description', ($event.target as HTMLInputElement).value)"
                  />
                </label>

                <label v-if="selectedUiField.control === 'text' || selectedUiField.control === 'number'" class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">占位提示</span>
                  <input
                    :value="selectedUiField.placeholder"
                    class="app-input"
                    @input="$emit('update-ui-field', selectedUiField.id, 'placeholder', ($event.target as HTMLInputElement).value)"
                  />
                </label>

                <label v-if="selectedUiField.control === 'radio' || selectedUiField.control === 'select'" class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">选项</span>
                  <textarea
                    :value="selectedUiField.optionsText"
                    class="app-textarea min-h-[100px]"
                    placeholder="每行一个选项"
                    @input="$emit('update-ui-field', selectedUiField.id, 'optionsText', ($event.target as HTMLTextAreaElement).value)"
                  />
                </label>
              </div>
            </div>

            <EmptyState
              v-else
              title="选择一个字段"
              description="点击中间字段列表或上方预览项，下面会切换到当前字段的可编辑内容。"
            />
          </div>

          <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <p class="text-sm font-semibold text-[var(--app-text-strong)]">绑定关系</p>
            <div class="mt-3 space-y-3">
              <div
                v-for="field in uiSchema.fields"
                :key="field.id"
                class="flex items-center justify-between gap-3 rounded-[14px] border border-[var(--app-border)] bg-white/30 px-3 py-3"
              >
                <span class="truncate text-[var(--app-text-strong)]">{{ field.label || field.key || '未命名字段' }}</span>
                <span class="truncate text-right text-[var(--app-text-faint)]">{{ field.inputKey || '未绑定' }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-else-if="activePanel === 'steps'" class="grid min-h-0 gap-4 xl:grid-cols-[minmax(0,480px)_minmax(360px,1fr)]">
        <div class="min-h-0 max-w-[480px] space-y-3 overflow-y-auto pr-1 custom-scrollbar">
          <article
            v-for="(step, index) in steps"
            :key="step.id ?? `${step.op}-${index}`"
            class="app-list-item space-y-3"
            :class="{
              'border-[var(--app-state-active-border)] bg-[var(--app-state-active-bg)]': selectedStepIndex === index,
              'bg-[var(--app-panel-muted)]': selectedStepIndex !== index,
              'editor-step-drop-target': overStepIndex === index && draggingStepIndex !== index,
            }"
            :data-testid="`editor-step-card-${index}`"
            draggable="true"
            @dragenter.prevent="overStepIndex = index"
            @dragover.prevent="handleStepDragOver($event, index)"
            @dragleave="handleStepLeave(index)"
            @dragstart="handleStepDragStart($event, index)"
            @drop.prevent="handleStepDrop(index)"
          >
            <div class="flex items-start justify-between gap-3">
              <button class="min-w-0 flex-1 text-left" type="button" @click="$emit('select-step', index)">
                <div class="flex items-center gap-2">
                  <span class="editor-drag-handle" :data-testid="`editor-step-drag-${index}`">
                    拖动
                  </span>
                  <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">
                    {{ step.label?.trim() || `步骤 ${index + 1}` }}
                  </p>
                </div>
                <p class="mt-2 text-sm leading-6 text-[var(--app-text-soft)]">{{ describeStep(step) }}</p>
                <p v-if="nestedSummary(step)" class="mt-2 text-xs text-[var(--app-text-faint)]">{{ nestedSummary(step) }}</p>
              </button>

              <span class="rounded-full border border-[var(--app-border)] px-2 py-1 text-[11px] font-medium text-[var(--app-text-soft)]">
                {{ step.op }}
              </span>
            </div>

            <div class="mt-4 flex flex-wrap gap-2">
              <button
                class="app-button app-button-ghost app-toolbar-button"
                type="button"
                :data-testid="`editor-step-duplicate-${index}`"
                @click="$emit('duplicate-step', index)"
              >
                复制
              </button>
              <button class="app-button app-button-danger app-toolbar-button" type="button" @click="$emit('remove-step', index)">删除</button>
            </div>
          </article>

          <EmptyState
            v-if="!steps.length"
            title="还没有步骤"
            description="先从中间插入模板，右侧再针对选中步骤查看和微调。"
          />
        </div>

        <div class="min-h-0 space-y-4 overflow-y-auto pr-1 custom-scrollbar">
          <div v-if="selectedStep" class="space-y-4">
            <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
              <div class="flex items-start justify-between gap-3">
                <div>
                  <p class="text-sm font-semibold text-[var(--app-text-strong)]">步骤详情</p>
                  <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ describeStep(selectedStep) }}</p>
                </div>
                <span class="rounded-full bg-white/50 px-3 py-1 text-xs text-[var(--app-text-soft)]">{{ selectedStep.op }}</span>
              </div>

              <label class="mt-4 block space-y-2">
                <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">步骤标题</span>
                <input :value="selectedStep.label || ''" class="app-input" @input="updateStepLabel(($event.target as HTMLInputElement).value)" />
              </label>
            </div>

            <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
              <p class="text-sm font-semibold text-[var(--app-text-strong)]">关键字段</p>
              <div class="mt-3 space-y-3">
                <template v-if="selectedStep.op === 'action' && selectedAction?.ac === 'capture'">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输出变量</span>
                    <input :value="selectedAction.output_var || ''" class="app-input" @input="updateActionField('output_var', ($event.target as HTMLInputElement).value)" />
                  </label>
                </template>

                <template v-else-if="selectedStep.op === 'action' && (selectedAction?.ac === 'launchApp' || selectedAction?.ac === 'stopApp')">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">包名</span>
                    <input :value="selectedAction.pkg_name || ''" class="app-input" @input="updateActionField('pkg_name', ($event.target as HTMLInputElement).value)" />
                  </label>
                </template>

                <template v-else-if="selectedStep.op === 'flowControl' && selectedFlow?.type === 'waitMs'">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">等待毫秒</span>
                    <input :value="String(selectedFlow.ms ?? 1000)" class="app-input" type="number" @input="updateNumberField('ms', ($event.target as HTMLInputElement).value)" />
                  </label>
                </template>

                <template v-else-if="selectedStep.op === 'flowControl' && selectedFlow?.type === 'link'">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标任务</span>
                    <input :value="selectedFlow.target || ''" class="app-input" @input="updateFlowField('target', ($event.target as HTMLInputElement).value)" />
                  </label>
                </template>

                <template v-else-if="selectedStep.op === 'flowControl' && flowWithCondition">
                  <label v-if="flowWithCondition.con.type === 'rawExpr'" class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">条件表达式</span>
                    <input :value="String(flowWithCondition.con.expr || '')" class="app-input" @input="updateConditionExpr(($event.target as HTMLInputElement).value)" />
                  </label>
                  <div v-else class="rounded-[14px] border border-[var(--app-border)] bg-white/30 px-3 py-3 text-sm text-[var(--app-text-soft)]">
                    当前条件节点为 `{{ flowWithCondition.con.type }}`，后续继续补专用 ConditionNode 编辑器。
                  </div>
                </template>

                <template v-else-if="selectedStep.op === 'dataHanding' && selectedData?.type === 'setVar'">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">变量名</span>
                    <input :value="selectedData.name || ''" class="app-input" @input="updateDataField('name', ($event.target as HTMLInputElement).value)" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">默认值</span>
                    <input :value="String(selectedData.val ?? '')" class="app-input" @input="updateDataField('val', ($event.target as HTMLInputElement).value)" />
                  </label>
                </template>

                <template v-else-if="selectedStep.op === 'dataHanding' && selectedData?.type === 'getVar'">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">变量名</span>
                    <input :value="selectedData.name || ''" class="app-input" @input="updateDataField('name', ($event.target as HTMLInputElement).value)" />
                  </label>
                </template>

                <template v-else-if="selectedStep.op === 'vision' && selectedVision?.type === 'visionSearch'">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输出变量</span>
                    <input :value="selectedVision.out_var || ''" class="app-input" @input="updateVisionField('out_var', ($event.target as HTMLInputElement).value)" />
                  </label>
                </template>

                <p v-else class="text-sm leading-6 text-[var(--app-text-soft)]">
                  当前步骤暂未提供专用表单，必要时可从右上角打开底层结构调试。
                </p>
              </div>
            </div>

            <div v-if="selectedNestedGroups.length" class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
              <p class="text-sm font-semibold text-[var(--app-text-strong)]">嵌套结构</p>
              <div class="mt-3 space-y-3">
                <div
                  v-for="group in selectedNestedGroups"
                  :key="group.label"
                  class="rounded-[14px] border border-[var(--app-border)] bg-white/30 px-3 py-3"
                >
                  <div class="flex items-center justify-between gap-3">
                    <span class="text-sm font-medium text-[var(--app-text-strong)]">{{ group.label }}</span>
                    <span class="text-xs text-[var(--app-text-faint)]">{{ group.steps.length }} 项</span>
                  </div>
                  <div class="mt-3 space-y-2">
                    <div
                      v-for="(nestedStep, nestedIndex) in group.steps"
                      :key="nestedStep.id ?? `${group.label}-${nestedIndex}`"
                      class="rounded-[12px] border border-[var(--app-border)] bg-white/50 px-3 py-2 text-sm text-[var(--app-text-soft)]"
                    >
                      {{ nestedIndex + 1 }}. {{ nestedStep.label?.trim() || describeStep(nestedStep) }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <EmptyState
            v-else
            title="选择一个步骤"
            description="右侧默认展示步骤概览，选中后可调整标题、关键字段和嵌套关系。"
          />
        </div>
      </div>

      <div v-else class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-5 py-5">
        <p class="text-sm font-semibold text-[var(--app-text-strong)]">任务概览</p>
      </div>
    </template>

    <EmptyState
      v-else
      title="没有选中任务"
      description="先从左侧选择任务，右侧工作区才会显示步骤概览和 UI 预览。"
    />
  </SurfacePanel>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { Step } from '@/types/bindings/Step';
import type { ScriptTaskTable } from '@/types/bindings/ScriptTaskTable';
import { describeStep } from '@/views/script-editor/editorStepTemplates';
import {
  buildInputJson,
  cloneJson,
  getInputTypeLabel,
  getUiControlLabel,
  type EditorInputEntry,
  type EditorUiField,
  type EditorPanelId,
  type EditorUiSchema,
} from '@/views/script-editor/editorSchema';

const props = defineProps<{
  task: ScriptTaskTable | null;
  activePanel: EditorPanelId;
  steps: Step[];
  selectedStepIndex: number | null;
  uiSchema: EditorUiSchema;
  selectedUiFieldId: string | null;
  inputEntries: EditorInputEntry[];
}>();

const emit = defineEmits<{
  'update-input': [entryId: string, field: 'key' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
  'remove-input': [entryId: string];
  'select-ui-field': [fieldId: string];
  'update-ui-field': [fieldId: string, field: 'label' | 'key' | 'inputKey' | 'description' | 'placeholder' | 'optionsText', value: string];
  'remove-ui-field': [fieldId: string];
  'select-step': [index: number];
  'reorder-step': [from: number, to: number];
  'duplicate-step': [index: number];
  'remove-step': [index: number];
  'update-step': [index: number, step: Step];
  'open-raw': [section: 'inputs' | 'ui' | 'steps'];
}>();

const draggingStepIndex = ref<number | null>(null);
const overStepIndex = ref<number | null>(null);
const uiPreviewExpanded = ref(true);

const workspaceTitle = computed(() => {
  if (props.activePanel === 'steps') return '步骤概览';
  if (props.activePanel === 'ui') return 'UI 预览';
  if (props.activePanel === 'inputs') return '输入设置';
  return '任务概览';
});
const rawSection = computed(() => {
  if (props.activePanel === 'steps') return 'steps';
  if (props.activePanel === 'ui') return 'ui';
  return 'inputs';
});
const inputTypeOptions = [
  { label: '文本', value: 'string', description: '普通字符串。' },
  { label: '数字', value: 'number', description: '次数、阈值、索引。' },
  { label: '布尔', value: 'boolean', description: '开关状态。' },
  { label: 'JSON', value: 'json', description: '复杂结构。' },
];

const selectedUiField = computed(() => props.uiSchema.fields.find((field) => field.id === props.selectedUiFieldId) ?? null);
const selectedUiFieldIndex = computed(() =>
  selectedUiField.value ? props.uiSchema.fields.findIndex((field) => field.id === selectedUiField.value?.id) : -1,
);

const bindOptions = computed(() => [
  { label: '未绑定', value: null, description: '纯展示字段或说明文本。' },
  ...props.inputEntries.map((entry) => ({
    label: entry.key || '未命名输入',
    value: entry.key || null,
    description: getInputTypeLabel(entry.type),
  })),
]);

const resolvePreviewValue = (field: EditorUiField) => {
  try {
    const inputs = buildInputJson(props.inputEntries);
    return field.inputKey ? inputs[field.inputKey] ?? null : null;
  } catch {
    return null;
  }
};

const firstOption = (field: EditorUiField) =>
  field.optionsText
    .split('\n')
    .map((item) => item.trim())
    .find(Boolean) ?? '';

const selectedStep = computed(() => {
  if (props.selectedStepIndex === null) return null;
  return props.steps[props.selectedStepIndex] ?? null;
});

const selectedAction = computed<Record<string, unknown> | null>(() =>
  selectedStep.value?.op === 'action' ? (selectedStep.value.a as Record<string, unknown>) : null,
);
const selectedFlow = computed<Record<string, unknown> | null>(() =>
  selectedStep.value?.op === 'flowControl' ? (selectedStep.value.a as Record<string, unknown>) : null,
);
const selectedData = computed<Record<string, unknown> | null>(() =>
  selectedStep.value?.op === 'dataHanding' ? (selectedStep.value.a as Record<string, unknown>) : null,
);
const selectedVision = computed<Record<string, unknown> | null>(() =>
  selectedStep.value?.op === 'vision' ? (selectedStep.value.a as Record<string, unknown>) : null,
);

const flowWithCondition = computed(() => {
  if (!selectedFlow.value) return null;
  const type = selectedFlow.value.type;
  if ((type === 'if' || type === 'while' || type === 'for') && selectedFlow.value.con && typeof selectedFlow.value.con === 'object') {
    return {
      type: type as 'if' | 'while' | 'for',
      con: selectedFlow.value.con as Record<string, unknown>,
    };
  }
  return null;
});

const selectedNestedGroups = computed<Array<{ label: string; steps: Step[] }>>(() => {
  if (!selectedStep.value) return [];
  if (selectedStep.value.op === 'sequence') {
    return [{ label: '顺序步骤', steps: selectedStep.value.steps }];
  }
  if (selectedStep.value.op === 'flowControl') {
    if (selectedStep.value.a.type === 'if') {
      return [
        { label: 'Then', steps: selectedStep.value.a.then },
        { label: 'Else', steps: selectedStep.value.a.else_steps ?? [] },
      ].filter((group) => group.steps.length);
    }
    if (selectedStep.value.a.type === 'while' || selectedStep.value.a.type === 'for') {
      return [{ label: '循环步骤', steps: selectedStep.value.a.flow }];
    }
  }
  if (selectedStep.value.op === 'vision' && selectedStep.value.a.type === 'visionSearch') {
    return [{ label: '命中后执行', steps: selectedStep.value.a.then_steps }];
  }
  return [];
});

const updateSelectedStep = (mutator: (step: Step & { a?: Record<string, unknown> }) => void) => {
  if (props.selectedStepIndex === null || !selectedStep.value) return;
  const nextStep = cloneJson(selectedStep.value) as Step & { a?: Record<string, unknown> };
  mutator(nextStep);
  emit('update-step', props.selectedStepIndex, nextStep);
};

const updateStepLabel = (value: string) => {
  updateSelectedStep((step) => {
    step.label = value;
  });
};

const updateActionField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    step.a = { ...(step.a ?? {}), [field]: value };
  });
};

const updateFlowField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    step.a = { ...(step.a ?? {}), [field]: value };
  });
};

const updateDataField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    step.a = { ...(step.a ?? {}), [field]: value };
  });
};

const updateVisionField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    step.a = { ...(step.a ?? {}), [field]: value };
  });
};

const updateNumberField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    step.a = { ...(step.a ?? {}), [field]: Number(value) };
  });
};

const updateConditionExpr = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== 'flowControl') return;
    if ((step.a.type === 'if' || step.a.type === 'while' || step.a.type === 'for') && step.a.con.type === 'rawExpr') {
      step.a.con.expr = value;
    }
  });
};

const handleStepDragStart = (event: DragEvent, index: number) => {
  draggingStepIndex.value = index;
  event.dataTransfer?.setData('text/plain', String(index));
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move';
  }
};

const handleStepDragOver = (event: DragEvent, index: number) => {
  overStepIndex.value = index;
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move';
  }
};

const handleStepLeave = (index: number) => {
  if (overStepIndex.value === index) overStepIndex.value = null;
};

const handleStepDrop = (targetIndex: number) => {
  if (draggingStepIndex.value === null || draggingStepIndex.value === targetIndex) {
    draggingStepIndex.value = null;
    overStepIndex.value = null;
    return;
  }
  emit('reorder-step', draggingStepIndex.value, targetIndex);
  draggingStepIndex.value = null;
  overStepIndex.value = null;
};

const nestedSummary = (step: Step) => {
  if (step.op === 'sequence' && step.steps.length) return `顺序容器 · ${step.steps.length} 个子步骤`;
  if (step.op === 'flowControl') {
    if (step.a.type === 'if') return `Then ${step.a.then.length} · Else ${(step.a.else_steps ?? []).length}`;
    if (step.a.type === 'while' || step.a.type === 'for') return `嵌套 ${step.a.flow.length} 个步骤`;
  }
  if (step.op === 'vision' && step.a.type === 'visionSearch' && step.a.then_steps.length) {
    return `命中后 ${step.a.then_steps.length} 个步骤`;
  }
  return '';
};
</script>

<style scoped>
.editor-ui-chip {
  display: inline-flex;
  align-items: center;
  gap: 0.65rem;
  border-radius: 18px;
  border: 1px solid var(--app-border);
  background: rgba(255, 255, 255, 0.72);
  padding: 0.8rem 0.95rem;
  text-align: left;
  transition: border-color 0.16s ease, background 0.16s ease;
}

.editor-ui-chip:hover {
  border-color: rgba(70, 110, 255, 0.22);
}

.editor-ui-chip-active {
  border-color: var(--app-state-active-border);
  background: var(--app-state-active-bg);
}

.editor-ui-chip-static {
  background: rgba(255, 255, 255, 0.84);
}

.editor-ui-chip-vertical {
  justify-content: flex-start;
  width: 100%;
}

.editor-ui-task-name {
  color: var(--app-text-strong);
  font-size: 1rem;
  font-weight: 600;
}

.editor-ui-inline-value {
  min-width: 72px;
  border-radius: 12px;
  border: 1px solid var(--app-border);
  background: white;
  padding: 0.45rem 0.75rem;
  text-align: center;
  color: var(--app-text-strong);
}

.editor-ui-inline-options {
  display: inline-flex;
  flex-wrap: wrap;
  gap: 0.45rem;
}

.editor-ui-inline-pill {
  border-radius: 999px;
  border: 1px solid var(--app-border);
  background: white;
  padding: 0.3rem 0.7rem;
  font-size: 0.75rem;
  color: var(--app-text-soft);
}

.editor-ui-inline-pill-active {
  border-color: var(--app-state-active-border);
  background: var(--app-state-active-bg);
  color: var(--app-text-strong);
}

.editor-drag-handle {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  border: 1px dashed var(--app-border);
  background: rgba(255, 255, 255, 0.46);
  color: var(--app-text-faint);
  padding: 0.24rem 0.55rem;
  font-size: 0.72rem;
  cursor: grab;
}

.editor-drag-handle:active {
  cursor: grabbing;
}

.editor-step-drop-target {
  box-shadow: inset 0 0 0 1px rgba(70, 110, 255, 0.22);
}
</style>
