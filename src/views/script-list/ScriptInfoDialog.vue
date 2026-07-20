<template>
  <AppDialog
    :open="open"
    :title="mode === 'edit' ? '编辑脚本' : '新建脚本'"
    :width-class="dialogWidthClass"
    @close="$emit('close')"
  >
    <form v-if="form" class="grid min-h-0 gap-5 lg:grid-cols-[156px_minmax(0,1fr)]" :class="formClass" @submit.prevent="submit">
      <aside class="space-y-2 overflow-y-auto pr-1 custom-scrollbar">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          class="app-list-item"
          :data-testid="`script-dialog-tab-${tab.id}`"
          :class="{ 'app-list-item-active': activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          <p class="text-sm font-semibold text-(--app-text-strong)">{{ tab.label }}</p>
        </button>
      </aside>

      <div class="flex min-h-0 flex-col gap-5">
        <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
          <div
            v-if="validationIssues.length"
            class="rounded-[20px] border border-amber-300/70 bg-amber-50 px-4 py-4 text-sm text-amber-950"
            data-testid="script-info-validation-summary"
          >
            <p class="font-semibold">请先补齐脚本信息后再继续。</p>
            <ul class="mt-2 space-y-1">
              <li v-for="issue in validationIssues" :key="issue.field">
                {{ issue.label }}：{{ issue.message }}
              </li>
            </ul>
          </div>

          <div :class="validationIssues.length ? 'mt-5' : ''">
            <template v-if="activeTab === 'basic'">
          <SurfacePanel tone="muted" padding="sm" class="space-y-5">
            <div class="space-y-4">
              <label class="dialog-form-row dialog-form-row-wide">
                <span class="dialog-form-label">脚本名称</span>
                <input
                  v-model.trim="form.data.name"
                  class="app-input"
                  data-testid="script-basic-name"
                  maxlength="40"
                  placeholder="例如：每日清体力"
                />
              </label>

              <label class="dialog-form-row dialog-form-row-wide dialog-form-row-start">
                <span class="dialog-form-label">描述</span>
                <textarea
                  v-model="descriptionValue"
                  class="app-textarea min-h-[130px]"
                  data-testid="script-basic-description"
                  maxlength="240"
                  placeholder="简述脚本作用、运行前提和风险提示。"
                />
              </label>

              <div class="dialog-form-grid">
                <!-- <label class="dialog-form-row">
                  <span class="dialog-form-label">运行时</span>
                  <AppSelect v-model="form.data.runtimeType" :options="runtimeOptions" test-id="script-basic-runtime-type" />
                </label> -->

                <!-- <label class="dialog-form-row">
                  <span class="dialog-form-label">脚本平台</span>
                  <AppSelect v-model="scriptPlatformValue" :options="platformOptions" test-id="script-basic-platform" />
                </label> -->
              </div>

              <div class="dialog-form-grid">
                <label class="dialog-form-row">
                  <span class="dialog-form-label">版本名称</span>
                  <input
                    v-model.trim="form.data.verName"
                    class="app-input"
                    data-testid="script-basic-version-name"
                    maxlength="20"
                    placeholder="0.1.0"
                  />
                </label>

                <label class="dialog-form-row">
                  <span class="dialog-form-label">版本号</span>
                  <input
                    v-model.number="form.data.verNum"
                    class="app-input"
                    data-testid="script-basic-version-num"
                    min="1"
                    type="number"
                  />
                </label>
              </div>

              <div class="dialog-form-grid">
                <div class="dialog-form-row">
                  <span class="dialog-form-label">作者</span>
                  <div class="dialog-form-readonly">{{ form.data.userName || 'Local User' }}</div>
                </div>

                <div class="dialog-form-row" style="display: none">
                  <span class="dialog-form-label">脚本类型</span>
                  <div class="dialog-form-readonly">{{ scriptTypeLabel }}</div>
                </div>
              </div>

              <label class="dialog-form-row dialog-form-row-wide">
                <span class="dialog-form-label">允许克隆</span>
                <span class="dialog-form-inline-toggle">
                  <input
                    v-model="form.data.allowClone"
                    type="checkbox"
                    class="h-4 w-4"
                    data-testid="script-basic-allow-clone"
                    style="accent-color: var(--app-accent)"
                  />
                  <span class="text-sm text-(--app-text-soft)">关闭后，其他用户只能查看脚本信息，不能直接复制到本地。</span>
                </span>
              </label>
            </div>
          </SurfacePanel>
            </template>

            <template v-else-if="activeTab === 'models'">
          <div class="space-y-4">
            <div class="overflow-x-auto">
              <div class="editor-panel-tabs min-w-max">
                <button
                  v-for="tab in modelTabs"
                  :key="tab.id"
                  type="button"
                  class="editor-panel-tab"
                  :class="{ 'editor-panel-tab-active': activeModelTab === tab.id }"
                  :data-testid="`script-models-tab-${tab.id}`"
                  @click="activeModelTab = tab.id"
                >
                  {{ tab.label }}
                </button>
              </div>
            </div>

            <SurfacePanel v-if="activeModelTab === 'imgDet'" tone="muted" padding="sm">
              <VisionModelSettings mode="imgDet" v-model:detector-model="form.data.imgDetModel" test-id-prefix="script-models-img-det" />
            </SurfacePanel>
            <SurfacePanel v-else-if="activeModelTab === 'txtDet'" tone="muted" padding="sm">
              <VisionModelSettings mode="txtDet" v-model:detector-model="form.data.txtDetModel" test-id-prefix="script-models-txt-det" />
            </SurfacePanel>
            <SurfacePanel v-else tone="muted" padding="sm">
              <VisionModelSettings mode="txtRec" v-model:recognizer-model="form.data.txtRecModel" test-id-prefix="script-models-txt-rec" />
            </SurfacePanel>
          </div>
            </template>

            <template v-else-if="activeTab === 'runtime'">
          <SurfacePanel tone="muted" padding="sm" class="space-y-5">
            <div class="max-w-[720px] space-y-4">
              <label class="support-form-row">
                <span class="support-form-label">点击随机偏移</span>
                <div class="space-y-3">
                  <input
                    v-model.number="clickRandomOffsetValue"
                    class="app-input"
                    data-testid="script-runtime-click-random-offset"
                    min="0"
                    step="1"
                    type="number"
                  />
                  <p class="text-sm text-(--app-text-soft)">
                    执行点击前在 X/Y 方向随机偏移该像素范围，`0` 表示不偏移。
                  </p>
                </div>
              </label>

              <label class="support-form-row">
                <span class="support-form-label">恢复任务</span>
                <div class="space-y-3">
                  <AppSelect
                    v-model="recoveryTaskValue"
                    :options="recoveryTaskOptions"
                    :disabled="!hasTaskContext"
                    placeholder="不设置"
                    test-id="script-runtime-recovery-task"
                  />
                  <p class="text-sm text-(--app-text-soft)">
                    当设备执行策略选择 `RunRecoveryTask` 时，运行时会使用这里选定的普通 Task 作为恢复入口。
                  </p>
                  <p v-if="!hasTaskContext" class="text-xs text-(--app-text-faint)">
                    当前入口没有可选的任务上下文。请在脚本编辑器中打开脚本信息后配置恢复任务。
                  </p>
                </div>
              </label>
            </div>
          </SurfacePanel>
            </template>

            <template v-else-if="activeTab === 'content'">
          <SurfacePanel tone="muted" padding="sm" class="space-y-4">
            <div class="flex flex-wrap items-center gap-2">
              <button class="app-icon-button" type="button" title="标题" aria-label="标题" @click="wrapContentLine('# ')">
                <AppIcon name="heading-1" :size="16" />
              </button>
              <button class="app-icon-button" type="button" title="粗体" aria-label="粗体" @click="wrapContentSelection('**', '**')">
                <AppIcon name="bold" :size="16" />
              </button>
              <button class="app-icon-button" type="button" title="列表" aria-label="列表" @click="wrapContentLine('- ')">
                <AppIcon name="list" :size="16" />
              </button>
              <button class="app-icon-button" type="button" title="下划线" aria-label="下划线" @click="wrapContentSelection('<u>', '</u>')">
                <AppIcon name="underline" :size="16" />
              </button>
              <button class="app-icon-button" type="button" title="删除线" aria-label="删除线" @click="wrapContentSelection('~~', '~~')">
                <AppIcon name="strikethrough" :size="16" />
              </button>
            </div>

            <div class="grid gap-4 xl:grid-cols-2">
              <label class="grid gap-2">
                <span class="dialog-form-label">Markdown</span>
                <textarea
                  ref="contentTextarea"
                  v-model="contentMdValue"
                  class="app-textarea min-h-[280px]"
                  data-testid="script-content-md"
                  maxlength="4000"
                  placeholder="# 2026-05-06&#10;## v0.0.1 更新日志&#10;首次发布。"
                />
              </label>
              <div class="grid gap-2">
                <span class="dialog-form-label">预览</span>
                <div class="script-markdown-preview custom-scrollbar">
                  <MarkdownView :content="contentMdValue" empty-text="暂无脚本更新日志。" />
                </div>
              </div>
            </div>
          </SurfacePanel>
            </template>

            <template v-else>
          <SurfacePanel tone="muted" padding="sm" class="space-y-5">
            <div class="max-w-[720px] space-y-4">
              <label class="support-form-row">
                <span class="support-form-label">联系方式</span>
                <input
                  v-model.trim="contactInfoValue"
                  class="app-input"
                  data-testid="script-support-contact-info"
                  maxlength="80"
                  placeholder="QQ / Telegram / Email"
                />
              </label>

              <label class="support-form-row">
                <span class="support-form-label">赞助链接</span>
                <input
                  v-model.trim="sponsorshipUrlValue"
                  class="app-input"
                  data-testid="script-support-sponsorship-url"
                  maxlength="160"
                  placeholder="https://..."
                />
              </label>

              <SponsorshipQrField
                v-model="sponsorshipQrValue"
                clear-button-test-id="script-support-sponsorship-qr-clear"
                input-test-id="script-support-sponsorship-qr-input"
                preview-test-id="script-support-sponsorship-qr-preview"
                source-test-id="script-support-sponsorship-qr-source"
              />
            </div>
          </SurfacePanel>
            </template>
          </div>
        </div>

        <div class="flex justify-end gap-3">
          <button class="app-button app-button-ghost" type="button" @click="$emit('close')">取消</button>
          <button class="app-button app-button-primary" data-testid="script-submit" type="submit" :disabled="!canSubmit">
            {{ mode === 'edit' ? '保存信息' : '创建脚本' }}
          </button>
        </div>
      </div>
    </form>
  </AppDialog>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, toRaw, watch } from 'vue';
