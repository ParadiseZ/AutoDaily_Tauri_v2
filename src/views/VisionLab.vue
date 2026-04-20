<template>
  <div class="vision-lab-shell h-[100svh] overflow-hidden px-4 py-4 lg:px-6 lg:py-5">
    <div class="mx-auto flex h-full max-w-[1880px] flex-col gap-4">
      <header class="vision-lab-header rounded-[28px] border border-[var(--app-border)] px-5 py-4 lg:px-6">
        <div class="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
          <div class="space-y-2">
            <div class="flex flex-wrap items-center gap-3">
              <span class="rounded-full border border-[var(--app-border)] bg-white/55 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.24em] text-[var(--app-text-faint)]">
                Vision Lab
              </span>
              <span v-if="isStandalone" class="rounded-full bg-[var(--app-accent-soft)] px-3 py-1 text-xs font-medium text-[var(--app-accent)]">
                独立窗口
              </span>
            </div>
            <div class="space-y-1">
              <h1 class="text-2xl font-semibold tracking-[-0.05em] text-[var(--app-text-strong)] lg:text-3xl">视觉测试工作台</h1>
              <p class="max-w-3xl text-sm leading-6 text-[var(--app-text-soft)]">
                左侧组织图片与设备截图，中间做叠加预览和取色采样，右侧维护模型参数、分析动作与结果过滤。
              </p>
            </div>
          </div>

          <div class="flex flex-wrap items-center gap-2">
            <button class="app-button app-button-ghost" type="button" @click="pickImageDirectory">
              <AppIcon name="folder-open" :size="16" />
              打开图片目录
            </button>
            <button class="app-button app-button-ghost" type="button" @click="pickSaveDirectory">
              <AppIcon name="folder-output" :size="16" />
              保存目录
            </button>
            <button class="app-button app-button-primary shadow-lg shadow-[var(--app-accent-soft)]" type="button" @click="captureFromDevice">
              <AppIcon name="camera" :size="16" />
              设备截图
            </button>
          </div>
        </div>
      </header>

      <div class="grid min-h-0 flex-1 gap-4 xl:grid-cols-[320px_minmax(0,1fr)_420px]">
        <aside class="vision-side-panel min-h-0 overflow-hidden rounded-[26px] border border-[var(--app-border)] bg-[var(--app-panel)]">
          <div class="flex h-full flex-col">
            <div class="border-b border-[var(--app-border)] px-4 py-4">
              <div class="space-y-3">
                <div>
                  <p class="text-xs uppercase tracking-[0.2em] text-[var(--app-text-faint)]">数据源</p>
                  <p class="mt-1 text-sm text-[var(--app-text-soft)]">目录图像会长期保留，当前采集区用来管理设备截图和暂存图。</p>
                </div>
                <label class="space-y-2">
                  <span class="text-xs font-semibold text-[var(--app-text-faint)]">文件名筛选</span>
                  <input
                    v-model.trim="preferences.filterText"
                    class="app-input"
                    placeholder="按文件名筛选图片"
                    @change="persistPreferences"
                  />
                </label>
                <div class="space-y-2">
                  <span class="text-xs font-semibold text-[var(--app-text-faint)]">设备</span>
                  <AppSelect
                    v-model="selectedDeviceId"
                    :options="deviceOptions"
                    placeholder="请选择设备"
                    test-id="vision-lab-device"
                  />
                </div>
                <div class="space-y-2">
                  <div class="flex items-center justify-between text-xs text-[var(--app-text-faint)]">
                    <span>图片目录</span>
                    <span class="truncate pl-3 text-right">{{ preferences.imageDir || '未选择' }}</span>
                  </div>
                  <div class="flex items-center justify-between text-xs text-[var(--app-text-faint)]">
                    <span>保存目录</span>
                    <span class="truncate pl-3 text-right">{{ preferences.saveDir || '未设置' }}</span>
                  </div>
                </div>
              </div>
            </div>

            <div class="min-h-0 flex-1 overflow-y-auto px-4 py-4">
              <section class="space-y-3">
                <div class="flex items-center justify-between">
                  <div>
                    <p class="text-sm font-semibold text-[var(--app-text-strong)]">目录图像</p>
                    <p class="text-xs text-[var(--app-text-faint)]">{{ filteredFolderItems.length }} 张</p>
                  </div>
                  <button class="app-button app-button-ghost px-3 py-2 text-xs" type="button" @click="reloadImageDirectory" :disabled="!preferences.imageDir">
                    刷新
                  </button>
                </div>

                <div v-if="!filteredFolderItems.length" class="rounded-[22px] border border-dashed border-[var(--app-border)] px-4 py-6 text-sm text-[var(--app-text-soft)]">
                  先选择一个图像目录，或调整筛选条件。
                </div>

                <button
                  v-for="item in filteredFolderItems"
                  :key="item.id"
                  class="vision-list-item w-full rounded-[20px] border px-3 py-3 text-left transition"
                  :class="selectedItem?.id === item.id ? 'border-[var(--app-accent)] bg-[var(--app-accent-soft)]/40' : 'border-[var(--app-border)] hover:border-[var(--app-accent)]/35'"
                  type="button"
                  @click="selectItem(item)"
                >
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0">
                      <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ item.name }}</p>
                      <p class="mt-1 line-clamp-2 text-xs leading-5 text-[var(--app-text-faint)]">{{ item.path }}</p>
                    </div>
                    <span class="rounded-full bg-emerald-500/12 px-2 py-1 text-[11px] font-medium text-emerald-700">已保存</span>
                  </div>
                </button>
              </section>

              <section class="mt-6 space-y-3">
                <div class="flex items-center justify-between">
                  <div>
                    <p class="text-sm font-semibold text-[var(--app-text-strong)]">当前采集</p>
                    <p class="text-xs text-[var(--app-text-faint)]">{{ captureItems.length }} 张</p>
                  </div>
                </div>

                <div v-if="!captureItems.length" class="rounded-[22px] border border-dashed border-[var(--app-border)] px-4 py-6 text-sm text-[var(--app-text-soft)]">
                  使用设备截图后，这里会显示当前工作台的暂存图像。
                </div>

                <div
                  v-for="item in captureItems"
                  :key="item.id"
                  class="rounded-[22px] border px-3 py-3 transition"
                  :class="[
                    selectedItem?.id === item.id ? 'border-[var(--app-accent)] bg-[var(--app-accent-soft)]/35' : 'border-[var(--app-border)]',
                    item.saved ? 'shadow-[0_0_0_1px_rgba(16,185,129,0.1)]' : 'shadow-[0_0_0_1px_rgba(245,158,11,0.08)]',
                  ]"
                >
                  <button class="w-full text-left" type="button" @click="selectItem(item)">
                    <div class="flex items-start justify-between gap-3">
                      <div class="min-w-0">
                        <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ item.name }}</p>
                        <p class="mt-1 text-xs text-[var(--app-text-faint)]">
                          {{ item.saved ? item.savedPath || '已保存' : '未保存到自定义目录' }}
                        </p>
                      </div>
                      <span
                        class="rounded-full px-2 py-1 text-[11px] font-medium"
                        :class="item.saved ? 'bg-emerald-500/12 text-emerald-700' : 'bg-amber-500/14 text-amber-700'"
                      >
                        {{ item.saved ? '已保存' : '未保存' }}
                      </span>
                    </div>
                  </button>
                  <div class="mt-3 flex items-center gap-2">
                    <button
                      class="app-button app-button-ghost px-3 py-2 text-xs"
                      type="button"
                      :disabled="item.saved || savingCaptureId === item.id"
                      @click="saveCaptureItem(item)"
                    >
                      <AppIcon name="save" :size="14" />
                      {{ item.saved ? '已保存' : savingCaptureId === item.id ? '保存中...' : '保存到本地' }}
                    </button>
                    <span class="text-xs text-[var(--app-text-faint)]">{{ formatRelativeTime(item.createdAt) }}</span>
                  </div>
                </div>
              </section>
            </div>
          </div>
        </aside>

        <main class="min-h-0 overflow-hidden rounded-[26px] border border-[var(--app-border)] bg-[var(--app-panel)]">
          <div class="flex h-full flex-col">
            <div class="border-b border-[var(--app-border)] px-4 py-4">
              <div class="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
                <div class="min-w-0">
                  <p class="truncate text-lg font-semibold text-[var(--app-text-strong)]">{{ selectedItem?.name || '未选择图像' }}</p>
                  <p class="mt-1 truncate text-sm text-[var(--app-text-soft)]">{{ selectedItem?.path || selectedItem?.savedPath || '请从左侧选择图像或先截图' }}</p>
                </div>
                <div class="flex flex-wrap items-center gap-2">
                  <button
                    v-for="tool in previewTools"
                    :key="tool.value"
                    class="app-button app-button-ghost px-3 py-2 text-xs"
                    type="button"
                    :class="activeTool === tool.value ? '!border-[var(--app-accent)] !text-[var(--app-accent)]' : ''"
                    @click="activeTool = tool.value"
                  >
                    {{ tool.label }}
                  </button>
                  <button class="app-button app-button-ghost px-3 py-2 text-xs" type="button" @click="fitPreview">适配</button>
                  <button class="app-button app-button-ghost px-3 py-2 text-xs" type="button" @click="zoomOut">-</button>
                  <span class="rounded-full border border-[var(--app-border)] px-3 py-2 text-xs text-[var(--app-text-soft)]">{{ Math.round(zoom * 100) }}%</span>
                  <button class="app-button app-button-ghost px-3 py-2 text-xs" type="button" @click="zoomIn">+</button>
                </div>
              </div>
            </div>

            <div ref="previewContainerRef" class="min-h-0 flex-1 overflow-auto px-4 py-4">
              <div
                v-if="selectedPreviewUrl"
                class="vision-preview-frame mx-auto w-max rounded-[28px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] p-4"
              >
                <div
                  ref="previewSurfaceRef"
                  class="relative overflow-hidden rounded-[20px] bg-[#111827]"
                  :style="previewSurfaceStyle"
                  @mousedown="handlePreviewPointerDown"
                  @mousemove="handlePreviewPointerMove"
                  @mouseup="handlePreviewPointerUp"
                  @mouseleave="handlePreviewPointerUp"
                  @click="handlePreviewClick"
                >
                  <img
                    ref="previewImageRef"
                    :src="selectedPreviewUrl"
                    class="select-none"
                    :style="previewImageStyle"
                    draggable="false"
                    @load="handleImageLoaded"
                  />

                  <div class="pointer-events-none absolute inset-0">
                    <div
                      v-for="entry in overlayEntries"
                      :key="entry.key"
                      class="absolute rounded-[10px] border text-[10px] font-semibold shadow-lg"
                      :class="entry.kind === 'ocr' ? 'border-cyan-300/90 bg-cyan-500/15 text-cyan-50' : 'border-amber-300/90 bg-amber-500/18 text-amber-50'"
                      :style="getBoxStyle(entry.box)"
                    >
                      <span class="absolute left-1 top-1 rounded-full bg-black/55 px-1.5 py-0.5 backdrop-blur-sm">
                        {{ entry.label }}
                      </span>
                    </div>

                    <div
                      v-if="regionDraft"
                      class="absolute rounded-[10px] border border-dashed border-white/90 bg-white/12"
                      :style="getBoxStyle(regionDraft)"
                    />

                    <div
                      v-if="pickedPoint"
                      class="absolute h-4 w-4 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-white shadow-[0_0_0_2px_rgba(15,23,42,0.3)]"
                      :style="{
                        left: `${pickedPoint.x * zoom}px`,
                        top: `${pickedPoint.y * zoom}px`,
                        backgroundColor: pickedPoint.hex,
                      }"
                    />
                  </div>
                </div>
              </div>

              <div v-else class="flex h-full min-h-[360px] items-center justify-center rounded-[28px] border border-dashed border-[var(--app-border)] text-sm text-[var(--app-text-soft)]">
                选择一张目录图像，或先从设备采集截图。
              </div>
            </div>
          </div>
        </main>

        <aside class="vision-side-panel min-h-0 overflow-hidden rounded-[26px] border border-[var(--app-border)] bg-[var(--app-panel)]">
          <div class="flex h-full flex-col">
            <div class="border-b border-[var(--app-border)] px-4 py-4">
              <div class="flex items-center justify-between">
                <div>
                  <p class="text-sm font-semibold text-[var(--app-text-strong)]">分析控制台</p>
                  <p class="mt-1 text-xs text-[var(--app-text-faint)]">按模型参数运行检测、OCR、筛选与颜色采样。</p>
                </div>
              </div>
            </div>

            <div class="min-h-0 flex-1 overflow-y-auto px-4 py-4">
              <section class="space-y-3">
                <div class="flex items-center justify-between">
                  <p class="text-sm font-semibold text-[var(--app-text-strong)]">动作</p>
                  <div class="flex items-center gap-2 text-xs text-[var(--app-text-faint)]">
                    <label class="inline-flex items-center gap-1">
                      <input v-model="showDetOverlay" type="checkbox" />
                      <span>Det</span>
                    </label>
                    <label class="inline-flex items-center gap-1">
                      <input v-model="showOcrOverlay" type="checkbox" />
                      <span>OCR</span>
                    </label>
                  </div>
                </div>
                <div class="grid grid-cols-2 gap-2">
                  <button class="app-button app-button-ghost justify-center" type="button" :disabled="!canRunDetection || isRunningDet" @click="runDetection">
                    {{ isRunningDet ? '检测中...' : '目标检测' }}
                  </button>
                  <button class="app-button app-button-primary justify-center" type="button" :disabled="!canRunOcr || isRunningOcr" @click="runOcr">
                    {{ isRunningOcr ? 'OCR 中...' : '完整 OCR' }}
                  </button>
                </div>
              </section>

              <section class="mt-6 space-y-3">
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">目标检测模型</p>
                <label class="space-y-2">
                  <span class="text-xs font-semibold text-[var(--app-text-faint)]">模型类型</span>
                  <AppSelect
                    :model-value="imgDetKind"
                    :options="detectorOptions"
                    placeholder="选择模型"
                    test-id="vision-lab-img-det-kind"
                    @update:model-value="setImgDetKind"
                  />
                </label>
                <template v-if="imgDetYolo">
                  <label class="space-y-2">
                    <span class="text-xs font-semibold text-[var(--app-text-faint)]">模型路径</span>
                    <input v-model.trim="imgDetYolo.baseModel.modelPath" class="app-input" placeholder="D:\\models\\img-det.onnx" />
                  </label>
                  <div class="grid grid-cols-2 gap-3">
                    <label class="space-y-2">
                      <span class="text-xs font-semibold text-[var(--app-text-faint)]">类别数量</span>
                      <input v-model.number="imgDetYolo.classCount" class="app-input" min="1" type="number" />
                    </label>
                    <label class="space-y-2">
                      <span class="text-xs font-semibold text-[var(--app-text-faint)]">标签路径</span>
                      <input v-model.trim="imgDetYolo.labelPath" class="app-input" placeholder="D:\\models\\labels.yaml" />
                    </label>
                  </div>
                  <div class="grid grid-cols-2 gap-3">
                    <label class="space-y-2">
                      <span class="text-xs font-semibold text-[var(--app-text-faint)]">置信度</span>
                      <input v-model.number="imgDetYolo.confidenceThresh" class="app-input" min="0" max="1" step="0.01" type="number" />
                    </label>
                    <label class="space-y-2">
                      <span class="text-xs font-semibold text-[var(--app-text-faint)]">IOU</span>
                      <input v-model.number="imgDetYolo.iouThresh" class="app-input" min="0" max="1" step="0.01" type="number" />
                    </label>
                  </div>
                </template>
                <template v-else-if="imgDetDbNet">
                  <label class="space-y-2">
                    <span class="text-xs font-semibold text-[var(--app-text-faint)]">模型路径</span>
                    <input v-model.trim="imgDetDbNet.baseModel.modelPath" class="app-input" placeholder="D:\\models\\ocr-dbnet.onnx" />
                  </label>
                  <div class="grid grid-cols-2 gap-3">
                    <label class="space-y-2">
                      <span class="text-xs font-semibold text-[var(--app-text-faint)]">二值化阈值</span>
                      <input v-model.number="imgDetDbNet.dbThresh" class="app-input" min="0" max="1" step="0.01" type="number" />
                    </label>
                    <label class="space-y-2">
                      <span class="text-xs font-semibold text-[var(--app-text-faint)]">框阈值</span>
                      <input v-model.number="imgDetDbNet.dbBoxThresh" class="app-input" min="0" max="1" step="0.01" type="number" />
                    </label>
                  </div>
                </template>
              </section>

              <section class="mt-6 space-y-3">
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">文字检测 / 识别</p>
                <label class="space-y-2">
                  <span class="text-xs font-semibold text-[var(--app-text-faint)]">文字检测类型</span>
                  <AppSelect
                    :model-value="txtDetKind"
                    :options="detectorOptions"
                    placeholder="选择模型"
                    test-id="vision-lab-txt-det-kind"
                    @update:model-value="setTxtDetKind"
                  />
                </label>
                <template v-if="txtDetYolo">
                  <label class="space-y-2">
                    <span class="text-xs font-semibold text-[var(--app-text-faint)]">模型路径</span>
                    <input v-model.trim="txtDetYolo.baseModel.modelPath" class="app-input" placeholder="D:\\models\\txt-det.onnx" />
                  </label>
                  <div class="grid grid-cols-2 gap-3">
                    <label class="space-y-2">
                      <span class="text-xs font-semibold text-[var(--app-text-faint)]">标签路径</span>
                      <input v-model.trim="txtDetYolo.labelPath" class="app-input" placeholder="D:\\models\\labels.yaml" />
                    </label>
                    <label class="space-y-2">
                      <span class="text-xs font-semibold text-[var(--app-text-faint)]">文本索引</span>
                      <input v-model.number="txtDetYolo.txtIdx" class="app-input" min="0" type="number" />
                    </label>
                  </div>
                </template>
                <template v-else-if="txtDetDbNet">
                  <label class="space-y-2">
                    <span class="text-xs font-semibold text-[var(--app-text-faint)]">模型路径</span>
                    <input v-model.trim="txtDetDbNet.baseModel.modelPath" class="app-input" placeholder="D:\\models\\ocr-dbnet.onnx" />
                  </label>
                </template>

                <label class="space-y-2">
                  <span class="text-xs font-semibold text-[var(--app-text-faint)]">文字识别类型</span>
                  <AppSelect
                    :model-value="txtRecKind"
                    :options="recognizerOptions"
                    placeholder="选择识别模型"
                    test-id="vision-lab-txt-rec-kind"
                    @update:model-value="setTxtRecKind"
                  />
                </label>
                <template v-if="txtRecCrnn">
                  <label class="space-y-2">
                    <span class="text-xs font-semibold text-[var(--app-text-faint)]">模型路径</span>
                    <input v-model.trim="txtRecCrnn.baseModel.modelPath" class="app-input" placeholder="D:\\models\\ocr-rec.onnx" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-semibold text-[var(--app-text-faint)]">字典路径</span>
                    <input v-model.trim="txtRecCrnn.dictPath" class="app-input" placeholder="D:\\models\\keys.txt" />
                  </label>
                </template>
              </section>

              <section class="mt-6 space-y-3">
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">结果过滤</p>
                <label class="space-y-2">
                  <span class="text-xs font-semibold text-[var(--app-text-faint)]">OCR 文本</span>
                  <input v-model.trim="ocrFilterText" class="app-input" placeholder="包含关键字" />
                </label>
                <div class="grid grid-cols-2 gap-3">
                  <label class="space-y-2">
                    <span class="text-xs font-semibold text-[var(--app-text-faint)]">背景色</span>
                    <AppSelect v-model="ocrBgFilter" :options="colorFilterOptions" placeholder="全部" test-id="vision-lab-bg-filter" />
                  </label>
                  <label class="space-y-2">
                    <span class="text-xs font-semibold text-[var(--app-text-faint)]">文字色</span>
                    <AppSelect v-model="ocrFgFilter" :options="colorFilterOptions" placeholder="全部" test-id="vision-lab-fg-filter" />
                  </label>
                </div>
              </section>

              <section class="mt-6 space-y-3">
                <p class="text-sm font-semibold text-[var(--app-text-strong)]">取色 / 比对</p>
                <div class="grid grid-cols-2 gap-3">
                  <label class="space-y-2">
                    <span class="text-xs font-semibold text-[var(--app-text-faint)]">目标颜色</span>
                    <input v-model="targetColorHex" class="h-11 w-full rounded-[14px] border border-[var(--app-border)] bg-transparent px-3" type="color" />
                  </label>
                  <div class="rounded-[18px] border border-[var(--app-border)] px-3 py-3 text-xs text-[var(--app-text-soft)]">
                    <p>点取色：{{ pickedPoint?.hex || '--' }}</p>
                    <p>区域均色：{{ regionSample?.hex || '--' }}</p>
                  </div>
                </div>
                <div v-if="pickedPoint" class="rounded-[18px] border border-[var(--app-border)] px-3 py-3 text-xs leading-6 text-[var(--app-text-soft)]">
                  <p>点坐标：({{ pickedPoint.x }}, {{ pickedPoint.y }})</p>
                  <p>RGB：{{ pickedPoint.rgb.r }}, {{ pickedPoint.rgb.g }}, {{ pickedPoint.rgb.b }}</p>
                  <p>HSV：{{ pickedPoint.hsv.h }} / {{ pickedPoint.hsv.s }} / {{ pickedPoint.hsv.v }}</p>
                  <p>与目标色距离：{{ pickedPoint.deltaToTarget.toFixed(2) }}</p>
                </div>
                <div v-if="regionSample" class="rounded-[18px] border border-[var(--app-border)] px-3 py-3 text-xs leading-6 text-[var(--app-text-soft)]">
                  <p>区域：({{ regionSample.box.x1 }}, {{ regionSample.box.y1 }}) - ({{ regionSample.box.x2 }}, {{ regionSample.box.y2 }})</p>
                  <p>均值 RGB：{{ regionSample.rgb.r }}, {{ regionSample.rgb.g }}, {{ regionSample.rgb.b }}</p>
                  <p>HEX：{{ regionSample.hex }}</p>
                  <p>与目标色距离：{{ regionSample.deltaToTarget.toFixed(2) }}</p>
                </div>
              </section>

              <section class="mt-6 space-y-3">
                <div class="flex items-center justify-between">
                  <p class="text-sm font-semibold text-[var(--app-text-strong)]">OCR 结果</p>
                  <span class="text-xs text-[var(--app-text-faint)]">{{ filteredOcrResults.length }}/{{ ocrResults.length }}</span>
                </div>
                <div v-if="!ocrResults.length" class="rounded-[18px] border border-dashed border-[var(--app-border)] px-3 py-4 text-xs text-[var(--app-text-soft)]">
                  运行 OCR 后，这里会展示文本结果以及背景色/文字色分析。
                </div>
                <div v-for="(item, index) in filteredOcrResults" :key="`ocr-${index}`" class="rounded-[18px] border border-[var(--app-border)] px-3 py-3">
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0">
                      <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ item.txt || '(空文本)' }}</p>
                      <p class="mt-1 text-xs text-[var(--app-text-faint)]">框：{{ formatBox(item.bounding_box) }}</p>
                    </div>
                    <span class="rounded-full bg-cyan-500/12 px-2 py-1 text-[11px] font-medium text-cyan-700">
                      {{ averageScore(item).toFixed(3) }}
                    </span>
                  </div>
                  <div class="mt-2 flex flex-wrap gap-2 text-[11px]">
                    <span class="rounded-full bg-slate-500/10 px-2 py-1 text-slate-700">背景 {{ item.bgColor }}</span>
                    <span class="rounded-full bg-slate-500/10 px-2 py-1 text-slate-700">文字 {{ item.fgColor }}</span>
                  </div>
                </div>
              </section>

              <section class="mt-6 space-y-3 pb-4">
                <div class="flex items-center justify-between">
                  <p class="text-sm font-semibold text-[var(--app-text-strong)]">检测结果</p>
                  <span class="text-xs text-[var(--app-text-faint)]">{{ detResults.length }}</span>
                </div>
                <div v-if="!detResults.length" class="rounded-[18px] border border-dashed border-[var(--app-border)] px-3 py-4 text-xs text-[var(--app-text-soft)]">
                  运行目标检测后，这里会展示目标框与标签。
                </div>
                <div v-for="(item, index) in detResults" :key="`det-${index}`" class="rounded-[18px] border border-[var(--app-border)] px-3 py-3">
                  <div class="flex items-start justify-between gap-3">
                    <div class="min-w-0">
                      <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ item.label }}</p>
                      <p class="mt-1 text-xs text-[var(--app-text-faint)]">类目 #{{ item.index }} · {{ formatBox(item.bounding_box) }}</p>
                    </div>
                    <span class="rounded-full bg-amber-500/12 px-2 py-1 text-[11px] font-medium text-amber-700">
                      {{ item.score.toFixed(3) }}
                    </span>
                  </div>
                </div>
              </section>
            </div>
          </div>
        </aside>
      </div>
    </div>

    <canvas ref="hiddenCanvasRef" class="hidden"></canvas>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, reactive, ref } from 'vue';
