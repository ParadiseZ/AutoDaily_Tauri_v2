<template>
  <div class="grid min-h-0 gap-4 xl:grid-rows-[auto_minmax(0,1fr)]">
    <datalist v-if="variableSuggestions.length" :id="variableDatalistId">
      <option v-for="option in variableSuggestions" :key="option" :value="option" />
    </datalist>

    <EditorStepBreadcrumb
      :steps="steps"
      :active-branch-path="activeBranchPath"
      :selected-step-path="selectedStepPath"
      @navigate-branch="$emit('navigate-branch', $event)"
      @select-step-path="$emit('select-step-path', $event)"
    />

    <div class="grid min-h-0 gap-4 xl:grid-cols-[minmax(0,360px)_minmax(0,1fr)]">
      <div class="min-h-0 overflow-y-auto pr-1 custom-scrollbar">
        <EditorStepList
          v-if="currentContainerSteps.length"
          :steps="currentContainerSteps"
          :selected-index="currentSelectedIndex"
          @select="selectCurrentBranchStep"
          @remove="$emit('remove-step', $event)"
          @reorder="handleReorder"
        />

        <EmptyState
          v-else
          title="还没有步骤"
          description="当前层级还是空的。"
        />
      </div>

      <div class="min-h-0 overflow-y-auto pr-1 custom-scrollbar">
        <div v-if="selectedStep" class="space-y-4">
          <div class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <div class="flex items-start justify-between gap-3">
              <div>
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">步骤详情</p>
                <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ describeStepMeta(selectedStep) }}</p>
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
              <template v-if="selectedStep.op === STEP_OP.action && selectedAction?.ac === ACTION_TYPE.capture">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输出变量</span>
                  <input :value="selectedAction.output_var || ''" :list="variableDatalistId" class="app-input" @input="updateActionField('output_var', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

              <template
                v-else-if="
                  selectedStep.op === STEP_OP.action &&
                  (selectedAction?.ac === ACTION_TYPE.launchApp || selectedAction?.ac === ACTION_TYPE.stopApp)
                "
              >
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">包名</span>
                  <input :value="selectedAction.pkg_name || ''" class="app-input" @input="updateActionField('pkg_name', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.action && selectedAction?.ac === ACTION_TYPE.click">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">点击方式</span>
                  <AppSelect
                    :model-value="String(selectedAction.mode || ACTION_MODE.point)"
                    :options="clickModeOptions"
                    placeholder="点击方式"
                    @update:model-value="updateActionMode(String($event || ACTION_MODE.point))"
                  />
                </label>

                <div v-if="selectedAction.mode === ACTION_MODE.point || selectedAction.mode === ACTION_MODE.percent" class="grid gap-3 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">X</span>
                    <input
                      :value="String((selectedAction.p as { x?: number })?.x ?? '')"
                      class="app-input"
                      type="number"
                      @input="updateActionPointField('p', 'x', ($event.target as HTMLInputElement).value)"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">Y</span>
                    <input
                      :value="String((selectedAction.p as { y?: number })?.y ?? '')"
                      class="app-input"
                      type="number"
                      @input="updateActionPointField('p', 'y', ($event.target as HTMLInputElement).value)"
                    />
                  </label>
                </div>

                <label v-else-if="selectedAction.mode === ACTION_MODE.txt" class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标文字</span>
                  <input :value="String(selectedAction.txt ?? '')" class="app-input" @input="updateActionTextField('txt', ($event.target as HTMLInputElement).value)" />
                </label>

                <label v-else class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">标签索引</span>
                  <input :value="String(selectedAction.idx ?? 0)" class="app-input" type="number" @input="updateActionNumberField('idx', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.action && selectedAction?.ac === ACTION_TYPE.swipe">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">滑动方式</span>
                  <AppSelect
                    :model-value="String(selectedAction.mode || ACTION_MODE.point)"
                    :options="swipeModeOptions"
                    placeholder="滑动方式"
                    @update:model-value="updateActionMode(String($event || ACTION_MODE.point))"
                  />
                </label>

                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">持续时间 (ms)</span>
                  <input :value="String(selectedAction.duration ?? 300)" class="app-input" type="number" @input="updateActionNumberField('duration', ($event.target as HTMLInputElement).value)" />
                </label>

                <div v-if="selectedAction.mode === ACTION_MODE.point || selectedAction.mode === ACTION_MODE.percent" class="grid gap-3 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">起点 X</span>
                    <input
                      :value="String((selectedAction.from as { x?: number })?.x ?? '')"
                      class="app-input"
                      type="number"
                      @input="updateActionPointField('from', 'x', ($event.target as HTMLInputElement).value)"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">起点 Y</span>
                    <input
                      :value="String((selectedAction.from as { y?: number })?.y ?? '')"
                      class="app-input"
                      type="number"
                      @input="updateActionPointField('from', 'y', ($event.target as HTMLInputElement).value)"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">终点 X</span>
                    <input
                      :value="String((selectedAction.to as { x?: number })?.x ?? '')"
                      class="app-input"
                      type="number"
                      @input="updateActionPointField('to', 'x', ($event.target as HTMLInputElement).value)"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">终点 Y</span>
                    <input
                      :value="String((selectedAction.to as { y?: number })?.y ?? '')"
                      class="app-input"
                      type="number"
                      @input="updateActionPointField('to', 'y', ($event.target as HTMLInputElement).value)"
                    />
                  </label>
                </div>

                <div v-else-if="selectedAction.mode === ACTION_MODE.txt" class="grid gap-3 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">起点文字</span>
                    <input :value="String(selectedAction.from ?? '')" class="app-input" @input="updateActionTextField('from', ($event.target as HTMLInputElement).value)" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">终点文字</span>
                    <input :value="String(selectedAction.to ?? '')" class="app-input" @input="updateActionTextField('to', ($event.target as HTMLInputElement).value)" />
                  </label>
                </div>

                <div v-else class="grid gap-3 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">起点标签</span>
                    <input :value="String(selectedAction.from ?? 0)" class="app-input" type="number" @input="updateActionNumberField('from', ($event.target as HTMLInputElement).value)" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">终点标签</span>
                    <input :value="String(selectedAction.to ?? 1)" class="app-input" type="number" @input="updateActionNumberField('to', ($event.target as HTMLInputElement).value)" />
                  </label>
                </div>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.flowControl && selectedFlow?.type === FLOW_TYPE.waitMs">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">等待毫秒</span>
                  <input :value="String(selectedFlow.ms ?? 1000)" class="app-input" type="number" @input="updateNumberField('ms', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.flowControl && selectedFlow?.type === FLOW_TYPE.link">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标任务</span>
                  <input :value="selectedFlow.target || ''" class="app-input" @input="updateFlowField('target', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.flowControl && (selectedFlow?.type === FLOW_TYPE.continue || selectedFlow?.type === FLOW_TYPE.break)">
                <div class="rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4 text-sm leading-6 text-[var(--app-text-soft)]">
                  {{ selectedFlow.type === FLOW_TYPE.continue ? '该步骤会立即开始下一轮循环。' : '该步骤会立即跳出当前循环。' }}
                </div>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.flowControl && flowWithCondition">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">流程类型</span>
                  <AppSelect
                    :model-value="flowWithCondition.type"
                    :options="flowTypeOptions"
                    placeholder="流程类型"
                    @update:model-value="updateFlowType(String($event || FLOW_TYPE.if))"
                  />
                </label>

                <div class="flex flex-wrap items-center justify-between gap-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-3">
                  <span class="text-sm text-[var(--app-text-soft)]">
                    {{
                      flowWithCondition.type === FLOW_TYPE.if
                        ? `Then ${(((selectedFlow as { then?: unknown[] } | null)?.then)?.length ?? 0)} 步`
                        : `循环 ${(((selectedFlow as { flow?: unknown[] } | null)?.flow)?.length ?? 0)} 步`
                    }}
                  </span>
                  <button
                    v-if="flowWithCondition.type === FLOW_TYPE.if"
                    class="app-button app-button-ghost app-toolbar-button"
                    type="button"
                    @click="toggleElseBranch"
                  >
                    {{ hasElseBranch ? '移除 Else 分支' : '添加 Else 分支' }}
                  </button>
                </div>

                <EditorConditionBuilder
                  v-if="flowCondition"
                  :model-value="flowCondition"
                  :variable-datalist-id="variableDatalistId"
                  test-id-prefix="editor-condition"
                  @update:model-value="updateFlowCondition"
                />
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.dataHanding && selectedData?.type === DATA_TYPE.setVar">
                <div class="space-y-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标变量</span>
                    <AppSelect
                      :model-value="selectedData.name || null"
                      :options="writableCatalogVariableOptions"
                      placeholder="从变量列表中选择"
                      test-id="editor-set-var-name"
                      @update:model-value="updateSetVarTarget(String($event || ''))"
                    />
                  </label>

                  <div v-if="selectedSetVarTarget" class="space-y-3 rounded-[14px] border border-[var(--app-border)] bg-white/45 px-4 py-3">
                    <div class="flex flex-wrap gap-2">
                      <span class="rounded-full border border-[var(--app-border)] bg-white/60 px-3 py-1 text-xs text-[var(--app-text-soft)]">
                        {{ selectedSetVarTarget.namespace }}
                      </span>
                      <span class="rounded-full border border-[var(--app-border)] bg-white/60 px-3 py-1 text-xs text-[var(--app-text-soft)]">
                        {{ selectedSetVarTarget.valueType }}
                      </span>
                    </div>

                    <div class="grid gap-2 text-sm">
                      <div class="grid gap-1">
                        <span class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">键</span>
                        <span class="text-[var(--app-text-strong)]">{{ selectedSetVarTarget.key }}</span>
                      </div>
                      <div class="grid gap-1">
                        <span class="text-[11px] uppercase tracking-[0.12em] text-[var(--app-text-faint)]">备注</span>
                        <span class="text-[var(--app-text-soft)]">{{ selectedSetVarTarget.description || '无' }}</span>
                      </div>
                    </div>
                  </div>
                </div>

                <div v-if="selectedSetVarTarget && setVarCanSwitchMode" class="flex justify-end">
                  <button class="app-button app-button-ghost app-toolbar-button" type="button" @click="updateSetVarMode(setVarUsesExpression ? 'value' : 'expr')">
                    {{ setVarUsesExpression ? '改为直接值' : '改用表达式' }}
                  </button>
                </div>

                <template v-if="selectedSetVarTarget && !setVarUsesExpression">
                  <label v-if="!selectedSetVarKind" class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">值类型</span>
                    <AppSelect
                      :model-value="effectiveSetVarKind"
                      :options="varValueTypeOptions"
                      placeholder="值类型"
                      test-id="editor-set-var-type"
                      @update:model-value="updateSetVarType(String($event || 'string'))"
                    />
                  </label>

                  <label v-if="effectiveSetVarKind === 'bool'" class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
                    <input
                      :checked="setVarDraft.boolValue"
                      type="checkbox"
                      class="h-4 w-4"
                      data-testid="editor-set-var-bool"
                      style="accent-color: var(--app-accent)"
                      @change="updateSetVarBool(($event.target as HTMLInputElement).checked)"
                    />
                    <span class="text-sm text-[var(--app-text-soft)]">值为真</span>
                  </label>
                  <label v-else class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">值</span>
                    <input
                      :value="setVarDraft.textValue"
                      class="app-input"
                      :type="effectiveSetVarKind === 'string' ? 'text' : 'number'"
                      data-testid="editor-set-var-value"
                      @input="updateSetVarText(($event.target as HTMLInputElement).value)"
                    />
                  </label>
                </template>

                <div
                  v-else-if="selectedSetVarTarget && !selectedSetVarKind"
                  class="rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4 text-sm leading-6 text-[var(--app-text-soft)]"
                >
                  当前变量类型不适合直接写固定值，请使用表达式。
                </div>

                <label v-if="selectedSetVarTarget && setVarUsesExpression" class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">表达式</span>
                  <input
                    :value="selectedData.expr ?? ''"
                    class="app-input"
                    @input="updateDataNullableField('expr', ($event.target as HTMLInputElement).value)"
                  />
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.dataHanding && selectedData?.type === DATA_TYPE.getVar">
                <div class="space-y-3 rounded-[16px] border border-[var(--app-border)] bg-white/35 px-4 py-4">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">读取变量</span>
                    <AppSelect
                      :model-value="selectedData.name || null"
                      :options="includeCurrentVariableOption(readableCatalogVariableOptions, selectedData.name)"
                      placeholder="优先从变量目录里选择"
                      test-id="editor-get-var-name"
                      @update:model-value="updateDataField('name', String($event || ''))"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">变量键</span>
                    <input
                      :value="selectedData.name || ''"
                      :list="variableDatalistId"
                      class="app-input"
                      data-testid="editor-get-var-name-input"
                      @input="updateDataField('name', ($event.target as HTMLInputElement).value)"
                    />
                  </label>
                </div>
                <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
                  <input
                    :checked="getVarHasDefault"
                    type="checkbox"
                    class="h-4 w-4"
                    style="accent-color: var(--app-accent)"
                    @change="toggleGetVarDefault(($event.target as HTMLInputElement).checked)"
                  />
                  <span class="text-sm text-[var(--app-text-soft)]">启用默认值</span>
                </label>
                <template v-if="getVarHasDefault">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">默认值类型</span>
                    <AppSelect
                      :model-value="getVarDraft.kind"
                      :options="varValueTypeOptions"
                      placeholder="默认值类型"
                      test-id="editor-get-var-type"
                      @update:model-value="updateGetVarType(String($event || 'string'))"
                    />
                  </label>
                  <label v-if="getVarDraft.kind === 'bool'" class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
                    <input
                      :checked="getVarDraft.boolValue"
                      type="checkbox"
                      class="h-4 w-4"
                      data-testid="editor-get-var-bool"
                      style="accent-color: var(--app-accent)"
                      @change="updateGetVarBool(($event.target as HTMLInputElement).checked)"
                    />
                    <span class="text-sm text-[var(--app-text-soft)]">默认值为真</span>
                  </label>
                  <label v-else class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">默认值</span>
                    <input
                      :value="getVarDraft.textValue"
                      class="app-input"
                      :type="getVarDraft.kind === 'string' ? 'text' : 'number'"
                      data-testid="editor-get-var-value"
                      @input="updateGetVarText(($event.target as HTMLInputElement).value)"
                    />
                  </label>
                </template>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.dataHanding && selectedData?.type === DATA_TYPE.filter">
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输入变量</span>
                  <input :value="selectedData.input_var" :list="variableDatalistId" class="app-input" @input="updateDataField('input_var', ($event.target as HTMLInputElement).value)" />
                </label>
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输出变量</span>
                  <input :value="selectedData.out_name" :list="variableDatalistId" class="app-input" @input="updateDataField('out_name', ($event.target as HTMLInputElement).value)" />
                </label>
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">过滤模式</span>
                  <AppSelect
                    :model-value="selectedData.mode.type"
                    :options="filterModeOptions"
                    placeholder="过滤模式"
                    @update:model-value="updateFilterMode(String($event || FILTER_MODE_TYPE.filter))"
                  />
                </label>
                <label class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">逻辑表达式</span>
                  <input :value="selectedData.logic_expr" class="app-input" @input="updateDataField('logic_expr', ($event.target as HTMLInputElement).value)" />
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.sequence">
                <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
                  <input
                    :checked="selectedStep.reverse"
                    type="checkbox"
                    class="h-4 w-4"
                    style="accent-color: var(--app-accent)"
                    @change="updateSequenceReverse(($event.target as HTMLInputElement).checked)"
                  />
                  <span class="text-sm text-[var(--app-text-soft)]">倒序执行子步骤</span>
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.taskControl && selectedTaskControl">
                <div class="grid gap-3 md:grid-cols-2">
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">动作类型</span>
                    <AppSelect
                      :model-value="selectedTaskControl.type"
                      :options="taskControlTypeOptions"
                      placeholder="动作类型"
                      @update:model-value="updateTaskControlType(String($event || TASK_CONTROL_TYPE.setState))"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标类型</span>
                    <AppSelect
                      :model-value="selectedTaskControl.target.type"
                      :options="stateTargetTypeOptions"
                      placeholder="目标类型"
                      @update:model-value="updateTaskControlTargetType(String($event || STATE_TARGET_TYPE.task))"
                    />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">目标 ID</span>
                    <input :value="selectedTaskControl.target.id" class="app-input" @input="updateTaskControlTargetId(($event.target as HTMLInputElement).value)" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">状态类型</span>
                    <AppSelect
                      :model-value="selectedTaskControl.status.type"
                      :options="stateStatusTypeOptions"
                      placeholder="状态类型"
                      @update:model-value="updateTaskControlStatusType(String($event || STATE_STATUS_TYPE.done))"
                    />
                  </label>
                </div>
                <label class="flex items-center gap-3 rounded-[16px] border border-[var(--app-border)] px-4 py-3">
                  <input
                    :checked="Boolean(selectedTaskControl.status.value)"
                    type="checkbox"
                    class="h-4 w-4"
                    style="accent-color: var(--app-accent)"
                    @change="updateTaskControlStatusValue(($event.target as HTMLInputElement).checked)"
                  />
                  <span class="text-sm text-[var(--app-text-soft)]">状态值为真</span>
                </label>
              </template>

              <template v-else-if="selectedStep.op === STEP_OP.vision && selectedVision?.type === VISION_TYPE.visionSearch">
                <div class="space-y-4">
                  <div class="rounded-[16px] border border-[var(--app-border)] bg-white/40 px-4 py-4">
                    <p class="text-sm font-semibold text-[var(--app-text-strong)]">基础信息</p>
                    <label class="mt-3 space-y-2">
                      <span class="text-xs font-medium uppercase tracking-[0.12em] text-[var(--app-text-faint)]">输出变量</span>
                      <input :value="selectedVision.out_var || ''" :list="variableDatalistId" class="app-input" @input="updateVisionField('out_var', ($event.target as HTMLInputElement).value)" />
                    </label>
                  </div>

                  <div class="rounded-[16px] border border-[var(--app-border)] bg-white/40 px-4 py-4">
                    <p class="text-sm font-semibold text-[var(--app-text-strong)]">搜索规则</p>
                    <p class="mt-1 text-xs text-[var(--app-text-faint)]">根层固定为逻辑组，在组内继续添加规则或子组。</p>
                    <div class="mt-3">
                      <EditorSearchRuleBuilder
                        :model-value="selectedVision.rule"
                        force-group-root
                        test-id-prefix="editor-search-rule"
                        @update:model-value="updateVisionRule"
                      />
                    </div>
                  </div>

                  <div class="rounded-[16px] border border-[var(--app-border)] bg-white/40 px-4 py-4">
                    <div class="flex items-center justify-between gap-3">
                      <div>
                        <p class="text-sm font-semibold text-[var(--app-text-strong)]">命中后行为</p>
                        <p class="mt-1 text-xs text-[var(--app-text-faint)]">命中后继续进入步骤层级维护后续动作。</p>
                      </div>
                      <button
                        v-if="visionBranchTarget"
                        class="app-button app-button-ghost app-toolbar-button"
                        type="button"
                        @click="$emit('navigate-branch', visionBranchTarget.path)"
                      >
                        进入步骤
                      </button>
                    </div>
                    <p class="mt-3 text-sm text-[var(--app-text-soft)]">{{ visionBranchTarget?.count ?? 0 }} 个步骤</p>
                  </div>
                </div>
              </template>

              <p v-else class="text-sm leading-6 text-[var(--app-text-soft)]">
                当前步骤暂未提供专用表单，必要时可从右上角打开底层结构调试。
              </p>
            </div>
          </div>

          <div v-if="branchTargets.length" class="rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] px-4 py-4">
            <p class="text-sm font-semibold text-[var(--app-text-strong)]">可进入层级</p>
            <div class="mt-3 grid gap-3">
              <button
                v-for="target in branchTargets"
                :key="target.key"
                class="app-list-item"
                :class="{ 'app-list-item-active': isSameBranchPath(activeBranchPath, target.path) }"
                type="button"
                :data-testid="`editor-branch-${target.key}`"
                @click="$emit('navigate-branch', target.path)"
              >
                <div class="flex items-center justify-between gap-3">
                  <div class="min-w-0">
                    <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ target.label }}</p>
                    <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ target.count }} 个步骤</p>
                  </div>
                  <span class="text-xs text-[var(--app-text-faint)]">进入</span>
                </div>
              </button>
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
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import type { Action } from '@/types/bindings/Action';
import type { ConditionNode } from '@/types/bindings/ConditionNode';
import type { DataHanding } from '@/types/bindings/DataHanding';
import type { SearchRule } from '@/types/bindings/SearchRule';
import type { FlowControl } from '@/types/bindings/FlowControl';
import type { TaskControl } from '@/types/bindings/TaskControl';
import type { Step } from '@/types/bindings/Step';
import type { VisionNode } from '@/types/bindings/VisionNode';
import EditorConditionBuilder from '@/views/script-editor/EditorConditionBuilder.vue';
import EditorSearchRuleBuilder from '@/views/script-editor/EditorSearchRuleBuilder.vue';
import EditorStepBreadcrumb from '@/views/script-editor/EditorStepBreadcrumb.vue';
import EditorStepList from '@/views/script-editor/EditorStepList.vue';
import { createConditionNode } from '@/views/script-editor/editorCondition';
import {
  ACTION_MODE,
  ACTION_TYPE,
  DATA_TYPE,
  FILTER_MODE_TYPE,
  FLOW_TYPE,
  STATE_STATUS_TYPE,
  STATE_TARGET_TYPE,
  STEP_OP,
  TASK_CONTROL_TYPE,
  VISION_TYPE,
} from '@/views/script-editor/editorStepKinds';
import { describeStepMeta } from '@/views/script-editor/editorStepTemplates';
import {
  buildVarValue,
  parseVarValueDraft,
  varValueTypeOptions,
  type VarValueKind,
} from '@/views/script-editor/editorVarValue';
import type { EditorVariableOption } from '@/views/script-editor/editorVariables';
import {
  buildStepPath,
  getBranchSteps,
  getParentBranchPath,
  getStepByPath,
  isSameBranchPath,
  type StepBranchPath,
  type StepPath,
} from '@/views/script-editor/editorStepTree';
import { cloneJson } from '@/views/script-editor/editorSchema';

