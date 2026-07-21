<template>
  <div class="space-y-4">

    <template v-if="selectedAction.ac === ACTION_TYPE.capture">
      <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-4">
        <EditorVariableBindingField
          label="输出名称"
          :model-value="selectedAction.output_var || null"
          :options="resolvedCaptureOutputOptions"
          placeholder="选择或创建输出变量"
          test-id="editor-capture-output-var"
          create-label="新建 Runtime 变量"
          create-test-id="editor-capture-output-create"
          locate-test-id="editor-capture-output-locate"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedCaptureOutputTarget && jumpToVariable)"
          :locate-disabled="!selectedCaptureOutputTarget || !jumpToVariable"
          @update:model-value="$emit('update-field', 'output_var', String($event || ''))"
          @create="$emit('create-variable', 'captureOutput')"
          @locate="selectedCaptureOutputTarget ? $emit('jump-to-variable', selectedCaptureOutputTarget) : undefined"
        />
        <p class="text-xs leading-5 text-(--app-text-faint)">
          当前运行时会把截图图像对象写入 runtime 变量，不再默认转成字符串或文件路径。
        </p>
      </div>
    </template>

    <template v-else-if="selectedAction.ac === ACTION_TYPE.launchApp || selectedAction.ac === ACTION_TYPE.stopApp">
      <div class="space-y-3">
        <EditorPresetBindingSection
          label="包名"
          :model-value="launchPackageSource"
          :options="presetBindingModeOptions"
          placeholder="选择包名来源"
          test-id="editor-action-launch-pkg-source"
          @update:model-value="updateLaunchPackageSource(String($event || 'fixed'))"
        >
          <template #fixed>
            <input :value="selectedAction.pkg_name || ''" class="app-input" @input="$emit('update-field', 'pkg_name', ($event.target as HTMLInputElement).value)" />
          </template>
          <template #binding>
            <EditorVariableBindingField
              label="包名变量"
              :model-value="selectedAction.pkg_name_expr || null"
              :options="resolvedLaunchPackageVariableOptions"
              placeholder="绑定字符串变量"
              test-id="editor-action-launch-pkg-var"
              create-label="新建包名变量"
              :show-create="Boolean(createVariable)"
              :show-locate="Boolean(selectedLaunchPackageTarget && jumpToVariable)"
              :locate-disabled="!selectedLaunchPackageTarget || !jumpToVariable"
              @update:model-value="$emit('update-text-field', 'pkg_name_expr', String($event || ''))"
              @create="$emit('create-variable', 'launchPackage')"
              @locate="selectedLaunchPackageTarget ? $emit('jump-to-variable', selectedLaunchPackageTarget) : undefined"
            />
          </template>
        </EditorPresetBindingSection>

        <EditorPresetBindingSection
          v-if="selectedAction.ac === ACTION_TYPE.launchApp"
          label="Activity"
          :model-value="launchActivitySource"
          :options="presetBindingModeOptions"
          placeholder="选择 Activity 来源"
          test-id="editor-action-launch-activity-source"
          @update:model-value="updateLaunchActivitySource(String($event || 'fixed'))"
        >
          <template #fixed>
            <input :value="selectedAction.activity_name || ''" class="app-input" placeholder=".MainActivity" @input="$emit('update-field', 'activity_name', ($event.target as HTMLInputElement).value)" />
          </template>
          <template #binding>
            <EditorVariableBindingField
              label="Activity 变量"
              :model-value="selectedAction.activity_name_expr || null"
              :options="resolvedLaunchActivityVariableOptions"
              placeholder="绑定字符串变量"
              test-id="editor-action-launch-activity-var"
              create-label="新建 Activity 变量"
              :show-create="Boolean(createVariable)"
              :show-locate="Boolean(selectedLaunchActivityTarget && jumpToVariable)"
              :locate-disabled="!selectedLaunchActivityTarget || !jumpToVariable"
              @update:model-value="$emit('update-text-field', 'activity_name_expr', String($event || ''))"
              @create="$emit('create-variable', 'launchActivity')"
              @locate="selectedLaunchActivityTarget ? $emit('jump-to-variable', selectedLaunchActivityTarget) : undefined"
            />
          </template>
        </EditorPresetBindingSection>
      </div>
    </template>

    <template v-else-if="selectedAction.ac === ACTION_TYPE.posAdd || selectedAction.ac === ACTION_TYPE.posMinus">
      <div class="space-y-3">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">目标策略</span>
          <EditorSelectField
            :model-value="selectedAction.target || null"
            :options="resolvedPolicyTargetOptions"
            :show-description="true"
            placeholder="选择要调整当前位置的策略"
            test-id="editor-action-policy-position-target"
            @update:model-value="$emit('update-field', 'target', String($event || ''))"
          />
        </label>
        <div class="flex flex-wrap gap-2">
          <button
            v-if="createPolicy"
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            @click="$emit('create-policy-target')"
          >
            <AppIcon name="plus" :size="14" />
            新建策略
          </button>
          <button
            class="app-button app-button-ghost app-toolbar-button"
            type="button"
            :disabled="!selectedAction.target || !jumpToPolicy"
            @click="selectedAction.target ? $emit('jump-policy-target', selectedAction.target) : undefined"
          >
            <AppIcon name="locate-fixed" :size="14" />
            定位策略
          </button>
        </div>
        <p class="text-xs leading-5 text-(--app-text-faint)">
          只调整本次运行中的点击序号，不写回策略配置；多个目标会先按从上到下、从左到右排序，再按策略序号点击。
        </p>
      </div>
    </template>

    <template v-else-if="selectedAction.ac === ACTION_TYPE.dropSetNext">
      <EditorOverviewSection title="UI 变量切换">
        <EditorOverviewField label="目标任务">
          <EditorSelectField
            :model-value="selectedAction.task || null"
            :options="resolvedTaskTargetOptions"
            :show-description="true"
            placeholder="选择要切换 UI 变量的任务"
            test-id="editor-action-drop-set-task"
            @update:model-value="selectDropSetTarget(String($event || ''))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="UI 变量">
          <EditorSelectField
            :model-value="selectedAction.variable_id || null"
            :options="resolvedDropSetVariableOptions"
            :show-description="true"
            placeholder="选择 Select / Radio 类型绑定的变量"
            test-id="editor-action-drop-set-variable"
            @update:model-value="$emit('update-field', 'variable_id', String($event || ''))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="切换方向" width="radio">
          <EditorSelectField
            :model-value="selectedAction.direction"
            :options="dropSetDirectionOptions"
            :show-description="true"
            width="radio"
            test-id="editor-action-drop-set-direction"
            @update:model-value="$emit('update-field', 'direction', String($event || 'increase'))"
          />
        </EditorOverviewField>

        <EditorOverviewField label="循环切换" width="compact">
          <label class="flex items-center gap-3 rounded-[12px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3 text-sm text-(--app-text-soft)">
            <input
              :checked="selectedAction.cycle"
              type="checkbox"
              class="h-4 w-4"
              data-testid="editor-action-drop-set-cycle"
              style="accent-color: var(--app-accent)"
              @change="$emit('update-field', 'cycle', ($event.target as HTMLInputElement).checked ? 'true' : 'false')"
            />
            <span>到首尾后回到另一端</span>
          </label>
        </EditorOverviewField>
      </EditorOverviewSection>
    </template>

    <template v-else-if="selectedAction.ac === ACTION_TYPE.click">

      <template v-if="selectedAction.mode === ACTION_MODE.txt || selectedAction.mode === ACTION_MODE.labelIdx">
        <EditorVariableBindingField
          label="输入结果变量"
          :model-value="selectedActionInput || null"
          :options="resolvedActionInputOptions"
          placeholder="选择 OCR / 检测 / 处理结果变量"
          test-id="editor-action-click-input-var"
          create-label="新建"
          locate-test-id="editor-action-click-input-locate"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedActionInputTarget && jumpToVariable)"
          :locate-disabled="!selectedActionInputTarget || !jumpToVariable"
          @update:model-value="$emit('update-field', 'input_var', String($event || ''))"
          @create="$emit('create-variable', 'actionInput')"
          @locate="selectedActionInputTarget ? $emit('jump-to-variable', selectedActionInputTarget) : undefined"
        />
      </template>

      <div v-if="selectedAction.mode !== ACTION_MODE.point && selectedAction.mode === ACTION_MODE.percent" class="border-1"></div>
      
      <div v-if="selectedAction.mode === ACTION_MODE.txt" class="space-y-3">
        <label class="md:col-span-2 flex items-center gap-3 rounded-[16px] border border-(--app-border) bg-white/55 px-4 py-3">
          <input
            :checked="selectedAction.enable_filter ?? true"
            type="checkbox"
            class="h-4 w-4"
            style="accent-color: var(--app-accent)"
            @change="$emit('update-field', 'enable_filter', ($event.target as HTMLInputElement).checked ? 'true' : 'false')"
          />
          <div class="space-y-1">
            <p class="text-sm font-medium text-(--app-text-strong)">筛选后点击</p>
            <p class="text-xs leading-5 text-(--app-text-soft)">默认开启，按"多目标时选择第几个"点击，若不在策略中则默认第一个</p>
          </div>
        </label>
        <template v-if="selectedAction.enable_filter ?? true">
          <EditorPresetBindingSection
            label="筛选取值"
            :model-value="clickTextFilterSource"
            :options="presetBindingModeOptions"
            placeholder="选择取值来源"
            test-id="editor-action-click-text-filter-source"
            @update:model-value="updateClickTextFilterSource(String($event || 'fixed'))"
          >
            <template #fixed>
              <label class="space-y-2">
                <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">目标文字</span>
                <input :value="String(selectedAction.txt ?? '')" class="app-input" @input="$emit('update-text-field', 'txt', ($event.target as HTMLInputElement).value)" />
              </label>
            </template>
            <template #binding>
              <EditorVariableBindingField
              label="目标变量"
              :model-value="selectedAction.txt_expr || null"
              :options="resolvedClickTextVariableOptions"
              placeholder="绑定 input 文本变量"
              test-id="editor-action-click-text-var"
              create-label="新建文字变量"
              :show-create="Boolean(createVariable)"
              :show-locate="Boolean(selectedClickTextTarget && jumpToVariable)"
              :locate-disabled="!selectedClickTextTarget || !jumpToVariable"
              @update:model-value="$emit('update-text-field', 'txt_expr', String($event || ''))"
              @create="$emit('create-variable', 'clickText')"
              @locate="selectedClickTextTarget ? $emit('jump-to-variable', selectedClickTextTarget) : undefined"
            />
            </template>
          </EditorPresetBindingSection>
        </template>
      </div>

      <div v-else-if="selectedAction.mode === ACTION_MODE.labelIdx" class="space-y-3">
        <div class="space-y-3">
          <label class="flex items-center gap-3 rounded-[16px] border border-(--app-border) bg-white/55 px-4 py-3">
            <input
              :checked="selectedAction.enable_filter ?? true"
              type="checkbox"
              class="h-4 w-4"
              style="accent-color: var(--app-accent)"
              @change="$emit('update-field', 'enable_filter', ($event.target as HTMLInputElement).checked ? 'true' : 'false')"
            />
            <div class="space-y-1">
            <p class="text-sm font-medium text-(--app-text-strong)">筛选后点击</p>
            <p class="text-xs leading-5 text-(--app-text-soft)">默认开启，按"多目标时选择第几个"点击，若不在策略中则默认第一个</p>
          </div>
          </label>
          <template v-if="selectedAction.enable_filter ?? true">
            <EditorPresetBindingSection
              label="筛选取值"
              :model-value="clickLabelFilterSource"
              :options="presetBindingModeOptions"
              placeholder="选择取值来源"
              test-id="editor-action-click-label-filter-source"
              @update:model-value="updateClickLabelFilterSource(String($event || 'fixed'))"
            >
              <template #fixed>
                <div class="space-y-2">
                  <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">标签</span>
                  <AppSelect
                    :model-value="selectedAction.idx ?? null"
                    :options="resolvedLabelIdxOptions"
                    :placeholder="labelSelectPlaceholder"
                    :disabled="!(labelIndexOptions?.length)"
                    test-id="editor-action-click-label-idx"
                    @update:model-value="$emit('update-number-field', 'idx', String($event ?? 0))"
                  />
                  <p v-if="labelSelectHint" class="text-xs leading-5 text-amber-700">{{ labelSelectHint }}</p>
                </div>
              </template>
              <template #binding>
              <EditorVariableBindingField
                label="目标变量"
                :model-value="selectedAction.idx_expr || null"
                :options="resolvedClickLabelVariableOptions"
                placeholder="请选择"
                test-id="editor-action-click-label-var"
                create-label="新建整数变量"
                :show-create="Boolean(createVariable)"
                :show-locate="Boolean(selectedClickLabelTarget && jumpToVariable)"
                :locate-disabled="!selectedClickLabelTarget || !jumpToVariable"
                @update:model-value="$emit('update-text-field', 'idx_expr', String($event || ''))"
                @create="$emit('create-variable', 'clickLabel')"
                @locate="selectedClickLabelTarget ? $emit('jump-to-variable', selectedClickLabelTarget) : undefined"
              />
              </template>
            </EditorPresetBindingSection>
          </template>
        </div>
      </div>
      <div v-if="selectedAction.mode !== ACTION_MODE.point && selectedAction.mode === ACTION_MODE.percent" class="border-1"></div>
      <div class="editor-compact-grid editor-compact-grid--triple">
        <!-- <label class="editor-compact-field">
          <span class="editor-compact-field__label">点击方式</span>
          <EditorSelectField
            :model-value="String(selectedAction.mode || ACTION_MODE.point)"
            :options="clickModeOptions"
            placeholder="点击方式"
            @update:model-value="$emit('update-mode', String($event || ACTION_MODE.point))"
          />
        </label> -->

        <label class="editor-compact-field">
          <span class="editor-compact-field__label">偏移 X</span>
          <input
            :value="String(selectedAction.offset_x ?? 0)"
            class="app-input"
            type="number"
            @input="$emit('update-number-field', 'offset_x', ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="editor-compact-field">
          <span class="editor-compact-field__label">偏移 Y</span>
          <input
            :value="String(selectedAction.offset_y ?? 0)"
            class="app-input"
            type="number"
            @input="$emit('update-number-field', 'offset_y', ($event.target as HTMLInputElement).value)"
          />
        </label>
      </div>

      <EditorPresetBindingSection
        v-if="selectedAction.mode === ACTION_MODE.point || selectedAction.mode === ACTION_MODE.percent"
        :label="selectedAction.mode === ACTION_MODE.point ? '坐标来源' : '百分比来源'"
        :model-value="clickPointSource"
        :options="presetBindingModeOptions"
        placeholder="选择点位来源"
        test-id="editor-action-click-point-source"
        @update:model-value="updateClickPointSource(String($event || 'fixed'))"
      >
        <template #fixed>
          <div class="editor-compact-grid">
            <label class="editor-compact-field">
              <span class="editor-compact-field__label">X</span>
              <input
                :value="String((selectedAction.p as { x?: number })?.x ?? '')"
                class="app-input"
                type="number"
                @input="$emit('update-point-field', 'p', 'x', ($event.target as HTMLInputElement).value)"
              />
            </label>
            <label class="editor-compact-field">
              <span class="editor-compact-field__label">Y</span>
              <input
                :value="String((selectedAction.p as { y?: number })?.y ?? '')"
                class="app-input"
                type="number"
                @input="$emit('update-point-field', 'p', 'y', ($event.target as HTMLInputElement).value)"
              />
            </label>
          </div>
        </template>
        <template #binding>
          <EditorVariableBindingField
            label="点位变量"
            :model-value="selectedAction.p_expr || null"
            :options="resolvedClickPointVariableOptions"
            placeholder="绑定 JSON 点位变量"
            test-id="editor-action-click-point-var"
            create-label="新建点位变量"
            :show-create="Boolean(createVariable)"
            :show-locate="Boolean(selectedClickPointTarget && jumpToVariable)"
            :locate-disabled="!selectedClickPointTarget || !jumpToVariable"
            @update:model-value="$emit('update-text-field', 'p_expr', String($event || ''))"
            @create="$emit('create-variable', 'clickPoint')"
            @locate="selectedClickPointTarget ? $emit('jump-to-variable', selectedClickPointTarget) : undefined"
          />
          <p class="text-xs leading-5 text-(--app-text-soft)">
            变量值使用 JSON `{ "x": ..., "y": ... }`。坐标模式读绝对值；百分比模式读 0 到 1。
          </p>
        </template>
      </EditorPresetBindingSection>

    </template>

    <template v-else-if="selectedAction.ac === ACTION_TYPE.swipe">
      <div class="editor-compact-grid">
        <label class="editor-compact-field">
          <span class="editor-compact-field__label">滑动方式</span>
          <EditorSelectField
            :model-value="String(selectedAction.mode || ACTION_MODE.point)"
            :options="swipeModeOptions"
            placeholder="滑动方式"
            @update:model-value="$emit('update-mode', String($event || ACTION_MODE.point))"
          />
        </label>
        <label class="editor-compact-field">
          <span class="editor-compact-field__label">持续时间</span>
          <input :value="String(selectedAction.duration ?? 300)" class="app-input" type="number" @input="$emit('update-number-field', 'duration', ($event.target as HTMLInputElement).value)" />
        </label>
      </div>

      <template v-if="selectedAction.mode === ACTION_MODE.txt || selectedAction.mode === ACTION_MODE.labelIdx">
        <EditorVariableBindingField
          label="输入结果变量"
          :model-value="selectedActionInput || null"
          :options="resolvedActionInputOptions"
          placeholder="选择 OCR / 检测 / 处理结果变量"
          test-id="editor-action-swipe-input-var"
          create-label="新建结果变量"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedActionInputTarget && jumpToVariable)"
          :locate-disabled="!selectedActionInputTarget || !jumpToVariable"
          @update:model-value="$emit('update-field', 'input_var', String($event || ''))"
          @create="$emit('create-variable', 'actionInput')"
          @locate="selectedActionInputTarget ? $emit('jump-to-variable', selectedActionInputTarget) : undefined"
        />
      </template>

      <div v-if="selectedAction.mode === ACTION_MODE.point || selectedAction.mode === ACTION_MODE.percent" class="editor-compact-grid editor-compact-grid--quad">
        <label class="editor-compact-field">
          <span class="editor-compact-field__label">起点 X</span>
          <input
            :value="String((selectedAction.from as { x?: number })?.x ?? '')"
            aria-label="起点 X"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'from', 'x', ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="editor-compact-field">
          <span class="editor-compact-field__label">起点 Y</span>
          <input
            :value="String((selectedAction.from as { y?: number })?.y ?? '')"
            aria-label="起点 Y"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'from', 'y', ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="editor-compact-field">
          <span class="editor-compact-field__label">终点 X</span>
          <input
            :value="String((selectedAction.to as { x?: number })?.x ?? '')"
            aria-label="终点 X"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'to', 'x', ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="editor-compact-field">
          <span class="editor-compact-field__label">终点 Y</span>
          <input
            :value="String((selectedAction.to as { y?: number })?.y ?? '')"
            aria-label="终点 Y"
            class="app-input"
            type="number"
            @input="$emit('update-point-field', 'to', 'y', ($event.target as HTMLInputElement).value)"
          />
        </label>
      </div>

      <div v-if="selectedAction.mode === ACTION_MODE.txt" class="grid gap-3 md:grid-cols-2">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">固定起点文字</span>
          <input :value="String(selectedAction.from ?? '')" class="app-input" @input="$emit('update-text-field', 'from', ($event.target as HTMLInputElement).value)" />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">固定终点文字</span>
          <input :value="String(selectedAction.to ?? '')" class="app-input" @input="$emit('update-text-field', 'to', ($event.target as HTMLInputElement).value)" />
        </label>
        <EditorVariableBindingField
          label="起点文字变量"
          :model-value="selectedAction.from_expr || null"
          :options="resolvedSwipeFromTextVariableOptions"
          placeholder="绑定文字变量"
          test-id="editor-action-swipe-from-text-var"
          create-label="新建起点文字变量"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedSwipeFromTextTarget && jumpToVariable)"
          :locate-disabled="!selectedSwipeFromTextTarget || !jumpToVariable"
          @update:model-value="$emit('update-text-field', 'from_expr', String($event || ''))"
          @create="$emit('create-variable', 'swipeFromText')"
          @locate="selectedSwipeFromTextTarget ? $emit('jump-to-variable', selectedSwipeFromTextTarget) : undefined"
        />
        <EditorVariableBindingField
          label="终点文字变量"
          :model-value="selectedAction.to_expr || null"
          :options="resolvedSwipeToTextVariableOptions"
          placeholder="绑定文字变量"
          test-id="editor-action-swipe-to-text-var"
          create-label="新建终点文字变量"
          :show-create="Boolean(createVariable)"
          :show-locate="Boolean(selectedSwipeToTextTarget && jumpToVariable)"
          :locate-disabled="!selectedSwipeToTextTarget || !jumpToVariable"
          @update:model-value="$emit('update-text-field', 'to_expr', String($event || ''))"
          @create="$emit('create-variable', 'swipeToText')"
          @locate="selectedSwipeToTextTarget ? $emit('jump-to-variable', selectedSwipeToTextTarget) : undefined"
        />
      </div>

      <div v-else-if="selectedAction.mode === ACTION_MODE.labelIdx" class="grid gap-3 md:grid-cols-2">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">起点标签</span>
          <AppSelect
            :model-value="typeof selectedAction.from === 'number' ? selectedAction.from : null"
            :options="resolvedSwipeFromLabelOptions"
            :placeholder="labelSelectPlaceholder"
            :disabled="!(labelIndexOptions?.length)"
            test-id="editor-action-swipe-label-from"
            @update:model-value="$emit('update-number-field', 'from', String($event ?? 0))"
          />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">终点标签</span>
          <AppSelect
            :model-value="typeof selectedAction.to === 'number' ? selectedAction.to : null"
            :options="resolvedSwipeToLabelOptions"
            :placeholder="labelSelectPlaceholder"
            :disabled="!(labelIndexOptions?.length)"
            test-id="editor-action-swipe-label-to"
            @update:model-value="$emit('update-number-field', 'to', String($event ?? 0))"
          />
        </label>
        <p v-if="labelSelectHint" class="md:col-span-2 text-xs leading-5 text-amber-700">{{ labelSelectHint }}</p>
      </div>

      <div v-else-if="selectedAction.mode === ACTION_MODE.mixed" class="grid gap-3 md:grid-cols-2">
        <MixedSwipeTargetEditor
          label="起点"
          target-key="from"
          :target="selectedAction.from"
          :source-options="swipeTargetSourceOptions"
          :variable-options="resolvedActionInputOptions"
          :label-options="resolvedSwipeFromLabelOptions"
          :label-placeholder="labelSelectPlaceholder"
          @update-field="(field, value) => $emit('update-swipe-target-field', 'from', field, value)"
        />
        <MixedSwipeTargetEditor
          label="终点"
          target-key="to"
          :target="selectedAction.to"
          :source-options="swipeTargetSourceOptions"
          :variable-options="resolvedActionInputOptions"
          :label-options="resolvedSwipeToLabelOptions"
          :label-placeholder="labelSelectPlaceholder"
          @update-field="(field, value) => $emit('update-swipe-target-field', 'to', field, value)"
        />
        <p v-if="labelSelectHint" class="md:col-span-2 text-xs leading-5 text-amber-700">{{ labelSelectHint }}</p>
      </div>
    </template>

    <div class="border-1"></div>
    <EditorOverviewField label="执行次数" width="compact">
      <input
        :value="String(actionExecMax)"
        class="app-input"
        type="number"
        min="0"
        data-testid="editor-action-exec-max"
        @input="$emit('update-exec-max', ($event.target as HTMLInputElement).value)"
        placeholder="0表示无限次"
      />
    </EditorOverviewField>
  </div>