import AppDialog from '@/components/shared/AppDialog.vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import MarkdownView from '@/components/shared/MarkdownView.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { DetectorType } from '@/types/bindings/DetectorType';
import type { ScriptTableRecord } from '@/types/app/domain';
import type { ScriptInfoValidationIssue } from '@/utils/scriptInfoValidation';
import { validateScriptInfo } from '@/utils/scriptInfoValidation';
import {
  rewritePublishedDetectorModelPath,
  rewritePublishedRecognizerModelPath,
  syncYoloPostprocessFields,
} from '@/utils/visionModelPresets';
import VisionModelSettings from '@/views/script-list/script-info/VisionModelSettings.vue';
import SponsorshipQrField from '@/views/script-list/script-info/SponsorshipQrField.vue';

type DialogTab = 'basic' | 'models' | 'runtime' | 'content' | 'support';
type ModelTab = 'imgDet' | 'txtDet' | 'txtRec';
type TaskOption = { label: string; value: string | null; description?: string };

const props = defineProps<{
  open: boolean;
  mode: 'create' | 'edit';
  script: ScriptTableRecord | null;
  taskOptions?: TaskOption[];
}>();

const emit = defineEmits(['close', 'save']);

const tabs = [
  { id: 'basic' as const, label: '基本信息' },
  { id: 'models' as const, label: '模型信息' },
  { id: 'runtime' as const, label: '运行恢复' },
  { id: 'content' as const, label: '更新日志' },
  { id: 'support' as const, label: '赞助信息' },
];
const modelTabs = [
  { id: 'imgDet' as const, label: '目标检测' },
  { id: 'txtDet' as const, label: '文字检测' },
  { id: 'txtRec' as const, label: '文字识别' },
];

