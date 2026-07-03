<template>
  <div class="flex h-full min-h-0 flex-col gap-4">
    <AppPageHeader
      title="运行日志"
    />

    <SurfacePanel class="flex flex-wrap items-center gap-3">
      <div class="w-full max-w-[220px] flex-1">
        <AppSelect v-model="selectedDeviceId" :options="deviceOptions" placeholder="全部设备" />
      </div>
      <div class="w-full max-w-[180px] flex-1">
        <AppSelect v-model="selectedLevel" :options="levelOptions" placeholder="全部级别" />
      </div>
      <div class="min-w-[320px] flex-1">
        <div class="flex items-center gap-2">
          <input v-model.trim="searchText" class="app-input min-w-0 flex-1" type="search" placeholder="搜索日志内容" />
          <div class="flex shrink-0 items-center gap-1 rounded-[12px] border border-(--app-border) bg-(--app-panel-muted)/75 px-2 py-1">
            <span class="min-w-[54px] text-center text-xs font-medium text-(--app-text-faint)">
              {{ searchHitLabel }}
            </span>
            <button
              class="app-icon-button h-8 w-8"
              type="button"
              title="上一个命中"
              :disabled="!searchHits.length"
              @click="moveSearchHit(-1)"
            >
              <AppIcon name="chevron-up" :size="14" />
            </button>
            <button
              class="app-icon-button h-8 w-8"
              type="button"
              title="下一个命中"
              :disabled="!searchHits.length"
              @click="moveSearchHit(1)"
            >
              <AppIcon name="chevron-down" :size="14" />
            </button>
          </div>
        </div>
      </div>
      <AppSelect
        v-if="selectedDeviceId"
        class="w-full max-w-[180px] flex-1"
        v-model="deviceLogLevel"
        :options="levelOptions.filter((item) => item.value)"
      />
      <button class="app-icon-button h-10 w-10" type="button" title="滚动到顶部" @click="scrollToTop">
        <AppIcon name="arrow-up-to-line" :size="16" />
      </button>
      <button class="app-icon-button h-10 w-10" type="button" title="滚动到底部" @click="scrollToBottom">
        <AppIcon name="arrow-down-to-line" :size="16" />
      </button>
      <button class="app-button app-button-ghost" type="button" @click="handleClearLogs">
        清空
      </button>
    </SurfacePanel>

    <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
    <SurfacePanel v-if="historyLoading" class="h-full">
      <AppLoadingState label="正在加载本日日志..." />
    </SurfacePanel>

    <SurfacePanel v-else-if="!visibleLogs.length" class="space-y-4">
      <EmptyState
        :title="emptyLogTitle"
        :description="emptyLogDescription"
        icon="file-text"
      />
      <div class="grid gap-3 text-sm text-(--app-text-soft) md:grid-cols-3">
        <div class="rounded-[16px] border border-(--app-border) bg-(--app-panel-muted)/70 px-4 py-3">
          <p class="font-semibold text-(--app-text-strong)">先运行队列</p>
          <p class="mt-1 text-xs leading-5">设备开始执行后，日志会在这里追加。</p>
        </div>
        <div class="rounded-[16px] border border-(--app-border) bg-(--app-panel-muted)/70 px-4 py-3">
          <p class="font-semibold text-(--app-text-strong)">检查筛选</p>
          <p class="mt-1 text-xs leading-5">设备或级别筛选过窄时会隐藏已有日志。</p>
        </div>
        <div class="rounded-[16px] border border-(--app-border) bg-(--app-panel-muted)/70 px-4 py-3">
          <p class="font-semibold text-(--app-text-strong)">本日会话</p>
          <p class="mt-1 text-xs leading-5">这里显示的是本日打开应用后收到的日志。</p>
        </div>
      </div>
    </SurfacePanel>

    <SurfacePanel v-else class="h-full overflow-hidden p-0">
      <div
        ref="logContainer"
        class="log-viewer h-full overflow-y-auto bg-[#081019] px-6 py-4 font-mono text-xs text-slate-200"
        draggable="false"
        @copy="handleLogCopy"
        @mousedown.stop
        @scroll="handleLogScroll"
        @dragstart.prevent
      >
        <div>
          <div class="log-spacer" :style="{ height: `${topSpacerHeight}px` }" />
          <div
            v-for="log in virtualLogs"
            :key="log.key"
            :ref="(el) => setLogRowRef(log.index, el)"
            class="log-line grid gap-y-0.5 border-b border-white/5 pb-1.5 md:gap-x-2 md:grid-cols-[130px_50px_1fr]"
            draggable="false"
            @dragstart.prevent
          >
            <span class="log-line__meta pt-[1px] font-sans tracking-wide text-slate-500">{{ log.entry.time }} {{ resolveDeviceName(log.entry.deviceId) }}</span>
            <span class="log-line__level font-sans uppercase" :class="levelClass(log.entry.level)">{{ log.entry.level }}</span>
            <span class="log-line__message whitespace-pre-wrap break-all leading-5 text-slate-100/90">
              <template v-for="segment in log.segments" :key="segment.key">
                <mark
                  v-if="segment.isMatch"
                  :ref="(el) => setSearchHitRef(segment.hitId!, el)"
                  class="log-search-hit"
                  :class="{ 'log-search-hit-active': segment.hitId === currentSearchHitId }"
                >
                  {{ segment.text }}
                </mark>
                <template v-else>{{ segment.text }}</template>
              </template>
            </span>
          </div>
          <div class="log-spacer" :style="{ height: `${bottomSpacerHeight}px` }" />
        </div>
      </div>
    </SurfacePanel>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import AppLoadingState from '@/components/shared/AppLoadingState.vue';
