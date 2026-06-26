<template>
  <div class="space-y-4">
    <template v-if="selectedVision.type === VISION_TYPE.detect || selectedVision.type === VISION_TYPE.ocr">
      <div class="space-y-3">
        <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">输入图像变量</span>
            <EditorSelectField
              :model-value="selectedVision.input_var || null"
              :options="resolvedVisionInputOptions"
              :show-description="true"
              placeholder="选择截图或图像变量"
              :test-id="selectedVision.type === VISION_TYPE.detect ? 'editor-vision-detect-input-var' : 'editor-vision-ocr-input-var'"
              @update:model-value="$emit('update-field', 'input_var', String($event || ''))"
            />
          </label>
          <div class="flex flex-wrap gap-2">
            <button
              v-if="createVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="$emit('create-variable', 'visionInput')"
            >
              <AppIcon name="plus" :size="14" />
              新建图像变量
            </button>
            <button
              v-if="selectedVisionInputTarget && jumpToVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="$emit('jump-to-variable', selectedVisionInputTarget)"
            >
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>
        </div>

        <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">输出结果变量</span>
            <EditorSelectField
              :model-value="selectedVision.out_var || null"
              :options="resolvedVisionOutputOptions"
              :show-description="true"
              placeholder="选择或创建结果变量"
              :test-id="selectedVision.type === VISION_TYPE.detect ? 'editor-vision-detect-output-var' : 'editor-vision-ocr-output-var'"
              @update:model-value="$emit('update-field', 'out_var', String($event || ''))"
            />
          </label>
          <div class="flex flex-wrap gap-2">
            <button
              v-if="createVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="$emit('create-variable', 'visionOutput')"
            >
              <AppIcon name="plus" :size="14" />
              新建结果变量
            </button>
            <button
              v-if="selectedVisionOutputTarget && jumpToVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="$emit('jump-to-variable', selectedVisionOutputTarget)"
            >
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>
        </div>
      </div>
    </template>

    <template v-else-if="selectedVision.type === VISION_TYPE.countCompare">
      <div class="grid gap-3 xl:grid-cols-2">
        <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">输入结果变量</span>
            <EditorSelectField
              :model-value="selectedVision.input_var || null"
              :options="resolvedVisionReadableOptions"
              :show-description="true"
              placeholder="选择检测结果或 OCR 结果变量"
              test-id="editor-vision-count-compare-input-var"
              @update:model-value="$emit('update-field', 'input_var', String($event || ''))"
            />
          </label>
          <div class="flex flex-wrap gap-2">
            <button
              v-if="createVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="$emit('create-variable', 'visionInput')"
            >
              <AppIcon name="plus" :size="14" />
              新建结果变量
            </button>
            <button
              v-if="selectedVisionInputTarget && jumpToVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="$emit('jump-to-variable', selectedVisionInputTarget)"
            >
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>
        </div>

        <div class="space-y-3 rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">输出布尔变量</span>
            <EditorSelectField
              :model-value="selectedVision.out_var || null"
              :options="resolvedVisionBoolOutputOptions"
              :show-description="true"
              placeholder="选择或创建布尔变量"
              test-id="editor-vision-count-compare-output-var"
              @update:model-value="$emit('update-field', 'out_var', String($event || ''))"
            />
          </label>
          <div class="flex flex-wrap gap-2">
            <button
              v-if="createVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="$emit('create-variable', 'visionOutput')"
            >
              <AppIcon name="plus" :size="14" />
              新建布尔变量
            </button>
            <button
              v-if="selectedVisionOutputTarget && jumpToVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              @click="$emit('jump-to-variable', selectedVisionOutputTarget)"
            >
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>
        </div>
      </div>

      <div class="grid gap-3 md:grid-cols-3">
        <label class="space-y-2 md:col-span-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">目标标签 / 文字</span>
          <input
            :value="selectedVision.target_value ?? ''"
            class="app-input"
            placeholder="留空则统计全部结果"
            test-id="editor-vision-count-compare-target-value"
            @input="$emit('update-nullable-field', 'target_value', ($event.target as HTMLInputElement).value)"
          />
        </label>
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">比较方式</span>
          <EditorSelectField
            :model-value="selectedVision.op ?? 'ge'"
            :options="countCompareOpOptions"
            placeholder="比较方式"
            test-id="editor-vision-count-compare-op"
            @update:model-value="$emit('update-field', 'op', String($event || 'ge'))"
          />
        </label>
      </div>

      <div class="grid gap-3 md:grid-cols-[minmax(0,1fr)_240px]">
        <label class="space-y-2">
          <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">指定数量</span>
          <input
            :value="String(selectedVision.expected_count ?? 0)"
            class="app-input"
            type="number"
            test-id="editor-vision-count-compare-expected-count"
            @input="$emit('update-number-field', 'expected_count', ($event.target as HTMLInputElement).value)"
          />
        </label>

        <div class="rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
          <div class="editor-inline-grid">
            <div class="editor-inline-label">命中后</div>
            <div class="editor-inline-content flex items-center justify-between gap-3">
              <span class="text-sm text-(--app-text-soft)">{{ visionBranchTarget?.count ?? 0 }} 个步骤</span>
              <button
                v-if="visionBranchTarget"
                class="app-button app-button-ghost app-toolbar-button"
                type="button"
                data-testid="editor-branch-visionThen"
                @click="$emit('navigate-branch', visionBranchTarget.path)"
              >
                进入步骤
              </button>
            </div>
          </div>
        </div>
      </div>
    </template>

    <template v-else>
      <div class="grid gap-3 xl:grid-cols-2">
        <div class="rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">OCR 结果输入</span>
            <EditorSelectField
              :model-value="selectedVision.ocr_res_var || null"
              :options="resolvedVisionSearchOcrInputOptions"
              :show-description="true"
              placeholder="未绑定时使用当前 OCR 结果"
              test-id="editor-vision-search-ocr-input-var"
              @update:model-value="$emit('update-nullable-field', 'ocr_res_var', String($event || ''))"
            />
          </label>
          <div v-if="createVariable || (selectedVisionOcrInputTarget && jumpToVariable)" class="mt-3 flex flex-wrap gap-2">
            <button v-if="createVariable" class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('create-variable', 'visionSearchOcrInput')">
              <AppIcon name="plus" :size="14" />
              新建 OCR 变量
            </button>
            <button v-if="selectedVisionOcrInputTarget && jumpToVariable" class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('jump-to-variable', selectedVisionOcrInputTarget)">
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>
        </div>

        <div class="rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">检测结果输入</span>
            <EditorSelectField
              :model-value="selectedVision.det_res_var || null"
              :options="resolvedVisionSearchDetInputOptions"
              :show-description="true"
              placeholder="未绑定时使用当前检测结果"
              test-id="editor-vision-search-det-input-var"
              @update:model-value="$emit('update-nullable-field', 'det_res_var', String($event || ''))"
            />
          </label>
          <div v-if="createVariable || (selectedVisionDetInputTarget && jumpToVariable)" class="mt-3 flex flex-wrap gap-2">
            <button v-if="createVariable" class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('create-variable', 'visionSearchDetInput')">
              <AppIcon name="plus" :size="14" />
              新建检测变量
            </button>
            <button v-if="selectedVisionDetInputTarget && jumpToVariable" class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('jump-to-variable', selectedVisionDetInputTarget)">
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>
        </div>

        <div class="rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">SearchHit 输出</span>
            <EditorSelectField
              :model-value="selectedVision.out_var || null"
              :options="resolvedVisionOutputOptions"
              :show-description="true"
              placeholder="选择或创建 SearchHit 结果变量"
              test-id="editor-vision-output-var"
              @update:model-value="$emit('update-field', 'out_var', String($event || ''))"
            />
          </label>
          <div v-if="createVariable || (selectedVisionOutputTarget && jumpToVariable)" class="mt-3 flex flex-wrap gap-2">
            <button
              v-if="createVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              data-testid="editor-vision-output-create"
              @click="$emit('create-variable', 'visionOutput')"
            >
              <AppIcon name="plus" :size="14" />
              新建 SearchHit 变量
            </button>
            <button
              v-if="selectedVisionOutputTarget && jumpToVariable"
              class="app-button app-button-ghost app-toolbar-button"
              type="button"
              data-testid="editor-vision-output-locate"
              @click="$emit('jump-to-variable', selectedVisionOutputTarget)"
            >
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>
        </div>

        <div class="rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">检测筛选输出</span>
            <EditorSelectField
              :model-value="selectedVision.out_det_var || null"
              :options="resolvedVisionSearchDetOutputOptions"
              :show-description="true"
              placeholder="未绑定时不输出检测筛选结果"
              test-id="editor-vision-search-det-output-var"
              @update:model-value="$emit('update-nullable-field', 'out_det_var', String($event || ''))"
            />
          </label>
          <div v-if="createVariable || (selectedVisionDetOutputTarget && jumpToVariable)" class="mt-3 flex flex-wrap gap-2">
            <button v-if="createVariable" class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('create-variable', 'visionSearchDetOutput')">
              <AppIcon name="plus" :size="14" />
              新建检测输出
            </button>
            <button v-if="selectedVisionDetOutputTarget && jumpToVariable" class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('jump-to-variable', selectedVisionDetOutputTarget)">
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>
        </div>

        <div class="rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
          <label class="space-y-2">
            <span class="text-xs font-medium uppercase tracking-[0.12em] text-(--app-text-faint)">OCR 筛选输出</span>
            <EditorSelectField
              :model-value="selectedVision.out_ocr_var || null"
              :options="resolvedVisionSearchOcrOutputOptions"
              :show-description="true"
              placeholder="未绑定时不输出 OCR 筛选结果"
              test-id="editor-vision-search-ocr-output-var"
              @update:model-value="$emit('update-nullable-field', 'out_ocr_var', String($event || ''))"
            />
          </label>
          <div v-if="createVariable || (selectedVisionOcrOutputTarget && jumpToVariable)" class="mt-3 flex flex-wrap gap-2">
            <button v-if="createVariable" class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('create-variable', 'visionSearchOcrOutput')">
              <AppIcon name="plus" :size="14" />
              新建 OCR 输出
            </button>
            <button v-if="selectedVisionOcrOutputTarget && jumpToVariable" class="app-button app-button-ghost app-toolbar-button" type="button" @click="$emit('jump-to-variable', selectedVisionOcrOutputTarget)">
              <AppIcon name="locate-fixed" :size="14" />
              定位变量
            </button>
          </div>
        </div>

        <div class="rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4 xl:col-span-2">
          <div class="editor-inline-grid">
            <div class="editor-inline-label">命中后</div>
            <div class="editor-inline-content flex items-center justify-between gap-3">
              <span class="text-sm text-(--app-text-soft)">{{ visionBranchTarget?.count ?? 0 }} 个步骤</span>
              <button
                v-if="visionBranchTarget"
                class="app-button app-button-ghost app-toolbar-button"
                type="button"
                data-testid="editor-branch-visionThen"
                @click="$emit('navigate-branch', visionBranchTarget.path)"
              >
                进入步骤
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="rounded-[16px] border border-(--app-border) bg-white/40 px-4 py-4">
        <p class="text-sm font-semibold text-(--app-text-strong)">搜索规则</p>
        <div class="mt-3">
          <EditorSearchRuleBuilder
            :model-value="selectedVision.rule!"
            force-group-root
            test-id-prefix="editor-search-rule"
            :label-index-options="labelIndexOptions"
            :label-select-placeholder="labelSelectPlaceholder"
            :label-select-hint="labelSelectHint"
            @update:model-value="$emit('update-rule', $event)"
          />
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import EditorSelectField from '@/views/script-editor/EditorSelectField.vue';
import type { CompareOp } from '@/types/bindings/CompareOp';
import type { SearchRule } from '@/types/bindings/SearchRule';
import type { Step } from '@/types/bindings/Step';
import type { VisionNode } from '@/types/bindings/VisionNode';
import EditorSearchRuleBuilder from '@/views/script-editor/EditorSearchRuleBuilder.vue';
import type { StepBranchPath } from '@/views/script-editor/editor-step/editorStepTree';
import type { EditorInputType, EditorVariableOption } from '@/views/script-editor/editorVariables';
import { VISION_TYPE } from '@/views/script-editor/editor-step/editorStepKinds';

