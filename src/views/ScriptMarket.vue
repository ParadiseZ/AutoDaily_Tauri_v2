<template>
  <div class="flex h-full min-h-0 flex-col gap-4">
    <AppPageHeader
      title="脚本市场"
    />

    <SurfacePanel v-if="!userStore.isLoggedIn" class="mx-auto w-full max-w-3xl">
      <div class="flex flex-col gap-6 rounded-[28px] border border-dashed border-(--app-border) bg-(--app-panel-muted)/50 px-6 py-10 text-center">
        <div class="space-y-3">
          <p class="text-sm font-semibold uppercase tracking-[0.18em] text-(--app-text-faint)">访问受限</p>
          <h2 class="text-2xl font-semibold text-(--app-text-strong)">此功能需登录且赞助或开发者用户才能使用</h2>
        </div>
        <div class="flex flex-wrap items-center justify-center gap-3">
          <button class="app-button app-button-primary px-5" type="button" @click="userStore.openAuthModal()">
            登录
          </button>
          <button class="app-button app-button-ghost px-5" type="button" @click="router.push('/scripts')">
            看本地库
          </button>
        </div>
      </div>
    </SurfacePanel>

    <template v-else>
    <SurfacePanel class="grid gap-3 lg:grid-cols-[1.2fr_1fr_220px_120px]">
      <input v-model.trim="filters.keyword" class="app-input" placeholder="搜索脚本名或描述" />
      <input v-model.trim="filters.author" class="app-input" placeholder="按作者筛选" />
      <AppSelect v-model="filters.runtimeType" :options="runtimeOptions" placeholder="运行时" />
      <button class="app-button app-button-primary" type="button" @click="search">
        搜索
      </button>
    </SurfacePanel>

    <div class="min-h-0 flex-1 overflow-y-auto pr-1 custom-scrollbar">
    <div class="grid min-h-full gap-4 xl:grid-cols-[360px_minmax(0,1fr)]">
      <SurfacePanel class="space-y-3">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-semibold text-(--app-text-strong)">搜索结果</p>
          </div>
        </div>

        <AppLoadingState
            v-if="scriptStore.marketLoading"
            label="正在检索..."
        />
        <div v-else-if="!scriptStore.marketPage.records.length" class="space-y-3">
          <EmptyState title="没有找到匹配脚本" description="可以放宽关键字、作者或运行时筛选后重新搜索。" icon="search" />
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
            <p class="truncate text-sm font-semibold text-(--app-text-strong)">{{ script.name || '未命名脚本' }}</p>
            <p class="mt-1 truncate text-xs text-(--app-text-faint)">{{ script.description || '暂无描述' }}</p>
          </button>
        </div>
      </SurfacePanel>

      <SurfacePanel class="space-y-6">
        <template v-if="selectedScript">
          <div class="space-y-2">
            <div class="flex flex-wrap items-center gap-2">
              <h2 class="text-2xl font-semibold text-(--app-text-strong)">{{ selectedScript.name || '未命名脚本' }}</h2>
              <StatusBadge label="云端脚本" tone="info" />
              <StatusBadge v-if="isSelectedIncompatible" label="需要升级程序" tone="warning" />
            </div>
            <p class="text-sm leading-6 text-(--app-text-soft)">
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

          <div v-if="isSelectedIncompatible" class="rounded-lg border border-amber-300/60 bg-amber-50 px-4 py-3 text-sm text-amber-900">
            {{ selectedScript.compatibility?.reason || '该脚本需要当前程序尚未支持的新能力，请先更新程序。' }}
          </div>

          <div class="rounded-lg border border-(--app-border) bg-(--app-panel-muted)/60 p-4">
            <div class="mb-3 flex items-center justify-between gap-3">
              <p class="text-sm font-semibold text-(--app-text-strong)">更新日志</p>
              <span v-if="changeLogsLoading" class="text-xs text-(--app-text-faint)">加载中</span>
            </div>
            <div v-if="selectedChangeLogs.length" class="space-y-4">
              <div v-for="log in selectedChangeLogs" :key="log.id ?? `${log.versionNum}-${log.versionName}`" class="border-t border-(--app-border) pt-3 first:border-t-0 first:pt-0">
                <p class="mb-2 text-sm font-semibold text-(--app-text-strong)">
                  {{ log.versionName || `版本 ${log.versionNum ?? '-'}` }}
                </p>
                <MarkdownView :content="log.contentMd" empty-text="该版本没有填写更新日志。" />
              </div>
            </div>
            <MarkdownView v-else :content="null" empty-text="该脚本还没有云端更新日志。" />
          </div>

          <div class="grid gap-3 text-sm text-(--app-text-soft) md:grid-cols-2">
            <div class="rounded-[20px] border border-(--app-border) p-4">
              <p class="text-xs uppercase tracking-[0.16em] text-(--app-text-faint)">发布时间</p>
              <p class="mt-2 text-(--app-text-strong)">{{ formatDate(selectedScript.createTime) }}</p>
            </div>
          </div>

          <div class="flex flex-wrap gap-3">
            <button class="app-button app-button-primary" type="button" :disabled="isSelectedIncompatible" @click="downloadSelected">
              {{ isSelectedIncompatible ? '需要升级程序' : '下载到本地' }}
            </button>
            <button class="app-button app-button-ghost" type="button" @click="router.push('/scripts')">
              查看本地库
            </button>
          </div>
        </template>

        <EmptyState
          v-else
          title="选择一个脚本后查看详情"
          description="搜索结果会显示在左侧，选中后可查看元信息"
          icon="info"
        />
      </SurfacePanel>
    </div>
    </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue';