import AppIcon from '@/components/shared/AppIcon.vue';
import AppSelect from '@/components/shared/AppSelect.vue';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import { useDeviceStore } from '@/store/device';
import { useLogsStore } from '@/store/logs';
import { deviceService } from '@/services/deviceService';
import { showToast } from '@/utils/toast';
import type { LogLevel } from '@/types/bindings/LogLevel';

const deviceStore = useDeviceStore();
const logsStore = useLogsStore();
const selectedLevel = ref('');
const searchText = ref('');
const deviceLogLevel = ref<LogLevel>('Info');
const logContainer = ref<HTMLDivElement | null>(null);
const currentSearchHitIndex = ref(-1);
const searchHitElements = new Map<string, HTMLElement>();
const suppressDeviceLogLevelSync = ref(false);
const pageReady = ref(false);
const historyLoading = ref(false);
const scrollTop = ref(0);
const viewportHeight = ref(0);
const rowHeights = ref<number[]>([]);
const rowElements = new Map<number, HTMLElement>();
const selectedRowRange = ref<{ start: number; end: number } | null>(null);
const suspendRowMeasurement = ref(false);
const ESTIMATED_ROW_HEIGHT = 30;
const VIRTUAL_OVERSCAN = 10;
let rowResizeObserver: ResizeObserver | null = null;
let containerResizeObserver: ResizeObserver | null = null;
let scrollSettleTimer: number | null = null;

const selectedDeviceId = computed({
  get: () => logsStore.selectedDeviceId,
  set: (value: string) => {
    void logsStore.setSelectedDevice(value);
  },
});

const deviceOptions = computed(() => [
  { label: '全部设备', value: '' },
  ...deviceStore.devices.map((device) => ({
    label: device.data.deviceName,
    value: device.id,
  })),
]);

const deviceNameMap = computed(() =>
  new Map(deviceStore.devices.map((device) => [device.id, device.data.deviceName])),
);

const deviceLogLevelMap = computed(() =>
  new Map(deviceStore.devices.map((device) => [device.id, device.data.logLevel])),
);

const levelOptions = [
  { label: '全部级别', value: '' },
  { label: 'Debug', value: 'Debug' },
  { label: 'Info', value: 'Info' },
  { label: 'Warn', value: 'Warn' },
  { label: 'Error', value: 'Error' },
];