defineOptions({ name: 'EditorStepVisionPanel' });

type EditableVisionNode = {
  type: VisionNode['type'];
  input_var?: string;
  out_var?: string;
  det_res_var?: string | null;
  ocr_res_var?: string | null;
  out_det_var?: string | null;
  out_ocr_var?: string | null;
  target_value?: string | null;
  op?: CompareOp;
  expected_count?: number;
  then_steps?: Step[];
  rule?: SearchRule;
};

const props = defineProps<{
  selectedVision: EditableVisionNode;
  variableDatalistId: string;
  writableCatalogVariableOptions?: Array<{ label: string; value: string; description: string; disabled?: boolean }>;
  readableCatalogVariableOptions?: Array<{ label: string; value: string; description: string; disabled?: boolean }>;
  labelIndexOptions?: Array<{ label: string; value: number; description?: string; disabled?: boolean }>;
  labelSelectPlaceholder?: string;
  labelSelectHint?: string | null;
  selectedVisionInputTarget?: EditorVariableOption | null;
  selectedVisionOutputTarget?: EditorVariableOption | null;
  selectedVisionDetInputTarget?: EditorVariableOption | null;
  selectedVisionOcrInputTarget?: EditorVariableOption | null;
  selectedVisionDetOutputTarget?: EditorVariableOption | null;
  selectedVisionOcrOutputTarget?: EditorVariableOption | null;
  visionBranchTarget: { count: number; path: StepBranchPath } | null;
  createVariable?: (namespace?: 'input' | 'runtime', inputType?: EditorInputType) => Promise<string>;
  jumpToVariable?: (option: EditorVariableOption) => void;
}>();