/* const runtimeOptions = [
  { label: 'Rhai', value: 'rhai', description: '适合轻量流程和原生脚本。' },
  { label: 'JavaScript', value: 'javaScript', description: '适合复杂逻辑和工具链扩展。' },
  { label: 'Lua', value: 'lua', description: '适合轻量运行时脚本。' },
  { label: 'AI + Vision', value: 'aIAndVision', description: '适合依赖 OCR 与视觉判断的脚本。' },
  { label: 'AI', value: 'aI', description: '适合纯 AI 处理流程。' },
]; */
const runtimeOptions = [
  { label: 'Rhai', value: 'rhai', description: '适合轻量流程和原生脚本。' },
]; 

const platformOptions = [
  { label: 'Android', value: 'android', description: '当前 ADB / start_activity / 截图链路默认支持的平台。' },
  { label: '桌面程序', value: 'desktop', description: '用于桌面端脚本归类与分配约束，执行适配器后续再接。' },
];

const defaultScriptRequiredFeatures = ['onnxInference', 'runtime:rhai', 'device:android'];

const activeTab = ref<DialogTab>('basic');
const activeModelTab = ref<ModelTab>('imgDet');
const form = ref<ScriptTableRecord | null>(null);
const validationIssues = ref<ScriptInfoValidationIssue[]>([]);
const contentTextarea = ref<HTMLTextAreaElement | null>(null);
const imgLabelPathHint = ref<string | null>(null);
const txtLabelPathHint = ref<string | null>(null);
const imgLabelLoading = ref(false);
const txtLabelLoading = ref(false);
let imgLabelRequestId = 0;
let txtLabelRequestId = 0;

