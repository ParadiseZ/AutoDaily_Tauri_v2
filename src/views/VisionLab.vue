<template>
  <div class="vision-lab-shell h-[100svh] overflow-hidden px-4 py-4 lg:px-6 lg:py-5">
    <div class="mx-auto flex h-full max-w-[1900px] flex-col gap-4">
      <header class="vision-lab-header rounded-[28px] border border-[var(--app-border)] px-5 py-4 lg:px-6">
        <div class="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
          <div class="space-y-2">
            <div class="flex flex-wrap items-center gap-3">
              <span class="rounded-full border border-[var(--app-border)] bg-white/55 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.24em] text-[var(--app-text-faint)]">
                Vision Lab
              </span>
            </div>
            <div class="space-y-1">
              <h1 class="text-2xl font-semibold tracking-[-0.05em] text-[var(--app-text-strong)] lg:text-3xl">视觉测试工作台</h1>
              <p class="text-sm text-[var(--app-text-soft)]">{{ selectedItem?.name || '未选择图像' }}</p>
            </div>
          </div>

          <div class="flex flex-wrap items-center gap-2">
            <button class="app-button app-button-ghost" type="button" data-testid="vision-lab-open-devtools" @click="openCurrentDevtools">
              <AppIcon name="bug" :size="16" />
              开发者工具
            </button>
            <button class="app-button app-button-ghost" type="button" data-testid="vision-lab-reload-page" @click="reloadCurrentPage">
              <AppIcon name="refresh-cw" :size="16" />
              刷新页面
            </button>
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

      <div class="grid min-h-0 flex-1 gap-4 xl:grid-cols-[300px_minmax(0,1fr)_minmax(460px,540px)] 2xl:grid-cols-[320px_minmax(0,1fr)_minmax(500px,580px)]">
        <aside class="vision-side-panel min-h-0 overflow-hidden rounded-[26px] border border-[var(--app-border)] bg-[var(--app-panel)]">
          <div class="flex h-full flex-col">
            <div class="border-b border-[var(--app-border)] px-4 py-4">
              <div class="space-y-3">
                <div>
                  <p class="text-xs uppercase tracking-[0.2em] text-[var(--app-text-faint)]">数据源</p>
                  <p class="mt-1 text-sm text-[var(--app-text-soft)]">{{ filteredFolderItems.length }} 张目录图像 · {{ captureItems.length }} 张当前采集</p>
                </div>

                <div class="space-y-2">
                  <div class="flex items-center justify-between gap-3">
                    <span class="text-xs font-semibold text-[var(--app-text-faint)]">设备</span>
                    <span class="text-[11px] text-[var(--app-text-faint)]">{{ deviceOptions.length }} 台</span>
                  </div>
                  <div class="min-w-0">
                    <AppSelect
                      v-model="selectedDeviceId"
                      :options="deviceOptions"
                      placeholder="请选择设备"
                      test-id="vision-lab-device"
                    />
                  </div>
                  <p v-if="!deviceOptions.length" class="text-xs text-[var(--app-text-faint)]">当前没有可用设备，请先在设备列表中配置。</p>
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

                <div class="space-y-2 rounded-[18px] border border-[var(--app-border)] bg-[var(--app-panel-muted)]/70 px-3 py-3 text-xs text-[var(--app-text-faint)]">
                  <div class="flex items-start justify-between gap-3">
                    <span>图片目录</span>
                    <span class="truncate text-right">{{ preferences.imageDir || '未选择' }}</span>
                  </div>
                  <div class="flex items-start justify-between gap-3">
                    <span>保存目录</span>
                    <span class="truncate text-right">{{ preferences.saveDir || '未设置' }}</span>
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
                  <button class="app-button app-button-ghost px-3 py-2 text-xs" type="button" :disabled="!preferences.imageDir" @click="reloadImageDirectory">
                    刷新
                  </button>
                </div>

                <div v-if="!filteredFolderItems.length" class="rounded-[22px] border border-dashed border-[var(--app-border)] px-4 py-6 text-sm text-[var(--app-text-soft)]">
                  先选择一个图像目录。
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
                  当前还没有设备截图。
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
              <div class="flex flex-col gap-4">
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
                    <button class="app-button app-button-ghost px-3 py-2 text-xs" type="button" @click="resetZoom">原始大小</button>
                    <button class="app-button app-button-ghost px-3 py-2 text-xs" type="button" @click="zoomOut">-</button>
                    <span class="rounded-full border border-[var(--app-border)] px-3 py-2 text-xs text-[var(--app-text-soft)]">{{ Math.round(zoom * 100) }}%</span>
                    <button class="app-button app-button-ghost px-3 py-2 text-xs" type="button" @click="zoomIn">+</button>
                  </div>
                </div>

                <div class="grid gap-3 lg:grid-cols-[minmax(0,1fr)_220px]">
                  <div class="grid gap-3 sm:grid-cols-2">
                    <div class="rounded-[18px] border border-[var(--app-border)] px-3 py-3 text-xs text-[var(--app-text-soft)]">
                      <p class="font-semibold text-[var(--app-text-strong)]">点取色</p>
                      <template v-if="pickedPoint">
                        <p class="mt-2">坐标：({{ pickedPoint.x }}, {{ pickedPoint.y }})</p>
                        <p>HEX：{{ pickedPoint.hex }}</p>
                        <p>RGB：{{ pickedPoint.rgb.r }}, {{ pickedPoint.rgb.g }}, {{ pickedPoint.rgb.b }}</p>
                      </template>
                      <p v-else class="mt-2">切到“点取色”后点击图像。</p>
                    </div>
                    <div class="rounded-[18px] border border-[var(--app-border)] px-3 py-3 text-xs text-[var(--app-text-soft)]">
                      <p class="font-semibold text-[var(--app-text-strong)]">区域采样</p>
                      <template v-if="regionSample">
                        <p class="mt-2">区域：{{ formatBox(regionSample.box) }}</p>
                        <p>HEX：{{ regionSample.hex }}</p>
                        <p>RGB：{{ regionSample.rgb.r }}, {{ regionSample.rgb.g }}, {{ regionSample.rgb.b }}</p>
                      </template>
                      <p v-else class="mt-2">切到“区域采样”后拖拽选框。</p>
                    </div>
                  </div>
                  <div class="rounded-[18px] border border-[var(--app-border)] px-3 py-3 text-xs text-[var(--app-text-soft)]">
                    <p class="font-semibold text-[var(--app-text-strong)]">目标颜色</p>
                    <input v-model="targetColorHex" class="mt-2 h-11 w-full rounded-[14px] border border-[var(--app-border)] bg-transparent px-3" type="color" />
                    <p class="mt-2">用于点取色和区域采样的差异对比。</p>
                  </div>
                </div>
              </div>
            </div>

            <div ref="previewContainerRef" class="vision-preview-scroll min-h-0 flex-1 overflow-auto px-4 py-4" @wheel="handlePreviewWheel">
              <div
                v-if="selectedPreviewUrl"
                class="vision-preview-frame mx-auto w-max rounded-[28px] border border-[var(--app-border)] bg-[var(--app-panel-muted)] p-4"
              >
                <div
                  ref="previewSurfaceRef"
                  class="relative overflow-hidden rounded-[20px] bg-[#111827]"
                  :class="previewSurfaceClass"
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
                      :class="entry.kind === 'ocr' ? 'border-cyan-300/90 bg-cyan-500/15 text-cyan-50' : entry.kind === 'textDet' ? 'border-fuchsia-300/90 bg-fuchsia-500/16 text-fuchsia-50' : 'border-amber-300/90 bg-amber-500/18 text-amber-50'"
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
              <div class="space-y-3">
                  <div class="flex items-center justify-between">
                    <div>
                      <p class="text-sm font-semibold text-[var(--app-text-strong)]">分析控制台</p>
                    <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ activeTabLabel }}</p>
                    </div>
                  </div>

                <div class="overflow-x-auto">
                  <div class="editor-panel-tabs min-w-max">
                  <button
                    v-for="tab in tabOptions"
                    :key="tab.value"
                    class="editor-panel-tab"
                    :class="{ 'editor-panel-tab-active': activeTab === tab.value }"
                    type="button"
                    @click="activeTab = tab.value"
                  >
                    {{ tab.label }}
                  </button>
                  </div>
                </div>
              </div>
            </div>

            <div class="min-h-0 flex-1 overflow-y-auto px-4 py-4">
              <section v-if="activeTab === 'det'" class="space-y-5">
                <section class="rounded-[20px] border border-[var(--app-border)] bg-[var(--app-panel-muted)]/70">
                  <button class="flex w-full items-center justify-between px-4 py-3 text-left" type="button" @click="panelOpen.detConfig = !panelOpen.detConfig">
                    <span class="text-sm font-semibold text-[var(--app-text-strong)]">检测模型配置</span>
                    <span class="text-xs text-[var(--app-text-faint)]">{{ panelOpen.detConfig ? '收起' : '展开' }}</span>
                  </button>
                  <div v-if="panelOpen.detConfig" class="space-y-3 border-t border-[var(--app-border)] px-4 py-4">
                    <label class="space-y-2">
                      <span class="text-xs font-semibold text-[var(--app-text-faint)]">模型类型</span>
                      <AppSelect :model-value="imgDetKind" :options="imgDetectorOptions" placeholder="选择模型" test-id="vision-lab-img-det-kind" @update:model-value="setImgDetKind" />
                    </label>
                    <template v-if="imgDetYolo">
                      <ModelBaseFields :model="imgDetYolo.baseModel" :built-in-enabled="false" compact path-placeholder="例如：D:\\models\\img-det.onnx" test-id-prefix="vision-lab-img-det-base" />
                      <div class="grid gap-3 sm:grid-cols-2">
                        <label class="space-y-2">
                          <span class="text-xs font-semibold text-[var(--app-text-faint)]">类别数量</span>
                          <input v-model.number="imgDetYolo.classCount" class="app-input" min="1" type="number" />
                        </label>
                        <label class="space-y-2">
                          <span class="text-xs font-semibold text-[var(--app-text-faint)]">标签路径</span>
                          <div class="vision-path-row">
                            <input v-model.trim="imgDetYolo.labelPath" class="app-input" placeholder="D:\\models\\labels.yaml" />
                            <button class="app-button app-button-ghost vision-path-button" type="button" @click="pickImgDetLabelPath">
                              <AppIcon name="folder-open" :size="16" />
                            </button>
                          </div>
                        </label>
                      </div>
                      <div class="grid gap-3 sm:grid-cols-2">
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
                    <button class="app-button app-button-primary w-full justify-center" type="button" :disabled="!canRunDetection || isRunningDet" @click="runDetection">
                      {{ isRunningDet ? '检测中...' : '目标检测' }}
                    </button>
                  </div>
                </section>

                <section class="rounded-[20px] border border-[var(--app-border)] bg-[var(--app-panel-muted)]/70">
                  <button class="flex w-full items-center justify-between px-4 py-3 text-left" type="button" @click="panelOpen.detResults = !panelOpen.detResults">
                    <span class="text-sm font-semibold text-[var(--app-text-strong)]">检测结果集</span>
                    <span class="text-xs text-[var(--app-text-faint)]">{{ panelOpen.detResults ? '收起' : '展开' }}</span>
                  </button>
                  <div v-if="panelOpen.detResults" class="space-y-3 border-t border-[var(--app-border)] px-4 py-4">
                    <div class="grid gap-3 sm:grid-cols-[minmax(0,1fr)_160px]">
                      <label class="space-y-2">
                        <span class="text-xs font-semibold text-[var(--app-text-faint)]">名称筛选</span>
                        <input v-model.trim="detSearchText" class="app-input" placeholder="标签或名称" />
                      </label>
                      <label class="space-y-2">
                        <span class="text-xs font-semibold text-[var(--app-text-faint)]">标签筛选</span>
                        <AppSelect v-model="detLabelFilter" :options="detLabelFilterOptions" placeholder="全部" test-id="vision-lab-det-label-filter" />
                      </label>
                    </div>

                    <div v-if="!filteredDetResults.length" class="rounded-[18px] border border-dashed border-[var(--app-border)] px-3 py-4 text-xs text-[var(--app-text-soft)]">
                      运行目标检测后，这里展示结果。
                    </div>
                    <div v-for="(item, index) in filteredDetResults" :key="`det-${index}`" class="rounded-[18px] border border-[var(--app-border)] px-3 py-3">
                      <div class="flex items-start justify-between gap-3">
                        <div class="min-w-0">
                          <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ item.index }}: {{ item.label }}</p>
                          <p class="mt-1 text-xs text-[var(--app-text-faint)]">类目 #{{ item.index }} · {{ formatBox(item.bounding_box) }}</p>
                        </div>
                        <span class="rounded-full bg-amber-500/12 px-2 py-1 text-[11px] font-medium text-amber-700">{{ item.score.toFixed(3) }}</span>
                      </div>
                    </div>
                  </div>
                </section>
              </section>

              <section v-else-if="activeTab === 'ocr'" class="space-y-5">
                <section class="rounded-[20px] border border-[var(--app-border)] bg-[var(--app-panel-muted)]/70">
                  <button class="flex w-full items-center justify-between px-4 py-3 text-left" type="button" @click="panelOpen.ocrConfig = !panelOpen.ocrConfig">
                    <span class="text-sm font-semibold text-[var(--app-text-strong)]">OCR 模型配置</span>
                    <span class="text-xs text-[var(--app-text-faint)]">{{ panelOpen.ocrConfig ? '收起' : '展开' }}</span>
                  </button>
                  <div v-if="panelOpen.ocrConfig" class="space-y-4 border-t border-[var(--app-border)] px-4 py-4">
                    <div class="space-y-3 rounded-[18px] border border-[var(--app-border)] px-3 py-3">
                      <p class="text-sm font-semibold text-[var(--app-text-strong)]">文字检测</p>
                      <label class="space-y-2">
                        <span class="text-xs font-semibold text-[var(--app-text-faint)]">模型类型</span>
                        <AppSelect :model-value="txtDetKind" :options="txtDetectorOptions" placeholder="选择模型" test-id="vision-lab-txt-det-kind" @update:model-value="setTxtDetKind" />
                      </label>
                      <template v-if="txtDetYolo">
                        <ModelBaseFields :model="txtDetYolo.baseModel" :built-in-enabled="false" compact path-placeholder="例如：D:\\models\\txt-det.onnx" test-id-prefix="vision-lab-txt-det-base" />
                        <div class="grid gap-3 sm:grid-cols-2">
                          <label class="space-y-2">
                            <span class="text-xs font-semibold text-[var(--app-text-faint)]">类别数量</span>
                            <input v-model.number="txtDetYolo.classCount" class="app-input" min="1" type="number" />
                          </label>
                          <label class="space-y-2">
                            <span class="text-xs font-semibold text-[var(--app-text-faint)]">标签路径</span>
                            <div class="vision-path-row">
                              <input v-model.trim="txtDetYolo.labelPath" class="app-input" placeholder="D:\\models\\labels.yaml" />
                              <button class="app-button app-button-ghost vision-path-button" type="button" @click="pickTxtDetLabelPath">
                                <AppIcon name="folder-open" :size="16" />
                              </button>
                            </div>
                          </label>
                        </div>
                        <div class="grid gap-3 sm:grid-cols-2">
                          <label class="space-y-2">
                            <span class="text-xs font-semibold text-[var(--app-text-faint)]">文本索引</span>
                            <AppSelect
                              v-if="txtDetLabelOptions.length"
                              v-model="txtDetLabelSelectValue"
                              :options="txtDetLabelOptions"
                              placeholder="选择标签"
                              test-id="vision-lab-txt-det-label-idx"
                            />
                            <input v-else v-model.number="txtDetYolo.txtIdx" class="app-input" min="0" type="number" placeholder="未加载标签时手动输入 idx" />
                          </label>
                          <label class="space-y-2">
                            <span class="text-xs font-semibold text-[var(--app-text-faint)]">置信度</span>
                            <input v-model.number="txtDetYolo.confidenceThresh" class="app-input" min="0" max="1" step="0.01" type="number" />
                          </label>
                        </div>
                        <div class="grid gap-3 sm:grid-cols-2">
                          <label class="space-y-2">
                            <span class="text-xs font-semibold text-[var(--app-text-faint)]">IOU</span>
                            <input v-model.number="txtDetYolo.iouThresh" class="app-input" min="0" max="1" step="0.01" type="number" />
                          </label>
                        </div>
                        <p v-if="txtDetLabelHint" class="text-xs text-[var(--app-text-faint)]">{{ txtDetLabelHint }}</p>
                      </template>
                      <template v-else-if="txtDetDbNet">
                        <ModelBaseFields :model="txtDetDbNet.baseModel" :built-in-enabled="false" compact path-placeholder="例如：D:\\models\\ocr-dbnet.onnx" test-id-prefix="vision-lab-txt-det-base" />
                      </template>
                    </div>

                    <div class="space-y-3 rounded-[18px] border border-[var(--app-border)] px-3 py-3">
                      <p class="text-sm font-semibold text-[var(--app-text-strong)]">文字识别</p>
                      <label class="space-y-2">
                        <span class="text-xs font-semibold text-[var(--app-text-faint)]">模型类型</span>
                        <AppSelect :model-value="txtRecKind" :options="recognizerOptions" placeholder="选择模型" test-id="vision-lab-txt-rec-kind" @update:model-value="setTxtRecKind" />
                      </label>
                      <template v-if="txtRecCrnn">
                        <ModelBaseFields :model="txtRecCrnn.baseModel" compact path-placeholder="例如：D:\\models\\ocr-rec.onnx" test-id-prefix="vision-lab-txt-rec-base" />
                        <label class="space-y-2">
                          <span class="text-xs font-semibold text-[var(--app-text-faint)]">字典路径</span>
                          <div class="vision-path-row">
                            <input v-model.trim="txtRecCrnn.dictPath" class="app-input" placeholder="D:\\models\\keys.txt" />
                            <button class="app-button app-button-ghost vision-path-button" type="button" @click="pickTxtRecDictPath">
                              <AppIcon name="folder-open" :size="16" />
                            </button>
                          </div>
                        </label>
                        <p class="text-xs text-[var(--app-text-faint)]">切换为内置后会保留内置模型来源配置；如需覆盖字典，继续填写自定义字典路径即可。</p>
                      </template>
                    </div>

                    <div class="grid gap-2 sm:grid-cols-2">
                      <button class="app-button app-button-ghost justify-center" type="button" :disabled="!canRunTextDetection || isRunningTextDet" @click="runTextDetection">
                        {{ isRunningTextDet ? '检测中...' : '文字检测' }}
                      </button>
                      <button class="app-button app-button-primary justify-center" type="button" :disabled="!canRunOcr || isRunningOcr" @click="runOcr">
                        {{ isRunningOcr ? 'OCR 中...' : '完整OCR' }}
                      </button>
                    </div>
                  </div>
                </section>

                <section class="rounded-[20px] border border-[var(--app-border)] bg-[var(--app-panel-muted)]/70">
                  <button class="flex w-full items-center justify-between px-4 py-3 text-left" type="button" @click="panelOpen.ocrResults = !panelOpen.ocrResults">
                    <span class="text-sm font-semibold text-[var(--app-text-strong)]">OCR 结果集</span>
                    <span class="text-xs text-[var(--app-text-faint)]">{{ panelOpen.ocrResults ? '收起' : '展开' }}</span>
                  </button>
                  <div v-if="panelOpen.ocrResults" class="space-y-4 border-t border-[var(--app-border)] px-4 py-4">
                    <label class="space-y-2">
                      <span class="text-xs font-semibold text-[var(--app-text-faint)]">文本筛选</span>
                      <input v-model.trim="ocrFilterText" class="app-input" placeholder="包含关键字" />
                    </label>

                    <div class="space-y-3">
                      <div class="flex items-center justify-between">
                        <p class="text-xs font-semibold uppercase tracking-[0.16em] text-[var(--app-text-faint)]">文字检测结果</p>
                        <span class="text-xs text-[var(--app-text-faint)]">{{ textDetResults.length }}</span>
                      </div>
                      <div v-if="!textDetResults.length" class="rounded-[18px] border border-dashed border-[var(--app-border)] px-3 py-4 text-xs text-[var(--app-text-soft)]">
                        运行文字检测后，这里显示文本框。
                      </div>
                      <div v-for="(item, index) in textDetResults" :key="`text-det-${index}`" class="rounded-[18px] border border-[var(--app-border)] px-3 py-3">
                        <p class="text-xs text-[var(--app-text-faint)]">文本框 {{ index + 1 }} · {{ formatBox(item.bounding_box) }}</p>
                        <p class="mt-1 text-sm font-semibold text-[var(--app-text-strong)]">{{ item.index }}: {{ item.label }}</p>
                      </div>
                    </div>

                    <div class="space-y-3">
                      <div class="flex items-center justify-between">
                        <p class="text-xs font-semibold uppercase tracking-[0.16em] text-[var(--app-text-faint)]">OCR 结果</p>
                        <span class="text-xs text-[var(--app-text-faint)]">{{ filteredOcrResults.length }}/{{ ocrResults.length }}</span>
                      </div>
                      <div v-if="!filteredOcrResults.length" class="rounded-[18px] border border-dashed border-[var(--app-border)] px-3 py-4 text-xs text-[var(--app-text-soft)]">
                        运行 OCR 后，这里展示文本识别结果。
                      </div>
                      <div v-for="(item, index) in filteredOcrResults" :key="`ocr-${index}`" class="rounded-[18px] border border-[var(--app-border)] px-3 py-3">
                        <div class="flex items-start justify-between gap-3">
                          <div class="min-w-0">
                            <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ item.txt || '(空文本)' }}</p>
                            <p class="mt-1 text-xs text-[var(--app-text-faint)]">框：{{ formatBox(item.bounding_box) }}</p>
                          </div>
                          <span class="rounded-full bg-cyan-500/12 px-2 py-1 text-[11px] font-medium text-cyan-700">{{ averageScore(item).toFixed(3) }}</span>
                        </div>
                        <div class="mt-2 grid gap-2 text-[11px] text-[var(--app-text-soft)] sm:grid-cols-2">
                          <div class="rounded-[12px] border border-[var(--app-border)] px-2 py-2">
                            <span class="mr-2">背景色</span>
                            <span class="inline-flex h-3 w-3 rounded-full align-middle" :style="{ backgroundColor: item.bgColorHex }" />
                            <span class="ml-2">{{ item.bgColorHex }}</span>
                          </div>
                          <div class="rounded-[12px] border border-[var(--app-border)] px-2 py-2">
                            <span class="mr-2">文字色</span>
                            <span class="inline-flex h-3 w-3 rounded-full align-middle" :style="{ backgroundColor: item.fgColorHex }" />
                            <span class="ml-2">{{ item.fgColorHex }}</span>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </section>
              </section>

              <section v-else class="space-y-5">
                <section class="rounded-[20px] border border-[var(--app-border)] bg-[var(--app-panel-muted)]/70 px-4 py-4">
                  <div class="space-y-3">
                    <div>
                      <p class="text-sm font-semibold text-[var(--app-text-strong)]">组合分析</p>
                      <p class="mt-1 text-xs text-[var(--app-text-faint)]">同时运行目标检测和 OCR，并为结果区域提取真实颜色值。</p>
                    </div>
                    <button class="app-button app-button-primary w-full justify-center" type="button" :disabled="!canRunCombo || isRunningCombo" @click="runComboAnalysis">
                      {{ isRunningCombo ? '分析中...' : '视觉分析' }}
                    </button>
                  </div>
                </section>

                <section class="rounded-[20px] border border-[var(--app-border)] bg-[var(--app-panel-muted)]/70">
                  <button class="flex w-full items-center justify-between px-4 py-3 text-left" type="button" @click="panelOpen.comboResults = !panelOpen.comboResults">
                    <span class="text-sm font-semibold text-[var(--app-text-strong)]">视觉结果集</span>
                    <span class="text-xs text-[var(--app-text-faint)]">{{ panelOpen.comboResults ? '收起' : '展开' }}</span>
                  </button>
                  <div v-if="panelOpen.comboResults" class="space-y-3 border-t border-[var(--app-border)] px-4 py-4">
                    <div v-if="!comboResults.length" class="rounded-[18px] border border-dashed border-[var(--app-border)] px-3 py-4 text-xs text-[var(--app-text-soft)]">
                      点击“视觉分析”后，这里展示目标检测区和 OCR 区的颜色结果。
                    </div>
                    <div v-for="item in comboResults" :key="item.key" class="rounded-[18px] border border-[var(--app-border)] px-3 py-3">
                      <div class="flex items-start justify-between gap-3">
                        <div class="min-w-0">
                          <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">
                            {{ item.kind === 'ocr' ? item.text || '(空文本)' : item.label || '检测框' }}
                          </p>
                          <p class="mt-1 text-xs text-[var(--app-text-faint)]">{{ item.kind === 'ocr' ? 'OCR' : 'Det' }} · {{ formatBox(item.box) }}</p>
                        </div>
                        <span class="rounded-full px-2 py-1 text-[11px] font-medium" :class="item.kind === 'ocr' ? 'bg-cyan-500/12 text-cyan-700' : 'bg-amber-500/12 text-amber-700'">
                          {{ item.score.toFixed(3) }}
                        </span>
                      </div>
                      <div class="mt-2 grid gap-2 text-[11px] text-[var(--app-text-soft)] sm:grid-cols-2">
                        <div class="rounded-[12px] border border-[var(--app-border)] px-2 py-2">
                          <span class="mr-2">背景色</span>
                          <span class="inline-flex h-3 w-3 rounded-full align-middle" :style="{ backgroundColor: item.bgColorHex }" />
                          <span class="ml-2">{{ item.bgColorHex }}</span>
                        </div>
                        <div class="rounded-[12px] border border-[var(--app-border)] px-2 py-2">
                          <template v-if="item.fgColorHex">
                            <span class="mr-2">文字色</span>
                            <span class="inline-flex h-3 w-3 rounded-full align-middle" :style="{ backgroundColor: item.fgColorHex }" />
                            <span class="ml-2">{{ item.fgColorHex }}</span>
                          </template>
                          <template v-else>
                            <span>文字色</span>
                            <span class="ml-2 text-[var(--app-text-faint)]">目标检测区域不提取</span>
                          </template>
                        </div>
                      </div>
                    </div>
                  </div>
                </section>
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
import { computed, nextTick, onMounted, reactive, ref, watch } from 'vue';
import { confirm, open } from '@tauri-apps/plugin-dialog';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import ModelBaseFields from '@/views/script-list/script-info/ModelBaseFields.vue';
import { deviceService } from '@/services/deviceService';
import { openCurrentDevtools, reloadCurrentPage } from '@/services/devtoolsService';
import { scriptService } from '@/services/scriptService';
import { visionLabConfigService } from '@/services/visionLabConfigService';
import { visionLabService } from '@/services/visionLabService';
import { useDeviceStore } from '@/store/device';
import {
  deviceKey,
  getFromStore,
  setToStore,
  visionLabActiveTabKey,
  visionLabLaunchPresetKey,
  visionLabPreferencesKey,
} from '@/store/store';
import { showToast } from '@/utils/toast';
import {
  createDetectorByKind,
  createRecognizerByKind,
  extractCrnn,
  extractDbNet,
  extractYoloDetector,
  resolveDetectorKind,
  resolveRecognizerKind,
  type DetectorKind,
  type RecognizerKind,
} from '@/utils/visionModelPresets';
import {
  DEFAULT_VISION_LAB_PREFERENCES,
  type VisionLabLaunchPreset,
  type VisionLabModelConfig,
  type VisionLabPreferences,
} from '@/types/app/domain';
import type { BoundingBox } from '@/types/bindings/BoundingBox';
import type { DetResult } from '@/types/bindings/DetResult';
import type { DetectorType } from '@/types/bindings/DetectorType';
import type { OcrResult } from '@/types/bindings/OcrResult';
import type { RecognizerType } from '@/types/bindings/RecognizerType';
import type { DeviceTable } from '@/types/bindings/DeviceTable';