</template>

<script setup lang="ts">
import { computed, defineComponent, h, ref, watch, type PropType } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import EditorOverviewSection from '@/views/script-editor/EditorOverviewSection.vue';
import EditorPresetBindingSection from '@/views/script-editor/EditorPresetBindingSection.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import EditorVariableBindingField from '@/views/script-editor/EditorVariableBindingField.vue';
import type { Action } from '@/types/bindings/Action';
import { ACTION_MODE, ACTION_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';
import type { EditorReferenceOption, EditorTaskUiVariableOption } from '@/views/script-editor/editorReferences';
import type { EditorInputType, EditorVariableOption } from '@/views/script-editor/editorVariables';
import EditorOverviewField from '../EditorOverviewField.vue';

defineOptions({ name: 'EditorStepActionPanel' });

const props = defineProps<{
  selectedAction: Action;
  actionExecMax: number;
  variableDatalistId: string;
  writableCatalogVariableOptions?: Array<{ label: string; value: string; description: string; disabled?: boolean }>;
  resultCatalogVariableOptions?: SelectOption[];
  textVariableOptions?: SelectOption[];
  numberVariableOptions?: SelectOption[];
  labelIndexOptions?: LabelSelectOption[];
  labelSelectPlaceholder?: string;
  labelSelectHint?: string | null;
  selectedCaptureOutputTarget?: EditorVariableOption | null;
  selectedActionInputTarget?: EditorVariableOption | null;
  selectedClickTextTarget?: EditorVariableOption | null;
  selectedClickLabelTarget?: EditorVariableOption | null;
  selectedSwipeFromTextTarget?: EditorVariableOption | null;
  selectedSwipeToTextTarget?: EditorVariableOption | null;
  policyReferenceOptions?: EditorReferenceOption[];
  taskReferenceOptions?: EditorReferenceOption[];
  taskUiVariableOptions?: EditorTaskUiVariableOption[];
  clickModeOptions: Array<{ label: string; value: string; description: string }>;
  swipeModeOptions: Array<{ label: string; value: string; description: string }>;
  createVariable?: (namespace?: 'input' | 'runtime', inputType?: EditorInputType, options?: { preferredKey?: string; name?: string; select?: boolean; silent?: boolean; focusEditor?: boolean }) => Promise<string>;
  jumpToVariable?: (option: EditorVariableOption) => void;
  createPolicy?: () => Promise<string>;
  jumpToPolicy?: (id: string) => void;
  jsonVariableOptions?: SelectOption[];
  selectedLaunchPackageTarget?: EditorVariableOption | null;
  selectedLaunchActivityTarget?: EditorVariableOption | null;
  selectedClickPointTarget?: EditorVariableOption | null;
}>();

const emit = defineEmits<{
  'update-exec-max': [value: string];
  'update-field': [field: string, value: string];
  'update-mode': [mode: string];
  'update-point-field': [field: 'p' | 'from' | 'to', axis: 'x' | 'y', value: string];
  'update-number-field': [field: string, value: string];
  'update-text-field': [field: string, value: string];
  'update-swipe-target-field': [target: 'from' | 'to', field: string, value: string | number | null];
  'create-variable': [target: 'captureOutput' | 'actionInput' | 'clickText' | 'clickLabel' | 'swipeFromText' | 'swipeToText' | 'launchPackage' | 'launchActivity' | 'clickPoint'];
  'jump-to-variable': [option: EditorVariableOption];
  'create-policy-target': [];
  'jump-policy-target': [id: string];
  'update-drop-set-target': [taskId: string, variableId: string];
  'update-input': [entryId: string, field: 'key' | 'name' | 'description' | 'namespace' | 'type' | 'stringValue' | 'booleanValue', value: string | boolean];
}>();

type SelectOption = { label: string; value: string; description?: string; disabled?: boolean };
type LabelSelectOption = { label: string; value: number; description?: string; disabled?: boolean };
type MixedSwipeTarget = {
  source: typeof ACTION_MODE.txt | typeof ACTION_MODE.labelIdx;
  input_var: string;
  value?: string | null;
  value_expr?: string | null;
  idx?: number;
};

const swipeTargetSourceOptions = [
  { label: '文字', value: ACTION_MODE.txt, description: '从 OCR 结果中取目标中心点。' },
  { label: '标签', value: ACTION_MODE.labelIdx, description: '从检测结果中取目标中心点。' },
];

const filterSourceOptions = [
  { label: '预设', value: 'fixed', description: '使用步骤里填写的目标文字或目标标签。' },
  { label: '绑定变量', value: 'expr', description: '绑定变量' },
];
const dropSetDirectionOptions: SelectOption[] = [
  { label: '向前', value: 'increase', description: '' },
  { label: '向后', value: 'decrease', description: '' },
];
const presetBindingModeOptions = [
  { label: '预设', value: 'fixed', description: '使用步骤里直接填写的固定值。' },
  { label: '绑定变量', value: 'expr', description: '从变量里读取当前值。' },
];

const MixedSwipeTargetEditor = defineComponent({
  name: 'MixedSwipeTargetEditor',
  props: {
    label: { type: String, required: true },
    targetKey: { type: String as PropType<'from' | 'to'>, required: true },
    target: { type: Object as PropType<MixedSwipeTarget>, required: true },
    sourceOptions: { type: Array as PropType<SelectOption[]>, required: true },
    variableOptions: { type: Array as PropType<SelectOption[]>, required: true },
    labelOptions: { type: Array as PropType<LabelSelectOption[]>, required: true },
    labelPlaceholder: { type: String, default: '选择标签' },
  },
  emits: ['update-field'],
  setup(componentProps, { emit: componentEmit }) {
    return () =>
      h('div', { class: 'space-y-3 rounded-[16px] border border-(--app-border) bg-(--app-panel-muted) px-4 py-3' }, [
        h('p', { class: 'text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)' }, componentProps.label),
        h(EditorSelectField, {
          modelValue: componentProps.target.source,
          options: componentProps.sourceOptions,
          placeholder: '目标来源',
          'onUpdate:modelValue': (value: unknown) => componentEmit('update-field', 'source', String(value || ACTION_MODE.txt)),
        }),
        h(EditorSelectField, {
          modelValue: componentProps.target.input_var || null,
          options: componentProps.variableOptions,
          placeholder: '结果变量',
          'onUpdate:modelValue': (value: unknown) => componentEmit('update-field', 'input_var', String(value || '')),
        }),
        componentProps.target.source === ACTION_MODE.labelIdx
          ? h(AppSelect, {
              modelValue: componentProps.target.idx ?? null,
              options: componentProps.labelOptions,
              placeholder: componentProps.labelPlaceholder,
              'onUpdate:modelValue': (value: unknown) => componentEmit('update-field', 'idx', Number(value ?? 0)),
            })
          : h('input', {
              value: String(componentProps.target.value ?? ''),
              class: 'app-input',
              placeholder: '目标文字',
              onInput: (event: Event) => componentEmit('update-field', 'value', (event.target as HTMLInputElement).value),
            }),
      ]);
  },
});

const withCurrentVariableOption = (options: SelectOption[], value: string) => {
  const trimmedValue = value.trim();
  if (!trimmedValue || options.some((option) => option.value === trimmedValue)) {
    return options;
  }

  return [
    {
      label: `（不存在）：${trimmedValue}`,
      value: trimmedValue,
      description: '变量目录里找不到该绑定，保存时仍会保留当前值。',
    },
    ...options,
  ];
};

const withCurrentLabelOption = (options: LabelSelectOption[], value: number | null | undefined) => {
  if (value === null || value === undefined || Number.isNaN(value)) {
    return options;
  }

  if (options.some((option) => option.value === value)) {
    return options;
  }

  return [
    {
      label: `${value}: 未找到标签`,
      value,
      description: '标签文件中不存在该索引，保存时仍会保留当前 idx。',
    },
    ...options,
  ];
};

const resolvedCaptureOutputOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.capture
    ? withCurrentVariableOption(props.writableCatalogVariableOptions ?? [], props.selectedAction.output_var ?? '')
    : props.writableCatalogVariableOptions ?? [],
);