function normalizeScriptModels() {
  if (form.value?.data.imgDetModel && 'PaddleDbNet' in form.value.data.imgDetModel) {
    form.value.data.imgDetModel = null;
  }
}
const scriptTypeLabel = computed(() => (form.value?.data.scriptType === 'published' ? '云端版本' : '本地开发'));
const canSubmit = computed(() => Boolean(form.value?.data.name.trim()));
const dialogWidthClass = computed(() => 'max-w-6xl min-h-[84vh] max-h-[calc(100vh-3rem)] flex flex-col');
const formClass = computed(() => 'min-h-0 flex-1 overflow-hidden');


const descriptionValue = computed({
  get: () => form.value?.data.description || '',
  set: (value: string) => {
    if (form.value) form.value.data.description = value || null;
  },
});

const contentMdValue = computed({
  get: () => form.value?.data.contentMd || '',
  set: (value: string) => {
    if (form.value) form.value.data.contentMd = value || null;
  },
});

const scriptPlatformValue = computed({
  get: () => form.value?.data.platform || 'android',
  set: (value: string) => {
    if (form.value) {
      form.value.data.platform = value as ScriptTableRecord['data']['platform'];
    }
  },
});

const contactInfoValue = computed({
  get: () => form.value?.data.contactInfo || '',
  set: (value: string) => {
    if (form.value) form.value.data.contactInfo = value || null;
  },
});

const sponsorshipUrlValue = computed({
  get: () => form.value?.data.sponsorshipUrl || '',
  set: (value: string) => {
    if (form.value) form.value.data.sponsorshipUrl = value || null;
  },
});