const normalizedSearchText = computed(() => searchText.value.trim().toLowerCase());

const visibleLogs = computed(() => {
  const pool = selectedDeviceId.value
    ? logsStore.getDeviceLogs(selectedDeviceId.value)
    : logsStore.allLogs;

  return pool.filter((entry) => {
    if (selectedLevel.value && entry.level !== selectedLevel.value) {
      return false;
    }
    return true;
  });
});

const buildSearchSegments = (message: string, keyword: string, logIndex: number) => {
  if (!keyword) {
    return [
      {
        key: `${logIndex}-plain`,
        text: message,
        isMatch: false,
        hitId: null,
      },
    ];
  }

  const lowerMessage = message.toLowerCase();
  const segments: Array<{ key: string; text: string; isMatch: boolean; hitId: string | null }> = [];
  let cursor = 0;
  let matchIndex = 0;

  while (cursor < message.length) {
    const foundAt = lowerMessage.indexOf(keyword, cursor);
    if (foundAt === -1) {
      if (cursor < message.length) {
        segments.push({
          key: `${logIndex}-plain-${cursor}`,
          text: message.slice(cursor),
          isMatch: false,
          hitId: null,
        });
      }
      break;
    }

    if (foundAt > cursor) {
      segments.push({
        key: `${logIndex}-plain-${cursor}`,
        text: message.slice(cursor, foundAt),
        isMatch: false,
        hitId: null,
      });
    }

    segments.push({
      key: `${logIndex}-hit-${matchIndex}`,
      text: message.slice(foundAt, foundAt + keyword.length),
      isMatch: true,
      hitId: `${logIndex}:${matchIndex}`,
    });
    cursor = foundAt + keyword.length;
    matchIndex += 1;
  }

  return segments;
};

const decoratedLogs = computed(() =>
  visibleLogs.value.map((entry, logIndex) => {
    const segments = buildSearchSegments(entry.message, normalizedSearchText.value, logIndex);
    return {
      key: `${entry.deviceId}-${entry.time}-${entry.level}-${logIndex}`,
      entry,
      segments,
    };
  }),
);

const rowMetrics = computed(() => {
  const offsets: number[] = new Array(decoratedLogs.value.length);
  let totalHeight = 0;

  for (let index = 0; index < decoratedLogs.value.length; index += 1) {
    offsets[index] = totalHeight;
    totalHeight += rowHeights.value[index] ?? ESTIMATED_ROW_HEIGHT;
  }

  return { offsets, totalHeight };
});

const locateRowIndex = (position: number) => {
  const { offsets } = rowMetrics.value;
  let low = 0;
  let high = offsets.length - 1;

  while (low <= high) {
    const mid = Math.floor((low + high) / 2);
    const top = offsets[mid];
    const bottom = top + (rowHeights.value[mid] ?? ESTIMATED_ROW_HEIGHT);

    if (position < top) {
      high = mid - 1;
      continue;
    }
    if (position >= bottom) {
      low = mid + 1;
      continue;
    }
    return mid;
  }

  return Math.max(0, Math.min(offsets.length - 1, low));
};

const virtualRange = computed(() => {
  if (!decoratedLogs.value.length) {
    return { start: 0, end: -1 };
  }

  let start = Math.max(0, locateRowIndex(Math.max(0, scrollTop.value)) - VIRTUAL_OVERSCAN);
  let end = Math.min(
    decoratedLogs.value.length - 1,
    locateRowIndex(scrollTop.value + Math.max(viewportHeight.value, ESTIMATED_ROW_HEIGHT)) + VIRTUAL_OVERSCAN,
  );

  if (selectedRowRange.value) {
    start = Math.min(start, selectedRowRange.value.start);
    end = Math.max(end, selectedRowRange.value.end);
  }

  return { start, end };
});