type NestedGroupKey = 'sequence' | 'then' | 'else' | 'flow' | 'visionThen' | 'filterThen';

const props = defineProps<{
  steps: Step[];
  selectedStepPath: StepPath | null;
  activeBranchPath: StepBranchPath;
  variableOptions: EditorVariableOption[];
  catalogVariableOptions: EditorVariableOption[];
}>();

const emit = defineEmits<{
  'select-step-path': [path: StepPath];
  'navigate-branch': [branchPath: StepBranchPath];
  'reorder-step': [from: number, to: number];
  'remove-step': [index: number];
  'update-step': [index: number, step: Step];
}>();

const variableDatalistId = 'editor-variable-suggestions';
const variableSuggestions = computed(() =>
  Array.from(new Set(props.variableOptions.map((item) => item.key).filter(Boolean))),
);
const readableCatalogVariableOptions = computed(() =>
  props.catalogVariableOptions
    .filter((item) => item.readable)
    .map((item) => ({
      label: item.label || item.key,
      value: item.key,
      description: `${item.namespace} · ${item.valueType}`,
    })),
);
const writableCatalogVariableOptions = computed(() =>
  props.catalogVariableOptions
    .filter((item) => item.writable)
    .map((item) => ({
      label: item.label || item.key,
      value: item.key,
      description: `${item.namespace} · ${item.valueType}`,
    })),
);
const includeCurrentVariableOption = (
  options: Array<{ label: string; value: string; description: string }>,
  currentValue: string | null | undefined,
) => {
  const trimmed = currentValue?.trim();
  if (!trimmed || options.some((item) => item.value === trimmed)) {
    return options;
  }

  return [
    {
      label: `${trimmed} (当前)`,
      value: trimmed,
      description: '当前值尚未登记到变量目录',
    },
    ...options,
  ];
};