const sponsorshipQrValue = computed({
  get: () => form.value?.data.sponsorshipQr || '',
  set: (value: string) => {
    if (form.value) form.value.data.sponsorshipQr = value || null;
  },
});

const hasTaskContext = computed(() => Boolean(props.taskOptions?.length));

const recoveryTaskOptions = computed<TaskOption[]>(() => [
  { label: '不设置', value: null, description: '当前脚本不提供恢复任务。' },
  ...(props.taskOptions ?? []),
]);

const recoveryTaskValue = computed<string | null>({
  get: () => form.value?.data.runtimeSettings?.recoveryTaskId || null,
  set: (value) => {
    if (!form.value) {
      return;
    }

    form.value.data.runtimeSettings = {
      ...(form.value.data.runtimeSettings ?? { recoveryTaskId: null, clickRandomOffset: 0 }),
      recoveryTaskId: value || null,
    };
  },
});

const clickRandomOffsetValue = computed({
  get: () => form.value?.data.runtimeSettings?.clickRandomOffset ?? 0,
  set: (value: number) => {
    if (!form.value) {
      return;
    }

    form.value.data.runtimeSettings = {
      ...(form.value.data.runtimeSettings ?? { recoveryTaskId: null, clickRandomOffset: 0 }),
      clickRandomOffset: Math.max(0, Math.floor(Number(value) || 0)),
    };
  },
});

function updateContentText(value: string, selectionStart: number, selectionEnd: number) {
  contentMdValue.value = value;
  nextTick(() => {
    const textarea = contentTextarea.value;
    if (!textarea) return;
    textarea.focus();
    textarea.setSelectionRange(selectionStart, selectionEnd);
  });
}

function wrapContentSelection(prefix: string, suffix: string) {
  const textarea = contentTextarea.value;
  const text = contentMdValue.value;
  if (!textarea) {
    contentMdValue.value = `${prefix}${text}${suffix}`;
    return;
  }

  const start = textarea.selectionStart;
  const end = textarea.selectionEnd;
  const selected = text.slice(start, end) || '文本';
  const next = `${text.slice(0, start)}${prefix}${selected}${suffix}${text.slice(end)}`;
  updateContentText(next, start + prefix.length, start + prefix.length + selected.length);
}

function wrapContentLine(prefix: string) {
  const textarea = contentTextarea.value;
  const text = contentMdValue.value;
  if (!textarea) {
    contentMdValue.value = `${prefix}${text || '内容'}`;
    return;
  }

  const start = textarea.selectionStart;
  const lineStart = text.lastIndexOf('\n', Math.max(0, start - 1)) + 1;
  const next = `${text.slice(0, lineStart)}${prefix}${text.slice(lineStart)}`;
  updateContentText(next, start + prefix.length, textarea.selectionEnd + prefix.length);
}

function cloneScriptRecord(script: unknown): ScriptTableRecord {
  return JSON.parse(JSON.stringify(toRaw(script))) as ScriptTableRecord;
}

function ensureRuntimeSettings(script: ScriptTableRecord) {
  script.data.contentMd = script.data.contentMd || null;
  script.data.platform = script.data.platform || 'android';
  script.data.requiredFeatures = Array.isArray(script.data.requiredFeatures) && script.data.requiredFeatures.length
    ? script.data.requiredFeatures
    : [...defaultScriptRequiredFeatures];
  script.data.runtimeSettings = {
    recoveryTaskId: script.data.runtimeSettings?.recoveryTaskId || null,
    clickRandomOffset: Math.max(0, Math.floor(Number(script.data.runtimeSettings?.clickRandomOffset ?? 0) || 0)),
  };
}

