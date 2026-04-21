<template>
  <AppDialog
    :open="open"
    :title="mode === 'edit' ? '编辑脚本' : '新建脚本'"
    :width-class="dialogWidthClass"
    @close="$emit('close')"
  >
    <form v-if="form" class="grid gap-5 lg:grid-cols-[156px_minmax(0,1fr)]" :class="formClass" @submit.prevent="submit">
      <aside class="space-y-2">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          type="button"
          class="app-list-item"
          :data-testid="`script-dialog-tab-${tab.id}`"
          :class="{ 'app-list-item-active': activeTab === tab.id }"
          @click="activeTab = tab.id"
        >
          <p class="text-sm font-semibold text-[var(--app-text-strong)]">{{ tab.label }}</p>
        </button>
      </aside>

      <div class="space-y-5">
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
                <label class="dialog-form-row">
                  <span class="dialog-form-label">运行时</span>
                  <AppSelect v-model="form.data.runtimeType" :options="runtimeOptions" test-id="script-basic-runtime-type" />
                </label>

                <label class="dialog-form-row">
                  <span class="dialog-form-label">包名</span>
                  <input
                    v-model.trim="pkgNameValue"
                    class="app-input"
                    data-testid="script-basic-package-name"
                    maxlength="80"
                    placeholder="com.example.app"
                  />
                </label>
              </div>

              <div class="dialog-form-grid">
                <label class="dialog-form-row">
                  <span class="dialog-form-label">Activity</span>
                  <input
                    v-model.trim="activityNameValue"
                    class="app-input"
                    data-testid="script-basic-activity-name"
                    maxlength="160"
                    placeholder=".MainActivity"
                  />
                </label>

                <label class="dialog-form-row">
                  <span class="dialog-form-label">脚本平台</span>
                  <AppSelect v-model="scriptPlatformValue" :options="platformOptions" test-id="script-basic-platform" />
                </label>
              </div>

              <p class="text-xs leading-5 text-[var(--app-text-faint)]">
                `包名 + Activity` 会作为脚本全局应用入口信息，供运行时和调试链路复用。
              </p>

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

                <div class="dialog-form-row">
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
                  <span class="text-sm text-[var(--app-text-soft)]">关闭后，其他用户只能查看脚本信息，不能直接复制到本地。</span>
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

            <SurfacePanel v-if="activeModelTab === 'imgDet'" tone="muted" padding="sm" class="space-y-4">
              <label class="dialog-form-row">
                <span class="dialog-form-label">模型类型</span>
                <AppSelect
                  :model-value="imgDetKind"
                  :options="imgDetectorOptions"
                  test-id="script-models-img-det-kind"
                  @update:model-value="setDetectorKind('imgDetModel', $event)"
                />
              </label>
              <template v-if="imgYoloModel">
                <ModelBaseFields
                  :model="imgYoloModel.baseModel"
                  :built-in-enabled="false"
                  path-placeholder="例如：D:\\models\\img-det.onnx"
                  test-id-prefix="script-models-img-det-base"
                />
                <div class="dialog-form-grid">
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">类别数量</span>
                    <input
                      v-model.number="imgYoloModel.classCount"
                      class="app-input"
                      data-testid="script-models-img-det-class-count"
                      min="1"
                      type="number"
                    />
                  </label>
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">标签路径</span>
                    <div class="dialog-path-row">
                      <input
                        v-model.trim="imgLabelPathValue"
                        class="app-input"
                        data-testid="script-models-img-det-label-path"
                        placeholder="例如：D:\\models\\labels.yaml"
                      />
                      <button class="app-button app-button-ghost dialog-path-button" type="button" @click="pickImgLabelPath">
                        <AppIcon name="folder-open" :size="16" />
                      </button>
                    </div>
                  </label>
                </div>
                <div class="dialog-form-grid">
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">置信度阈值</span>
                    <input
                      v-model.number="imgYoloModel.confidenceThresh"
                      class="app-input"
                      data-testid="script-models-img-det-confidence"
                      max="1"
                      min="0"
                      step="0.01"
                      type="number"
                    />
                  </label>
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">IOU 阈值</span>
                    <input
                      v-model.number="imgYoloModel.iouThresh"
                      class="app-input"
                      data-testid="script-models-img-det-iou"
                      max="1"
                      min="0"
                      step="0.01"
                      type="number"
                    />
                  </label>
                </div>
              </template>
            </SurfacePanel>

            <SurfacePanel v-else-if="activeModelTab === 'txtDet'" tone="muted" padding="sm" class="space-y-4">
              <label class="dialog-form-row">
                <span class="dialog-form-label">模型类型</span>
                <AppSelect
                  :model-value="txtDetKind"
                  :options="txtDetectorOptions"
                  test-id="script-models-txt-det-kind"
                  @update:model-value="setDetectorKind('txtDetModel', $event)"
                />
              </label>
              <template v-if="txtYoloModel">
                <ModelBaseFields
                  :model="txtYoloModel.baseModel"
                  :built-in-enabled="false"
                  path-placeholder="例如：D:\\models\\txt-det.onnx"
                  test-id-prefix="script-models-txt-det-base"
                />
                <div class="dialog-form-grid">
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">类别数量</span>
                    <input v-model.number="txtYoloModel.classCount" class="app-input" data-testid="script-models-txt-det-class-count" min="1" type="number" />
                  </label>
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">标签路径</span>
                    <div class="dialog-path-row">
                      <input v-model.trim="txtLabelPathValue" class="app-input" data-testid="script-models-txt-det-label-path" placeholder="例如：D:\\models\\labels.yaml" />
                      <button class="app-button app-button-ghost dialog-path-button" type="button" @click="pickTxtLabelPath">
                        <AppIcon name="folder-open" :size="16" />
                      </button>
                    </div>
                  </label>
                </div>
                <div class="dialog-form-grid">
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">文本类别索引</span>
                    <input v-model.number="txtIdxValue" class="app-input" data-testid="script-models-txt-det-txt-idx" min="0" type="number" />
                  </label>
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">置信度阈值</span>
                    <input v-model.number="txtYoloModel.confidenceThresh" class="app-input" data-testid="script-models-txt-det-confidence" max="1" min="0" step="0.01" type="number" />
                  </label>
                </div>
                <div class="dialog-form-grid">
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">IOU 阈值</span>
                    <input v-model.number="txtYoloModel.iouThresh" class="app-input" data-testid="script-models-txt-det-iou" max="1" min="0" step="0.01" type="number" />
                  </label>
                </div>
              </template>
              <template v-else-if="txtDetKind === 'PaddleDbNet' && form.data.txtDetModel && 'PaddleDbNet' in form.data.txtDetModel">
                <ModelBaseFields
                  :model="form.data.txtDetModel.PaddleDbNet.baseModel"
                  :built-in-enabled="false"
                  path-placeholder="例如：D:\\models\\ocr-dbnet.onnx"
                  test-id-prefix="script-models-txt-det-base"
                />
                <div class="dialog-form-grid">
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">二值化阈值</span>
                    <input
                      v-model.number="form.data.txtDetModel.PaddleDbNet.dbThresh"
                      class="app-input"
                      data-testid="script-models-txt-det-db-thresh"
                      max="1"
                      min="0"
                      step="0.01"
                      type="number"
                    />
                  </label>
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">框阈值</span>
                    <input
                      v-model.number="form.data.txtDetModel.PaddleDbNet.dbBoxThresh"
                      class="app-input"
                      data-testid="script-models-txt-det-db-box-thresh"
                      max="1"
                      min="0"
                      step="0.01"
                      type="number"
                    />
                  </label>
                </div>
                <div class="dialog-form-grid">
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">扩张比例</span>
                    <input
                      v-model.number="form.data.txtDetModel.PaddleDbNet.unclipRatio"
                      class="app-input"
                      data-testid="script-models-txt-det-unclip-ratio"
                      min="0"
                      step="0.1"
                      type="number"
                    />
                  </label>
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">启用膨胀</span>
                    <span class="dialog-form-inline-toggle">
                      <input
                        v-model="form.data.txtDetModel.PaddleDbNet.useDilation"
                        type="checkbox"
                        class="h-4 w-4"
                        data-testid="script-models-txt-det-use-dilation"
                        style="accent-color: var(--app-accent)"
                      />
                      <span class="text-sm text-[var(--app-text-soft)]">对弱文本边缘更友好，但可能带来额外噪点。</span>
                    </span>
                  </label>
                </div>
              </template>
            </SurfacePanel>

            <SurfacePanel v-else tone="muted" padding="sm" class="space-y-4">
              <label class="dialog-form-row">
                <span class="dialog-form-label">模型类型</span>
                <AppSelect
                  :model-value="txtRecKind"
                  :options="recognizerOptions"
                  test-id="script-models-txt-rec-kind"
                  @update:model-value="setRecognizerKind($event)"
                />
              </label>
              <template v-if="txtRecKind === 'PaddleCrnn' && form.data.txtRecModel && 'PaddleCrnn' in form.data.txtRecModel">
                <ModelBaseFields
                  :model="txtCrnnModel.baseModel"
                  path-placeholder="例如：D:\\models\\ocr-rec.onnx"
                  test-id-prefix="script-models-txt-rec-base"
                />
                <div class="dialog-form-grid">
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">字典路径</span>
                    <div class="dialog-path-row">
                      <input
                        v-model.trim="dictPathValue"
                        class="app-input"
                        data-testid="script-models-txt-rec-dict-path"
                        placeholder="例如：D:\\models\\keys.txt"
                      />
                      <button class="app-button app-button-ghost dialog-path-button" type="button" @click="pickDictPath">
                        <AppIcon name="folder-open" :size="16" />
                      </button>
                    </div>
                  </label>
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">缩放插值</span>
                    <AppSelect
                      v-model="txtCrnnModel.resizeFilter"
                      :options="recResizeFilterOptions"
                      test-id="script-models-txt-rec-resize-filter"
                    />
                  </label>
                </div>
                <div class="dialog-form-grid">
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">识别执行模式</span>
                    <AppSelect
                      v-model="txtCrnnModel.processingMode"
                      :options="recProcessingModeOptions"
                      test-id="script-models-txt-rec-processing-mode"
                    />
                  </label>
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">Micro-batch 大小</span>
                    <input
                      v-model.number="txtCrnnModel.microBatchSize"
                      class="app-input"
                      data-testid="script-models-txt-rec-micro-batch-size"
                      min="1"
                      step="1"
                      type="number"
                    />
                  </label>
                </div>
                <div class="dialog-form-grid">
                  <label class="dialog-form-row">
                    <span class="dialog-form-label">宽度分桶步长</span>
                    <input
                      v-model.number="txtCrnnModel.widthBucketStep"
                      class="app-input"
                      data-testid="script-models-txt-rec-width-bucket-step"
                      min="8"
                      step="8"
                      type="number"
                    />
                  </label>
                </div>
              </template>
            </SurfacePanel>
          </div>
        </template>

        <template v-else-if="activeTab === 'runtime'">
          <SurfacePanel tone="muted" padding="sm" class="space-y-5">
            <div class="max-w-[720px] space-y-4">
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
                  <p class="text-sm text-[var(--app-text-soft)]">
                    当设备执行策略选择 `RunRecoveryTask` 时，运行时会使用这里选定的普通 Task 作为恢复入口。
                  </p>
                  <p v-if="!hasTaskContext" class="text-xs text-[var(--app-text-faint)]">
                    当前入口没有可选的任务上下文。请在脚本编辑器中打开脚本信息后配置恢复任务。
                  </p>
                </div>
              </label>
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
import { computed, ref, toRaw, watch } from 'vue';
import { open as dialogOpen } from '@tauri-apps/plugin-dialog';
import AppDialog from '@/components/shared/AppDialog.vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import type { BaseModel } from '@/types/bindings/BaseModel';
import type { DetectorType } from '@/types/bindings/DetectorType';
import type { PaddleDetDbNet } from '@/types/bindings/PaddleDetDbNet';
import type { PaddleRecCrnn } from '@/types/bindings/PaddleRecCrnn';
import type { RecognizerType } from '@/types/bindings/RecognizerType';
import type { ScriptTableRecord } from '@/types/app/domain';
import type { YoloDet } from '@/types/bindings/YoloDet';
import {
  rewritePublishedDetectorModelPath,
  rewritePublishedRecognizerModelPath,
} from '@/utils/visionModelPresets';
import ModelBaseFields from '@/views/script-list/script-info/ModelBaseFields.vue';
import SponsorshipQrField from '@/views/script-list/script-info/SponsorshipQrField.vue';