const clickModeOptions = [
  { label: '坐标', value: ACTION_MODE.point, description: '绝对坐标点击。' },
  { label: '百分比', value: ACTION_MODE.percent, description: '相对坐标点击。' },
  { label: '文字', value: ACTION_MODE.txt, description: '按 OCR 文本点击。' },
  { label: '标签', value: ACTION_MODE.labelIdx, description: '按视觉标签点击。' },
];

const swipeModeOptions = [
  { label: '坐标', value: ACTION_MODE.point, description: '绝对坐标滑动。' },
  { label: '百分比', value: ACTION_MODE.percent, description: '相对坐标滑动。' },
  { label: '文字', value: ACTION_MODE.txt, description: '按 OCR 文本滑动。' },
  { label: '标签', value: ACTION_MODE.labelIdx, description: '按视觉标签滑动。' },
];

const flowTypeOptions = [
  { label: '条件分支', value: FLOW_TYPE.if, description: 'Then / Else 分支。' },
  { label: 'While', value: FLOW_TYPE.while, description: '满足条件时循环。' },
  { label: 'For', value: FLOW_TYPE.for, description: '条件控制的遍历循环。' },
];

const filterModeOptions = [
  { label: '过滤', value: FILTER_MODE_TYPE.filter, description: '保留符合条件的元素。' },
  { label: '映射', value: FILTER_MODE_TYPE.map, description: '将输入映射为新结果。' },
];
const taskControlTypeOptions = [
  { label: '设置状态', value: TASK_CONTROL_TYPE.setState, description: '写入目标状态。' },
  { label: '读取状态', value: TASK_CONTROL_TYPE.getState, description: '读取目标状态。' },
];