import { useRoute } from 'vue-router';
import { open } from '@tauri-apps/plugin-dialog';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import { scriptService } from '@/services/scriptService';
import { visionLabService } from '@/services/visionLabService';
import { useDeviceStore } from '@/store/device';
import { getFromStore, setToStore, visionLabLaunchPresetKey, visionLabPreferencesKey } from '@/store/store';
import { showToast } from '@/utils/toast';
import {
  createDetectorByKind,
  createRecognizerByKind,
  ensureDetectorModel,
  ensureRecognizerModel,
  extractCrnn,
  extractDbNet,
  extractYoloDetector,
  resolveDetectorKind,
  resolveRecognizerKind,
  type DetectorKind,
  type RecognizerKind,
} from '@/utils/visionModelPresets';
import { DEFAULT_VISION_LAB_PREFERENCES, type VisionLabLaunchPreset, type VisionLabPreferences } from '@/types/app/domain';
import type { BoundingBox } from '@/types/bindings/BoundingBox';
import type { DetResult } from '@/types/bindings/DetResult';
import type { DetectorType } from '@/types/bindings/DetectorType';
import type { OcrResult } from '@/types/bindings/OcrResult';
import type { RecognizerType } from '@/types/bindings/RecognizerType';
import type { DeviceTable } from '@/types/bindings/DeviceTable';