type VisionItemKind = 'folder' | 'capture';
type PreviewTool = 'browse' | 'pick' | 'region';
type VisionTab = 'det' | 'ocr' | 'combo';
type VisionImgDetectorKind = Exclude<DetectorKind, 'PaddleDbNet'>;

interface VisionSourceItem {
  id: string;
  kind: VisionItemKind;
  name: string;
  path: string | null;
  previewUrl: string | null;
  imageData: string | null;
  savedPath: string | null;
  createdAt: string;
  saved: boolean;
}

interface RgbColor {
  r: number;
  g: number;
  b: number;
}

interface PointSample {
  x: number;
  y: number;
  rgb: RgbColor;
  hex: string;
  deltaToTarget: number;
}

interface RegionSample {
  box: BoundingBox;
  rgb: RgbColor;
  hex: string;
  deltaToTarget: number;
}

interface VisionOcrResult extends OcrResult {
  bgColorHex: string;
  fgColorHex: string;
}

interface ComboResultItem {
  key: string;
  kind: 'det' | 'ocr';
  label?: string;
  text?: string;
  box: BoundingBox;
  score: number;
  bgColorHex: string;
  fgColorHex?: string | null;
}

interface OverlayEntry {
  key: string;
  kind: 'det' | 'textDet' | 'ocr';
  label: string;
  box: BoundingBox;
}