const stateTargetTypeOptions = [
  { label: '任务', value: STATE_TARGET_TYPE.task, description: '针对任务状态。' },
  { label: '策略', value: STATE_TARGET_TYPE.policy, description: '针对策略状态。' },
];

const stateStatusTypeOptions = [
  { label: '完成', value: STATE_STATUS_TYPE.done, description: 'done 状态。' },
  { label: '跳过', value: STATE_STATUS_TYPE.skip, description: 'skip 状态。' },
];

const currentContainerSteps = computed(() => getBranchSteps(props.steps, props.activeBranchPath));
const selectedStep = computed(() => getStepByPath(props.steps, props.selectedStepPath));
const currentSelectedIndex = computed(() => {
  if (!props.selectedStepPath?.length) return null;
  const branchPath = getParentBranchPath(props.selectedStepPath);
  if (!isSameBranchPath(branchPath, props.activeBranchPath)) return null;
  return props.selectedStepPath[props.selectedStepPath.length - 1]?.index ?? null;
});

const selectedAction = computed<Action | null>(() => (selectedStep.value?.op === STEP_OP.action ? selectedStep.value.a : null));
const selectedFlow = computed<FlowControl | null>(() => (selectedStep.value?.op === STEP_OP.flowControl ? selectedStep.value.a : null));
const selectedData = computed<DataHanding | null>(() => (selectedStep.value?.op === STEP_OP.dataHanding ? selectedStep.value.a : null));
const selectedTaskControl = computed<TaskControl | null>(() => (selectedStep.value?.op === STEP_OP.taskControl ? selectedStep.value.a : null));
const selectedVision = computed<VisionNode | null>(() => (selectedStep.value?.op === STEP_OP.vision ? selectedStep.value.a : null));
const setVarKindPreference = ref<VarValueKind | null>(null);
const getVarKindPreference = ref<VarValueKind | null>(null);
const setVarDraft = computed(() =>
  selectedData.value?.type === DATA_TYPE.setVar
    ? parseVarValueDraft(selectedData.value.val, setVarKindPreference.value ?? undefined)
    : parseVarValueDraft(''),
);
const getVarHasDefault = computed(() => Boolean(selectedData.value?.type === DATA_TYPE.getVar && selectedData.value.default_val !== null));
const getVarDraft = computed(() =>
  selectedData.value?.type === DATA_TYPE.getVar
    ? parseVarValueDraft(selectedData.value.default_val, getVarKindPreference.value ?? undefined)
    : parseVarValueDraft(''),
);
const mapVariableTypeToVarKind = (valueType: EditorVariableOption['valueType']): VarValueKind | null => {
  switch (valueType) {
    case 'int':
      return 'int';
    case 'float':
      return 'float';
    case 'bool':
      return 'bool';
    case 'string':
      return 'string';
    default:
      return null;
  }
};
const createDefaultVarValueDraft = (kind: VarValueKind) =>
  parseVarValueDraft(kind === 'string' ? '' : kind === 'bool' ? false : 0, kind);