type VisionItemKind = 'folder' | 'capture';
type PreviewTool = 'browse' | 'pick' | 'region';
type ColorTag = 'RED' | 'ORANGE' | 'YELLOW' | 'GREEN' | 'CYAN' | 'BLUE' | 'PURPLE' | 'PINK' | 'BLACK' | 'WHITE' | 'GRAY' | 'OTHER';

interface VisionSourceItem {
  id: string;
  kind: VisionItemKind;
  name: string;
  path: string | null;
  previewUrl: string | null;
  stagedPath: string | null;
  savedPath: string | null;
  createdAt: string;
  saved: boolean;
}

interface RgbColor {
  r: number;
  g: number;
  b: number;
}

interface HsvColor {
  h: number;
  s: number;
  v: number;
}

interface PointSample {
  x: number;
  y: number;
  rgb: RgbColor;
  hsv: HsvColor;
  hex: string;
  deltaToTarget: number;
}

interface RegionSample {
  box: BoundingBox;
  rgb: RgbColor;
  hsv: HsvColor;
  hex: string;
  deltaToTarget: number;
}

interface VisionOcrResult extends OcrResult {
  bgColor: ColorTag;
  fgColor: ColorTag;
}

interface OverlayEntry {
  key: string;
  kind: 'ocr' | 'det';
  label: string;
  box: BoundingBox;
}