type DialogTab = 'basic' | 'models' | 'runtime' | 'support';
type ModelTab = 'imgDet' | 'txtDet' | 'txtRec';
type DetectorKind = 'none' | 'Yolo11' | 'PaddleDbNet' | 'Yolo26';
type RecognizerKind = 'none' | 'PaddleCrnn';
type EditableDetectorField = 'imgDetModel' | 'txtDetModel';
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
  { id: 'support' as const, label: '赞助信息' },
];
const modelTabs = [
  { id: 'imgDet' as const, label: '目标检测' },
  { id: 'txtDet' as const, label: '文字检测' },
  { id: 'txtRec' as const, label: '文字识别' },
];

const runtimeOptions = [
  { label: 'Rhai', value: 'rhai', description: '适合轻量流程和原生脚本。' },
  { label: 'JavaScript', value: 'javaScript', description: '适合复杂逻辑和工具链扩展。' },
  { label: 'Lua', value: 'lua', description: '适合轻量运行时脚本。' },
  { label: 'AI + Vision', value: 'aIAndVision', description: '适合依赖 OCR 与视觉判断的脚本。' },
  { label: 'AI', value: 'aI', description: '适合纯 AI 处理流程。' },
];

const platformOptions = [
  { label: 'Android', value: 'android', description: '当前 ADB / start_activity / 截图链路默认支持的平台。' },
  { label: '桌面程序', value: 'desktop', description: '用于桌面端脚本归类与分配约束，执行适配器后续再接。' },
];