const currentSetVarName = computed(() =>
  selectedData.value?.type === DATA_TYPE.setVar ? selectedData.value.name : '',
);
const selectedSetVarTarget = computed(() =>
  currentSetVarName.value ? props.catalogVariableOptions.find((item) => item.key === currentSetVarName.value) ?? null : null,
);
const selectedSetVarKind = computed(() => (selectedSetVarTarget.value ? mapVariableTypeToVarKind(selectedSetVarTarget.value.valueType) : null));
const setVarUsesExpression = computed(() => {
  if (selectedData.value?.type !== DATA_TYPE.setVar) {
    return false;
  }

  if (selectedSetVarTarget.value && !selectedSetVarKind.value) {
    return true;
  }

  return Boolean(selectedData.value.expr);
});
const effectiveSetVarKind = computed<VarValueKind>(() => selectedSetVarKind.value ?? setVarDraft.value.kind);
const setVarCanSwitchMode = computed(() => Boolean(selectedSetVarTarget.value && selectedSetVarKind.value));

watch(
  () => props.selectedStepPath?.map((segment) => `${segment.branch}:${segment.index}`).join('/') ?? '',
  () => {
    setVarKindPreference.value = null;
    getVarKindPreference.value = null;
  },
  { immediate: true },
);

const flowWithCondition = computed(() => {
  if (!selectedFlow.value) return null;
  const type = selectedFlow.value.type;
  if ((type === FLOW_TYPE.if || type === FLOW_TYPE.while || type === FLOW_TYPE.for) && selectedFlow.value.con) {
    return {
      type,
      con: selectedFlow.value.con,
    };
  }
  return null;
});