const resolvedPolicyTargetOptions = computed(() => {
  const options = props.policyReferenceOptions ?? [];
  if (props.selectedAction.ac !== ACTION_TYPE.posAdd && props.selectedAction.ac !== ACTION_TYPE.posMinus) {
    return options;
  }
  const target = props.selectedAction.target?.trim() ?? '';
  if (!target || options.some((option) => option.value === target)) {
    return options;
  }
  return [
    {
      label: target,
      value: target,
      description: '未解析策略',
    },
    ...options,
  ];
});

const resolvedTaskTargetOptions = computed(() => {
  const options = props.taskReferenceOptions ?? [];
  if (props.selectedAction.ac !== ACTION_TYPE.dropSetNext) {
    return options;
  }
  const taskId = props.selectedAction.task?.trim() ?? '';
  if (!taskId || options.some((option) => option.value === taskId)) {
    return options;
  }
  return [
    {
      label: taskId,
      value: taskId,
      description: '未解析任务',
    },
    ...options,
  ];
});

const dropSetVariableOptions = computed(() => {
  if (props.selectedAction.ac !== ACTION_TYPE.dropSetNext) {
    return [];
  }
  const taskId = props.selectedAction.task?.trim() ?? '';
  return (props.taskUiVariableOptions ?? [])
    .filter((option) => !taskId || option.taskId === taskId)
    .map((option) => ({
      label: option.taskLabel,
      value: option.variableId,
      description: option.description ?? `${option.taskLabel} · ${option.options.length} 个选项`,
    }));
});