const emit = defineEmits<{
  'update-field': [field: string, value: string];
  'update-nullable-field': [field: string, value: string];
  'update-number-field': [field: string, value: string];
  'update-rule': [rule: SearchRule];
  'navigate-branch': [branchPath: StepBranchPath];
  'create-variable': [target: 'visionInput' | 'visionOutput' | 'visionSearchDetInput' | 'visionSearchOcrInput' | 'visionSearchDetOutput' | 'visionSearchOcrOutput'];
  'jump-to-variable': [option: EditorVariableOption];
}>();

type SelectOption = { label: string; value: string; description: string; disabled?: boolean };

const countCompareOpOptions: SelectOption[] = [
  { label: '等于', value: 'eq', description: '匹配数量等于指定数量。' },
  { label: '不等于', value: 'ne', description: '匹配数量不等于指定数量。' },
  { label: '大于', value: 'gt', description: '匹配数量大于指定数量。' },
  { label: '大于等于', value: 'ge', description: '匹配数量大于等于指定数量。' },
  { label: '小于', value: 'lt', description: '匹配数量小于指定数量。' },
  { label: '小于等于', value: 'le', description: '匹配数量小于等于指定数量。' },
];

const withCurrentVariableOption = (options: SelectOption[], value: string) => {
  const trimmedValue = value.trim();
  if (!trimmedValue || options.some((option) => option.value === trimmedValue)) {
    return options;
  }

  return [
    {
      label: trimmedValue,
      value: trimmedValue,
      description: '未解析变量',
    },
    ...options,
  ];
};