const deviceStore = useDeviceStore();

const preferences = reactive<VisionLabPreferences>({ ...DEFAULT_VISION_LAB_PREFERENCES });
const folderItems = ref<VisionSourceItem[]>([]);
const captureItems = ref<VisionSourceItem[]>([]);
const selectedItemId = ref<string | null>(null);
const selectedPreviewUrl = ref<string | null>(null);
const selectedDeviceId = ref<string | null>(null);
const activeTab = ref<VisionTab>('det');

const imgDetModel = ref<DetectorType | null>(null);
const txtDetModel = ref<DetectorType | null>(null);
const txtRecModel = ref<RecognizerType | null>(null);

const isRunningDet = ref(false);
const isRunningTextDet = ref(false);
const isRunningOcr = ref(false);
const isRunningCombo = ref(false);
const savingCaptureId = ref<string | null>(null);
const hydratingModelConfig = ref(true);

const detResults = ref<DetResult[]>([]);
const textDetResults = ref<DetResult[]>([]);
const ocrResults = ref<VisionOcrResult[]>([]);
const comboResults = ref<ComboResultItem[]>([]);

const detSearchText = ref('');
const detLabelFilter = ref<string>('ALL');
const ocrFilterText = ref('');
const imgDetLabelOptions = ref<Array<{ label: string; value: string; description?: string }>>([]);
const txtDetLabelOptions = ref<Array<{ label: string; value: string; description?: string }>>([]);
const txtDetLabelHint = ref<string | null>('请先设置文字检测标签路径，或选择“不过滤”验证原始检测输出。');