const flowCondition = computed<ConditionNode | null>(() => flowWithCondition.value?.con ?? null);
const hasElseBranch = computed(() => Boolean(selectedFlow.value?.type === FLOW_TYPE.if && selectedFlow.value.else_steps));

const branchTargets = computed<Array<{ key: NestedGroupKey; label: string; count: number; path: StepBranchPath }>>(() => {
  if (!selectedStep.value || !props.selectedStepPath) return [];

  if (selectedStep.value.op === STEP_OP.sequence) {
    return [{ key: 'sequence', label: '顺序步骤', count: selectedStep.value.steps.length, path: { parentStepPath: props.selectedStepPath, branch: 'sequence' } }];
  }

  if (selectedFlow.value?.type === FLOW_TYPE.if) {
      const targets: Array<{ key: NestedGroupKey; label: string; count: number; path: StepBranchPath }> = [
        { key: 'then', label: 'Then', count: selectedFlow.value.then.length, path: { parentStepPath: props.selectedStepPath, branch: 'then' } },
      ];
      if (selectedFlow.value.else_steps) {
        targets.push({ key: 'else', label: 'Else', count: selectedFlow.value.else_steps.length, path: { parentStepPath: props.selectedStepPath, branch: 'else' } });
      }
      return targets;
  }

  if (selectedFlow.value?.type === FLOW_TYPE.while || selectedFlow.value?.type === FLOW_TYPE.for) {
    return [{ key: 'flow', label: '循环体', count: selectedFlow.value.flow.length, path: { parentStepPath: props.selectedStepPath, branch: 'flow' } }];
  }

  if (selectedVision.value?.type === VISION_TYPE.visionSearch) {
    return [{ key: 'visionThen', label: '命中后执行', count: selectedVision.value.then_steps.length, path: { parentStepPath: props.selectedStepPath, branch: 'visionThen' } }];
  }

  if (selectedData.value?.type === DATA_TYPE.filter) {
    return [{ key: 'filterThen', label: '过滤命中后', count: selectedData.value.then_steps.length, path: { parentStepPath: props.selectedStepPath, branch: 'filterThen' } }];
  }

  return [];
});
const visionBranchTarget = computed(() => branchTargets.value.find((target) => target.key === 'visionThen') ?? null);