const resolvedDropSetVariableOptions = computed(() => {
  const options = dropSetVariableOptions.value;
  if (props.selectedAction.ac !== ACTION_TYPE.dropSetNext) {
    return options;
  }
  const variableId = props.selectedAction.variable_id?.trim() ?? '';
  if (!variableId || options.some((option) => option.value === variableId)) {
    return options;
  }
  return [
    {
      label: variableId,
      value: variableId,
      description: '未解析 UI 变量',
    },
    ...options,
  ];
});

const selectDropSetTarget = (taskId: string) => {
  if (props.selectedAction.ac !== ACTION_TYPE.dropSetNext) {
    return;
  }
  const currentVariableId = props.selectedAction.variable_id?.trim() ?? '';
  const nextOptions = (props.taskUiVariableOptions ?? []).filter((option) => option.taskId === taskId);
  const variableId = nextOptions.some((option) => option.variableId === currentVariableId)
    ? currentVariableId
    : nextOptions[0]?.variableId ?? '';
  emit('update-drop-set-target', taskId, variableId);
};

const selectedActionInput = computed(() => {
  if (props.selectedAction.ac !== ACTION_TYPE.click && props.selectedAction.ac !== ACTION_TYPE.swipe) {
    return '';
  }

  if (props.selectedAction.mode === ACTION_MODE.txt || props.selectedAction.mode === ACTION_MODE.labelIdx) {
    return String((props.selectedAction as { input_var?: string }).input_var ?? '');
  }

  return '';
});