const route = useRoute();
const deviceStore = useDeviceStore();
const isStandalone = computed(() => route.query.standalone === '1');

const detectorOptions = [
  { label: '不设置', value: 'none', description: '当前字段留空。' },
  { label: 'YOLO11', value: 'Yolo11', description: '通用目标检测方案。' },
  { label: 'Paddle DBNet', value: 'PaddleDbNet', description: '适合文本区域检测。' },
  { label: 'YOLO26', value: 'Yolo26', description: '端到端检测方案。' },
];
const recognizerOptions = [
  { label: '不设置', value: 'none', description: '不启用识别模型。' },
  { label: 'Paddle CRNN', value: 'PaddleCrnn', description: '当前绑定里可用的文本识别模型。' },
];
const colorFilterOptions = [
  { label: '全部', value: 'ALL' },
  { label: 'RED', value: 'RED' },
  { label: 'ORANGE', value: 'ORANGE' },
  { label: 'YELLOW', value: 'YELLOW' },
  { label: 'GREEN', value: 'GREEN' },
  { label: 'CYAN', value: 'CYAN' },
  { label: 'BLUE', value: 'BLUE' },
  { label: 'PURPLE', value: 'PURPLE' },
  { label: 'PINK', value: 'PINK' },
  { label: 'BLACK', value: 'BLACK' },
  { label: 'WHITE', value: 'WHITE' },
  { label: 'GRAY', value: 'GRAY' },
  { label: 'OTHER', value: 'OTHER' },
];
const previewTools = [
  { label: '浏览', value: 'browse' as const },
  { label: '点取色', value: 'pick' as const },
  { label: '区域采样', value: 'region' as const },
];