const virtualLogs = computed(() =>
  decoratedLogs.value
    .slice(virtualRange.value.start, virtualRange.value.end + 1)
    .map((log, offset) => {
      const index = virtualRange.value.start + offset;
      return {
        ...log,
        index,
      };
    }),
);

const topSpacerHeight = computed(() => rowMetrics.value.offsets[virtualRange.value.start] ?? 0);

const bottomSpacerHeight = computed(() => {
  if (virtualRange.value.end < virtualRange.value.start) {
    return 0;
  }

  const renderedHeight =
    (rowMetrics.value.offsets[virtualRange.value.end] ?? 0) +
    (rowHeights.value[virtualRange.value.end] ?? ESTIMATED_ROW_HEIGHT);

  return Math.max(0, rowMetrics.value.totalHeight - renderedHeight);
});

const searchHitMeta = computed(() =>
  decoratedLogs.value.flatMap((log, rowIndex) =>
    log.segments
      .filter((segment) => segment.isMatch && segment.hitId)
      .map((segment) => ({ hitId: segment.hitId as string, rowIndex })),
  ),
);

const searchHits = computed(() => searchHitMeta.value.map((item) => item.hitId));

const currentSearchHitId = computed(() =>
  currentSearchHitIndex.value >= 0 ? searchHits.value[currentSearchHitIndex.value] ?? null : null,
);

const currentSearchHitMeta = computed(() =>
  currentSearchHitIndex.value >= 0 ? searchHitMeta.value[currentSearchHitIndex.value] ?? null : null,
);

const searchHitLabel = computed(() => {
  if (!normalizedSearchText.value) {
    return '-- / --';
  }
  if (!searchHits.value.length) {
    return '0 / 0';
  }
  return `${currentSearchHitIndex.value + 1} / ${searchHits.value.length}`;
});

const hasAnyLogs = computed(() => Object.values(logsStore.logsByDevice).some((entries) => entries.length > 0));
const hasActiveFilter = computed(() => Boolean(selectedDeviceId.value || selectedLevel.value));

const emptyLogTitle = computed(() => {
  if (hasAnyLogs.value && hasActiveFilter.value) {
    return '筛选后没有日志';
  }
  return '还没有运行日志';
});

const emptyLogDescription = computed(() => {
  if (hasAnyLogs.value && hasActiveFilter.value) {
    return '已有日志被当前设备或级别条件过滤了。';
  }
  return '启动设备队列或调试运行后，这里会显示实时日志。';
});

const resolveDeviceName = (deviceId: string) => {
  return deviceNameMap.value.get(deviceId) || '未知设备';
};

const levelClass = (level: string) => {
  if (level === 'Error') return 'text-red-400';
  if (level === 'Warn') return 'text-amber-300';
  if (level === 'Info') return 'text-sky-300';
  return 'text-slate-400';
};


const setSearchHitRef = (hitId: string, el: unknown) => {
  if (el instanceof HTMLElement) {
    searchHitElements.set(hitId, el);
    return;
  }
  searchHitElements.delete(hitId);
};

const ensureObservers = () => {
  if (!rowResizeObserver && typeof ResizeObserver !== 'undefined') {
    rowResizeObserver = new ResizeObserver((entries) => {
      if (suspendRowMeasurement.value) {
        return;
      }

      const nextHeights = [...rowHeights.value];
      let changed = false;

      entries.forEach((entry) => {
        const element = entry.target as HTMLElement;
        const index = Number(element.dataset.logIndex ?? '-1');
        if (index < 0) {
          return;
        }

        const nextHeight = Math.max(ESTIMATED_ROW_HEIGHT, Math.ceil(entry.contentRect.height));
        if (nextHeights[index] !== nextHeight) {
          nextHeights[index] = nextHeight;
          changed = true;
        }
      });

      if (changed) {
        rowHeights.value = nextHeights;
      }
    });
  }

  if (!containerResizeObserver && typeof ResizeObserver !== 'undefined') {
    containerResizeObserver = new ResizeObserver(() => {
      viewportHeight.value = logContainer.value?.clientHeight ?? 0;
    });
  }
};