const activeTool = ref<PreviewTool>('browse');
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

const panState = reactive({
  active: false,
  startClientX: 0,
  startClientY: 0,
  startScrollLeft: 0,
  startScrollTop: 0,
});

const panelOpen = reactive({
  detConfig: true,
  detResults: true,
  ocrConfig: true,
  ocrResults: true,
  comboResults: true,
});

const tabOptions = [
  { label: '目标检测', value: 'det' as const },
  { label: 'OCR', value: 'ocr' as const },
  { label: '组合', value: 'combo' as const },
];
const imgDetectorOptions = [
  { label: '不设置', value: 'none', description: '当前字段留空。' },
  { label: 'YOLO11', value: 'Yolo11', description: '通用目标检测方案。' },
  { label: 'YOLO26', value: 'Yolo26', description: '端到端检测方案。' },
];
const txtDetectorOptions = [
  { label: '不设置', value: 'none', description: '当前字段留空。' },
  { label: 'YOLO11', value: 'Yolo11', description: '适合文本框或字符框检测。' },
  { label: 'Paddle DBNet', value: 'PaddleDbNet', description: '适合文本区域检测。' },
  { label: 'YOLO26', value: 'Yolo26', description: '端到端检测方案。' },
];
const recognizerOptions = [
  { label: '不设置', value: 'none', description: '不启用识别模型。' },
  { label: 'Paddle CRNN', value: 'PaddleCrnn', description: '当前绑定里可用的文本识别模型。' },
];
const previewTools = [
  { label: '浏览', value: 'browse' as const },
  { label: '点取色', value: 'pick' as const },
  { label: '区域采样', value: 'region' as const },
];