const selectCurrentBranchStep = (index: number) => {
  emit('select-step-path', buildStepPath(props.activeBranchPath, index));
};

const handleReorder = (from: number, to: number) => {
  emit('reorder-step', from, to);
};

const updateSelectedStep = (mutator: (step: Step & { a?: Action | FlowControl | DataHanding | VisionNode }) => void) => {
  if (currentSelectedIndex.value === null || !selectedStep.value) return;
  const nextStep = cloneJson(selectedStep.value) as Step & { a?: Action | FlowControl | DataHanding | VisionNode };
  mutator(nextStep);
  emit('update-step', currentSelectedIndex.value, nextStep);
};

const updateStepLabel = (value: string) => {
  updateSelectedStep((step) => {
    step.label = value;
  });
};

const updateActionField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action) return;
    step.a = { ...(step.a ?? {}), [field]: value } as Action;
  });
};

const createClickAction = (mode: string): Action => {
  switch (mode) {
    case ACTION_MODE.percent:
      return { ac: ACTION_TYPE.click, mode: ACTION_MODE.percent, p: { x: 0.5, y: 0.5 } };
    case ACTION_MODE.txt:
      return { ac: ACTION_TYPE.click, mode: ACTION_MODE.txt, txt: '开始' };
    case ACTION_MODE.labelIdx:
      return { ac: ACTION_TYPE.click, mode: ACTION_MODE.labelIdx, idx: 0 };
    default:
      return { ac: ACTION_TYPE.click, mode: ACTION_MODE.point, p: { x: 640, y: 360 } };
  }
};

const createSwipeAction = (mode: string): Action => {
  switch (mode) {
    case ACTION_MODE.percent:
      return {
        ac: ACTION_TYPE.swipe,
        mode: ACTION_MODE.percent,
        duration: 300 as never,
        from: { x: 0.5, y: 0.75 },
        to: { x: 0.5, y: 0.25 },
      };
    case ACTION_MODE.txt:
      return {
        ac: ACTION_TYPE.swipe,
        mode: ACTION_MODE.txt,
        duration: 300 as never,
        from: '开始',
        to: '结束',
      };
    case ACTION_MODE.labelIdx:
      return {
        ac: ACTION_TYPE.swipe,
        mode: ACTION_MODE.labelIdx,
        duration: 300 as never,
        from: 0,
        to: 1,
      };
    default:
      return {
        ac: ACTION_TYPE.swipe,
        mode: ACTION_MODE.point,
        duration: 300 as never,
        from: { x: 640, y: 560 },
        to: { x: 640, y: 180 },
      };
  }
};

const updateActionModel = (value: Action) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action) return;
    step.a = value;
  });
};

const updateActionMode = (mode: string) => {
  if (!selectedAction.value) return;
  if (selectedAction.value.ac === ACTION_TYPE.click) {
    updateActionModel(createClickAction(mode));
    return;
  }
  if (selectedAction.value.ac === ACTION_TYPE.swipe) {
    updateActionModel(createSwipeAction(mode));
  }
};

const toNumber = (value: string) => {
  const next = Number(value);
  return Number.isFinite(next) ? next : 0;
};

const updateActionPointField = (field: 'p' | 'from' | 'to', axis: 'x' | 'y', value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action) return;
    const action = step.a as Record<string, unknown>;
    const point = { ...((action[field] as Record<string, number> | undefined) ?? { x: 0, y: 0 }) };
    point[axis] = toNumber(value);
    step.a = { ...action, [field]: point } as Action;
  });
};

const updateActionNumberField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action) return;
    step.a = { ...(step.a as Record<string, unknown>), [field]: toNumber(value) } as Action;
  });
};

const updateActionTextField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.action) return;
    step.a = { ...(step.a as Record<string, unknown>), [field]: value } as Action;
  });
};

const updateFlowField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl) return;
    step.a = { ...(step.a ?? {}), [field]: value } as FlowControl;
  });
};

const updateDataField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding) return;
    step.a = { ...(step.a ?? {}), [field]: value } as DataHanding;
  });
};

const updateDataNullableField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding) return;
    step.a = { ...(step.a ?? {}), [field]: value.trim() ? value : null } as DataHanding;
  });
};

const getSetVarDraftForKind = (kind: VarValueKind) => (setVarDraft.value.kind === kind ? setVarDraft.value : createDefaultVarValueDraft(kind));

const updateSetVarTarget = (value: string) => {
  const matched = props.catalogVariableOptions.find((item) => item.key === value) ?? null;
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.setVar) return;

    const nextKind = matched ? mapVariableTypeToVarKind(matched.valueType) : null;
    const nextExpr = matched && !nextKind ? step.a.expr ?? '' : step.a.expr;
    const nextVal = nextKind ? buildVarValue(getSetVarDraftForKind(nextKind)) : nextKind === null && matched ? null : step.a.val;

    step.a = {
      ...step.a,
      name: value,
      val: nextExpr ? null : nextVal,
      expr: matched && !nextKind ? nextExpr : step.a.expr,
    };
  });
};

const updateSetVarMode = (mode: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.setVar) return;
    const forcedKind = selectedSetVarKind.value;
    const nextDraft = forcedKind ? getSetVarDraftForKind(forcedKind) : setVarDraft.value;
    step.a = {
      ...step.a,
      val: mode === 'expr' ? null : buildVarValue(nextDraft),
      expr: mode === 'expr' ? (step.a.expr || 'true') : null,
    };
  });
};