const setLogRowRef = (index: number, el: unknown) => {
  ensureObservers();

  const previous = rowElements.get(index);
  if (previous && previous !== el) {
    rowResizeObserver?.unobserve(previous);
    rowElements.delete(index);
  }

  if (!(el instanceof HTMLElement)) {
    return;
  }

  el.dataset.logIndex = String(index);
  rowElements.set(index, el);
  rowResizeObserver?.observe(el);

  if (suspendRowMeasurement.value) {
    return;
  }

  const measuredHeight = Math.max(ESTIMATED_ROW_HEIGHT, Math.ceil(el.getBoundingClientRect().height));
  if (rowHeights.value[index] !== measuredHeight) {
    const nextHeights = [...rowHeights.value];
    nextHeights[index] = measuredHeight;
    rowHeights.value = nextHeights;
  }
};

const initializeViewport = () => {
  ensureObservers();
  if (!logContainer.value) {
    return;
  }

  viewportHeight.value = logContainer.value.clientHeight;
  scrollTop.value = logContainer.value.scrollTop;
  containerResizeObserver?.observe(logContainer.value);
};

const handleLogScroll = () => {
  if (!logContainer.value) {
    return;
  }

  suspendRowMeasurement.value = true;
  scrollTop.value = logContainer.value.scrollTop;
  viewportHeight.value = logContainer.value.clientHeight;

  if (scrollSettleTimer !== null) {
    window.clearTimeout(scrollSettleTimer);
  }
  scrollSettleTimer = window.setTimeout(() => {
    suspendRowMeasurement.value = false;
    scheduleVisibleRowMeasurement();
    scrollSettleTimer = null;
  }, 120);
};

const normalizeCopiedText = (text: string) =>
  text
    .replace(/\u00a0/g, ' ')
    .replace(/[ \t\f\v]+/g, ' ')
    .replace(/\s*\n\s*/g, ' ')
    .trim();

const serializeLogFragment = (root: ParentNode) => {
  const parts = [
    root.querySelector<HTMLElement>('.log-line__meta')?.textContent ?? '',
    root.querySelector<HTMLElement>('.log-line__level')?.textContent ?? '',
    root.querySelector<HTMLElement>('.log-line__message')?.textContent ?? '',
  ]
    .map(normalizeCopiedText)
    .filter(Boolean);

  if (parts.length > 0) {
    return parts.join(' ');
  }

  return normalizeCopiedText(root.textContent ?? '');
};

const serializeSelectionRange = (range: Range) => {
  const fragment = range.cloneContents();
  const wrapper = document.createElement('div');
  wrapper.appendChild(fragment);

  wrapper.querySelectorAll<HTMLElement>('.log-line').forEach((row) => {
    const line = serializeLogFragment(row);
    row.replaceWith(document.createTextNode(`${line}\n`));
  });

  wrapper.querySelectorAll<HTMLElement>('.log-line__meta').forEach((element) => {
    const text = normalizeCopiedText(element.textContent ?? '');
    element.replaceWith(document.createTextNode(text ? `${text} ` : ''));
  });
  wrapper.querySelectorAll<HTMLElement>('.log-line__level').forEach((element) => {
    const text = normalizeCopiedText(element.textContent ?? '');
    element.replaceWith(document.createTextNode(text ? `${text} ` : ''));
  });
  wrapper.querySelectorAll<HTMLElement>('.log-line__message').forEach((element) => {
    element.replaceWith(document.createTextNode(normalizeCopiedText(element.textContent ?? '')));
  });

  return (wrapper.textContent ?? '')
    .replace(/[ \t]+\n/g, '\n')
    .replace(/\n{3,}/g, '\n\n')
    .trimEnd();
};