const selectedItem = computed(() => [...captureItems.value, ...folderItems.value].find((item) => item.id === selectedItemId.value) ?? null);
const selectedDevice = computed<DeviceTable | null>(() => deviceStore.devices.find((device) => device.id === selectedDeviceId.value) ?? null);
const selectedImagePath = computed(() => selectedItem.value?.path ?? null);
const selectedImageData = computed(() => selectedItem.value?.imageData ?? null);
const canRunVision = computed(() => Boolean(selectedImagePath.value || selectedImageData.value));
const canRunDetection = computed(() => canRunVision.value && Boolean(imgDetModel.value));
const canRunTextDetection = computed(() => canRunVision.value && Boolean(txtDetModel.value));
const canRunOcr = computed(() => canRunVision.value && Boolean(txtDetModel.value) && Boolean(txtRecModel.value));
const canRunCombo = computed(() => canRunDetection.value && canRunOcr.value);
const activeTabLabel = computed(() => tabOptions.find((tab) => tab.value === activeTab.value)?.label ?? '分析');

const imgDetKind = computed<VisionImgDetectorKind>(() => {
  const kind = resolveDetectorKind(imgDetModel.value);
  return kind === 'PaddleDbNet' ? 'none' : kind;
});
const txtDetKind = computed<DetectorKind>(() => resolveDetectorKind(txtDetModel.value));
const txtRecKind = computed<RecognizerKind>(() => resolveRecognizerKind(txtRecModel.value));
const imgDetYolo = computed(() => extractYoloDetector(imgDetModel.value));
const txtDetYolo = computed(() => extractYoloDetector(txtDetModel.value));
const txtDetDbNet = computed(() => extractDbNet(txtDetModel.value));
const txtRecCrnn = computed(() => extractCrnn(txtRecModel.value));

const filteredFolderItems = computed(() => {
  const keyword = preferences.filterText.trim().toLowerCase();
  if (!keyword) return folderItems.value;
  return folderItems.value.filter((item) => item.name.toLowerCase().includes(keyword));
});

const filteredDetResults = computed(() => {
  const keyword = detSearchText.value.trim().toLowerCase();
  return detResults.value.filter((item) => {
    if (detLabelFilter.value !== 'ALL' && String(item.index) !== detLabelFilter.value) {
      return false;
    }
    if (keyword && !item.label.toLowerCase().includes(keyword) && !String(item.index).includes(keyword)) {
      return false;
    }
    return true;
  });
});

const filteredOcrResults = computed(() => {
  const keyword = ocrFilterText.value.trim();
  if (!keyword) return ocrResults.value;
  return ocrResults.value.filter((item) => item.txt.includes(keyword));
});

const overlayEntries = computed<OverlayEntry[]>(() => {
  const entries: OverlayEntry[] = [];
  detResults.value.forEach((item, index) => {
    entries.push({ key: `det-${index}`, kind: 'det', label: item.label, box: item.bounding_box });
  });
  textDetResults.value.forEach((item, index) => {
    entries.push({ key: `text-det-${index}`, kind: 'textDet', label: item.label || `文本框 ${index + 1}`, box: item.bounding_box });
  });
  ocrResults.value.forEach((item, index) => {
    entries.push({ key: `ocr-${index}`, kind: 'ocr', label: item.txt || '(空)', box: item.bounding_box });
  });
  return entries;
});

const detLabelFilterOptions = computed(() => {
  const labels = imgDetLabelOptions.value.length
    ? imgDetLabelOptions.value
    : Array.from(new Map(detResults.value.map((item) => [String(item.index), item.label])).entries()).map(([index, label]) => ({
        label: `${index}: ${label}`,
        value: index,
      }));
  return [{ label: '全部', value: 'ALL' }, ...labels];
});

const txtDetLabelSelectValue = computed<string>({
  get: () => {
    const value = txtDetYolo.value?.txtIdx;
    return value == null ? '__ALL__' : String(value);
  },
  set: (value) => {
    if (!txtDetYolo.value) return;
    txtDetYolo.value.txtIdx = value === '__ALL__' ? null : Number(value);
  },
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
}));

const previewSurfaceClass = computed(() => {
  if (activeTool.value === 'browse') {
    return panState.active ? 'cursor-grabbing' : 'cursor-grab';
  }
  return 'cursor-crosshair';
});

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value)) as T;
}

function normalizeItemName(path: string) {
  return path.split(/[\\/]/).pop() || path;
}