const resolvedActionInputOptions = computed(() =>
  withCurrentVariableOption(props.resultCatalogVariableOptions ?? [], selectedActionInput.value),
);
const resolvedLaunchPackageVariableOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.launchApp || props.selectedAction.ac === ACTION_TYPE.stopApp
    ? withCurrentVariableOption(props.textVariableOptions ?? [], props.selectedAction.pkg_name_expr ?? '')
    : props.textVariableOptions ?? [],
);
const resolvedLaunchActivityVariableOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.launchApp
    ? withCurrentVariableOption(props.textVariableOptions ?? [], props.selectedAction.activity_name_expr ?? '')
    : props.textVariableOptions ?? [],
);
const resolvedClickPointVariableOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.click &&
  (props.selectedAction.mode === ACTION_MODE.point || props.selectedAction.mode === ACTION_MODE.percent)
    ? withCurrentVariableOption(props.jsonVariableOptions ?? [], props.selectedAction.p_expr ?? '')
    : props.jsonVariableOptions ?? [],
);
const resolvedClickTextVariableOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.click && props.selectedAction.mode === ACTION_MODE.txt
    ? withCurrentVariableOption(props.textVariableOptions ?? [], props.selectedAction.txt_expr ?? '')
    : props.textVariableOptions ?? [],
);
const resolvedClickLabelVariableOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.click && props.selectedAction.mode === ACTION_MODE.labelIdx
    ? withCurrentVariableOption(props.numberVariableOptions ?? [], props.selectedAction.idx_expr ?? '')
    : props.numberVariableOptions ?? [],
);
const resolvedSwipeFromTextVariableOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.swipe && props.selectedAction.mode === ACTION_MODE.txt
    ? withCurrentVariableOption(props.textVariableOptions ?? [], props.selectedAction.from_expr ?? '')
    : props.textVariableOptions ?? [],
);
const resolvedSwipeToTextVariableOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.swipe && props.selectedAction.mode === ACTION_MODE.txt
    ? withCurrentVariableOption(props.textVariableOptions ?? [], props.selectedAction.to_expr ?? '')
    : props.textVariableOptions ?? [],
);

const resolvedLabelIdxOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.click && props.selectedAction.mode === ACTION_MODE.labelIdx
    ? withCurrentLabelOption(props.labelIndexOptions ?? [], props.selectedAction.idx ?? null)
    : props.labelIndexOptions ?? [],
);

const resolvedSwipeFromLabelOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.swipe && props.selectedAction.mode === ACTION_MODE.labelIdx
    ? withCurrentLabelOption(props.labelIndexOptions ?? [], typeof props.selectedAction.from === 'number' ? props.selectedAction.from : null)
    : props.labelIndexOptions ?? [],
);

const resolvedSwipeToLabelOptions = computed(() =>
  props.selectedAction.ac === ACTION_TYPE.swipe && props.selectedAction.mode === ACTION_MODE.labelIdx
    ? withCurrentLabelOption(props.labelIndexOptions ?? [], typeof props.selectedAction.to === 'number' ? props.selectedAction.to : null)
    : props.labelIndexOptions ?? [],
);

const clickTextFilterSource = computed(() =>
  clickTextFilterSourceState.value,
);

const clickLabelFilterSource = computed(() =>
  clickLabelFilterSourceState.value,
);

const clickTextFilterSourceState = ref<'fixed' | 'expr'>('fixed');
const clickLabelFilterSourceState = ref<'fixed' | 'expr'>('fixed');
const launchPackageSourceState = ref<'fixed' | 'expr'>('fixed');
const launchActivitySourceState = ref<'fixed' | 'expr'>('fixed');
const clickPointSourceState = ref<'fixed' | 'expr'>('fixed');