const imgDetectorOptions = [
  { label: '不设置', value: 'none', description: '当前字段留空，不启用该类模型。' },
  { label: 'YOLO11', value: 'Yolo11', description: '通用目标检测方案。' },
  { label: 'YOLO26', value: 'Yolo26', description: '端到端 NMS-free 检测方案。' },
];

const txtDetectorOptions = [
  { label: '不设置', value: 'none', description: '当前字段留空，不启用该类模型。' },
  { label: 'YOLO11', value: 'Yolo11', description: '适合文本区域检测或字符框检测。' },
  { label: 'Paddle DBNet', value: 'PaddleDbNet', description: '适合文本区域检测。' },
  { label: 'YOLO26', value: 'Yolo26', description: '端到端 NMS-free 检测方案。' },
];

const recognizerOptions = [
  { label: '不设置', value: 'none', description: '当前字段留空，不启用识别模型。' },
  { label: 'Paddle CRNN', value: 'PaddleCrnn', description: '当前绑定里可用的文本识别模型。' },
];

const recResizeFilterOptions = [
  { label: 'Triangle', value: 'Triangle', description: '默认推荐，速度和识别稳定性更平衡。' },
  { label: 'Gaussian', value: 'Gaussian', description: '比 Triangle 更平滑，适合想减轻缩放噪点的场景。' },
  { label: 'CatmullRom', value: 'CatmullRom', description: '更锐利，适合想保留字形边缘细节的场景。' },
  { label: 'Lanczos3', value: 'Lanczos3', description: '更重的高质量插值，通常只在精度敏感时尝试。' },
  { label: 'Nearest', value: 'Nearest', description: '最快，但锯齿明显，通常不建议 OCR 默认使用。' },
];