import { useRouter } from 'vue-router';
import AppSelect from '@/components/shared/AppSelect.vue';
import AppPageHeader from '@/components/shared/AppPageHeader.vue';
import EmptyState from '@/components/shared/EmptyState.vue';
import SurfacePanel from '@/components/shared/SurfacePanel.vue';
import StatusBadge from '@/components/shared/StatusBadge.vue';
import MarkdownView from '@/components/shared/MarkdownView.vue';
import { useScriptStore } from '@/store/script';
import { useUserStore } from '@/store/user';
import { scriptService } from '@/services/scriptService';
import { showToast } from '@/utils/toast';
import { formatDate, formatRuntimeLabel } from '@/utils/presenters';
import AppLoadingState from '@/components/shared/AppLoadingState.vue';

const router = useRouter();
const scriptStore = useScriptStore();
const userStore = useUserStore();
const selectedScriptId = ref<string | null>(null);
const selectedChangeLogs = ref<Awaited<ReturnType<typeof scriptService.listChangeLogs>>>([]);
const changeLogsLoading = ref(false);
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
  { label: '全部运行时', value: '' },
  { label: 'Rhai', value: 'rhai' }
];

const selectedScript = computed(
  () => scriptStore.marketPage.records.find((script) => script.id === selectedScriptId.value) ?? null,
);
const isSelectedIncompatible = computed(() => selectedScript.value?.compatibility?.compatible === false);

const loadSelectedChangeLogs = async () => {
  if (!selectedScriptId.value) {
    selectedChangeLogs.value = [];
    return;
  }

  changeLogsLoading.value = true;
  try {
    selectedChangeLogs.value = await scriptService.listChangeLogs(selectedScriptId.value);
  } catch {
    selectedChangeLogs.value = [];
  } finally {
    changeLogsLoading.value = false;
  }
};

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
  if (isSelectedIncompatible.value) {
    showToast(selectedScript.value.compatibility?.reason || '该脚本需要先升级程序', 'warning');
    return;
  }

  try {
    const result = await scriptStore.downloadMarketScript(
      selectedScript.value.id,
      selectedScript.value.runtimeType || 'rhai',
      userStore.userProfile?.id || null,
    );
    if (!result.success) {
      throw new Error(result.message || '下载失败');
    }
    showToast(result.message || '脚本已写入本地库', 'success');
  } catch (error) {
    showToast(error instanceof Error ? error.message : '下载失败', 'error');
  }
};

onMounted(async () => {
  if (!userStore.isLoggedIn) {
    userStore.openAuthModal();
    return;
  }
  await search();
});

watch(selectedScriptId, () => {
  void loadSelectedChangeLogs();
});

watch(
  () => userStore.isLoggedIn,
  async (loggedIn, previousLoggedIn) => {
    if (!loggedIn) {
      selectedScriptId.value = null;
      selectedChangeLogs.value = [];
      if (previousLoggedIn) {
        userStore.openAuthModal();
      }
      return;
    }

    await search();
  },
);
</script>