watch(
  () => [
    props.selectedAction.ac,
    (props.selectedAction.ac === ACTION_TYPE.launchApp || props.selectedAction.ac === ACTION_TYPE.stopApp) ? props.selectedAction.pkg_name_expr ?? '' : '',
  ],
  ([ac, pkgExpr]) => {
    if (ac !== ACTION_TYPE.launchApp && ac !== ACTION_TYPE.stopApp) {
      launchPackageSourceState.value = 'fixed';
      return;
    }
    if (String(pkgExpr).trim()) {
      launchPackageSourceState.value = 'expr';
      return;
    }
    if (launchPackageSourceState.value !== 'expr') {
      launchPackageSourceState.value = 'fixed';
    }
  },
  { immediate: true },
);

watch(
  () => [
    props.selectedAction.ac,
    props.selectedAction.ac === ACTION_TYPE.launchApp ? props.selectedAction.activity_name_expr ?? '' : '',
  ],
  ([ac, activityExpr]) => {
    if (ac !== ACTION_TYPE.launchApp) {
      launchActivitySourceState.value = 'fixed';
      return;
    }
    if (String(activityExpr).trim()) {
      launchActivitySourceState.value = 'expr';
      return;
    }
    if (launchActivitySourceState.value !== 'expr') {
      launchActivitySourceState.value = 'fixed';
    }
  },
  { immediate: true },
);

watch(
  () => [
    props.selectedAction.ac,
    props.selectedAction.ac === ACTION_TYPE.click ? props.selectedAction.mode : null,
    props.selectedAction.ac === ACTION_TYPE.click &&
    (props.selectedAction.mode === ACTION_MODE.point || props.selectedAction.mode === ACTION_MODE.percent)
      ? props.selectedAction.p_expr ?? ''
      : '',
  ],
  ([ac, mode, pointExpr]) => {
    if (ac !== ACTION_TYPE.click || (mode !== ACTION_MODE.point && mode !== ACTION_MODE.percent)) {
      clickPointSourceState.value = 'fixed';
      return;
    }
    if (String(pointExpr).trim()) {
      clickPointSourceState.value = 'expr';
      return;
    }
    if (clickPointSourceState.value !== 'expr') {
      clickPointSourceState.value = 'fixed';
    }
  },
  { immediate: true },
);