const recProcessingModeOptions = [
  { label: '单张', value: 'Single', description: '逐张识别，适合文本框数量少或宽度差异大的场景。' },
  { label: 'Micro-batch', value: 'MicroBatch', description: '按宽度分桶后做小批次识别，适合框较多的场景。' },
];

const activeTab = ref<DialogTab>('basic');
const activeModelTab = ref<ModelTab>('imgDet');
const form = ref<ScriptTableRecord | null>(null);
const imgDetKind = ref<DetectorKind>('none');
const txtDetKind = ref<DetectorKind>('none');
const txtRecKind = ref<RecognizerKind>('none');

function createBaseModel(
  modelType: BaseModel['modelType'],
  width: number,
  height: number,
  modelSource: BaseModel['modelSource'] = 'Custom',
): BaseModel {
  return {
    intraThreadNum: 4,
    intraSpinning: true,
    interThreadNum: 1,
    interSpinning: true,
    executionProvider: 'CPU',
    inputWidth: width,
    inputHeight: height,
    modelSource,
    modelPath: '',
    modelType,
  };
}

function createYoloDet(kind: 'Yolo11' | 'Yolo26', textMode: boolean): YoloDet {
  return {
    baseModel: createBaseModel(kind, 640, 640, 'Custom'),
    classCount: 80,
    confidenceThresh: 0.25,
    iouThresh: 0.45,
    labelPath: null,
    txtIdx: textMode ? 0 : null,
  };
}