const preferences = reactive<VisionLabPreferences>({ ...DEFAULT_VISION_LAB_PREFERENCES });
const folderItems = ref<VisionSourceItem[]>([]);
const captureItems = ref<VisionSourceItem[]>([]);
const selectedItemId = ref<string | null>(null);
const selectedPreviewUrl = ref<string | null>(null);
const selectedDeviceId = ref<string | null>(null);
const imgDetModel = ref<DetectorType | null>(ensureDetectorModel(null, false));
const txtDetModel = ref<DetectorType | null>(ensureDetectorModel(null, true));
const txtRecModel = ref<RecognizerType | null>(ensureRecognizerModel(null));
const isRunningDet = ref(false);
const isRunningOcr = ref(false);
const savingCaptureId = ref<string | null>(null);
const detResults = ref<DetResult[]>([]);
const ocrResults = ref<VisionOcrResult[]>([]);
const ocrFilterText = ref('');
const ocrBgFilter = ref<'ALL' | ColorTag>('ALL');
const ocrFgFilter = ref<'ALL' | ColorTag>('ALL');
const activeTool = ref<PreviewTool>('browse');
const showDetOverlay = ref(true);
const showOcrOverlay = ref(true);
const zoom = ref(1);
const naturalSize = reactive({ width: 0, height: 0 });
const previewContainerRef = ref<HTMLElement | null>(null);
const previewSurfaceRef = ref<HTMLElement | null>(null);
const previewImageRef = ref<HTMLImageElement | null>(null);
const hiddenCanvasRef = ref<HTMLCanvasElement | null>(null);
const pickedPoint = ref<PointSample | null>(null);
const regionSample = ref<RegionSample | null>(null);
const regionDraft = ref<BoundingBox | null>(null);
const regionStart = ref<{ x: number; y: number } | null>(null);
const targetColorHex = ref('#ffffff');

const filteredFolderItems = computed(() => {
  const keyword = preferences.filterText.trim().toLowerCase();
  if (!keyword) {
    return folderItems.value;
  }
  return folderItems.value.filter((item) => item.name.toLowerCase().includes(keyword));
});

const selectedItem = computed(() => {
  const all = [...captureItems.value, ...folderItems.value];
  return all.find((item) => item.id === selectedItemId.value) ?? null;
});

const selectedDevice = computed<DeviceTable | null>(() =>
  deviceStore.devices.find((device) => device.id === selectedDeviceId.value) ?? null,
);

const selectedImagePath = computed(() => selectedItem.value?.path ?? selectedItem.value?.stagedPath ?? null);
const canRunVision = computed(() => Boolean(selectedImagePath.value));
const canRunDetection = computed(() => canRunVision.value && Boolean(imgDetModel.value));
const canRunOcr = computed(() => canRunVision.value && Boolean(txtDetModel.value) && Boolean(txtRecModel.value));

const imgDetKind = computed<DetectorKind>(() => resolveDetectorKind(imgDetModel.value));
const txtDetKind = computed<DetectorKind>(() => resolveDetectorKind(txtDetModel.value));
const txtRecKind = computed<RecognizerKind>(() => resolveRecognizerKind(txtRecModel.value));
const imgDetYolo = computed(() => extractYoloDetector(imgDetModel.value));
const imgDetDbNet = computed(() => extractDbNet(imgDetModel.value));
const txtDetYolo = computed(() => extractYoloDetector(txtDetModel.value));
const txtDetDbNet = computed(() => extractDbNet(txtDetModel.value));
const txtRecCrnn = computed(() => extractCrnn(txtRecModel.value));

const filteredOcrResults = computed(() => {
  const keyword = ocrFilterText.value.trim();
  return ocrResults.value.filter((item) => {
    if (keyword && !item.txt.includes(keyword)) {
      return false;
    }
    if (ocrBgFilter.value !== 'ALL' && item.bgColor !== ocrBgFilter.value) {
      return false;
    }
    if (ocrFgFilter.value !== 'ALL' && item.fgColor !== ocrFgFilter.value) {
      return false;
    }
    return true;
  });
});

