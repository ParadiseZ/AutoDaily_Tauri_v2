<template>
  <div class="space-y-6">
    <AppPageHeader
      title="脚本市场"
    />

    <SurfacePanel class="grid gap-3 lg:grid-cols-[1.2fr_1fr_220px_120px]">
      <input v-model.trim="filters.keyword" class="app-input" placeholder="搜索脚本名或描述" />
      <input v-model.trim="filters.author" class="app-input" placeholder="按作者筛选" />
      <AppSelect v-model="filters.runtimeType" :options="runtimeOptions" placeholder="运行时" />
      <button class="app-button app-button-primary" type="button" @click="search">
        搜索
      </button>
    </SurfacePanel>

    <div class="grid gap-4 xl:grid-cols-[360px_minmax(0,1fr)]">
      <SurfacePanel class="space-y-3">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-semibold text-[var(--app-text-strong)]">搜索结果</p>
          </div>
        </div>

        <div v-if="scriptStore.marketLoading" class="py-12 text-sm text-[var(--app-text-soft)]">正在检索...</div>
        <div v-else-if="!scriptStore.marketPage.records.length">
          <EmptyState title="没有找到匹配脚本" description="换一个关键字、作者名或运行时试试？" />
        </div>
        <div v-else class="space-y-2">
          <button
            v-for="script in scriptStore.marketPage.records"
            :key="script.id"
            type="button"
            class="app-list-item"
            :class="{ 'app-list-item-active': script.id === selectedScriptId }"
            @click="selectedScriptId = script.id"
          >
            <p class="truncate text-sm font-semibold text-[var(--app-text-strong)]">{{ script.name || '未命名脚本' }}</p>
            <p class="mt-1 truncate text-xs text-[var(--app-text-faint)]">{{ script.description || '暂无描述' }}</p>
          </button>
        </div>
      </SurfacePanel>

      <SurfacePanel class="space-y-6">
        <template v-if="selectedScript">
          <div class="space-y-2">
            <div class="flex flex-wrap items-center gap-2">
              <h2 class="text-2xl font-semibold text-[var(--app-text-strong)]">{{ selectedScript.name || '未命名脚本' }}</h2>
              <StatusBadge label="云端脚本" tone="info" />
            </div>
            <p class="text-sm leading-6 text-[var(--app-text-soft)]">
              {{ selectedScript.description || '脚本作者还没有补充详细说明。' }}
            </p>
          </div>

          <div class="grid gap-3 md:grid-cols-2">
            <div class="app-stat">
              <p class="app-stat-label">作者</p>
              <p class="app-stat-value text-base">{{ selectedScript.userName || '未知作者' }}</p>
            </div>
            <div class="app-stat">
              <p class="app-stat-label">运行时</p>
              <p class="app-stat-value text-base">{{ formatRuntimeLabel(selectedScript.runtimeType) }}</p>
            </div>
            <div class="app-stat">
              <p class="app-stat-label">版本</p>
              <p class="app-stat-value text-base">{{ selectedScript.verName || '未标记' }}</p>
            </div>
            <div class="app-stat">
              <p class="app-stat-label">下载次数</p>
              <p class="app-stat-value text-base">{{ selectedScript.downloadCount || 0 }}</p>
            </div>
          </div>

          <div class="grid gap-3 text-sm text-[var(--app-text-soft)] md:grid-cols-2">
            <div class="rounded-[20px] border border-[var(--app-border)] p-4">
              <p class="text-xs uppercase tracking-[0.16em] text-[var(--app-text-faint)]">发布时间</p>
              <p class="mt-2 text-[var(--app-text-strong)]">{{ formatDate(selectedScript.createTime) }}</p>
            </div>
            <div class="rounded-[20px] border border-[var(--app-border)] p-4">
              <p class="text-xs uppercase tracking-[0.16em] text-[var(--app-text-faint)]">包名</p>
              <p class="mt-2 text-[var(--app-text-strong)]">{{ selectedScript.pkgName || '未指定' }}</p>
            </div>
          </div>

          <div class="flex flex-wrap gap-3">
            <button class="app-button app-button-primary" type="button" @click="downloadSelected">
              下载到本地
            </button>
            <button class="app-button app-button-ghost" type="button" @click="router.push('/scripts')">
              查看本地库
            </button>
          </div>
        </template>

        <EmptyState
          v-else
          title="选择一个脚本查看详情"
        />
      </SurfacePanel>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { useRouter } from 'vue-router';
import AppSelect from '@/components/shared/AppSelect.vue';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import StatusBadge from '@/components/shared/StatusBadge.vue';
import { useScriptStore } from '@/store/script';
import { useUserStore } from '@/store/user';
import { showToast } from '@/utils/toast';
import { formatDate, formatRuntimeLabel } from '@/utils/presenters';

const router = useRouter();
const scriptStore = useScriptStore();
const userStore = useUserStore();
const selectedScriptId = ref<string | null>(null);
const filters = reactive({
  keyword: '',
  author: '',
  runtimeType: '',
});
/*const runtimeOptions = [
  { label: '全部运行时', value: '' },
  { label: 'Rhai', value: 'rhai' },
/!*  { label: 'JavaScript', value: 'javaScript' },
  { label: 'Lua', value: 'lua' },*!/
  { label: 'AI + Vision', value: 'aIAndVision' },
  { label: 'AI', value: 'aI' },
];*/
const runtimeOptions = [
  { label: 'Rhai', value: 'rhai' }
];

const selectedScript = computed(
  () => scriptStore.marketPage.records.find((script) => script.id === selectedScriptId.value) ?? null,
);

const search = async () => {
  await scriptStore.searchMarket({
    page: 1,
    keyword: filters.keyword,
    author: filters.author,
    runtimeType: filters.runtimeType,
  });
  selectedScriptId.value = scriptStore.marketPage.records[0]?.id ?? null;
};

const downloadSelected = async () => {
  if (!selectedScript.value) {
    return;
  }

  try {
    const result = await scriptStore.downloadMarketScript(selectedScript.value.id, userStore.userProfile?.id || null);
    if (!result.success) {
      throw new Error(result.message || '下载失败');
    }
    showToast(result.message || '脚本已写入本地库', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '下载失败', 'error');
  }
};

onMounted(async () => {
  await search();
});
</script>