const serializeSelectedLogRow = (row: HTMLElement, selectionRange: Range) => {
  const rowRange = document.createRange();
  rowRange.selectNodeContents(row);

  if (row.contains(selectionRange.startContainer)) {
    rowRange.setStart(selectionRange.startContainer, selectionRange.startOffset);
  }
  if (row.contains(selectionRange.endContainer)) {
    rowRange.setEnd(selectionRange.endContainer, selectionRange.endOffset);
  }

  return serializeSelectionRange(rowRange);
};

const handleLogCopy = (event: ClipboardEvent) => {
  if (!logContainer.value || !event.clipboardData) {
    return;
  }

  const selection = window.getSelection();
  if (!selection || selection.isCollapsed || selection.rangeCount === 0) {
    return;
  }
  if (
    !selection.anchorNode ||
    !selection.focusNode ||
    !logContainer.value.contains(selection.anchorNode) ||
    !logContainer.value.contains(selection.focusNode)
  ) {
    return;
  }

  const range = selection.getRangeAt(0);
  const anchorIndex = resolveSelectionRowIndex(selection.anchorNode);
  const focusIndex = resolveSelectionRowIndex(selection.focusNode);

  let text = '';
  if (anchorIndex !== null && focusIndex !== null) {
    const start = Math.min(anchorIndex, focusIndex);
    const end = Math.max(anchorIndex, focusIndex);
    const lines: string[] = [];

    for (let index = start; index <= end; index += 1) {
      const row = rowElements.get(index);
      if (!row) {
        text = serializeSelectionRange(range);
        break;
      }

      const line = serializeSelectedLogRow(row, range);
      if (line) {
        lines.push(line);
      }
    }

    if (!text) {
      text = lines.join('\n');
    }
  } else {
    text = serializeSelectionRange(range);
  }

  event.clipboardData.setData('text/plain', text);
  event.preventDefault();
};

const resolveSelectionRowIndex = (node: Node | null) => {
  if (!node || !logContainer.value) {
    return null;
  }

  const element =
    node instanceof HTMLElement ? node : node.parentElement;
  const row = element?.closest('.log-line') as HTMLElement | null;
  if (!row || !logContainer.value.contains(row)) {
    return null;
  }

  const index = Number(row.dataset.logIndex ?? '-1');
  return Number.isInteger(index) && index >= 0 ? index : null;
};

const updateSelectionPinned = () => {
  const selection = window.getSelection();
  if (!selection || selection.rangeCount === 0 || selection.isCollapsed || !logContainer.value) {
    selectedRowRange.value = null;
    return;
  }

  const anchorNode = selection.anchorNode;
  const focusNode = selection.focusNode;
  if (
    !anchorNode ||
    !focusNode ||
    !logContainer.value.contains(anchorNode) ||
    !logContainer.value.contains(focusNode)
  ) {
    selectedRowRange.value = null;
    return;
  }

  const anchorIndex = resolveSelectionRowIndex(anchorNode);
  const focusIndex = resolveSelectionRowIndex(focusNode);
  if (anchorIndex === null || focusIndex === null) {
    selectedRowRange.value = null;
    return;
  }

  selectedRowRange.value = {
    start: Math.min(anchorIndex, focusIndex),
    end: Math.max(anchorIndex, focusIndex),
  };
};

const scrollToTop = () => {
  logContainer.value?.scrollTo({ top: 0, behavior: 'smooth' });
};

const scrollToBottom = () => {
  if (!logContainer.value) {
    return;
  }
  logContainer.value.scrollTo({ top: logContainer.value.scrollHeight, behavior: 'smooth' });
};

const moveSearchHit = (direction: -1 | 1) => {
  if (!searchHits.value.length) {
    return;
  }

  const nextIndex =
    currentSearchHitIndex.value < 0
      ? 0
      : (currentSearchHitIndex.value + direction + searchHits.value.length) % searchHits.value.length;
  currentSearchHitIndex.value = nextIndex;
};