function createDbNet(): PaddleDetDbNet {
  return {
    baseModel: createBaseModel('PaddleDet5', 640, 640, 'Custom'),
    dbThresh: 0.3,
    dbBoxThresh: 0.5,
    unclipRatio: 1.5,
    useDilation: false,
  };
}

function createCrnn(): PaddleRecCrnn {
  return {
    baseModel: createBaseModel('PaddleCrnn5', 320, 48, 'BuiltIn'),
    dictPath: null,
    resizeFilter: 'Triangle',
    processingMode: 'Single',
    microBatchSize: 4,
    widthBucketStep: 32,
  };
}

function normalizeCrnnModel(model: PaddleRecCrnn): PaddleRecCrnn {
  if (!model.resizeFilter) model.resizeFilter = 'Triangle';
  if (!model.processingMode) model.processingMode = 'Single';
  if (!model.microBatchSize || model.microBatchSize < 1) model.microBatchSize = 4;
  if (!model.widthBucketStep || model.widthBucketStep < 8) model.widthBucketStep = 32;
  return model;
}

function resolveDetectorKind(model: DetectorType | null): DetectorKind {
  if (!model) return 'none';
  if ('Yolo11' in model) return 'Yolo11';
  if ('Yolo26' in model) return 'Yolo26';
  if ('PaddleDbNet' in model) return 'PaddleDbNet';
  return 'none';
}

function resolveRecognizerKind(model: RecognizerType | null): RecognizerKind {
  if (!model) return 'none';
  if ('PaddleCrnn' in model) return 'PaddleCrnn';
  return 'none';
}

