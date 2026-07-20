<template>
  <div class="space-y-4">
    <label class="model-row"><span class="model-label">模型类型</span><AppSelect :model-value="kind" :options="options" :test-id="`${testIdPrefix}-kind`" @update:model-value="setKind" /></label>
    <template v-if="yolo">
      <ModelBaseFields :model="yolo.baseModel" :built-in-enabled="false" input-size-mode="longestSide" :path-placeholder="modelPathPlaceholder" :test-id-prefix="`${testIdPrefix}-base`">
        <template #after-model-path>
          <label class="model-row model-row-wide"><span class="model-label">标签路径</span><div class="space-y-2"><div class="model-path-row"><input v-model.trim="labelPath" class="app-input" :data-testid="`${testIdPrefix}-label-path`" placeholder="例如：D:\\models\\labels.yaml" /><button class="app-button app-button-ghost model-path-button" type="button" @click="pickLabelPath"><AppIcon name="folder-open" :size="16" /></button></div><p v-if="labelLoading || labelHint" class="text-xs text-(--app-text-faint)">{{ labelLoading ? '正在读取标签文件...' : labelHint }}</p></div></label>
          <label class="model-row model-row-wide"><span class="model-label">类别数量</span><input :value="String(yolo.classCount ?? 0)" class="app-input app-input-readonly" :data-testid="`${testIdPrefix}-class-count`" readonly type="number" /></label>
        </template>
      </ModelBaseFields>
      <div class="model-grid"><label class="model-row"><span class="model-label">后处理</span><AppSelect v-model="postprocessKind" :options="yoloPostprocessOptions" :test-id="`${testIdPrefix}-postprocess-kind`" /></label><label v-if="mode === 'txtDet'" class="model-row"><span class="model-label">文本类别索引</span><input v-model.number="yolo.txtIdx" class="app-input" :data-testid="`${testIdPrefix}-txt-idx`" min="0" type="number" /></label></div>
      <div class="model-grid"><label class="model-row"><span class="model-label">置信度阈值</span><input v-model.number="yolo.confidenceThresh" class="app-input" :data-testid="`${testIdPrefix}-confidence`" max="1" min="0" step="0.01" type="number" /></label><label v-if="postprocessKind === 'LegacyNms'" class="model-row"><span class="model-label">IOU 阈值</span><input v-model.number="yolo.iouThresh" class="app-input" :data-testid="`${testIdPrefix}-iou`" max="1" min="0" step="0.01" type="number" /></label></div>
    </template>
    <template v-else-if="dbNet">
      <ModelBaseFields :model="dbNet.baseModel" :built-in-enabled="false" input-width-hint="动态宽度有效；静态宽度以模型输入为准。" input-height-hint="动态高度有效；静态高度以模型输入为准。" path-placeholder="例如：D:\\models\\ocr-dbnet.onnx" :test-id-prefix="`${testIdPrefix}-base`" />
      <div class="model-grid"><label class="model-row"><span class="model-label">二值化阈值</span><input v-model.number="dbNet.dbThresh" class="app-input" :data-testid="`${testIdPrefix}-db-thresh`" max="1" min="0" step="0.01" type="number" /></label><label class="model-row"><span class="model-label">框阈值</span><input v-model.number="dbNet.dbBoxThresh" class="app-input" :data-testid="`${testIdPrefix}-db-box-thresh`" max="1" min="0" step="0.01" type="number" /></label></div>
      <div class="model-grid"><label class="model-row"><span class="model-label">扩张比例</span><input v-model.number="dbNet.unclipRatio" class="app-input" :data-testid="`${testIdPrefix}-unclip-ratio`" min="0" step="0.1" type="number" /></label><label class="model-row"><span class="model-label">启用膨胀</span><span class="model-toggle"><input v-model="dbNet.useDilation" type="checkbox" /><span>对弱文本边缘更友好，但可能带来额外噪点。</span></span></label></div>
    </template>
    <template v-else-if="crnn">
      <ModelBaseFields :model="crnn.baseModel" input-width-hint="静态宽度有效；实际宽度以模型输入为准。" input-height-hint="动态高度有效；静态高度以模型输入为准。" path-placeholder="例如：D:\\models\\ocr-rec.onnx" :test-id-prefix="`${testIdPrefix}-base`"><template #after-model-path><label v-if="crnn.baseModel.modelSource === 'Custom'" class="model-row model-row-wide"><span class="model-label">字典路径</span><div class="model-path-row"><input v-model.trim="dictPath" class="app-input" :data-testid="`${testIdPrefix}-dict-path`" placeholder="例如：D:\\models\\keys.txt" /><button class="app-button app-button-ghost model-path-button" type="button" @click="pickDictPath"><AppIcon name="folder-open" :size="16" /></button></div></label></template></ModelBaseFields>
      <div class="model-grid"><label class="model-row"><span class="model-label">单会话算子内线程数量</span><input v-model.number="crnn.parallelCpuSessionIntraThreads" class="app-input" :data-testid="`${testIdPrefix}-parallel-session-intra-threads`" min="1" step="1" type="number" /></label><label class="model-row"><span class="model-label">缩放插值方式</span><AppSelect v-model="crnn.resizeFilter" :options="resizeFilterOptions" :test-id="`${testIdPrefix}-resize-filter`" /></label></div>
      <div class="model-grid"><label class="model-row"><span class="model-label">识别执行模式</span><AppSelect v-model="crnn.processingMode" :options="processingModeOptions" :test-id="`${testIdPrefix}-processing-mode`" /></label><label v-if="crnn.processingMode === 'MicroBatch'" class="model-row"><span class="model-label">批次大小</span><input v-model.number="crnn.microBatchSize" class="app-input" :data-testid="`${testIdPrefix}-micro-batch-size`" min="1" step="1" type="number" /></label></div>
      <div v-if="crnn.processingMode === 'MicroBatch'" class="model-grid"><label class="model-row"><span class="model-label">宽度分桶步长</span><input v-model.number="crnn.widthBucketStep" class="app-input" :data-testid="`${testIdPrefix}-width-bucket-step`" min="8" step="8" type="number" /></label></div>
    </template>
  </div>