const overlayEntries = computed<OverlayEntry[]>(() => {
  const entries: OverlayEntry[] = [];
  if (showDetOverlay.value) {
    detResults.value.forEach((item, index) => {
      entries.push({
        key: `det-${index}`,
        kind: 'det',
        label: item.label,
        box: item.bounding_box,
      });
    });
  }
  if (showOcrOverlay.value) {
    filteredOcrResults.value.forEach((item, index) => {
      entries.push({
        key: `ocr-${index}`,
        kind: 'ocr',
        label: item.txt || '(空)',
        box: item.bounding_box,
      });
    });
  }
  return entries;
});

const deviceOptions = computed(() =>
  deviceStore.devices.map((device) => ({
    label: device.data.deviceName,
    value: device.id,
    description: device.data.platform,
  })),
);

const previewImageStyle = computed(() => ({
  width: `${Math.max(1, naturalSize.width * zoom.value)}px`,
  height: `${Math.max(1, naturalSize.height * zoom.value)}px`,
}));

const previewSurfaceStyle = computed(() => ({
  width: `${Math.max(1, naturalSize.width * zoom.value)}px`,
  height: `${Math.max(1, naturalSize.height * zoom.value)}px`,
  cursor: activeTool.value === 'pick' ? 'crosshair' : activeTool.value === 'region' ? 'crosshair' : 'default',
}));

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

async function persistPreferences() {
  await setToStore(visionLabPreferencesKey, clone(preferences));
}

function normalizeItemName(path: string) {
  return path.split(/[\\/]/).pop() || path;
}

function averageScore(item: OcrResult) {
  if (!item.score.length) {
    return 0;
  }
  return item.score.reduce((sum, value) => sum + value, 0) / item.score.length;
}

function formatBox(box: BoundingBox) {
  return `${box.x1}, ${box.y1}, ${box.x2}, ${box.y2}`;
}

function formatRelativeTime(value: string) {
  const timestamp = new Date(value).getTime();
  const minutes = Math.max(0, Math.round((Date.now() - timestamp) / 60000));
  if (minutes < 1) return '刚刚';
  if (minutes < 60) return `${minutes} 分钟前`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours} 小时前`;
  return `${Math.round(hours / 24)} 天前`;
}

function getBoxStyle(box: BoundingBox) {
  return {
    left: `${box.x1 * zoom.value}px`,
    top: `${box.y1 * zoom.value}px`,
    width: `${Math.max(1, (box.x2 - box.x1) * zoom.value)}px`,
    height: `${Math.max(1, (box.y2 - box.y1) * zoom.value)}px`,
  };
}

function rgbToHex(color: RgbColor) {
  return `#${[color.r, color.g, color.b].map((value) => value.toString(16).padStart(2, '0')).join('')}`;
}

function parseHexColor(value: string): RgbColor {
  const normalized = value.replace('#', '');
  const raw = normalized.length === 3
    ? normalized.split('').map((char) => `${char}${char}`).join('')
    : normalized.padEnd(6, '0').slice(0, 6);

  return {
    r: parseInt(raw.slice(0, 2), 16),
    g: parseInt(raw.slice(2, 4), 16),
    b: parseInt(raw.slice(4, 6), 16),
  };
}

function rgbDistance(left: RgbColor, right: RgbColor) {
  const dr = left.r - right.r;
  const dg = left.g - right.g;
  const db = left.b - right.b;
  return Math.sqrt(dr * dr + dg * dg + db * db);
}

function rgbToHsv(color: RgbColor): HsvColor {
  const r = color.r / 255;
  const g = color.g / 255;
  const b = color.b / 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const delta = max - min;

  let h = 0;
  if (delta !== 0) {
    if (max === r) h = ((g - b) / delta) % 6;
    else if (max === g) h = (b - r) / delta + 2;
    else h = (r - g) / delta + 4;
    h *= 60;
    if (h < 0) h += 360;
  }

  const s = max === 0 ? 0 : delta / max;
  const v = max;

  return {
    h: Math.round(h),
    s: Number(s.toFixed(3)),
    v: Number(v.toFixed(3)),
  };
}

function rgbToTag(color: RgbColor): ColorTag {
  const r = color.r / 255;
  const g = color.g / 255;
  const b = color.b / 255;
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const delta = max - min;
  const l = (max + min) / 2;
  const s = max === 0 ? 0 : delta / max;

  if (l < 0.15) return 'BLACK';
  if (l > 0.85 && s < 0.1) return 'WHITE';
  if (s < 0.1) return 'GRAY';

  let h = 0;
  if (delta === 0) h = 0;
  else if (max === r) h = ((g - b) / delta) % 6;
  else if (max === g) h = (b - r) / delta + 2;
  else h = (r - g) / delta + 4;
  h *= 60;
  if (h < 0) h += 360;

  if (h < 20 || h >= 330) return 'RED';
  if (h < 45) return 'ORANGE';
  if (h < 75) return 'YELLOW';
  if (h < 150) return 'GREEN';
  if (h < 200) return 'CYAN';
  if (h < 260) return 'BLUE';
  if (h < 300) return 'PURPLE';
  return 'PINK';
}

function ensureCanvasContext() {
  const canvas = hiddenCanvasRef.value;
  if (!canvas) return null;
  return canvas.getContext('2d', { willReadFrequently: true });
}

function getCanvasPixel(x: number, y: number): RgbColor | null {
  const ctx = ensureCanvasContext();
  if (!ctx || !naturalSize.width || !naturalSize.height) return null;
  const clampedX = Math.max(0, Math.min(naturalSize.width - 1, Math.round(x)));
  const clampedY = Math.max(0, Math.min(naturalSize.height - 1, Math.round(y)));
  const data = ctx.getImageData(clampedX, clampedY, 1, 1).data;
  return { r: data[0], g: data[1], b: data[2] };
}

function sampleRegionAverage(box: BoundingBox): RgbColor | null {
  const ctx = ensureCanvasContext();
  if (!ctx) return null;
  const x1 = Math.max(0, Math.min(naturalSize.width - 1, box.x1));
  const y1 = Math.max(0, Math.min(naturalSize.height - 1, box.y1));
  const x2 = Math.max(x1 + 1, Math.min(naturalSize.width, box.x2));
  const y2 = Math.max(y1 + 1, Math.min(naturalSize.height, box.y2));
  const width = Math.max(1, x2 - x1);
  const height = Math.max(1, y2 - y1);
  const imageData = ctx.getImageData(x1, y1, width, height).data;

  let sumR = 0;
  let sumG = 0;
  let sumB = 0;
  let count = 0;
  for (let index = 0; index < imageData.length; index += 4) {
    sumR += imageData[index];
    sumG += imageData[index + 1];
    sumB += imageData[index + 2];
    count += 1;
  }

  if (!count) return null;
  return {
    r: Math.round(sumR / count),
    g: Math.round(sumG / count),
    b: Math.round(sumB / count),
  };
}