function syncKinds() {
  if (!form.value) return;
  if (form.value.data.imgDetModel && 'PaddleDbNet' in form.value.data.imgDetModel) {
    form.value.data.imgDetModel = null;
  }
  const imgYolo = extractYoloDetector(form.value.data.imgDetModel);
  if (imgYolo && imgYolo.baseModel.modelSource === 'BuiltIn') {
    imgYolo.baseModel.modelSource = 'Custom';
  }
  if (form.value.data.txtDetModel) {
    if ('PaddleDbNet' in form.value.data.txtDetModel && form.value.data.txtDetModel.PaddleDbNet.baseModel.modelSource === 'BuiltIn') {
      form.value.data.txtDetModel.PaddleDbNet.baseModel.modelSource = 'Custom';
    }
    const txtYolo = extractYoloDetector(form.value.data.txtDetModel);
    if (txtYolo && txtYolo.baseModel.modelSource === 'BuiltIn') {
      txtYolo.baseModel.modelSource = 'Custom';
    }
  }
  imgDetKind.value = resolveDetectorKind(form.value.data.imgDetModel);
  txtDetKind.value = resolveDetectorKind(form.value.data.txtDetModel);
  txtRecKind.value = resolveRecognizerKind(form.value.data.txtRecModel);
}

function setDetectorKind(field: EditableDetectorField, nextValue: string | number | null) {
  if (!form.value) return;

  const kind = (nextValue ?? 'none') as DetectorKind;
  if (field === 'imgDetModel') imgDetKind.value = kind;
  else txtDetKind.value = kind;

  if (kind === 'none') {
    form.value.data[field] = null;
    return;
  }

  if (kind === 'Yolo11') {
    form.value.data[field] = { Yolo11: createYoloDet('Yolo11', field === 'txtDetModel') };
    return;
  }

  if (kind === 'Yolo26') {
    form.value.data[field] = { Yolo26: createYoloDet('Yolo26', field === 'txtDetModel') };
    return;
  }

  if (field === 'imgDetModel') {
    form.value.data[field] = null;
    imgDetKind.value = 'none';
    return;
  }

  form.value.data[field] = { PaddleDbNet: createDbNet() };
}

function setRecognizerKind(nextValue: string | number | null) {
  if (!form.value) return;
  const kind = (nextValue ?? 'none') as RecognizerKind;
  txtRecKind.value = kind;
  form.value.data.txtRecModel = kind === 'PaddleCrnn' ? { PaddleCrnn: createCrnn() } : null;
}

const scriptTypeLabel = computed(() => (form.value?.data.scriptType === 'published' ? '云端版本' : '本地开发'));
const canSubmit = computed(() => Boolean(form.value?.data.name.trim()));
const dialogWidthClass = computed(() =>
  props.mode === 'create' ? 'max-w-6xl min-h-[80vh] max-h-[calc(100vh-3rem)] flex flex-col' : 'max-w-6xl',
);
const formClass = computed(() =>
  props.mode === 'create' ? 'min-h-0 flex-1 overflow-y-auto pr-1' : '',
);

function extractYoloDetector(model: DetectorType | null): YoloDet | null {
  if (!model) return null;
  if ('Yolo11' in model) return model.Yolo11;
  if ('Yolo26' in model) return model.Yolo26;
  return null;
}

function getYoloDetector(field: EditableDetectorField): YoloDet | null {
  if (!form.value) return null;
  return extractYoloDetector(form.value.data[field]);
}

const imgYoloModel = computed(() => getYoloDetector('imgDetModel'));
const txtYoloModel = computed(() => getYoloDetector('txtDetModel'));
const txtCrnnModel = computed(() => {
  if (!form.value?.data.txtRecModel || !('PaddleCrnn' in form.value.data.txtRecModel)) {
    return createCrnn();
  }
  return normalizeCrnnModel(form.value.data.txtRecModel.PaddleCrnn);
});

const descriptionValue = computed({
  get: () => form.value?.data.description || '',
  set: (value: string) => {
    if (form.value) form.value.data.description = value || null;
  },
});

const pkgNameValue = computed({
  get: () => form.value?.data.pkgName || '',
  set: (value: string) => {
    if (form.value) form.value.data.pkgName = value || null;
  },
});

const activityNameValue = computed({
  get: () => form.value?.data.activityName || '',
  set: (value: string) => {
    if (form.value) form.value.data.activityName = value || null;
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
      ...(form.value.data.runtimeSettings ?? { recoveryTaskId: null }),
      recoveryTaskId: value || null,
    };
  },
});