const scrollCurrentSearchHitIntoView = () => {
  if (!currentSearchHitId.value || !currentSearchHitMeta.value || !logContainer.value) {
    return;
  }

  const targetTop = rowMetrics.value.offsets[currentSearchHitMeta.value.rowIndex] ?? 0;
  logContainer.value.scrollTop = Math.max(0, targetTop - logContainer.value.clientHeight * 0.35);

  nextTick(() => {
    searchHitElements.get(currentSearchHitId.value!)?.scrollIntoView({
      block: 'center',
      behavior: 'smooth',
    });
  });
};

const updateDeviceLogLevel = async () => {
  if (!selectedDeviceId.value) {
    return;
  }

  try {
    await deviceService.updateChildLogLevel(selectedDeviceId.value, deviceLogLevel.value);
    showToast('设备日志级别已更新', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '更新失败', 'error');
  }
};

const syncSelectedDeviceLogLevel = (deviceId: string) => {
  const nextLevel = deviceId ? deviceLogLevelMap.value.get(deviceId) || 'Info' : 'Info';
  if (deviceLogLevel.value === nextLevel) {
    suppressDeviceLogLevelSync.value = false;
    return;
  }
  suppressDeviceLogLevelSync.value = true;
  deviceLogLevel.value = nextLevel;
};

const handleClearLogs = async () => {
  try {
    await logsStore.clearLogs(selectedDeviceId.value || null);
    showToast(selectedDeviceId.value ? '本日日志已清空' : '全部本日日志已清空', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '清空日志失败', 'error');
  }
};

const loadHistoryLogs = async (deviceId?: string | null) => {
  historyLoading.value = true;
  try {
    if (pageReady.value) {
      await logsStore.ensureTodayLogsLoaded(deviceId ?? null);
    } else {
      await logsStore.reloadTodayLogs(deviceId ?? null);
    }
  } finally {
    historyLoading.value = false;
    await nextTick();
    initializeViewport();
    scheduleVisibleRowMeasurement();
  }
};

const resetVirtualRows = () => {
  rowElements.forEach((element) => rowResizeObserver?.unobserve(element));
  rowElements.clear();
  rowHeights.value = [];
  scrollTop.value = logContainer.value?.scrollTop ?? 0;
  viewportHeight.value = logContainer.value?.clientHeight ?? 0;
};

const scheduleVisibleRowMeasurement = () => {
  nextTick(() => {
    virtualLogs.value.forEach((log) => {
      const element = rowElements.get(log.index);
      if (!element) {
        return;
      }

      const measuredHeight = Math.max(ESTIMATED_ROW_HEIGHT, Math.ceil(element.getBoundingClientRect().height));
      if (rowHeights.value[log.index] !== measuredHeight) {
        const nextHeights = [...rowHeights.value];
        nextHeights[log.index] = measuredHeight;
        rowHeights.value = nextHeights;
      }
    });
  });
};

watch(
  () => visibleLogs.value.length,
  async () => {
    if (normalizedSearchText.value) {
      return;
    }
    await nextTick();
    scrollToBottom();
  },
);

watch(
  searchHits,
  async (hits, previousHits) => {
    if (!normalizedSearchText.value || !hits.length) {
      currentSearchHitIndex.value = -1;
      return;
    }

    if (!(previousHits?.length) || currentSearchHitIndex.value < 0 || currentSearchHitIndex.value >= hits.length) {
      currentSearchHitIndex.value = 0;
    }

    await nextTick();
    scrollCurrentSearchHitIntoView();
  },
  { immediate: true },
);

watch(
  currentSearchHitId,
  async (hitId, previousHitId) => {
    if (!hitId || hitId === previousHitId) {
      return;
    }

    await nextTick();
    scrollCurrentSearchHitIntoView();
  },
);

watch(
  () => [selectedDeviceId.value, selectedLevel.value, normalizedSearchText.value],
  () => {
    resetVirtualRows();
  },
);

watch(
  () => decoratedLogs.value.length,
  (length, previousLength) => {
    if (length < previousLength) {
      resetVirtualRows();
      return;
    }

    if (rowHeights.value.length > length) {
      rowHeights.value = rowHeights.value.slice(0, length);
    }
    scheduleVisibleRowMeasurement();
  },
);