function averageScore(item: OcrResult) {
  if (!item.score.length) return 0;
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

function rgbToHex(color: RgbColor) {
  return `#${[color.r, color.g, color.b].map((value) => value.toString(16).padStart(2, '0')).join('')}`;
}

function parseHexColor(value: string): RgbColor {
  const normalized = value.replace('#', '');
  const raw = normalized.length === 3 ? normalized.split('').map((char) => `${char}${char}`).join('') : normalized.padEnd(6, '0').slice(0, 6);
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

function getBoxStyle(box: BoundingBox) {
  return {
    left: `${box.x1 * zoom.value}px`,
    top: `${box.y1 * zoom.value}px`,
    width: `${Math.max(1, (box.x2 - box.x1) * zoom.value)}px`,
    height: `${Math.max(1, (box.y2 - box.y1) * zoom.value)}px`,
  };
}

async function persistPreferences() {
  await setToStore(visionLabPreferencesKey, clone(preferences));
}

async function persistActiveTab() {
  await setToStore(visionLabActiveTabKey, activeTab.value);
}

let persistModelTimer: number | null = null;
function queuePersistModelConfig() {
  if (hydratingModelConfig.value) return;
  if (persistModelTimer) {
    window.clearTimeout(persistModelTimer);
  }
  persistModelTimer = window.setTimeout(() => {
    void visionLabConfigService.setModelConfig({
      imgDetModel: clone(imgDetModel.value),
      txtDetModel: clone(txtDetModel.value),
      txtRecModel: clone(txtRecModel.value),
    });
  }, 200);
}

function getCanvasContext() {
  const canvas = hiddenCanvasRef.value;
  if (!canvas) return null;
  return canvas.getContext('2d', { willReadFrequently: true });
}

function getCanvasPixel(x: number, y: number): RgbColor | null {
  const ctx = getCanvasContext();
  if (!ctx || !naturalSize.width || !naturalSize.height) return null;
  const clampedX = Math.max(0, Math.min(naturalSize.width - 1, Math.round(x)));
  const clampedY = Math.max(0, Math.min(naturalSize.height - 1, Math.round(y)));
  const data = ctx.getImageData(clampedX, clampedY, 1, 1).data;
  return { r: data[0], g: data[1], b: data[2] };
}

function sampleRegionAverage(box: BoundingBox): RgbColor | null {
  const ctx = getCanvasContext();
  if (!ctx) return null;
  const x1 = Math.max(0, Math.min(naturalSize.width - 1, box.x1));
  const y1 = Math.max(0, Math.min(naturalSize.height - 1, box.y1));
  const x2 = Math.max(x1 + 1, Math.min(naturalSize.width, box.x2));
  const y2 = Math.max(y1 + 1, Math.min(naturalSize.height, box.y2));
  const imageData = ctx.getImageData(x1, y1, Math.max(1, x2 - x1), Math.max(1, y2 - y1)).data;

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

function analyzeOcrColors(box: BoundingBox) {
  const x1 = Math.max(0, Math.min(naturalSize.width - 1, box.x1));
  const y1 = Math.max(0, Math.min(naturalSize.height - 1, box.y1));
  const x2 = Math.max(0, Math.min(naturalSize.width - 1, box.x2));
  const y2 = Math.max(0, Math.min(naturalSize.height - 1, box.y2));
  const stepX = Math.max(1, Math.floor((x2 - x1) / 3));
  const stepY = Math.max(1, Math.floor((y2 - y1) / 3));

  const samples: RgbColor[] = [];
  for (let y = y1; y <= y2; y += stepY) {
    for (let x = x1; x <= x2; x += stepX) {
      const pixel = getCanvasPixel(x, y);
      if (pixel) samples.push(pixel);
    }
  }

  const bgColor = sampleRegionAverage(box) ?? { r: 255, g: 255, b: 255 };
  if (!samples.length) {
    return {
      bgColorHex: rgbToHex(bgColor),
      fgColorHex: rgbToHex(bgColor),
    };
  }

  let farthest = samples[0];
  let farthestDistance = rgbDistance(samples[0], bgColor);
  for (const sample of samples) {
    const distance = rgbDistance(sample, bgColor);
    if (distance > farthestDistance) {
      farthest = sample;
      farthestDistance = distance;
    }
  }

  return {
    bgColorHex: rgbToHex(bgColor),
    fgColorHex: rgbToHex(farthest),
  };
}

function buildPointSample(x: number, y: number): PointSample | null {
  const rgb = getCanvasPixel(x, y);
  if (!rgb) return null;
  return {
    x,
    y,
    rgb,
    hex: rgbToHex(rgb),
    deltaToTarget: rgbDistance(rgb, parseHexColor(targetColorHex.value)),
  };
}

function buildRegionSample(box: BoundingBox): RegionSample | null {
  const rgb = sampleRegionAverage(box);
  if (!rgb) return null;
  return {
    box,
    rgb,
    hex: rgbToHex(rgb),
    deltaToTarget: rgbDistance(rgb, parseHexColor(targetColorHex.value)),
  };
}

function getPointerPosition(event: MouseEvent) {
  const rect = previewSurfaceRef.value?.getBoundingClientRect();
  if (!rect || !zoom.value) return null;
  return {
    x: Math.max(0, Math.min(naturalSize.width, Math.round((event.clientX - rect.left) / zoom.value))),
    y: Math.max(0, Math.min(naturalSize.height, Math.round((event.clientY - rect.top) / zoom.value))),
  };
}

function sanitizeImageDetector(model: DetectorType | null): DetectorType | null {
  if (!model || 'PaddleDbNet' in model) {
    return null;
  }
  const yolo = extractYoloDetector(model);
  if (yolo && yolo.baseModel.modelSource === 'BuiltIn') {
    yolo.baseModel.modelSource = 'Custom';
  }
  return model;
}

function sanitizeTextDetector(model: DetectorType | null): DetectorType | null {
  if (!model) {
    return null;
  }
  if ('PaddleDbNet' in model) {
    if (model.PaddleDbNet.baseModel.modelSource === 'BuiltIn') {
      model.PaddleDbNet.baseModel.modelSource = 'Custom';
    }
    return model;
  }
  const yolo = extractYoloDetector(model);
  if (yolo && yolo.baseModel.modelSource === 'BuiltIn') {
    yolo.baseModel.modelSource = 'Custom';
  }
  return model;
}

function setImgDetKind(value: string | number | null) {
  const kind = String(value ?? 'none') as VisionImgDetectorKind;
  imgDetModel.value = kind === 'none' ? null : createDetectorByKind(kind, false);
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
    imageData: null,
    savedPath: path,
    createdAt: new Date().toISOString(),
    saved: true,
  }));
}

async function reloadImageDirectory() {
  if (!preferences.imageDir) return;
  try {
    await loadImageDirectory(preferences.imageDir);
  } catch (error) {
    showToast(error instanceof Error ? error.message : '读取图像目录失败', 'error');
  }
}

async function pickImageDirectory() {
  const value = await open({ directory: true, multiple: false });
  if (typeof value !== 'string' || !value) return;
  preferences.imageDir = value;
  await persistPreferences();
  await reloadImageDirectory();
}

async function pickSaveDirectory() {
  const value = await open({ directory: true, multiple: false });
  if (typeof value !== 'string' || !value) return;
  preferences.saveDir = value;
  await persistPreferences();
  showToast('保存目录已更新', 'success');
}

async function selectItem(item: VisionSourceItem) {
  selectedItemId.value = item.id;
  clearVisionResults();
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
    return;
  }
  if (item.savedPath) {
    selectedPreviewUrl.value = await scriptService.convertLocalImageToDataUrl(item.savedPath);
    item.previewUrl = selectedPreviewUrl.value;
  }
}

function clearVisionResults() {
  detResults.value = [];
  textDetResults.value = [];
  ocrResults.value = [];
  comboResults.value = [];
}

async function ensureSaveDirectory() {
  if (preferences.saveDir) return preferences.saveDir;
  const value = await open({ directory: true, multiple: false });
  if (typeof value !== 'string' || !value) return null;
  preferences.saveDir = value;
  await persistPreferences();
  return value;
}