function analyzeOcrBoxColors(box: BoundingBox): { bgColor: ColorTag; fgColor: ColorTag } {
  const ctx = ensureCanvasContext();
  if (!ctx) {
    return { bgColor: 'OTHER', fgColor: 'OTHER' };
  }

  const x1 = Math.max(0, Math.min(naturalSize.width - 1, box.x1));
  const y1 = Math.max(0, Math.min(naturalSize.height - 1, box.y1));
  const x2 = Math.max(0, Math.min(naturalSize.width - 1, box.x2));
  const y2 = Math.max(0, Math.min(naturalSize.height - 1, box.y2));
  if (x1 >= x2 || y1 >= y2) {
    return { bgColor: 'OTHER', fgColor: 'OTHER' };
  }

  const width = x2 - x1;
  const height = y2 - y1;
  const stepX = Math.max(1, Math.floor(width / 3));
  const stepY = Math.max(1, Math.floor(height / 3));
  const tags: ColorTag[] = [];

  for (let y = y1; y <= y2; y += stepY) {
    for (let x = x1; x <= x2; x += stepX) {
      const pixel = getCanvasPixel(x, y);
      if (pixel) {
        tags.push(rgbToTag(pixel));
      }
    }
  }

  if (!tags.length) {
    return { bgColor: 'OTHER', fgColor: 'OTHER' };
  }

  const counts = new Map<ColorTag, number>();
  tags.forEach((tag) => counts.set(tag, (counts.get(tag) ?? 0) + 1));
  const sorted = [...counts.entries()].sort((left, right) => right[1] - left[1]);
  const bgColor = sorted[0]?.[0] ?? 'OTHER';
  const fgColor = sorted.find(([tag]) => tag !== bgColor)?.[0] ?? bgColor;
  return { bgColor, fgColor };
}

function buildPointSample(x: number, y: number): PointSample | null {
  const rgb = getCanvasPixel(x, y);
  if (!rgb) return null;
  const hsv = rgbToHsv(rgb);
  return {
    x,
    y,
    rgb,
    hsv,
    hex: rgbToHex(rgb),
    deltaToTarget: rgbDistance(rgb, parseHexColor(targetColorHex.value)),
  };
}

function buildRegionSample(box: BoundingBox): RegionSample | null {
  const rgb = sampleRegionAverage(box);
  if (!rgb) return null;
  const hsv = rgbToHsv(rgb);
  return {
    box,
    rgb,
    hsv,
    hex: rgbToHex(rgb),
    deltaToTarget: rgbDistance(rgb, parseHexColor(targetColorHex.value)),
  };
}

function getPointerPosition(event: MouseEvent) {
  const rect = previewSurfaceRef.value?.getBoundingClientRect();
  if (!rect || !zoom.value) return null;
  const x = Math.max(0, Math.min(naturalSize.width, Math.round((event.clientX - rect.left) / zoom.value)));
  const y = Math.max(0, Math.min(naturalSize.height, Math.round((event.clientY - rect.top) / zoom.value)));
  return { x, y };
}

function setImgDetKind(value: string | number | null) {
  imgDetModel.value = createDetectorByKind(String(value ?? 'none') as DetectorKind, false);
}

function setTxtDetKind(value: string | number | null) {
  txtDetModel.value = createDetectorByKind(String(value ?? 'none') as DetectorKind, true);
}

function setTxtRecKind(value: string | number | null) {
  txtRecModel.value = createRecognizerByKind(String(value ?? 'none') as RecognizerKind);
}

async function loadImageDirectory(dirPath: string) {
  const paths = await visionLabService.listImageFiles(dirPath);
  folderItems.value = paths.map((path) => ({
    id: `folder:${path}`,
    kind: 'folder',
    name: normalizeItemName(path),
    path,
    previewUrl: null,
    stagedPath: null,
    savedPath: path,
    createdAt: new Date().toISOString(),
    saved: true,
  }));
}

async function reloadImageDirectory() {
  if (!preferences.imageDir) {
    return;
  }
  try {
    await loadImageDirectory(preferences.imageDir);
  } catch (error) {
    showToast(error instanceof Error ? error.message : '读取图像目录失败', 'error');
  }
}

async function pickImageDirectory() {
  const value = await open({ directory: true, multiple: false });
  if (typeof value !== 'string' || !value) {
    return;
  }
  preferences.imageDir = value;
  await persistPreferences();
  await reloadImageDirectory();
}

async function pickSaveDirectory() {
  const value = await open({ directory: true, multiple: false });
  if (typeof value !== 'string' || !value) {
    return;
  }
  preferences.saveDir = value;
  await persistPreferences();
  showToast('保存目录已更新', 'success');
}

async function selectItem(item: VisionSourceItem) {
  selectedItemId.value = item.id;
  detResults.value = [];
  ocrResults.value = [];
  pickedPoint.value = null;
  regionSample.value = null;
  regionDraft.value = null;
  if (item.previewUrl) {
    selectedPreviewUrl.value = item.previewUrl;
    return;
  }
  if (item.path) {
    selectedPreviewUrl.value = await scriptService.convertLocalImageToDataUrl(item.path);
    item.previewUrl = selectedPreviewUrl.value;
  } else if (item.savedPath) {
    selectedPreviewUrl.value = await scriptService.convertLocalImageToDataUrl(item.savedPath);
    item.previewUrl = selectedPreviewUrl.value;
  }
}

async function ensureSaveDirectory() {
  if (preferences.saveDir) {
    return preferences.saveDir;
  }
  const value = await open({ directory: true, multiple: false });
  if (typeof value !== 'string' || !value) {
    return null;
  }
  preferences.saveDir = value;
  await persistPreferences();
  return value;
}

async function saveCaptureItem(item: VisionSourceItem) {
  if (!item.stagedPath || item.saved) {
    return;
  }
  const saveDir = await ensureSaveDirectory();
  if (!saveDir) {
    showToast('未设置保存目录', 'warning');
    return;
  }
  savingCaptureId.value = item.id;
  try {
    const savedPath = await visionLabService.saveStagedImage(item.stagedPath, saveDir, item.name);
    item.saved = true;
    item.savedPath = savedPath;
    showToast('截图已保存到本地', 'success');
    if (preferences.imageDir && saveDir === preferences.imageDir) {
      await reloadImageDirectory();
    }
  } catch (error) {
    showToast(error instanceof Error ? error.message : '保存截图失败', 'error');
  } finally {
    savingCaptureId.value = null;
  }
}