</template>
<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import { scriptService } from '@/services/scriptService';
import type { DetectorType } from '@/types/bindings/DetectorType';
import type { RecognizerType } from '@/types/bindings/RecognizerType';
import ModelBaseFields from './ModelBaseFields.vue';
import { createDetectorByKind, createRecognizerByKind, defaultYoloPostprocessKind, extractCrnn, extractDbNet, extractYoloDetector, resolveDetectorKind, resolveRecognizerKind, syncYoloPostprocessFields, type DetectorKind, type RecognizerKind } from '@/utils/visionModelPresets';
const props = defineProps<{ mode: 'imgDet' | 'txtDet' | 'txtRec'; detectorModel?: DetectorType | null; recognizerModel?: RecognizerType | null; testIdPrefix: string; }>();
const emit = defineEmits<{ 'update:detectorModel': [value: DetectorType | null]; 'update:recognizerModel': [value: RecognizerType | null]; }>();
const imgDetectorOptions = [{ label: '不设置', value: 'none' }, { label: 'YOLO11', value: 'Yolo11' }, { label: 'YOLO26', value: 'Yolo26' }];
const txtDetectorOptions = [...imgDetectorOptions.slice(0, 2), { label: 'Paddle DBNet v5', value: 'PaddleDbNet5' }, { label: 'Paddle DBNet v6', value: 'PaddleDbNet6' }, imgDetectorOptions[2]];
const recognizerOptions = [{ label: '不设置', value: 'none' }, { label: 'Paddle CRNN v5', value: 'PaddleCrnn5' }, { label: 'Paddle CRNN v6', value: 'PaddleCrnn6' }];
const yoloPostprocessOptions = [{ label: 'Legacy NMS', value: 'LegacyNms' }, { label: 'End-to-End', value: 'EndToEnd' }];
const resizeFilterOptions = ['Triangle', 'Gaussian', 'CatmullRom', 'Lanczos3', 'Nearest'].map((value) => ({ label: value, value }));
const processingModeOptions = [{ label: '单文本框', value: 'Single' }, { label: '批处理', value: 'MicroBatch' }];
const options = computed(() => props.mode === 'imgDet' ? imgDetectorOptions : props.mode === 'txtDet' ? txtDetectorOptions : recognizerOptions);
const detector = computed({ get: () => props.detectorModel ?? null, set: (value: DetectorType | null) => emit('update:detectorModel', value) });
const recognizer = computed({ get: () => props.recognizerModel ?? null, set: (value: RecognizerType | null) => emit('update:recognizerModel', value) });
const kind = computed(() => props.mode === 'txtRec' ? resolveRecognizerKind(recognizer.value) : resolveDetectorKind(detector.value));
const yolo = computed(() => extractYoloDetector(detector.value)); const dbNet = computed(() => extractDbNet(detector.value)); const crnn = computed(() => extractCrnn(recognizer.value));
const labelHint = ref<string | null>(null); const labelLoading = ref(false); let labelRequestId = 0;
const modelPathPlaceholder = computed(() => props.mode === 'imgDet' ? '例如：D:\\models\\img-det.onnx' : '例如：D:\\models\\txt-det.onnx');
const labelPath = computed({ get: () => yolo.value?.labelPath ?? '', set: (value: string) => { if (yolo.value) yolo.value.labelPath = value || null; } });
const dictPath = computed({ get: () => crnn.value?.dictPath ?? '', set: (value: string) => { if (crnn.value) crnn.value.dictPath = value || null; } });
const postprocessKind = computed({ get: () => yolo.value?.postprocessKind ?? defaultYoloPostprocessKind(kind.value === 'Yolo26' ? 'Yolo26' : 'Yolo11'), set: (value) => { if (yolo.value) { yolo.value.postprocessKind = value; syncYoloPostprocessFields(yolo.value); } } });
function setKind(value: string | number | null) { if (props.mode === 'txtRec') { recognizer.value = createRecognizerByKind(String(value ?? 'none') as RecognizerKind); return; } detector.value = createDetectorByKind(String(value ?? 'none') as DetectorKind, props.mode === 'txtDet'); }
function normalize() { if (yolo.value) { yolo.value.baseModel.modelSource = 'Custom'; syncYoloPostprocessFields(yolo.value); } if (dbNet.value) dbNet.value.baseModel.modelSource = 'Custom'; if (crnn.value) { if (!crnn.value.resizeFilter) crnn.value.resizeFilter = 'Triangle'; if (!crnn.value.processingMode) crnn.value.processingMode = 'Single'; if (!crnn.value.microBatchSize || crnn.value.microBatchSize < 1) crnn.value.microBatchSize = 4; if (!crnn.value.widthBucketStep || crnn.value.widthBucketStep < 8) crnn.value.widthBucketStep = 32; if (!crnn.value.parallelCpuSessionIntraThreads || crnn.value.parallelCpuSessionIntraThreads < 1) crnn.value.parallelCpuSessionIntraThreads = 1; if (crnn.value.baseModel.modelSource === 'BuiltIn') crnn.value.dictPath = null; } }
async function hydrateClassCount(path: string | null | undefined) { if (!yolo.value) return; const value = path?.trim() ?? ''; if (!value) { yolo.value.classCount = 0; labelHint.value = null; return; } const requestId = ++labelRequestId; labelLoading.value = true; try { const labels = await scriptService.getYoloLabels(value); if (requestId !== labelRequestId || !yolo.value) return; yolo.value.classCount = labels.length; labelHint.value = labels.length ? `已从标签文件读取 ${labels.length} 个类别。` : '标签文件已读取，但未解析到任何 names。'; } catch (error) { if (requestId === labelRequestId) labelHint.value = error instanceof Error ? `标签文件读取失败：${error.message}` : '标签文件读取失败，请检查路径和格式。'; } finally { if (requestId === labelRequestId) labelLoading.value = false; } }
async function pickLabelPath() { const value = await open({ multiple: false, directory: false, filters: [{ name: 'Label Files', extensions: ['yaml', 'yml', 'json', 'txt'] }] }); if (typeof value === 'string' && value) labelPath.value = value; }
async function pickDictPath() { const value = await open({ multiple: false, directory: false, filters: [{ name: 'Dictionary Files', extensions: ['txt', 'dict'] }] }); if (typeof value === 'string' && value) dictPath.value = value; }
watch([detector, recognizer], normalize, { immediate: true, deep: true }); watch(() => yolo.value?.labelPath, hydrateClassCount, { immediate: true });
</script>
<style scoped>
.model-row { display: grid; gap: .9rem; align-items: center; } .model-grid { display: grid; gap: 1rem 1.25rem; } .model-label { font-size: .95rem; font-weight: 600; color: var(--app-text-strong); } .model-path-row { display: grid; grid-template-columns: minmax(0, 1fr) auto; gap: .65rem; align-items: center; } .model-path-button { min-width: 2.75rem; height: 2.75rem; padding: 0; } .model-toggle { display: inline-flex; align-items: center; gap: .75rem; font-size: .875rem; color: var(--app-text-soft); } @media (min-width: 768px) { .model-row, .model-row-wide { grid-template-columns: 96px minmax(0, 1fr); } .model-grid { grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); } }
</style>