const imgLabelPathValue = computed({
  get: () => imgYoloModel.value?.labelPath || '',
  set: (value: string) => {
    const model = getYoloDetector('imgDetModel');
    if (model) model.labelPath = value || null;
  },
});

const txtLabelPathValue = computed({
  get: () => txtYoloModel.value?.labelPath || '',
  set: (value: string) => {
    const model = getYoloDetector('txtDetModel');
    if (model) model.labelPath = value || null;
  },
});

const txtIdxValue = computed({
  get: () => txtYoloModel.value?.txtIdx ?? 0,
  set: (value: number) => {
    const model = getYoloDetector('txtDetModel');
    if (model) model.txtIdx = value;
  },
});

const dictPathValue = computed({
  get: () => txtCrnnModel.value.dictPath || '',
  set: (value: string) => {
    if (form.value?.data.txtRecModel && 'PaddleCrnn' in form.value.data.txtRecModel) {
      form.value.data.txtRecModel.PaddleCrnn.dictPath = value || null;
    }
  },
});

const pickImgLabelPath = async () => {
  const value = await dialogOpen({
    multiple: false,
    directory: false,
    filters: [{ name: 'Label Files', extensions: ['yaml', 'yml', 'json', 'txt'] }],
  });
  if (typeof value === 'string' && value) {
    imgLabelPathValue.value = value;
  }
};

const pickTxtLabelPath = async () => {
  const value = await dialogOpen({
    multiple: false,
    directory: false,
    filters: [{ name: 'Label Files', extensions: ['yaml', 'yml', 'json', 'txt'] }],
  });
  if (typeof value === 'string' && value) {
    txtLabelPathValue.value = value;
  }
};

const pickDictPath = async () => {
  const value = await dialogOpen({
    multiple: false,
    directory: false,
    filters: [{ name: 'Dictionary Files', extensions: ['txt', 'dict'] }],
  });
  if (typeof value === 'string' && value) {
    dictPathValue.value = value;
  }
};

function cloneScriptRecord(script: unknown): ScriptTableRecord {
  return JSON.parse(JSON.stringify(toRaw(script))) as ScriptTableRecord;
}

function ensureRuntimeSettings(script: ScriptTableRecord) {
  script.data.platform = script.data.platform || 'android';
  script.data.pkgName = script.data.pkgName || null;
  script.data.activityName = script.data.activityName || null;
  script.data.runtimeSettings = {
    recoveryTaskId: script.data.runtimeSettings?.recoveryTaskId || null,
  };
}

function submit() {
  if (!form.value || !canSubmit.value) return;
  form.value.data.name = form.value.data.name.trim();
  form.value.data.verName = form.value.data.verName.trim() || '0.1.0';
  form.value.data.updateTime = new Date().toISOString();
  const nextScript = cloneScriptRecord(form.value);
  if (nextScript.data.scriptType === 'published') {
    nextScript.data.imgDetModel = rewritePublishedDetectorModelPath(nextScript.data.imgDetModel, nextScript.id, 'det.onnx');
    nextScript.data.txtDetModel = rewritePublishedDetectorModelPath(nextScript.data.txtDetModel, nextScript.id, 'txt_det.onnx');
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
    activeTab.value = 'basic';
    activeModelTab.value = 'imgDet';
    syncKinds();
  },
  { immediate: true },
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

.editor-panel-tabs {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  border-bottom: 1px solid var(--app-border);
}

.editor-panel-tab {
  position: relative;
  margin-bottom: -1px;
  border-bottom: 2px solid transparent;
  padding: 0.75rem 0.35rem 0.85rem;
  color: var(--app-text-faint);
  font-size: 0.93rem;
  font-weight: 600;
  white-space: nowrap;
  transition: color 0.16s ease, border-color 0.16s ease;
}

.editor-panel-tab:hover {
  color: var(--app-text-soft);
}

.editor-panel-tab-active {
  border-bottom-color: var(--app-accent);
  color: var(--app-text-strong);
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