const updateSetVarType = (kind: string) => {
  setVarKindPreference.value = kind as VarValueKind;
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.setVar) return;
    const nextDraft = createDefaultVarValueDraft(kind as VarValueKind);
    step.a = {
      ...step.a,
      val: buildVarValue(nextDraft),
      expr: null,
    };
  });
};

const updateSetVarText = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.setVar) return;
    const nextKind = effectiveSetVarKind.value;
    step.a = {
      ...step.a,
      val: buildVarValue({
        ...getSetVarDraftForKind(nextKind),
        textValue: value,
      }),
      expr: null,
    };
  });
};

const updateSetVarBool = (value: boolean) => {
  setVarKindPreference.value = 'bool';
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.setVar) return;
    step.a = {
      ...step.a,
      val: buildVarValue({
        ...getSetVarDraftForKind('bool'),
        kind: 'bool',
        boolValue: value,
        textValue: value ? 'true' : 'false',
      }),
      expr: null,
    };
  });
};

const toggleGetVarDefault = (enabled: boolean) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.getVar) return;
    step.a = {
      ...step.a,
      default_val: enabled ? buildVarValue(getVarDraft.value) : null,
    };
  });
};

const updateGetVarType = (kind: string) => {
  getVarKindPreference.value = kind as VarValueKind;
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.getVar) return;
    step.a = {
      ...step.a,
      default_val: buildVarValue({
        kind: kind as 'int' | 'float' | 'bool' | 'string',
        textValue: kind === 'string' ? '' : '0',
        boolValue: false,
      }),
    };
  });
};

const updateGetVarText = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.getVar) return;
    step.a = {
      ...step.a,
      default_val: buildVarValue({
        ...getVarDraft.value,
        textValue: value,
      }),
    };
  });
};

const updateGetVarBool = (value: boolean) => {
  getVarKindPreference.value = 'bool';
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.getVar) return;
    step.a = {
      ...step.a,
      default_val: buildVarValue({
        ...getVarDraft.value,
        kind: 'bool',
        boolValue: value,
        textValue: value ? 'true' : 'false',
      }),
    };
  });
};

const updateFilterMode = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.dataHanding || step.a.type !== DATA_TYPE.filter) return;
    step.a = {
      ...step.a,
      mode: {
        type: value as typeof FILTER_MODE_TYPE.filter | typeof FILTER_MODE_TYPE.map,
      },
    };
  });
};

const updateSequenceReverse = (value: boolean) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.sequence) return;
    step.reverse = value;
  });
};

const updateTaskControlType = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    step.a = {
      ...step.a,
      type: value as typeof TASK_CONTROL_TYPE.setState | typeof TASK_CONTROL_TYPE.getState,
    };
  });
};

const updateTaskControlTargetType = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    step.a = {
      ...step.a,
      target: {
        ...step.a.target,
        type: value as typeof STATE_TARGET_TYPE.task | typeof STATE_TARGET_TYPE.policy,
      },
    };
  });
};

const updateTaskControlTargetId = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    step.a = {
      ...step.a,
      target: {
        ...step.a.target,
        id: value,
      },
    };
  });
};

const updateTaskControlStatusType = (value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    step.a = {
      ...step.a,
      status: {
        ...step.a.status,
        type: value as typeof STATE_STATUS_TYPE.done | typeof STATE_STATUS_TYPE.skip,
      },
    };
  });
};

const updateTaskControlStatusValue = (value: boolean) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.taskControl) return;
    step.a = {
      ...step.a,
      status: {
        ...step.a.status,
        value,
      },
    };
  });
};

const updateVisionField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.vision) return;
    step.a = { ...(step.a ?? {}), [field]: value } as VisionNode;
  });
};

const updateVisionRule = (rule: SearchRule) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.vision || step.a.type !== VISION_TYPE.visionSearch) return;
    step.a = {
      ...step.a,
      rule,
    };
  });
};

const updateNumberField = (field: string, value: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl) return;
    step.a = { ...(step.a ?? {}), [field]: Number(value) } as FlowControl;
  });
};

const updateFlowCondition = (condition: ConditionNode) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl) return;
    if (step.a.type === FLOW_TYPE.if || step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.for) {
      step.a.con = condition;
    }
  });
};

const updateFlowType = (type: string) => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl) return;
    const currentCondition = flowCondition.value ?? createConditionNode('rawExpr');
    if (type === FLOW_TYPE.if) {
      const flowSteps = step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.for ? step.a.flow : [];
      step.a = {
        type: FLOW_TYPE.if,
        con: currentCondition,
        then: flowSteps,
        else_steps: null,
      } as FlowControl;
      return;
    }

    if (type === FLOW_TYPE.while || type === FLOW_TYPE.for) {
      const branchSteps =
        step.a.type === FLOW_TYPE.if ? step.a.then : step.a.type === FLOW_TYPE.while || step.a.type === FLOW_TYPE.for ? step.a.flow : [];
      step.a = {
        type: type as typeof FLOW_TYPE.while | typeof FLOW_TYPE.for,
        con: currentCondition,
        flow: branchSteps,
      } as FlowControl;
    }
  });
};

const toggleElseBranch = () => {
  updateSelectedStep((step) => {
    if (step.op !== STEP_OP.flowControl || step.a.type !== FLOW_TYPE.if) return;
    step.a.else_steps = step.a.else_steps ? null : [];
  });
};
</script>
