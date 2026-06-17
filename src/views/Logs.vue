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
      <button class="app-button app-button-ghost" type="button" @click="logsStore.clearLogs(selectedDeviceId || undefined)">
        清空
      </button>
    </SurfacePanel>

    <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
    <SurfacePanel v-if="!visibleLogs.length" class="space-y-4">
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
          <p class="mt-1 text-xs leading-5">设备、级别或关键字过窄时会隐藏已有日志。</p>
        </div>
        <div class="rounded-[16px] border border-(--app-border) bg-(--app-panel-muted)/70 px-4 py-3">
          <p class="font-semibold text-(--app-text-strong)">当前会话</p>
          <p class="mt-1 text-xs leading-5">这里显示的是本次打开应用后收到的实时日志。</p>
        </div>
      </div>
    </SurfacePanel>

    <SurfacePanel v-else class="h-full overflow-hidden p-0">
      <div
        ref="logContainer"
        class="log-viewer h-full overflow-y-auto bg-[#081019] px-5 py-4 font-mono text-xs text-slate-200"
        draggable="false"
        @mousedown.stop
        @dragstart.prevent
      >
        <div class="space-y-1">
          <div
            v-for="log in decoratedLogs"
            :key="log.key"
            class="log-line grid gap-x-3 gap-y-0.5 border-b border-white/5 pb-1.5 md:grid-cols-[160px_90px_1fr]"
            draggable="false"
            @dragstart.prevent
          >
            <span class="log-line__meta pt-[1px] font-sans tracking-wide text-slate-500">{{ log.entry.time }} · {{ resolveDeviceName(log.entry.deviceId) }}</span>
            <div class="flex items-start gap-1.5 pt-[1px]" :class="levelClass(log.entry.level)">
              <AppIcon :name="levelIcon(log.entry.level)" :size="14" class="shrink-0 translate-y-[2px]" />
              <span class="font-sans text-xs font-semibold tracking-wider uppercase">{{ log.entry.level }}</span>
            </div>
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
        </div>
      </div>
    </SurfacePanel>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
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
const selectedDeviceId = ref('');
const selectedLevel = ref('');
const searchText = ref('');
const deviceLogLevel = ref<LogLevel>('Info');
const logContainer = ref<HTMLDivElement | null>(null);
const currentSearchHitIndex = ref(-1);
const searchHitElements = new Map<string, HTMLElement>();
const suppressDeviceLogLevelSync = ref(false);

const deviceOptions = computed(() => [
  { label: '全部设备', value: '' },
  ...deviceStore.devices.map((device) => ({
    label: device.data.deviceName,
    value: device.id,
  })),
]);

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
    : Object.values(logsStore.logsByDevice).flat();

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

const searchHits = computed(() =>
  decoratedLogs.value.flatMap((log) => log.segments.filter((segment) => segment.isMatch && segment.hitId).map((segment) => segment.hitId as string)),
);

const currentSearchHitId = computed(() =>
  currentSearchHitIndex.value >= 0 ? searchHits.value[currentSearchHitIndex.value] ?? null : null,
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
  return deviceStore.devices.find((device) => device.id === deviceId)?.data.deviceName || deviceId;
};

const levelClass = (level: string) => {
  if (level === 'Error') return 'text-red-400';
  if (level === 'Warn') return 'text-amber-300';
  if (level === 'Info') return 'text-sky-300';
  return 'text-slate-400';
};

const levelIcon = (level: string) => {
  if (level === 'Error') return 'hexagon'; // x-octagon alternative if not available, wait 'hexagon' has no 'x' usually but let's use 'alert-triangle'
  if (level === 'Warn') return 'triangle-alert';
  if (level === 'Debug') return 'bug';
  return 'info';
};

const setSearchHitRef = (hitId: string, el: unknown) => {
  if (el instanceof HTMLElement) {
    searchHitElements.set(hitId, el);
    return;
  }
  searchHitElements.delete(hitId);
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
  if (!currentSearchHitId.value) {
    return;
  }

  searchHitElements.get(currentSearchHitId.value)?.scrollIntoView({
    block: 'center',
    behavior: 'smooth',
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
  () => selectedDeviceId.value,
  (deviceId) => {
    suppressDeviceLogLevelSync.value = true;
    if (!deviceId) {
      deviceLogLevel.value = 'Info';
      return;
    }
    deviceLogLevel.value =
      deviceStore.devices.find((device) => device.id === deviceId)?.data.logLevel || 'Info';
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
  await Promise.all([deviceStore.refreshAll(), logsStore.initListener()]);
});
</script>

<style scoped>
.log-viewer,
.log-line,
.log-line__meta,
.log-line__message {
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