watch(
  virtualLogs,
  () => {
    if (suspendRowMeasurement.value) {
      return;
    }
    scheduleVisibleRowMeasurement();
  },
  { deep: true },
);

watch(
  () => selectedDeviceId.value,
  async (deviceId) => {
    syncSelectedDeviceLogLevel(deviceId);
    if (!pageReady.value) {
      return;
    }
    await loadHistoryLogs(deviceId || null);
  },
  { immediate: true },
);

watch(
  () => deviceLogLevel.value,
  async (level, previousLevel) => {
    if (suppressDeviceLogLevelSync.value) {
      suppressDeviceLogLevelSync.value = false;
      return;
    }
    if (!selectedDeviceId.value || level === previousLevel) {
      return;
    }
    await updateDeviceLogLevel();
  },
);

onMounted(async () => {
  try {
    historyLoading.value = true;
    document.addEventListener('selectionchange', updateSelectionPinned);
    await Promise.all([
      deviceStore.devices.length ? Promise.resolve() : deviceStore.refreshAll(),
      logsStore.ensurePersistedSelectionLoaded(),
    ]);

    if (
      selectedDeviceId.value &&
      !deviceStore.devices.some((device) => device.id === selectedDeviceId.value)
    ) {
      await logsStore.setSelectedDevice('');
    }

    await logsStore.reloadTodayLogs(selectedDeviceId.value || null);
    await logsStore.startListener();
    syncSelectedDeviceLogLevel(selectedDeviceId.value);
    pageReady.value = true;
  } finally {
    historyLoading.value = false;
    await nextTick();
    initializeViewport();
    scheduleVisibleRowMeasurement();
  }
});

onBeforeUnmount(() => {
  document.removeEventListener('selectionchange', updateSelectionPinned);
  if (scrollSettleTimer !== null) {
    window.clearTimeout(scrollSettleTimer);
    scrollSettleTimer = null;
  }
  rowElements.forEach((element) => rowResizeObserver?.unobserve(element));
  rowElements.clear();
  rowResizeObserver?.disconnect();
  rowResizeObserver = null;
  containerResizeObserver?.disconnect();
  containerResizeObserver = null;
  logsStore.stopListener();
});
</script>

<style scoped>
.log-viewer,
.log-line {
  user-select: none;
  -webkit-user-select: none;
}

.log-line__meta,
.log-line__level,
.log-line__message,
.log-line__meta *,
.log-line__level *,
.log-line__message * {
  user-select: text;
  -webkit-user-select: text;
}

.log-viewer ::selection,
.log-line ::selection,
.log-line__meta ::selection,
.log-line__message ::selection {
  background: rgba(96, 165, 250, 0.46);
  color: rgb(255, 255, 255);
}

.log-viewer ::-moz-selection,
.log-line ::-moz-selection,
.log-line__meta ::-moz-selection,
.log-line__message ::-moz-selection {
  background: rgba(96, 165, 250, 0.46);
  color: rgb(255, 255, 255);
}

.log-viewer {
  cursor: text;
}

.log-spacer {
  user-select: none;
  -webkit-user-select: none;
  pointer-events: none;
}

.log-line {
  align-items: start;
}

.log-line__meta,
.log-line__level,
.log-line__message {
  line-height: 1.25rem;
}

.log-line__message {
  display: block;
  width: 100%;
}

.log-search-hit {
  border-radius: 6px;
  background: rgba(251, 191, 36, 0.3);
  color: #fef3c7;
  padding: 0 0.18rem;
  box-shadow: inset 0 0 0 1px rgba(253, 224, 71, 0.18);
}

.log-search-hit-active {
  background: rgba(190, 242, 100, 0.96);
  color: #081019;
  box-shadow: 0 0 0 1px rgba(236, 252, 203, 0.94);
}
</style>