async function saveCaptureItem(item: VisionSourceItem) {
  if (!item.imageData || item.saved) return;
  const saveDir = await ensureSaveDirectory();
  if (!saveDir) {
    showToast('未设置保存目录', 'warning');
    return;
  }
  savingCaptureId.value = item.id;
  try {
    const savedPath = await visionLabService.saveCaptureImage(item.imageData, saveDir, item.name);
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
    let capture;
    try {
      capture = await visionLabService.captureDevice(selectedDevice.value);
    } catch (initialError) {
      if (selectedDevice.value.data.capMethod !== 'adb') {
        throw initialError;
      }

      const running = await deviceService.isRunning(selectedDevice.value.id);
      if (running) {
        throw initialError;
      }

      const approved = await confirm('设备当前未完成运行时准备。是否先启动设备并尝试连接后再截图？', {
        title: '设备截图',
        kind: 'warning',
      });
      if (!approved) {
        return;
      }

      const message = await deviceService.prepareCapture(selectedDevice.value.id);
      await deviceStore.refreshRunningDevices();
      showToast(message, 'success');
      capture = await visionLabService.captureDevice(selectedDevice.value);
    }

    const suggestedName = `${selectedDevice.value.data.deviceName}_${new Date().toISOString().replace(/[:.]/g, '-')}.png`;
    const item: VisionSourceItem = {
      id: `capture:${suggestedName}:${Date.now()}`,
      kind: 'capture',
      name: normalizeItemName(suggestedName),
      path: null,
      previewUrl: `data:image/png;base64,${capture.imageData}`,
      imageData: capture.imageData,
      savedPath: null,
      createdAt: new Date().toISOString(),
      saved: false,
    };
    captureItems.value = [item, ...captureItems.value];
    await selectItem(item);
    showToast('设备截图已加入当前采集区', 'success');
  } catch (error) {
    console.log(error);
    showToast(error instanceof Error ? error.message : '设备截图失败', 'error');
  }
}

const pickImgDetLabelPath = async () => {
  const value = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'Label Files', extensions: ['yaml', 'yml', 'json', 'txt'] }],
  });
  if (typeof value === 'string' && value && imgDetYolo.value) {
    imgDetYolo.value.labelPath = value;
  }
};

const pickTxtDetLabelPath = async () => {
  const value = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'Label Files', extensions: ['yaml', 'yml', 'json', 'txt'] }],
  });
  if (typeof value === 'string' && value && txtDetYolo.value) {
    txtDetYolo.value.labelPath = value;
  }
};

const pickTxtRecDictPath = async () => {
  const value = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'Dictionary Files', extensions: ['txt', 'dict'] }],
  });
  if (typeof value === 'string' && value && txtRecCrnn.value) {
    txtRecCrnn.value.dictPath = value;
  }
};

async function runDetection() {
  if (!imgDetModel.value || (!selectedImagePath.value && !selectedImageData.value)) return;
  isRunningDet.value = true;
  try {
    detResults.value = selectedImagePath.value
      ? await visionLabService.runDetection(clone(imgDetModel.value), selectedImagePath.value)
      : await visionLabService.runDetectionForImageData(clone(imgDetModel.value), selectedImageData.value!);
    showToast(`检测完成，共 ${detResults.value.length} 条结果`, 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '目标检测失败', 'error');
  } finally {
    isRunningDet.value = false;
  }
}

async function runTextDetection() {
  if (!txtDetModel.value || (!selectedImagePath.value && !selectedImageData.value)) return;
  isRunningTextDet.value = true;
  try {
    textDetResults.value = selectedImagePath.value
      ? await visionLabService.runDetection(clone(txtDetModel.value), selectedImagePath.value)
      : await visionLabService.runDetectionForImageData(clone(txtDetModel.value), selectedImageData.value!);
    showToast(`文字检测完成，共 ${textDetResults.value.length} 个文本框`, 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '文字检测失败', 'error');
  } finally {
    isRunningTextDet.value = false;
  }
}

async function runOcr() {
  if (!txtDetModel.value || !txtRecModel.value || (!selectedImagePath.value && !selectedImageData.value)) return;
  isRunningOcr.value = true;
  try {
    const results = selectedImagePath.value
      ? await visionLabService.runOcr(clone(txtDetModel.value), clone(txtRecModel.value), selectedImagePath.value)
      : await visionLabService.runOcrForImageData(clone(txtDetModel.value), clone(txtRecModel.value), selectedImageData.value!);
    ocrResults.value = results.map((item) => ({
      ...item,
      ...analyzeOcrColors(item.bounding_box),
    }));
    showToast(`OCR 完成，共 ${ocrResults.value.length} 条文本`, 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : 'OCR 失败', 'error');
  } finally {
    isRunningOcr.value = false;
  }
}

async function runComboAnalysis() {
  if (!canRunCombo.value) return;
  isRunningCombo.value = true;
  try {
    await Promise.all([runDetection(), runOcr()]);
    comboResults.value = [
      ...detResults.value.map((item, index) => ({
        key: `combo-det-${index}`,
        kind: 'det' as const,
        label: item.label,
        box: item.bounding_box,
        score: item.score,
        bgColorHex: rgbToHex(sampleRegionAverage(item.bounding_box) ?? { r: 255, g: 255, b: 255 }),
        fgColorHex: null,
      })),
      ...ocrResults.value.map((item, index) => ({
        key: `combo-ocr-${index}`,
        kind: 'ocr' as const,
        text: item.txt,
        box: item.bounding_box,
        score: averageScore(item),
        bgColorHex: item.bgColorHex,
        fgColorHex: item.fgColorHex,
      })),
    ];
    showToast(`视觉分析完成，共 ${comboResults.value.length} 条结果`, 'success');
  } finally {
    isRunningCombo.value = false;
  }
}

function fitPreview() {
  if (!previewContainerRef.value || !naturalSize.width || !naturalSize.height) return;
  const containerWidth = previewContainerRef.value.clientWidth - 48;
  const containerHeight = previewContainerRef.value.clientHeight - 48;
  const widthRatio = containerWidth / naturalSize.width;
  const heightRatio = containerHeight / naturalSize.height;
  zoom.value = Math.max(0.2, Math.min(1, widthRatio, heightRatio));
}

function resetZoom() {
  zoom.value = 1;
}

function zoomIn() {
  zoom.value = Math.min(5, Number((zoom.value + 0.1).toFixed(2)));
}

function zoomOut() {
  zoom.value = Math.max(0.2, Number((zoom.value - 0.1).toFixed(2)));
}

function handlePreviewWheel(event: WheelEvent) {
  if (!event.ctrlKey || !previewContainerRef.value || !previewSurfaceRef.value || !naturalSize.width || !naturalSize.height) {
    return;
  }

  event.preventDefault();

  const container = previewContainerRef.value;
  const surfaceRect = previewSurfaceRef.value.getBoundingClientRect();
  const offsetX = event.clientX - surfaceRect.left;
  const offsetY = event.clientY - surfaceRect.top;
  const imageX = offsetX / zoom.value;
  const imageY = offsetY / zoom.value;

  const delta = event.deltaY < 0 ? 0.12 : -0.12;
  const nextZoom = Math.max(0.2, Math.min(5, Number((zoom.value + delta).toFixed(2))));

  if (nextZoom === zoom.value) {
    return;
  }

  zoom.value = nextZoom;

  nextTick(() => {
    container.scrollLeft = Math.max(0, imageX * nextZoom - (event.clientX - container.getBoundingClientRect().left));
    container.scrollTop = Math.max(0, imageY * nextZoom - (event.clientY - container.getBoundingClientRect().top));
  });
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
        ...analyzeOcrColors(item.bounding_box),
      }));
    }
    if (comboResults.value.length) {
      comboResults.value = comboResults.value.map((item) => ({
        ...item,
        bgColorHex: item.kind === 'det'
          ? rgbToHex(sampleRegionAverage(item.box) ?? { r: 255, g: 255, b: 255 })
          : analyzeOcrColors(item.box).bgColorHex,
        fgColorHex: item.kind === 'ocr' ? analyzeOcrColors(item.box).fgColorHex : null,
      }));
    }
  });
}