watch(
  () => [
    props.selectedAction.ac,
    props.selectedAction.ac === ACTION_TYPE.click ? props.selectedAction.mode : null,
    props.selectedAction.ac === ACTION_TYPE.click && props.selectedAction.mode === ACTION_MODE.txt ? props.selectedAction.txt_expr ?? '' : '',
  ],
  ([ac, mode, txtExpr]) => {
    if (ac !== ACTION_TYPE.click || mode !== ACTION_MODE.txt) {
      clickTextFilterSourceState.value = 'fixed';
      return;
    }
    if (String(txtExpr).trim()) {
      clickTextFilterSourceState.value = 'expr';
      return;
    }
    if (clickTextFilterSourceState.value !== 'expr') {
      clickTextFilterSourceState.value = 'fixed';
    }
  },
  { immediate: true },
);

watch(
  () => [
    props.selectedAction.ac,
    props.selectedAction.ac === ACTION_TYPE.click ? props.selectedAction.mode : null,
    props.selectedAction.ac === ACTION_TYPE.click && props.selectedAction.mode === ACTION_MODE.labelIdx ? props.selectedAction.idx_expr ?? '' : '',
  ],
  ([ac, mode, idxExpr]) => {
    if (ac !== ACTION_TYPE.click || mode !== ACTION_MODE.labelIdx) {
      clickLabelFilterSourceState.value = 'fixed';
      return;
    }
    if (String(idxExpr).trim()) {
      clickLabelFilterSourceState.value = 'expr';
      return;
    }
    if (clickLabelFilterSourceState.value !== 'expr') {
      clickLabelFilterSourceState.value = 'fixed';
    }
  },
  { immediate: true },
);

const launchPackageSource = computed(() =>
  launchPackageSourceState.value,
);
const launchActivitySource = computed(() =>
  launchActivitySourceState.value,
);
const clickPointSource = computed(() =>
  clickPointSourceState.value,
);

const updateLaunchPackageSource = (value: string) => {
  launchPackageSourceState.value = value === 'expr' ? 'expr' : 'fixed';
  if (value === 'expr') {
    const fallbackValue =
      props.selectedAction.ac === ACTION_TYPE.launchApp || props.selectedAction.ac === ACTION_TYPE.stopApp
        ? props.selectedAction.pkg_name_expr?.trim() || resolvedLaunchPackageVariableOptions.value[0]?.value || ''
        : '';
    emit('update-text-field', 'pkg_name_expr', fallbackValue);
    return;
  }
  emit('update-text-field', 'pkg_name_expr', '');
};

const updateLaunchActivitySource = (value: string) => {
  launchActivitySourceState.value = value === 'expr' ? 'expr' : 'fixed';
  if (value === 'expr') {
    const fallbackValue =
      props.selectedAction.ac === ACTION_TYPE.launchApp
        ? props.selectedAction.activity_name_expr?.trim() || resolvedLaunchActivityVariableOptions.value[0]?.value || ''
        : '';
    emit('update-text-field', 'activity_name_expr', fallbackValue);
    return;
  }
  emit('update-text-field', 'activity_name_expr', '');
};

const updateClickPointSource = (value: string) => {
  clickPointSourceState.value = value === 'expr' ? 'expr' : 'fixed';
  if (value === 'expr') {
    const fallbackValue =
      props.selectedAction.ac === ACTION_TYPE.click &&
      (props.selectedAction.mode === ACTION_MODE.point || props.selectedAction.mode === ACTION_MODE.percent)
        ? props.selectedAction.p_expr?.trim() || resolvedClickPointVariableOptions.value[0]?.value || ''
        : '';
    emit('update-text-field', 'p_expr', fallbackValue);
    return;
  }
  emit('update-text-field', 'p_expr', '');
};

const updateClickTextFilterSource = (value: string) => {
  clickTextFilterSourceState.value = value === 'expr' ? 'expr' : 'fixed';
  if (value === 'expr') {
    const fallbackValue =
      props.selectedAction.ac === ACTION_TYPE.click && props.selectedAction.mode === ACTION_MODE.txt
        ? props.selectedAction.txt_expr?.trim() || resolvedClickTextVariableOptions.value[0]?.value || ''
        : '';
    emit('update-text-field', 'txt_expr', fallbackValue);
    return;
  }
  emit('update-text-field', 'txt_expr', '');
};

const updateClickLabelFilterSource = (value: string) => {
  clickLabelFilterSourceState.value = value === 'expr' ? 'expr' : 'fixed';
  if (value === 'expr') {
    const fallbackValue =
      props.selectedAction.ac === ACTION_TYPE.click && props.selectedAction.mode === ACTION_MODE.labelIdx
        ? props.selectedAction.idx_expr?.trim() || resolvedClickLabelVariableOptions.value[0]?.value || ''
        : '';
    emit('update-text-field', 'idx_expr', fallbackValue);
    return;
  }
  emit('update-text-field', 'idx_expr', '');
};
</script>

<style scoped>
.editor-config-strip {
  display: flex;
  flex-wrap: wrap;
  align-items: end;
  gap: 0.75rem 1rem;
  padding: 0.875rem 1rem;
  border: 1px solid var(--app-border);
  border-radius: 16px;
  background: color-mix(in srgb, var(--app-panel-muted) 76%, white);
}

.editor-config-strip__hint {
  flex: 1 1 14rem;
  min-height: 44px;
  display: flex;
  align-items: center;
  color: var(--app-text-faint);
  font-size: 0.78rem;
  line-height: 1.5;
}

.editor-compact-grid {
  display: grid;
  gap: 0.75rem;
}

@media (min-width: 300px) {
  .editor-compact-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .editor-compact-grid--triple {
    grid-template-columns: minmax(0, 1.2fr) repeat(2, minmax(0, 0.9fr));
  }

  .editor-compact-grid--quad {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
}

.editor-compact-field {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 0.5rem;
}

.editor-compact-field__label {
  color: var(--app-text-faint);
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}
</style>