const withUnboundOption = (options: SelectOption[], description: string) => [
  { label: '未绑定', value: '', description },
  ...options,
];

const resolvedVisionInputOptions = computed(() =>
  withCurrentVariableOption(
    (props.readableCatalogVariableOptions ?? []).filter((option) => option.description.includes('图像') || option.description.includes('Image')),
    props.selectedVision.input_var ?? '',
  ),
);

const resolvedVisionReadableOptions = computed(() =>
  withCurrentVariableOption(props.readableCatalogVariableOptions ?? [], props.selectedVision.input_var ?? ''),
);

const resolvedVisionOutputOptions = computed(() =>
  withCurrentVariableOption(
    (props.writableCatalogVariableOptions ?? []).filter((option) => option.description.includes('JSON') || option.description.includes('列表') || option.description.includes('对象')),
    props.selectedVision.out_var ?? '',
  ),
);

const resolvedVisionBoolOutputOptions = computed(() =>
  withCurrentVariableOption(
    (props.writableCatalogVariableOptions ?? []).filter((option) => option.description.includes('布尔') || option.description.includes('Boolean') || option.description.includes('bool')),
    props.selectedVision.out_var ?? '',
  ),
);

const filterStructuredOptions = (options: SelectOption[]) =>
  options.filter((option) => option.description.includes('JSON') || option.description.includes('列表') || option.description.includes('对象'));

const buildNullableStructuredOptions = (options: SelectOption[], value: string, description: string) =>
  withUnboundOption(withCurrentVariableOption(filterStructuredOptions(options), value), description);

const resolvedVisionSearchOcrInputOptions = computed(() =>
  buildNullableStructuredOptions(
    props.readableCatalogVariableOptions ?? [],
    props.selectedVision.ocr_res_var ?? '',
    '留空时使用当前上下文中的最近 OCR 结果。',
  ),
);

const resolvedVisionSearchDetInputOptions = computed(() =>
  buildNullableStructuredOptions(
    props.readableCatalogVariableOptions ?? [],
    props.selectedVision.det_res_var ?? '',
    '留空时使用当前上下文中的最近检测结果。',
  ),
);

const resolvedVisionSearchDetOutputOptions = computed(() =>
  buildNullableStructuredOptions(
    props.writableCatalogVariableOptions ?? [],
    props.selectedVision.out_det_var ?? '',
    '留空时不写出检测筛选结果。',
  ),
);

const resolvedVisionSearchOcrOutputOptions = computed(() =>
  buildNullableStructuredOptions(
    props.writableCatalogVariableOptions ?? [],
    props.selectedVision.out_ocr_var ?? '',
    '留空时不写出 OCR 筛选结果。',
  ),
);
</script>

<style scoped>
.editor-inline-grid {
  display: grid;
  gap: 0.75rem;
}

@media (min-width: 1280px) {
  .editor-inline-grid {
    grid-template-columns: 72px minmax(0, 1fr);
    align-items: center;
  }
}

.editor-inline-label {
  display: flex;
  align-items: center;
  min-height: 44px;
  color: var(--app-text-faint);
  font-size: 0.74rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.editor-inline-content {
  min-height: 44px;
}
</style>