function handlePreviewClick(event: MouseEvent) {
  if (activeTool.value !== 'pick' || panState.active) return;
  const position = getPointerPosition(event);
  if (!position) return;
  pickedPoint.value = buildPointSample(position.x, position.y);
}

function handlePreviewPointerDown(event: MouseEvent) {
  if (activeTool.value === 'browse') {
    if (!previewContainerRef.value) return;
    panState.active = true;
    panState.startClientX = event.clientX;
    panState.startClientY = event.clientY;
    panState.startScrollLeft = previewContainerRef.value.scrollLeft;
    panState.startScrollTop = previewContainerRef.value.scrollTop;
    return;
  }

  if (activeTool.value !== 'region') return;
  const position = getPointerPosition(event);
  if (!position) return;
  regionStart.value = position;
  regionDraft.value = { x1: position.x, y1: position.y, x2: position.x, y2: position.y };
}

function handlePreviewPointerMove(event: MouseEvent) {
  if (activeTool.value === 'browse' && panState.active && previewContainerRef.value) {
    const dx = event.clientX - panState.startClientX;
    const dy = event.clientY - panState.startClientY;
    previewContainerRef.value.scrollLeft = panState.startScrollLeft - dx;
    previewContainerRef.value.scrollTop = panState.startScrollTop - dy;
    return;
  }

  if (activeTool.value !== 'region' || !regionStart.value) return;
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
  if (activeTool.value === 'browse') {
    panState.active = false;
    return;
  }

  if (activeTool.value !== 'region' || !regionDraft.value) {
    regionStart.value = null;
    return;
  }
  regionSample.value = buildRegionSample(regionDraft.value);
  regionStart.value = null;
}

async function loadImgDetLabels() {
  const labelPath = imgDetYolo.value?.labelPath?.trim();
  if (!labelPath) {
    imgDetLabelOptions.value = [];
    detLabelFilter.value = 'ALL';
    return;
  }
  try {
    const labels = await scriptService.getYoloLabels(labelPath);
    imgDetLabelOptions.value = labels.map((item) => ({
      label: `${item.index}: ${item.label}`,
      value: String(item.index),
      description: item.label,
    }));
  } catch {
    imgDetLabelOptions.value = [];
  }
}

async function loadTxtDetLabels() {
  const labelPath = txtDetYolo.value?.labelPath?.trim();
  if (!labelPath) {
    txtDetLabelOptions.value = [];
    txtDetLabelHint.value = '当前未设置文字检测标签路径，可先选择“不过滤”验证模型输出。';
    return;
  }
  try {
    const labels = await scriptService.getYoloLabels(labelPath);
    txtDetLabelOptions.value = [
      { label: '不过滤', value: '__ALL__', description: '先查看所有检测结果' },
      ...labels.map((item) => ({
        label: `${item.index}: ${item.label}`,
        value: String(item.index),
        description: `idx ${item.index}`,
      })),
    ];
    txtDetLabelHint.value = labels.length ? null : '标签文件已读取，但未解析到 names。';
  } catch (error) {
    txtDetLabelOptions.value = [{ label: '不过滤', value: '__ALL__', description: '先查看所有检测结果' }];
    txtDetLabelHint.value = error instanceof Error ? `标签文件读取失败：${error.message}` : '标签文件读取失败，请检查路径或格式。';
  }
}

async function hydrateFromLocalConfig() {
  const config = await visionLabConfigService.getModelConfig().catch(() => null);
  imgDetModel.value = sanitizeImageDetector(clone(config?.imgDetModel ?? createDetectorByKind('Yolo11', false)));
  txtDetModel.value = sanitizeTextDetector(clone(config?.txtDetModel ?? createDetectorByKind('PaddleDbNet', true)));
  txtRecModel.value = clone(config?.txtRecModel ?? createRecognizerByKind('PaddleCrnn'));
}

async function hydrateFromLaunchPreset() {
  const preset = await getFromStore<VisionLabLaunchPreset | null>(visionLabLaunchPresetKey).catch(() => null);
  if (!preset) {
    await hydrateFromLocalConfig();
    return;
  }
  imgDetModel.value = sanitizeImageDetector(clone(preset.imgDetModel ?? createDetectorByKind('Yolo11', false)));
  txtDetModel.value = sanitizeTextDetector(clone(preset.txtDetModel ?? createDetectorByKind('PaddleDbNet', true)));
  txtRecModel.value = clone(preset.txtRecModel ?? createRecognizerByKind('PaddleCrnn'));
  if (preset.selectedDeviceId) {
    selectedDeviceId.value = preset.selectedDeviceId;
  }
  await setToStore(visionLabLaunchPresetKey, null);
}

async function loadPreferences() {
  const saved = await getFromStore<VisionLabPreferences>(visionLabPreferencesKey).catch(() => null);
  if (saved) {
    Object.assign(preferences, { ...DEFAULT_VISION_LAB_PREFERENCES, ...saved });
  }
  const savedTab = await getFromStore<VisionTab>(visionLabActiveTabKey).catch(() => null);
  if (savedTab) {
    activeTab.value = savedTab;
  }
}

watch(activeTab, () => {
  void persistActiveTab();
});

watch(
  () => [imgDetModel.value, txtDetModel.value, txtRecModel.value],
  () => {
    queuePersistModelConfig();
  },
  { deep: true },
);

watch(selectedDeviceId, (value) => {
  if (!value) return;
  void setToStore(deviceKey, value);
});

watch(
  () => deviceStore.devices.map((device) => device.id).join('|'),
  async () => {
    const storedDeviceId = await getFromStore<string>(deviceKey).catch(() => null);
    if (!selectedDeviceId.value && storedDeviceId && deviceStore.devices.some((device) => device.id === storedDeviceId)) {
      selectedDeviceId.value = storedDeviceId;
      return;
    }
    if (selectedDeviceId.value && deviceStore.devices.some((device) => device.id === selectedDeviceId.value)) {
      return;
    }
    selectedDeviceId.value = deviceStore.devices[0]?.id ?? null;
  },
  { immediate: true },
);

watch(
  () => imgDetYolo.value?.labelPath,
  () => {
    void loadImgDetLabels();
  },
  { immediate: true },
);

watch(
  () => txtDetYolo.value?.labelPath,
  () => {
    void loadTxtDetLabels();
  },
  { immediate: true },
);

onMounted(async () => {
  await Promise.all([deviceStore.refreshAll(), loadPreferences()]);
  await hydrateFromLaunchPreset();
  hydratingModelConfig.value = false;
  await Promise.all([loadImgDetLabels(), loadTxtDetLabels()]);
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
  transition: color 0.16s ease, border-color 0.16s ease;
}

.editor-panel-tab:hover {
  color: var(--app-text-soft);
}

.editor-panel-tab-active {
  border-bottom-color: var(--app-accent);
  color: var(--app-text-strong);
}

.vision-path-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 0.65rem;
  align-items: center;
}

.vision-path-button {
  min-width: 2.75rem;
  height: 2.75rem;
  padding: 0;
}
</style>