function submit() {
  if (!form.value || !canSubmit.value) return;
  //@ts-ignore
  const issues = validateScriptInfo(form.value);
  validationIssues.value = issues;
  if (issues.length) {
    activeTab.value = 'basic';
    return;
  }
  form.value.data.name = form.value.data.name.trim();
  form.value.data.verName = form.value.data.verName.trim() || '0.1.0';
  form.value.data.updateTime = new Date().toISOString();
  const nextScript = cloneScriptRecord(form.value);
  const normalizeDetector = (model: DetectorType | null) => {
    if (!model) {
      return;
    }
    if ('Yolo11' in model) {
      syncYoloPostprocessFields(model.Yolo11);
      return;
    }
    if ('Yolo26' in model) {
      syncYoloPostprocessFields(model.Yolo26);
    }
  };
  normalizeDetector(nextScript.data.imgDetModel);
  normalizeDetector(nextScript.data.txtDetModel);
  if (nextScript.data.txtRecModel && 'PaddleCrnn' in nextScript.data.txtRecModel && nextScript.data.txtRecModel.PaddleCrnn.baseModel.modelSource === 'BuiltIn') {
    nextScript.data.txtRecModel.PaddleCrnn.dictPath = null;
  }
  if (nextScript.data.scriptType === 'published') {
    nextScript.data.imgDetModel = rewritePublishedDetectorModelPath(nextScript.data.imgDetModel, nextScript.id, 'img_det_model.onnx');
    nextScript.data.txtDetModel = rewritePublishedDetectorModelPath(nextScript.data.txtDetModel, nextScript.id, 'txt_det_model.onnx');
    nextScript.data.txtRecModel = rewritePublishedRecognizerModelPath(nextScript.data.txtRecModel, nextScript.id);
  }
  emit('save', nextScript);
}

watch(
  () => [props.open, props.script?.id],
  ([open]) => {
    if (!open || !props.script) return;
    const nextForm = cloneScriptRecord(props.script);
    ensureRuntimeSettings(nextForm);
    form.value = nextForm;
    validationIssues.value = [];
    activeTab.value = 'basic';
    activeModelTab.value = 'imgDet';
    normalizeScriptModels();
  },
  { immediate: true },
);


watch(
  () => [form.value?.data.name, form.value?.data.description, form.value?.data.verName] as const,
  () => {
    if (!form.value || !validationIssues.value.length) {
      return;
    }

    validationIssues.value = validateScriptInfo(form.value);
  },
);
</script>

<style scoped>
.dialog-form-row {
  display: grid;
  gap: 0.9rem;
  align-items: center;
}

.dialog-form-grid {
  display: grid;
  gap: 1rem 1.25rem;
}

.dialog-form-row-start {
  align-items: start;
}

.dialog-form-label {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--app-text-strong);
}

.dialog-form-readonly {
  min-height: 2.9rem;
  display: flex;
  align-items: center;
  border-radius: 16px;
  border: 1px solid var(--app-border);
  background: color-mix(in srgb, var(--app-panel-muted) 94%, transparent);
  padding: 0.8rem 0.9rem;
  color: var(--app-text-strong);
}

.dialog-form-inline-toggle {
  display: inline-flex;
  align-items: center;
  gap: 0.75rem;
}

.dialog-path-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.65rem;
  align-items: center;
}

.dialog-path-button {
  min-width: 2.75rem;
  height: 2.75rem;
  padding: 0;
}

.app-input-readonly {
  background: color-mix(in srgb, var(--app-panel-muted) 94%, transparent);
  color: var(--app-text-soft);
}

.support-form-row {
  display: grid;
  gap: 0.9rem;
  align-items: center;
}

.support-form-label {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--app-text-strong);
}

.script-markdown-preview {
  min-height: 280px;
  max-height: 420px;
  overflow: auto;
  border-radius: 16px;
  border: 1px solid var(--app-border);
  background: color-mix(in srgb, var(--app-panel) 88%, transparent);
  padding: 1rem;
}

@media (min-width: 768px) {
  .dialog-form-row {
    grid-template-columns: 96px minmax(0, 1fr);
  }

  .dialog-form-grid {
    grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  }

  .dialog-form-row-wide {
    grid-template-columns: 96px minmax(0, 1fr);
  }

  .support-form-row {
    grid-template-columns: 96px minmax(0, 1fr);
  }
}
</style>