async function captureFromDevice() {
  if (!selectedDevice.value) {
    showToast('请先选择设备', 'warning');
    return;
  }
  try {
    const capture = await visionLabService.captureDevice(selectedDevice.value);
    const suggestedName = `${selectedDevice.value.data.deviceName}_${new Date().toISOString().replace(/[:.]/g, '-')}.png`;
    const stagedPath = await visionLabService.stageCaptureImage(capture.imageData, suggestedName);
    const previewUrl = `data:image/png;base64,${capture.imageData}`;
    const item: VisionSourceItem = {
      id: `capture:${stagedPath}`,
      kind: 'capture',
      name: normalizeItemName(suggestedName),
      path: null,
      previewUrl,
      stagedPath,
      savedPath: null,
      createdAt: new Date().toISOString(),
      saved: false,
    };
    captureItems.value = [item, ...captureItems.value];
    await selectItem(item);
    showToast('设备截图已加入当前采集区', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '设备截图失败', 'error');
  }
}

async function runDetection() {
  if (!selectedImagePath.value || !imgDetModel.value) {
    return;
  }
  isRunningDet.value = true;
  try {
    detResults.value = await visionLabService.runDetection(clone(imgDetModel.value), selectedImagePath.value);
    showToast(`检测完成，共 ${detResults.value.length} 条结果`, 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '目标检测失败', 'error');
  } finally {
    isRunningDet.value = false;
  }
}

async function runOcr() {
  if (!selectedImagePath.value || !txtDetModel.value || !txtRecModel.value) {
    return;
  }
  isRunningOcr.value = true;
  try {
    const results = await visionLabService.runOcr(clone(txtDetModel.value), clone(txtRecModel.value), selectedImagePath.value);
    ocrResults.value = results.map((item) => ({
      ...item,
      ...analyzeOcrBoxColors(item.bounding_box),
    }));
    showToast(`OCR 完成，共 ${ocrResults.value.length} 条文本`, 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : 'OCR 失败', 'error');
  } finally {
    isRunningOcr.value = false;
  }
}

function fitPreview() {
  if (!previewContainerRef.value || !naturalSize.width || !naturalSize.height) {
    return;
  }
  const containerWidth = previewContainerRef.value.clientWidth - 48;
  const containerHeight = previewContainerRef.value.clientHeight - 48;
  const widthRatio = containerWidth / naturalSize.width;
  const heightRatio = containerHeight / naturalSize.height;
  zoom.value = Math.max(0.2, Math.min(1, widthRatio, heightRatio));
}

function zoomIn() {
  zoom.value = Math.min(5, Number((zoom.value + 0.1).toFixed(2)));
}

function zoomOut() {
  zoom.value = Math.max(0.2, Number((zoom.value - 0.1).toFixed(2)));
}

function handleImageLoaded() {
  const image = previewImageRef.value;
  const canvas = hiddenCanvasRef.value;
  if (!image || !canvas) return;
  naturalSize.width = image.naturalWidth;
  naturalSize.height = image.naturalHeight;
  canvas.width = image.naturalWidth;
  canvas.height = image.naturalHeight;
  const ctx = canvas.getContext('2d', { willReadFrequently: true });
  if (ctx) {
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    ctx.drawImage(image, 0, 0);
  }
  nextTick(() => {
    fitPreview();
    if (ocrResults.value.length) {
      ocrResults.value = ocrResults.value.map((item) => ({
        ...item,
        ...analyzeOcrBoxColors(item.bounding_box),
      }));
    }
  });
}

function handlePreviewClick(event: MouseEvent) {
  if (activeTool.value !== 'pick') {
    return;
  }
  const position = getPointerPosition(event);
  if (!position) return;
  pickedPoint.value = buildPointSample(position.x, position.y);
}

function handlePreviewPointerDown(event: MouseEvent) {
  if (activeTool.value !== 'region') {
    return;
  }
  const position = getPointerPosition(event);
  if (!position) return;
  regionStart.value = position;
  regionDraft.value = {
    x1: position.x,
    y1: position.y,
    x2: position.x,
    y2: position.y,
  };
}

function handlePreviewPointerMove(event: MouseEvent) {
  if (activeTool.value !== 'region' || !regionStart.value) {
    return;
  }
  const position = getPointerPosition(event);
  if (!position) return;
  regionDraft.value = {
    x1: Math.min(regionStart.value.x, position.x),
    y1: Math.min(regionStart.value.y, position.y),
    x2: Math.max(regionStart.value.x, position.x),
    y2: Math.max(regionStart.value.y, position.y),
  };
}

function handlePreviewPointerUp() {
  if (activeTool.value !== 'region' || !regionDraft.value) {
    regionStart.value = null;
    return;
  }
  regionSample.value = buildRegionSample(regionDraft.value);
  regionStart.value = null;
}

async function loadPreferences() {
  const saved = await getFromStore<VisionLabPreferences>(visionLabPreferencesKey);
  if (saved) {
    Object.assign(preferences, { ...DEFAULT_VISION_LAB_PREFERENCES, ...saved });
  }
}

async function applyLaunchPreset() {
  if (!isStandalone.value) {
    return;
  }
  const preset = await getFromStore<VisionLabLaunchPreset | null>(visionLabLaunchPresetKey);
  if (!preset) {
    return;
  }
  imgDetModel.value = ensureDetectorModel(preset.imgDetModel, false);
  txtDetModel.value = ensureDetectorModel(preset.txtDetModel, true);
  txtRecModel.value = ensureRecognizerModel(preset.txtRecModel);
  selectedDeviceId.value = preset.selectedDeviceId;
  await setToStore(visionLabLaunchPresetKey, null);
}

onMounted(async () => {
  await Promise.all([deviceStore.refreshAll(), loadPreferences()]);
  selectedDeviceId.value = selectedDeviceId.value || deviceStore.devices[0]?.id || null;
  await applyLaunchPreset();
  if (preferences.imageDir) {
    await reloadImageDirectory();
  }
});
</script>

<style scoped>
.vision-lab-shell {
  background:
    radial-gradient(circle at 12% 10%, rgba(14, 165, 233, 0.12), transparent 22%),
    radial-gradient(circle at 88% 12%, rgba(249, 115, 22, 0.1), transparent 20%),
    linear-gradient(180deg, rgba(255, 255, 255, 0.16), rgba(255, 255, 255, 0)),
    transparent;
}

.vision-lab-header {
  background:
    radial-gradient(circle at 10% 18%, rgba(255, 255, 255, 0.42), transparent 28%),
    linear-gradient(135deg, rgba(255, 255, 255, 0.58), rgba(245, 248, 255, 0.3)),
    var(--app-panel);
  box-shadow: var(--app-shadow-soft);
  backdrop-filter: blur(16px);
}

.vision-side-panel {
  box-shadow: var(--app-shadow-soft);
}

.vision-preview-frame {
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.4), 0 22px 50px rgba(15, 23, 42, 0.12);
}

.vision-list-item {
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.46), rgba(255, 255, 255, 0.18));
}
</style>
